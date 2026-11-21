# Proposed Tobalaba scenario contract

Implement after the first comuna loop. One point replaces jobs with N dwellings; Rust estimates the change in local weekday Metro boardings. First case: MUT-style land-use replacement only. This is a transparent accounting model, not a causal forecast. No implementation or fitted parameters yet.

## Site geography and mode share

The initial [comuna demo](../mvp.md) changes Providencia (`13123`). The proposed MUT replacement site is in **Las Condes (`13114`)**, at Av. Apoquindo 2730. The [Las Condes municipal resolution of 14 February 2026](https://archivos.lascondes.cl/descargas/transparencia/actos_resoluciones/comerciales/2026/febrero/002327.pdf) grants a licence at that address. The [data owner's site report](https://discord.com/channels/1573514576400752640/1573579088684253235/1573584489890709507) identifies manzana `13114131009033`. Use Las Condes for this site's resident OD row and workplace OD column; proximity to Tobalaba does not change the site's comuna.

Manual aggregation of [`mode_share.csv` at data commit `09b99527`](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/09b99527f9b26a06bf0440f9e9317273a6461fc6/data/mvp/mode_share.csv) gives Las Condes 33,834 public-transport commuters / 107,707 known-comuna commuters = **31.4%**. Providencia has 25,512 / 64,203 = **39.7%**. These Census 2024 shares describe residents of each comuna, including commuters to destinations outside the RM. Code 2 combines bus, Metro, train, taxi and colectivo: neither percentage is a Metro share. Applying a comuna share to new site residents is a model assumption; the Metro and station-access shares remain explicit inputs. These resident shares do not describe workers arriving at MUT from other comunas.

## Evidence and missing parameters

[Verified Tobalaba observations](tobalaba-observations.md): 37 weekday half-hours, 81,612.8 validation-derived boardings per average weekday, April 13–17, 2026. Station TB-TOB is the parent of four L1/L4 platforms. These are boardings, not arrivals, passengers aboard trains, unique people, or MUT visitors.

[Verified Census sources](sources/verify-people-work.md) supply residents and housing fields. Use the corrected known-comuna commute OD, not all employed residents; preserve the core's explicit RM/external boundary. [Historical EOD](sources/verify-transport.md) can supply weighted mode/time priors after matching definitions. [MATSim Santiago](sources/prior-art-simulators.md) separates activities, modes and paths; follow that accounting separation without importing its full engine or historical calibration.

Required parameters have no hidden numerical defaults:

| Parameter | Basis or missing information |
| --- | --- |
| Site coordinate/CUT, baseline jobs/dwellings, jobs removed, dwellings added | Scenario inputs. Actual MUT worker count is unverified. SII headquarters counts do not measure site jobs. |
| Occupied share; persons per occupied dwelling | Compatible Census occupied-private-dwelling counts if available; otherwise user assumptions. Total dwellings include vacant units. The pitch's 2.1 comes from a different historical block, not measured MUT occupancy. |
| Known commuters per resident | Compatible filtered OD origin total / resident count, with exclusions and boundary. Assumes new residents resemble this population. Not employment rate. |
| Weekday attendance; filled job share | Explicit assumptions. Census usual workplace is not daily attendance. |
| Metro-entry share; station-access share | Site values unknown. Census main mode or weighted EOD can inform a proxy. Combined bus+Metro share is NOT Metro-entry share; keep mode categories and denominator explicit. |
| Departure-time profiles; maximum access distance | Declared assumptions below, not fitted behavior. |
| Population policy | Local increment for this layer. Relocation requires the core to remove residents/jobs from previous locations. No citywide conservation claim here. |

Each parameter carries its value, `source_kind` (observed aggregate, historical proxy, user assumption), and source/description. Missing inputs produce a list of required fields, not invented estimates. An illustrative run is allowed with assumptions visibly labeled.

## Accounting

Use expected counts, not sampled people. Compute the same site contribution C for baseline state 0 and scenario state 1:

```
residents[x] = dwellings[x] * occupied_share[x] * persons_per_occupied_dwelling[x]
home_departures[x,j] = residents[x] * known_commuters_per_resident[x]
                     * attendance_home[x] * work_destination_share[x,j]
worker_departures[x,i] = jobs[x] * filled_job_share[x]
                       * attendance_work[x] * home_origin_share[x,i]
C[x,t] = sum_j(home_departures[x,j] * metro_entry_share_home[x,j] * access_share_home[x,j]) * home_time_share[t]
       + sum_i(worker_departures[x,i] * metro_entry_share_work[x,i] * access_share_work[x,i]) * work_time_share[t]
residual[t] = observed[t] - C[0,t]
scenario[t] = residual[t] + C[1,t]
```

Scenario jobs = baseline jobs − jobs removed; scenario dwellings = baseline dwellings + dwellings added. Time profiles each sum to one; probability shares lie in [0,1]. Attendance counts one outward departure per attending commuter. Metro-entry share means a validation-covered entry; it excludes walk/car/bus-only trips and does not add an entry for every internal Metro transfer.

**Direction:** residents leaving home board locally in the morning. Workers arriving at the site in the morning ALIGHT there; their return departures can board locally in the evening. Do not multiply each round-trip worker by two at Tobalaba.

Normalize the scenario comuna OD row for residents' work destinations and its destination column for workers' home origins. This assumes within-comuna similarity, not observed building OD. Keep same-comuna trips but do not automatically assign them to Metro. With scalar Metro/access shares, destination redistribution alone cannot change the station total; disclose that simplification and do not draw invented routes.

## Access and timing

Choose mode before stop. Select the nearest eligible Metro entrance (GTFS location_type=2), then its parent station, by straight-line distance. Report entrance ID, distance and parent. For the supplied site, verify that this selects TB-TOB. This is a geometric access proxy, not walking-network routing; apply the explicit access share and maximum-distance assumption. A nearest bus stop does not imply Metro use.

Proposed initial profiles: home departures 06:00–10:00, worker return departures 16:00–21:00, local time. Normalize the observed weekday shape within each window. These windows are illustrative assumptions; source-purpose EOD profiles can replace them later. Reusing observed shape is not independent validation.

Shopping, students, deliveries, visitors and weekends stay in the fixed residual. Removing jobs does not model removing all MUT activity.

## Additive API and acceptance

Propose `POST /api/site-scenario`; leave `/api/world` and `/api/simulate` v1 unchanged. Request: site/baseline/scenario fields above, explicit parameters, and `population_policy=local_increment_unbalanced`. The core supplies its baseline/scenario OD shares.

Response: `observed[48]`, `base[48]`, `scenario[48]`, `delta[48]`, `site_base[48]`, `site_scenario[48]`, `flows[]`, measured `elapsed_ms`, parameter provenance and `assumptions[]`. Index k is local k×30 minutes. Absent observed bins have **null** observed/base/scenario/delta values, never zero. Base is reconstruction, not an independent prediction. Flows are expected point→work-comuna commuters, not measured point OD or routed train loads. Scene draws; Rust computes.

Metadata: `station_scope=TB-TOB`, `scenario_day=LABORAL`, `model_boundary=local_increment_unbalanced`, `global_conservation_claim=false`, `validation=accounting_only`, plus omitted activities. Other sites/days return the implemented scope limit until reviewed.

- No changed inputs returns the observed baseline exactly; do not add existing site residents/jobs twice.
- If C[0,t] exceeds observed[t], report inconsistent parameters. Do not silently clamp negative residuals or fit an unexplained scale factor.
- Show home and worker contributions separately, even when net daily change cancels. Never multiply all 81,612.8 boardings by the site's housing/jobs multiplier.
- Display “April observations” and “scenario estimate under assumptions.” Low/central/high inputs are sensitivity scenarios, not confidence intervals.
- Baseline reproduction and arithmetic closure are accounting checks. No available post-intervention data validates the causal effect.

Three pitch cases can share this later: tower additions generate home departures; job removal changes worker departures; stadium visitors add an **exit** departure term with explicit count, attendance, Metro/access shares and exit profile. Event arrivals are not local station boardings. Implement the reviewed MUT case first, then add event/tower variants without changing the accounting meaning.
