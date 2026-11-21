# Station access walking geometry

Build mapped walking routes for the three stations used in issue #289. Engine
snapshots supply the origins and shares. This pack supplies geometry only.

## Research and source

Context7 resolved `/project-osrm/osrm-backend` and fetched its Route service and
Waypoint documentation on 2026-11-21. The route endpoint returns the fastest
profile route, full GeoJSON geometry, route distance and waypoint snap distances:
https://github.com/Project-OSRM/osrm-backend/blob/master/docs/http.md

The existing `tools/pack/local_mover_routes.py` uses the same local foot endpoint,
250 m snap limit, OSM attribution and deterministic gzip. This generator keeps
the full geometry instead of simplifying it. The shortest returned route among
real station entrances chooses a modeled door, not an observed passenger choice.

Inputs are producer `e4871238941582773ef656dd1d68cb6252417d69` baseline revision 1
and Tobalaba-closed revision 2. Their compressed hashes are in the pack. The
producer's independent reset check found the original station table restored.
The snapshots are local `data/raw/station-access-routes/` inputs, not tracked data.

Entrances come from `web/data/metro_entrances.json.gz`, SHA-256
`c6086e24b740af0a93d9a84d1416ee66cd2de56461ff701adebddfbb008f0327`.
Use each record's `position`, not the cartographic `porch.position`. Los Leones
D/E share one coordinate. Their IDs remain in `entrance_ids`.

Local OSRM 26.11.0 used the existing foot MLD graph from regional OSM extract
SHA-256 `43596705fc3af495df63c47ed66b10ac19e699a02d54867a7b1786a6049de529`
and official foot profile SHA-256
`74e231f81301d89a10e2bd0e67b787147c761b3ee06f665426e21a9b736372f4`.
The graph was read only. The owned router on 127.0.0.1:19443 stopped after use.

## Run from the repository root

```sh
python3 -B tools/pack/station_access_routes.py \
  data/raw/station-access-routes/e487123-baseline.json.gz \
  data/raw/station-access-routes/e487123-tobalaba-closed.json.gz \
  --pbf ../offstation-walk-routes/data/raw/offstation-walk/region.osm.pbf \
  --profile /opt/homebrew/share/osrm/profiles/foot.lua
```

The local foot router must already run. `--plan` prints the workload without
requests. `--stations` selects physical IDs; defaults are TB-TOB, GO, LE-LEN.

## Consumer contract

`web/data/station_access_routes.json.gz` contains `sources`, source hashes,
`attribution`, `license`, `max_snap_m`, `door_choice`, `routes` and `paths`.
Match a route by exact `(station, origin[0], origin[1])`. A `Routed` row has
`entrance`, `entrance_ids`, `route` (index into WGS84 `paths`), `length_m`,
`duration_s` and two `snap_m` values. An `Unrouted` row has per-door failure codes
and no geometry. The path endpoints are snapped graph coordinates, not fabricated
links from the source point or to the door.

Only positive `walk` origins are selected. Counts and shares remain in the
current engine response: use `home` for bins 0–23 and `job` for 24–47, multiply
`scenario_half_hour[bin]` by its original share. Never normalize the selected walk
shares or reuse baseline weights after a change. Bus origins are not bus stops;
they do not authorize a full walking route. No new demand or trips are created.

## Generated result and limits

Pack SHA-256 `19a8a71f963a36e7e30698dd2072b139bfdd360bac42f97cbaeb5afe33c5ce3b`:
45,383 gzip bytes, 491 origin records, 490 paths, 18,309 points, 1,988 queries.
Routed origins: Tobalaba 169, Los Leones 165, El Golf 156. One El Golf origin
`[-70.593325,-33.408955]` has NoSegment for both doors. It represents 6.646% of
baseline home entries and 1.598% of job entries; after closure, 4.711%/0.937%.
Keep that missing geometry and share explicit. Do not draw a straight substitute.

Chosen snap distances: median 2.96 m, p95 49.61 m, maximum 96.85 m. The longest
route is 1,404.8 m. This geometry does not prove sidewalk legality, route use,
door capacity or observed passenger movements. No native view was run. The
consumer's rendering, matching and before/after capture remain separate gates.

Manual inspection opened the generated metadata, representative coordinate
arrays and source share coverage. `git diff --check` passed. No tests per repo.
