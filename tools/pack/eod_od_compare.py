"""V1 daily trip score: model work and study trips vs EOD 2012, by directed EOD sector pair.

Model side: the engine reproduces the Census comuna flows exactly (mode_od.csv for
workers, school_od.csv for students; see engine/src/population.rs). Each of the 45
survey comunas lies in one EOD sector (data/mvp/eod_sectors/zones.csv), so the model
sector pairs follow from the comuna flows. Comunas outside the survey are excluded.

Survey side: weekday trips with Proposito 1 (al trabajo) or 3 (al estudio), both ends in
sectors 1-7, weight = viajes.FactorLaboralNormal x personas.Factor_LaboralNormal
(the design in context/research/v1-daily-trip-comparison.md).

Score: total variation distance (TV) between the two share distributions, 0 = same, 1 = disjoint.

Usage: python3 tools/pack/eod_od_compare.py /path/to/EOD_STGO
"""
import collections, csv, sys

EOD = sys.argv[1]
MODE_EOD = {1: "car", 17: "walk", 18: "bicycle", **{c: "public" for c in (2, 3, 4, 5, 6, 7, 9, 10, 11, 12)}}
MODE_CENSUS = {"1": "car", "2": "public", "3": "walk", "4": "bicycle"}

sector = {}
for r in csv.DictReader(open("data/mvp/eod_sectors/zones.csv")):
    if r["sector"]:
        sector[r["polygon_cut"]] = r["sector"]


def tv(a, b):
    sa, sb = sum(a.values()), sum(b.values())
    return 0.5 * sum(abs(a.get(k, 0) / sa - b.get(k, 0) / sb) for k in set(a) | set(b))


def num(x):
    return float(x.replace(",", ".")) if x else 0.0


model = {"work": collections.Counter(), "study": collections.Counter()}
model_mode, outside, outside_rm = collections.Counter(), collections.Counter(), collections.Counter()
for purpose, path, count in (("work", "data/mvp/mode_od.csv", "commuters"), ("study", "data/mvp/school_od.csv", "students")):
    for r in csv.DictReader(open(path)):
        if not r["destination_cut"].startswith("13"):
            outside_rm[purpose] += float(r[count])
            continue
        n = float(r[count])
        o, d = sector.get(r["origin_cut"]), sector.get(r["destination_cut"])
        if not (o and d):
            outside[purpose] += n
            continue
        model[purpose][(o, d)] += n
        if purpose == "work":
            model_mode[MODE_CENSUS.get(r["mode"], "other")] += n

# Higher education (engine/src/population.rs): the travelling share of Census higher-education residents
# (role 1 in person_traits_residents.csv) goes to SIES campuses with P(campus | home comuna) proportional to
# (in-person + hybrid enrollment) x exp(-BETA x minutes). BETA and minutes follow engine/src/choice.rs:
# BETA is fitted below from the same pack (jobs.csv, work_od.csv, travel times), as the engine does on each load.
campus, enrolled = collections.Counter(), 0.0
for r in csv.DictReader(open("data/mvp/higher_ed.csv")):
    campus[r["cut"]] += float(r["in_person"] or 0) + float(r["hybrid"] or 0)
    enrolled += float(r["enrollment"] or 0)
travel = sum(campus.values()) / enrolled
import math
com = list(csv.DictReader(open("data/mvp/communes.csv")))
cuts = [r["cut"] for r in com]
xy = {r["cut"]: (float(r["longitude"]), float(r["latitude"])) for r in com}
line = lambda a, b: math.hypot((xy[a][0]-xy[b][0])*92.9, (xy[a][1]-xy[b][1])*110.6)
km = {(a, b): (min(line(a, e) for e in cuts if e != a)/2 if a == b else line(a, b)) for a in cuts for b in cuts}
minutes = {}
for r in csv.DictReader(open("data/mvp/travel_times.csv")):
    minutes[(r["origin_cut"], r["destination_cut"])] = float(r["minutes_door_to_door"])
xs = [(km[k], t) for k, t in minutes.items()]
m = len(xs); sx = sum(x for x,_ in xs); sy = sum(y for _,y in xs); sxx = sum(x*x for x,_ in xs); sxy = sum(x*y for x,y in xs)
b = (m*sxy - sx*sy)/(m*sxx - sx*sx); a = (sy - b*sx)/m
mins = lambda o, d: minutes.get((o, d), a + b*km[(o, d)])
jobs, flows = collections.Counter(), collections.Counter()
for r in csv.DictReader(open("data/mvp/jobs.csv")):
    jobs[r["cut"]] += float(r["jobs"])
for r in csv.DictReader(open("data/mvp/work_od.csv")):
    if r["destination_cut"].startswith("13"):
        flows[(r["origin_cut"], r["destination_cut"])] += float(r["workers"])


def loglik(b):  # gravity P(d | o) proportional to jobs_d x exp(-b x minutes_od), as choice.rs
    ll = 0.0
    for o in cuts:
        row = {d: jobs[d] * math.exp(-b * mins(o, d)) for d in cuts}
        t = sum(row.values())
        ll += sum(f * math.log(row[d] / t) for (oo, d), f in flows.items() if oo == o)
    return ll


lo, hi = 0.0, 1.0
for _ in range(50):
    a3, b3 = lo + (hi - lo) / 3, hi - (hi - lo) / 3
    lo, hi = (a3, hi) if loglik(a3) < loglik(b3) else (lo, b3)
BETA = (lo + hi) / 2
print(f"campus beta (fitted as choice.rs) {BETA:.5f} per minute")
share = {}
for o in cuts:
    row = {d: w*math.exp(-BETA*mins(o, d)) for d, w in campus.items()}
    t = sum(row.values()); share[o] = {d: v/t for d, v in row.items()}
for r in csv.DictReader(open("data/mvp/person_traits_residents.csv")):
    if r["role"] != "1":
        continue
    for d, f in share[r["cut"]].items():
        n = float(r["persons"]) * travel * f
        o, s_d = sector.get(r["cut"]), sector.get(d)
        if o and s_d:
            model["study"][(o, s_d)] += n
        else:
            outside["study"] += n

person = {}
for r in csv.DictReader(open(f"{EOD}/personas.csv", encoding="latin-1"), delimiter=";"):
    person[r["Persona"]] = num(r["Factor_LaboralNormal"])
survey = {"work": collections.Counter(), "study": collections.Counter()}
survey_mode, survey_n, survey_out, survey_rm = collections.Counter(), collections.Counter(), collections.Counter(), collections.Counter()
for r in csv.DictReader(open(f"{EOD}/viajes.csv", encoding="latin-1"), delimiter=";"):
    purpose = {"1": "work", "3": "study"}.get(r["Proposito"])
    o, d = r["SectorOrigen"], r["SectorDestino"]
    w = num(r["FactorLaboralNormal"]) * person.get(r["Persona"], 0.0)
    if not purpose or w <= 0:
        continue
    if o not in "1234567" or d not in "1234567" or not o or not d:
        # Sector 9 = exterior a RM; sector 0 or blank = no location given.
        (survey_rm if "9" in (o, d) else survey_out)[purpose] += w
        continue
    survey[purpose][(o, d)] += w
    survey_n[purpose] += 1
    if purpose == "work":
        survey_mode[MODE_EOD.get(int(r["ModoAgregado"] or 0), "other")] += w

names = {"1": "Norte", "2": "Poniente", "3": "Oriente", "4": "Centro", "5": "Sur", "6": "Sur-Oriente", "7": "Ext. Sur-Poniente"}
for purpose in ("work", "study"):
    m, s = model[purpose], survey[purpose]
    orig = lambda c: collections.Counter({k: sum(v for (o, _), v in c.items() if o == k) for k in names})
    dest = lambda c: collections.Counter({k: sum(v for (_, d), v in c.items() if d == k) for k in names})
    print(f"\n## {purpose}")
    print(f"model {sum(m.values()):,.0f} persons in survey comunas; excluded: {outside[purpose]:,.0f} with an end in the 7 RM comunas "
          f"outside the survey, {outside_rm[purpose]:,.0f} with a destination outside the RM")
    print(f"survey {sum(s.values()):,.0f} weighted trips from {survey_n[purpose]:,} records; "
          f"excluded: {survey_rm[purpose]:,.0f} weighted trips with an end outside the RM (sector 9), "
          f"{survey_out[purpose]:,.0f} with an unlocated end (sector 0) and no sector-9 end")
    print(f"TV sector pair {tv(m, s):.3f} | TV origin sector {tv(orig(m), orig(s)):.3f} | TV destination sector {tv(dest(m), dest(s)):.3f}")
    print("| destination sector | model share | survey share |")
    dm, ds = dest(m), dest(s)
    for k in names:
        print(f"| {names[k]} | {dm[k] / sum(dm.values()):.1%} | {ds[k] / sum(ds.values()):.1%} |")
    top = sorted(set(m) | set(s), key=lambda k: -abs(m[k] / sum(m.values()) - s[k] / sum(s.values())))[:5]
    print("largest pair gaps (model vs survey):", ", ".join(f"{names[o]}→{names[d]} model {m[(o, d)] / sum(m.values()):.1%} vs survey {s[(o, d)] / sum(s.values()):.1%}" for o, d in top))

print(f"\n## work mode (TV {tv(model_mode, survey_mode):.3f})")
print("| mode | model 2024 | survey 2012 |")
for k in ("car", "public", "walk", "bicycle", "other"):
    print(f"| {k} | {model_mode[k] / sum(model_mode.values()):.1%} | {survey_mode[k] / sum(survey_mode.values()):.1%} |")

