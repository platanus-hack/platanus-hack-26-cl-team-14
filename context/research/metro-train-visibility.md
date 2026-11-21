# Metro train visibility in the cached replay

The Metro scene shows a full service closure. The timed preview reports zero station entries from 07:00 to 10:00. The current viewer still draws Metro trains from the packed timetable. [Pitch proposal](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/354) · [Timed preview](https://discord.com/channels/1573514576400752640/1573809606277337280/1573820133032591372) · [Mover draw](../../viz/src/movers.rs)

## Result and viewer contract

- The preview engine commit is `c405b3e00d6383edc58cf8dc31a443ebf7047753`. It keeps the original line change in `applied`. The full Metro removal has `kind: "line"`, `op: "remove"`, and `target: {"type":"mode","id":"metro"}`. The timed preview publishes `closure_window.half_hours: [14,20]` for the effective window. The raw applied edit can retain a requested window that the engine rejects. [Window validation](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/371) [Result construction](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/c405b3e00d6383edc58cf8dc31a443ebf7047753/engine/src/scenario.rs)
- The viewer computes a half-hour slot from its replay clock. `Routes::load` uses only Metro paths for `tracks`. `trains` builds a list for each track from the weekday timetable. Riders index these lists by track, then find the run. [Viewer source](../../viz/src/movers.rs)
- Proposed change for the mover owner. On each scenario revision, confirm the Metro removal in `applied`, then read the effective interval from `closure_window.half_hours`. During the closure, keep each per-track list empty. This keeps the track indices valid and leaves the existing train drawing and pool cleanup in place. Treat an applied removal without an effective interval as active all day. Resume timetable trains when the slot reaches 20 or when the baseline snapshot replaces the scenario. Keep surface rail separate from the Metro lists. This proposal covers train visibility. Check line fade and station signs in their own draw paths.

## Matched check

| Clock | Packed active runs | Expected service in the timed scenario |
| --- | ---: | --- |
| 06:59 | 155 | Baseline |
| 07:00 | 156 | Hidden |
| 08:00 | 173 | Hidden |
| 09:59 | 141 | Hidden |
| 10:00 | 142 | Baseline |

The middle column counts runs whose first arrival is at or before the clock and whose last departure is after it. It covers all 14 line and direction entries in the pack. It does not count visible trains at one camera. [Timetable](../../web/data/metro_schedule.json.gz)

Use the same camera and fixed clock for baseline and closure at 08:00. Include the subway view so tunnel geometry does not hide the witness. Check 06:59 and 10:00 on the same timed result. The native run and the behavior of people already riding at the cutoff remain unverified.
