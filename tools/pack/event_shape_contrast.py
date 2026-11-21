"""Weekend shape contrast at three venue stations, April 11 and 12, 2026.

The rule was frozen and posted before any value was read:
https://discord.com/channels/1573514576400752640/1573579088684253235/1573640576606404610

Venue stations, fixed before the run: ESN Estadio Nacional (L6), PE Pedrero (L5,
Estadio Monumental), PQ Parque O'Higgins (L2, Movistar Arena). No other station enters.

Statistic, frozen: for each 30-minute bin, share = bin / the station's own observed day
total, for the venue and for every other station of its own line; the reported gap is
`share_venue - median(share_peers)` in percentage points of the day. Shares, not
boardings, so a large station cannot look like a peak. A missing bin stays null and
enters nothing.

`gap_pp` is the FROZEN primary statistic (each station divided by its own observed
bins). `gap_pp_common_mask` is a SENSITIVITY computed after the results were seen, on
the bins the venue observes and only over peers that observe every one of them; it is
not the frozen result.

What this cannot establish: causality, attendance, a Metro-entry share, or the end time
of any event. A peak aligned with a calendar date is a coincidence in time, nothing more.

Output: data/validation/event_shape_2026_04/shape_contrast.csv
        data/validation/event_shape_2026_04/manifest.json
Usage:  python3 -I tools/pack/event_shape_contrast.py
"""
import collections, csv, hashlib, io, json, pathlib, statistics, zipfile

OBS = pathlib.Path("data/mvp/metro_observations.json")
GTFS = pathlib.Path("data/raw/dtpm-gtfs-march/GTFS_20260321_v3.zip")
OUT = pathlib.Path("data/validation/event_shape_2026_04")
VENUES = {"ESN": "Estadio Nacional", "PE": "Pedrero", "PQ": "Parque O'Higgins"}
DAYS = ("SABADO", "DOMINGO")

records = json.loads(OBS.read_text())
records = records["stations"] if isinstance(records, dict) else records
by_id = {(s.get("station") or {}).get("id"): s for s in records}

z = zipfile.ZipFile(GTFS)
table = lambda n: csv.DictReader(io.TextIOWrapper(z.open(n), encoding="utf-8-sig"))
metro = {r["route_id"]: r["route_short_name"] for r in table("routes.txt") if r["route_type"] == "1"}
trip_line = {r["trip_id"]: metro[r["route_id"]] for r in table("trips.txt") if r["route_id"] in metro}
parent = {r["stop_id"]: (r.get("parent_station") or r["stop_id"]) for r in table("stops.txt")}
lines = collections.defaultdict(set)
for row in table("stop_times.txt"):
    line = trip_line.get(row["trip_id"])
    if line:
        lines[parent.get(row["stop_id"], row["stop_id"])].add(line)


def label(index):
    return f"{index * 30 // 60:02d}:{index * 30 % 60:02d}"


rows, summary = [], []
for day in DAYS:
    for venue, name in VENUES.items():
        own_lines = lines.get(venue, set())
        peers = sorted(i for i in by_id if i != venue and lines.get(i, set()) & own_lines)
        venue_bins = by_id[venue]["half_hour_bins"][day]
        venue_total = sum(v for v in venue_bins if v is not None)
        observed = [b for b in range(48) if venue_bins[b] is not None]
        # sensitivity denominator: peers that observe every bin the venue observes
        common = [p for p in peers
                  if all(by_id[p]["half_hour_bins"][day][b] is not None for b in observed)]
        venue_common_total = sum(venue_bins[b] for b in observed)
        best = (None, None)
        for b in range(48):
            peer_shares, peer_common = [], []
            for p in peers:
                bins = by_id[p]["half_hour_bins"][day]
                total = sum(v for v in bins if v is not None)
                if bins[b] is not None and total:
                    peer_shares.append(bins[b] / total)
            if b in observed:
                for p in common:
                    bins = by_id[p]["half_hour_bins"][day]
                    peer_common.append(bins[b] / sum(bins[x] for x in observed))
            share = venue_bins[b] / venue_total if venue_bins[b] is not None else None
            gap = (share - statistics.median(peer_shares)) * 100 if (share is not None and peer_shares) else None
            share_c = venue_bins[b] / venue_common_total if venue_bins[b] is not None else None
            gap_c = (share_c - statistics.median(peer_common)) * 100 if (share_c is not None and peer_common) else None
            if gap is not None and (best[0] is None or gap > best[0]):
                best = (gap, b)
            rows.append([day, venue, name, "/".join(sorted(own_lines)), b, label(b),
                         "" if venue_bins[b] is None else f"{venue_bins[b]:.1f}",
                         "" if share is None else f"{share * 100:.6f}",
                         "" if not peer_shares else f"{statistics.median(peer_shares) * 100:.6f}",
                         "" if gap is None else f"{gap:.6f}",
                         "" if gap_c is None else f"{gap_c:.6f}",
                         len(peer_shares), len(peers), len(common)])
        summary.append({"day_type": day, "station_id": venue, "station": name,
                        "lines": sorted(own_lines), "peers": peers,
                        "peer_count": len(peers), "peers_common_mask": len(common),
                        "observed_bins": len(observed), "missing_bins": 48 - len(observed),
                        "day_total_observed": venue_total,
                        "max_gap_pp": best[0], "max_gap_bin": label(best[1])})

OUT.mkdir(parents=True, exist_ok=True)
with (OUT / "shape_contrast.csv").open("w", newline="", encoding="utf-8") as fh:
    writer = csv.writer(fh)
    writer.writerow(["day_type", "station_id", "station", "lines", "bin_index", "bin",
                     "venue_boardings", "venue_share_pct", "peer_median_share_pct",
                     "gap_pp", "gap_pp_common_mask", "peers_with_data", "peers_total",
                     "peers_common_mask"])
    writer.writerows(rows)
(OUT / "manifest.json").write_text(json.dumps({
    "kind": "weekend_shape_contrast",
    "dates": {"SABADO": "2026-04-11", "DOMINGO": "2026-04-12"},
    "timezone": "America/Santiago",
    "frozen_rule": "https://discord.com/channels/1573514576400752640/1573579088684253235/1573640576606404610",
    "primary_statistic": "gap_pp, each station divided by its own observed day total",
    "sensitivity": "gap_pp_common_mask, computed after the results were seen; not the frozen statistic",
    "sources": {"observations": OBS.name,
                "observations_sha256": hashlib.sha256(OBS.read_bytes()).hexdigest(),
                "gtfs": GTFS.name,
                "gtfs_sha256": hashlib.sha256(GTFS.read_bytes()).hexdigest()},
    "limits": ["Descriptive only. A peak on the same date as an event is a coincidence in time.",
               "It does not establish causality, attendance, a Metro-entry share or an end time.",
               "Bins belong to one calendar date; 00:00-01:00 is that morning, never the next night.",
               "Nothing here bears on the weekday L4b score."],
    "cases": summary}, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
print(OUT / "shape_contrast.csv", len(rows), "rows")
for c in summary:
    print(f"  {c['day_type']:8s} {c['station_id']:4s} peers {c['peer_count']:2d}"
          f" (common {c['peers_common_mask']:2d}) observed {c['observed_bins']:2d}"
          f" max gap {c['max_gap_pp']:+6.2f} pp at {c['max_gap_bin']}")
