"""Build anonymous normal-weekday tour profiles from the local Santiago EOD 2012 archive."""

import argparse
import csv
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--survey", type=Path, default=ROOT / "data/raw/eod2012/x/eod2012/EOD_STGO")
parser.add_argument("--output", type=Path, default=ROOT / "data/mvp/eod_tours")
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=True)
STUDY = {
    "1": "preschool",
    "2": "preschool",
    "3": "preschool",
    "4": "school",
    "6": "higher_ed",
    "7": "higher_ed",
    "8": "higher_ed",
}
ERRANDS = ("shopping", "health", "public_errand")
PURPOSES = ("preschool", "school", "higher_ed", *ERRANDS)
SECTORS = set("1234567")
STAGE_MODES = {str(k) for k in range(1, 19)}


def table(name, encoding="utf-8-sig"):
    with (args.survey / f"{name}.csv").open(encoding=encoding, newline="") as f:
        yield from csv.DictReader(f, delimiter=";")


def factor(value):
    return float(value.replace(",", ".") or 0)


def covered(trip, study):
    code = trip["Proposito"]
    if code == "3":
        return study
    if code == "12" and trip["ActividadDestino"] == "7":
        return "public_errand"
    return {"5": "health", "11": "shopping"}.get(code)


def broad(purpose):
    if purpose in {"3", "4"}:
        return "ESTUDIO"
    if purpose in {"5", "6", "8", "9", "10", "11", "12", "13", "14"}:
        return "OTROS"


def minutes(value):
    hour, minute = map(int, value.split(":"))
    return hour * 60 + minute


def diary_times(trips):
    # Only times of day are published. Trip order sets the date when the clock wraps.
    intervals = []
    day = 0
    last_clock = None
    for trip in trips:
        if not trip["HoraIni"]:
            intervals.append(None)
            continue
        start = minutes(trip["HoraIni"])
        if last_clock is not None and start < last_clock:
            day += 1440
        last_clock = start
        if not trip["HoraFin"] or "5" in trip["CodigoTiempo"]:
            intervals.append(None)
            continue
        end = minutes(trip["HoraFin"])
        intervals.append((day + start, day + end + (1440 if end < start else 0)))
    bad = set()
    for i, a in enumerate(intervals):
        if a:
            for j in range(i + 1, len(intervals)):
                b = intervals[j]
                if b and b[0] < a[1] and a[0] < b[1]:
                    bad.update((i, j))
    return [None if i in bad else interval for i, interval in enumerate(intervals)]


def write(name, header, rows):
    with (args.output / f"{name}.csv").open("w", newline="") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(header)
        for row in rows:
            w.writerow([f"{value:.9f}" if isinstance(value, float) else value for value in row])


households = {r["Hogar"]: (r["Sector"], r["Zona"]) for r in table("Hogares")}
people = {}
residents = defaultdict(Counter)
study = defaultdict(Counter)
for r in table("personas"):
    weight = factor(r["Factor_LaboralNormal"])
    if not weight:
        continue
    person = r["Persona"]
    sector, zone = households[r["Hogar"]]
    group = STUDY.get(r["DondeEstudia"])
    people[person] = (sector, zone, group, weight)
    residents[sector][person] += weight
    if group:
        study[group, sector][person] += weight

stages = defaultdict(dict)
for r in table("Etapas", encoding="latin1"):
    stages[r["Viaje"]][int(r["Etapa"][-1])] = r["Modo"]

diaries = defaultdict(list)
for trip in table("viajes"):
    if trip["Persona"] in people:
        diaries[trip["Persona"]].append(trip)

final_metro = defaultdict(Counter)
unknown_stage = defaultdict(Counter)
metro_only_final = Counter()
covered_metro = Counter()
complete_metro = Counter()
complete_by_purpose = Counter()
raw_by_purpose = Counter()
factor_differences = 0
higher_seed = defaultdict(Counter)
audit = defaultdict(Counter)
tours = []
for person, trips in diaries.items():
    home, zone, education, person_weight = people[person]
    trips.sort(key=lambda r: int(r["Viaje"][-2:]))
    timeline = diary_times(trips)
    for i, out in enumerate(trips):
        weight = person_weight * factor(out["FactorLaboralNormal"])
        if weight == 0:
            continue
        category = broad(out["Proposito"])
        final_is_metro = False
        if category:
            legs = stages[out["Viaje"]]
            count = int(out["Etapas"])
            if set(legs) != set(range(1, count + 1)):
                unknown_stage[category][person] += weight
            else:
                last_mode = next((legs[k] for k in range(count, 0, -1) if legs[k] not in {"8", "9"}), None)
                if last_mode == "4":
                    final_metro[category][person] += weight
                    final_is_metro = True
                    if out["ModoAgregado"] == "4":
                        metro_only_final[category] += weight
                elif last_mode is not None and last_mode not in STAGE_MODES:
                    unknown_stage[category][person] += weight
        purpose = covered(out, education)
        if not purpose:
            continue
        audit["candidate"][person] += weight
        raw_by_purpose[purpose] += weight
        eligible_metro = final_is_metro and out["ModoAgregado"] == "4"
        if eligible_metro:
            covered_metro[category] += weight
        dest = out["SectorDestino"]
        number = int(out["Viaje"][-2:])
        previous = trips[i - 1] if i else None
        first_trip = i == 0 and number == 1
        follows_home = (
            previous is not None
            and previous["Viaje"][:-2] == person
            and int(previous["Viaje"][-2:]) + 1 == number
            and previous["Proposito"] == "7"
            and previous["ZonaDestino"] == zone
            and timeline[i - 1]
            and timeline[i]
            and timeline[i - 1][1] <= timeline[i][0]
        )
        if (
            out["Viaje"][:-2] != person
            or not (first_trip or follows_home)
            or out["ZonaOrigen"] != zone
            or out["SectorOrigen"] != home
            or dest not in SECTORS
            or out["ZonaDestino"] in {"", "0"}
        ):
            audit["not_home_or_no_sector_zone"][person] += weight
            continue
        if purpose == "higher_ed":
            higher_seed[home, dest][person] += weight
        ret = trips[i + 1] if i + 1 < len(trips) else None
        if (
            not ret
            or ret["Proposito"] != "7"
            or int(ret["Viaje"][-2:]) != int(out["Viaje"][-2:]) + 1
            or any(t["Viaje"][:-2] != person for t in (out, ret))
            or ret["ZonaDestino"] != zone
            or ret["ZonaOrigen"] != out["ZonaDestino"]
            or ret["SectorOrigen"] != dest
            or ret["SectorDestino"] != home
            or not factor(ret["FactorLaboralNormal"])
        ):
            audit["no_matching_weekday_return"][person] += weight
            continue
        a, b = timeline[i : i + 2]
        if not a or not b or b[0] < a[1] or b[1] - a[0] >= 1440:
            audit["missing_overlap_or_time"][person] += weight
            continue
        if not out["ModoAgregado"] or not ret["ModoAgregado"]:
            audit["missing_mode"][person] += weight
            continue
        if factor(ret["FactorLaboralNormal"]) != factor(out["FactorLaboralNormal"]):
            audit["different_expansion_factors"][person] += weight
            factor_differences += 1
        complete_by_purpose[purpose] += weight
        if eligible_metro:
            complete_metro[category] += weight
        start_day = (a[0] // 1440) * 1440
        bins = tuple((t - start_day) // 30 for t in (*a, *b))
        timing = (purpose, *bins, int(out["ModoAgregado"]), int(ret["ModoAgregado"]))
        tours.append(((purpose, home, dest), timing, person, weight, eligible_metro))


# Remove cells backed by only one diary in either output margin. Removing a cell
# from one margin can leave a singleton in the other, so recompute both margins.
def margins(tours):
    ods, times = defaultdict(Counter), defaultdict(Counter)
    for od, time, person, w, _ in tours:
        ods[od][person] += w
        times[time][person] += w
    return ods, times


complete_count = len(tours)
complete_weight = sum(t[3] for t in tours)
while True:
    ods, times = margins(tours)
    safe = [t for t in tours if len(ods[t[0]]) >= 2 and len(times[t[1]]) >= 2]
    if len(safe) == len(tours):
        break
    tours = safe

rate = Counter()
metro = defaultdict(Counter)
for (purpose, home, dest), time, person, w, eligible_metro in tours:
    rate[purpose, home] += w
    if eligible_metro:
        metro["OTROS" if purpose in ERRANDS else "ESTUDIO"][person] += w


def rate_rows():
    for purpose in PURPOSES:
        for home in sorted(SECTORS):
            people_at_home = residents[home] if purpose in ERRANDS else study[purpose, home]
            population = people_at_home.total()
            count = rate[purpose, home]
            yield purpose, home, len(people_at_home), population, count, count / population


write(
    "rates",
    [
        "purpose",
        "home_sector",
        "eligible_diaries",
        "weighted_eligible_people",
        "weighted_tours",
        "tours_per_eligible_person",
    ],
    rate_rows(),
)
write(
    "od",
    ["purpose", "home_sector", "destination_sector", "diaries", "weighted_tours"],
    ((*k, len(v), v.total()) for k, v in sorted(ods.items())),
)
write(
    "time_mode",
    [
        "purpose",
        "out_start_bin",
        "out_end_bin",
        "return_start_bin",
        "return_end_bin",
        "out_mode",
        "return_mode",
        "diaries",
        "weighted_tours",
    ],
    ((*k, len(v), v.total()) for k, v in sorted(times.items())),
)
write(
    "higher_ed_seed",
    ["home_sector", "destination_sector", "diaries", "weighted_outbound_study_trips"],
    ((*k, len(v), v.total()) for k, v in sorted(higher_seed.items()) if len(v) >= 2),
)
write(
    "metro_coverage",
    [
        "broad_purpose",
        "weighted_eligible_outward_metro_only",
        "weighted_all_final_motorized_stage_metro",
        "coverage_share",
        "weighted_broad_metro_only",
        "weighted_covered_metro_only",
        "weighted_complete_metro_only",
    ],
    (
        (
            k,
            metro[k].total(),
            final_metro[k].total(),
            metro[k].total() / final_metro[k].total(),
            metro_only_final[k],
            covered_metro[k],
            complete_metro[k],
        )
        for k in sorted(final_metro)
    ),
)

for key, weights in sorted(audit.items()):
    if len(weights) >= 2:
        print(key, len(weights), f"{weights.total():.3f}")
print("Complete pairs:", complete_count, f"{complete_weight:.3f}")
print("Retained pairs:", len(tours), f"{sum(t[3] for t in tours):.3f}")
print("Pairs with differing expansion factors:", factor_differences)
for purpose in PURPOSES:
    retained = sum(v for (p, _), v in rate.items() if p == purpose)
    print(
        purpose,
        "candidate / complete / retained",
        *(round(v, 3) for v in (raw_by_purpose[purpose], complete_by_purpose[purpose], retained)),
    )
dropped_stages = sum(unknown_stage.values(), Counter())
print(
    "Stage sequence unavailable:",
    len(dropped_stages),
    "diaries;",
    round(dropped_stages.total(), 3) if len(dropped_stages) >= 2 else "weight withheld",
)
print(
    "Higher-ed seed mass suppressed:", round(sum(v.total() for v in higher_seed.values() if len(v) < 2), 3)
)
