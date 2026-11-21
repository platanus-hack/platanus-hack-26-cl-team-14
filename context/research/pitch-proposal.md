# Pitch proposal: "Santiago en un segundo"

- Status: Proposed by brave-gecko, 2026-11-21 06:13 UTC. Rewritten by keen-koala (claude-fable-5-1, replaces brave-gecko) at 06:50 UTC, numbers checked by bright-heron and calm-condor at 06:40 UTC: the MUT site is Las Condes (`13114`), a fallback wow moment on the comuna engine, Variant C on the Las Condes car share, and questions for the humans. Humans accept, change, or reject it in `context/pitch.md`.
- Structure follows `context/pitch.md`. Spoken lines are in Chilean Spanish. All other text is simple English.
- Every number has the file that holds it. `claim` marks a press or publisher number that nobody recomputed. `thread` marks a number from a real run that an agent posted but no file holds yet: say it on stage only after a file holds it.

## Problem (one sentence)

Santiago is decided in pieces (a mall here, a tower there, a stadium somewhere) and nobody can see what each piece does to the 7.4 million people who live with the result.

## What exists tonight (06:50 UTC)

| Part | State | Who |
| --- | --- | --- |
| Comuna engine, `GET /api/world` and `POST /api/simulate` (`zone_id`, `jobs_multiplier`, `housing_multiplier`) | Merged in `main` (PR #25, #28). Runs on Emerson's laptop on :8787 with the real files. 0.09 ms per run, measured. Comuna points are population-weighted since PR #37. **It is a reconstruction of the Census 2024 matrix with a sensitivity control, not a prediction**: no full comparable historical matrix is verified yet to judge it (Census 2017 has no work comuna; the INE 2002 viewer gives the top 3 destinations only; a 2002 Redatam export is in check). On stage, the Santiago view is "reconstrucción del Censo 2024 + sensibilidad, sin validar" (witty-llama's audit, PR #47) | witty-llama, bright-otter |
| Station model `engine/src/stations.rs`, `--bin stations` | Merged (PR #38, #46); scorer `check_score.py` merged (PR #42). Predicts weekday boardings for the 126 Metro stations from blocks, jobs, work OD and GTFS; scored on 31 held-out stations. L1 = 0.4736, L2 (public transport share of the home comuna) = 0.4688. Next layers registered (L2b jobs 500 m, L3 bus feeder, L4 split, L5 schools, L6 travel times `travel_times.csv`) | lucky-llama builds, clever-heron reviews, calm-condor owns the score |
| Point-to-station module `engine/src/site.rs` (contract PR #26) | Merged (PR #30). **Route `POST /api/site-scenario` merged in `main` (PR #33, 06:50 UTC), running on :8787, and the Tobalaba Chrome loop passed** (clever-heron, 06:52 UTC: the page sends 4,679 jobs and 298 dwellings, 18:00 shows 7,516 → 7,431, daily −513, reset restores 7,516). Open: no mode split in the response, comuna arcs cover the labels at close zoom, Santiago startup guard | witty-llama, clever-heron reviews, bright-otter runs |
| Scene `web/` | v2.3 **merged in `main` (PR #34, 07:05 UTC)**; clever-heron posted Release v1 at 07:09 UTC. v2.5 on the shared branch adds the third view "Puntaje" (31 held-out stations, observed and predicted columns, the WAPE number; reads a temporary L1 file until lucky-llama's `station_score_run.json` lands; layer label still missing). Mode output (`resident_commuters_by_mode`, PR #36) is in the engine and on screen (PR #44): under the day change, "83 personas salen en auto al día" and "51 en transporte público", labelled nuevos residentes and supuesto: reparto modal de Las Condes; 0 / 298 / 1,500 dwellings give 0 / 83 / 419 by car. Fresh-clone run passes with PR #45's two-terminal instructions (engine, then `python3 -m http.server` at the repo root, open `/web/`); first build about 75 s. Santiago view runs on the live engine (browser pass: jobs ×2 on Providencia 225,827 → 409,109, reset exact). Tobalaba runs the site scenario end to end: 298 dwellings → −513 per day, 1,500 → −408, 0 → −539, reset exact; three columns (observado, base, escenario). keen-panda's own weak point: the three columns differ by 1% in height, only the numbers show the change. That is the finding below; the MUT moment needs the mode figures on screen | keen-panda |
| Data pack `data/mvp/` | `communes.csv`, `work_od.csv`, `mode_share.csv`, `blocks.csv`, `jobs.csv`, `schools.csv`, `buildings.geojson`, `communes_urban.geojson`, 3 Metro station files | lucky-llama, swift-fox, bright-heron |

Two pitches follow from this: the full one (site route in `main`, browser loop passed) and the fallback (comuna engine only, kept as the recorded safety net). Both are written below. The humans choose the story; the agents choose which one ships by the state of the browser loop at rehearsal time.

## The numbers we can say on stage

| Number | What it is | File |
| --- | --- | --- |
| 7,400,741 | Persons with a home comuna in the RM, Census 2024 | `data/mvp/communes.csv`, `sources/od-flows-admin.md` |
| 52,646 | Census 2024 blocks of the RM, each with persons and dwellings | `data/mvp/blocks.csv` |
| 6,260,194 | Persons in the 34 comunas of Gran Santiago (the camera frame of the city view) | `data/mvp/communes_urban.geojson`, `data/mvp/README.md` |
| 2,740,454 | RM residents who commute to a known comuna (9,128 pairs; 51,640 to a comuna outside the RM) | `data/mvp/work_od.csv` |
| 305,601 | Workers who go to a job in Las Condes each day from any RM comuna (column sum). 28,583 of them live in Santiago centro, checked against the INE visualizer | `data/mvp/work_od.csv`, `data/mvp/README.md` (jobs) |
| 107,707 | Las Condes residents who commute to a known comuna. 45,458 work inside Las Condes, 12,923 in Providencia, 12,409 in Santiago | `data/mvp/work_od.csv` |
| 51.1% and 31.4% | Las Condes commuters who go to work by **car** (55,009) and by **public transport** (33,834: bus, Metro, train, taxi, colectivo in one Census code). Walking 10.0%. Say "transporte público", never "Metro": the Census does not split them | `data/mvp/mode_share.csv`, `data/mvp/README.md` |
| 0.9251, 2.0847, 0.3577 | Las Condes measured site inputs for **apartment blocks**: occupied share, persons per occupied dwelling (320 blocks, 160,457 persons), RM commuters per resident (private dwellings). A tower is flats, so this is the pair for the MUT tower (swift-fox, clever-heron). 298 dwellings → about 206 commuters: 105 travel by car, 65 by public transport, if they behave like Las Condes. The comuna pair (0.9405, 2.4185: houses plus flats) gives 242, 124, 76. Always take a pair from one cohort. The rate 0.3545 (every dwelling kind) changes 2 commuters. **The stage number is the engine's number**, never a hard-coded one | `data/mvp/site_inputs.csv` (swift-fox), `mode_share.csv`; see Variant C |
| 39.7% | Providencia commuters by public transport. Providencia is the comuna of the **station** (TB-TOB is 26 m inside Providencia), not of the site. Use only if the story names Providencia | `data/mvp/mode_share.csv`; lucky-llama's point-in-polygon check, thread 06:32 UTC |
| 81,613 | Metro boardings at Tobalaba on an average weekday, April 13 to 17, 2026 (37 observed half hours; 11 bins are null, not zero) | `data/mvp/tobalaba_observations.json` (PR #23, #29), station `TB-TOB` |
| 7,516 | Boardings at Tobalaba in the busiest half hour, 18:00 to 18:30 (bin 36). Next: 18:30 with 6,704 and 17:30 with 6,244 | same file |
| 54,840 and 14,954 | Weekday boardings at Los Leones (`LE-LEN`) and El Golf (`GO`), 36 observed bins each | `data/mvp/los_leones_observations.json`, `el_golf_observations.json` (PR #29) |
| 298 | Average dwellings in an RM residential tower of 15 floors or more (361 permits, median 238, max 1,161) | `sources/construction-pipeline.md` |
| 477 | Average dwellings in an Estación Central tower of 20 floors or more (57 permits). 57 × 477 = 27,189 dwellings = +35.8% on the 75,992 dwellings of the comuna | `sources/construction-pipeline.md`, `data/mvp/communes.csv` |
| 2,156 | Persons in the Pila del Ganso block, Estación Central, Census 2017: 1,019 dwellings, about 2.1 persons per dwelling | `sources/construction-pipeline.md` |
| 3,220 | Operating RM schools with coordinates and enrolment, MINEDUC 2026 | `data/mvp/schools.csv` |
| 35,345 and 40,167 | Attendance at two U. de Chile home matches at Estadio Nacional, 2025; season average 29,564 | `sources/events-attendance.md` |
| about 48,000 | Capacity of Estadio Nacional, `claim` | `sources/events-attendance.md` |
| 4,679 | Modeled jobs on the MUT block `13114131009033`, Las Condes, Av. Apoquindo 2730, from floor area: offices 45,765 m² × 88% leased / 10 m² = 4,027, shops 40,000 m² × 75% / 46 m² = 652. Say "modelado". Replaces 7,124 (2,481 of those came from 2012 answers when the site was a pit). Press claim: 4,000 office workers | `data/mvp/jobs.csv` (bright-heron, 06:38 UTC), `data/mvp/README.md` |
| 31,516 | Building footprints with height around Tobalaba (Overture 2026-11-18, OSM + Google + Microsoft). 5 features tagged `is_mut`: the base 17.5 m and 4 towers, three at 58 to 65 m and Torre 4 without height data (6 m default). Gran Torre Costanera 300 m | `data/mvp/buildings.geojson` |
| 0.09 ms | Comuna engine run time on Emerson's laptop, 52 zones, measured by bright-otter and by this agent. Say "menos de un milisegundo", not "7 millones de agentes" | thread 06:31 UTC; `curl` on :8787 |
| 0.4688 | **Score of the simulation** (lucky-llama, PR #38 and #46, clever-heron reproduced each): `WAPE_bins` of the station model on 31 held-out Metro stations that the model never saw (1,123 half-hour cells, 570,369 boardings per day). Ladder: training-mean profile 0.6245 → L0 nearest station 0.5001 → L1 within 1,000 m 0.4736 → L2 public transport share of the home comuna 0.4688. Say "error absoluto agregado de 47% sobre 31 estaciones reservadas, puntaje de desarrollo". Not an average error per station, not a final test | `tools/pack/check_score.py`, `data/mvp/metro_observations.json` (PR #35), `engine/src/stations.rs` |
| −513 per day | **Real run on :8787, PR #33, 06:55 UTC, by this agent.** MUT request: 4,679 jobs removed, 298 dwellings added, apartment cohort 0.9251 × 2.0847 × 0.3577 (`site_inputs.csv`), the other shares from clever-heron's example as `user_assumption` (attendance 0.8, filled jobs 0.9, Metro entry 0.2, access 0.8, 500 m). Result: 575 residents, 164 home departures, home +26.3, work −539.0, net −512.7 boardings per day, **−0.63% of 81,612.8**. Peak bin 18:00: 7,516 → 7,431 (−85). Morning bins +1 to +5 each. 11 null bins, 0.007 ms. clever-heron's earlier run with 7,124 jobs gave −778. Print the engine's number on stage, never this table | `engine/README.md` request shape; `data/mvp/site_inputs.csv` |

Numbers we do NOT have: measured MUT workers or visitors, the Metro share inside the 31.4%, where stadium fans live, observed boardings at any station other than Tobalaba, Los Leones, El Golf, and the run time of the site route. Do not say them until a file holds them.

## The finding that changes the wow moment

The real run of `POST /api/site-scenario` with the MUT request (4,679 jobs, 298 dwellings, apartment cohort) moves Tobalaba by −513 boardings per day, 0.63%; the 18:00 column goes from 7,516 to 7,431. On a column of 7,516 boardings, a 1% change is one pixel on a projector. **The station columns cannot be the surprise for the MUT case.** Even 5,000 dwellings add about 720 morning boardings (0.14 per dwelling per day, from the same run), spread over 8 bins: 2% per bin.

Two things in the data are large enough to see from the back of the room:

1. **The street, not the station.** 51.1% of Las Condes commuters travel by car. In a 298-dwelling tower at the MUT, about 105 people travel to work by car and 65 by public transport (bus and Metro together). The surprise is the inversion: a tower 100 m from the biggest Metro station of Santiago makes more car commuters than public transport riders, if its residents behave like Las Condes. Say "personas que van en auto", never "autos": the Census gives the main mode, not vehicles or a road. (Variant C.)
2. **The stadium.** 40,000 persons exit in 2 hours. With a public transport share near 50%, that is about 20,000 boardings over 4 half hours: 5,000 per bin, the size of the Tobalaba evening peak, on stations that normally have far less. This moves columns. It needs scenario-only columns labelled "supuesto" at the stations near the stadium, because we have no observations there.

Recommendation: keep Variant A's story (18:00 at Tobalaba), but the juror's number produces two results on screen: the new residents who go to work by car (large, red) and the change at the station (small, honest: "el Metro casi no lo nota"). The columns move for real in the stadium case.

## Variant A: "Las seis de la tarde en Tobalaba" (recommended)

Emotional core: the daily commuter. Every person in the room has stood on the Tobalaba platform at 18:00. The city decided that for them, and nobody simulated it. Now a juror decides, live, and sees the answer in a second.

| Time | What the audience sees | What must work behind it |
| --- | --- | --- |
| 0:00 | Dark screen. One line: "Son las seis de la tarde en Tobalaba." The 3D isometric scene appears: real buildings, the station, the MUT towers. A column grows on the station for each half hour of the day; at 18:00 it hits 7,516. Spoken: "Siete mil quinientas personas suben al metro aquí, en media hora, todas las tardes. Nadie lo decidió. Simplemente pasó." | Scene with `data/mvp/buildings.geojson` and the 48 weekday bins of `tobalaba_observations.json` (null bins draw nothing). Exists in scene v1.2. |
| 0:25 | Zoom out to Gran Santiago. 52,646 blocks light up with real people. Spoken: "Santiago se decide a pedazos. Un mall acá, una torre allá, un estadio donde salga más barato. Y nadie puede ver qué le pasa a los siete millones que viven las consecuencias." | `blocks.csv` as a point layer sized by `population`; camera frame from `communes_urban.geojson` (lon −70.80 to −70.50, lat −33.63 to −33.34). Scene v2, keen-panda. |
| 0:50 | Three large numbers: 7,4 M personas, 2,74 M viajes al trabajo, 52 comunas. Thin arcs from home comunas to work comunas. Spoken: "Hicimos un motor. Casa, trabajo y cómo se mueve cada comuna. Datos reales: Censo 2024, DTPM. Todo en Rust: Santiago entero corre en menos de un milisegundo. Y lo medimos: predice las subidas de 31 estaciones de Metro que nunca vio, con un error agregado de 47 por ciento. Es poco. Cada capa lo baja." | `GET /api/world` flows as arcs (2,627 internal pairs); exists and the scene calls it. The score line needs the latest `WAPE_bins` from `check_score.py`; say the current value, and say it is a development score if a juror asks. Visual for the sentence (gentle-fox 4, keen-panda builds): a 5-second panel with the 31 held-out stations as points and two columns each, predicted and observed, plus the WAPE number; one static artifact from the latest reviewed run, no interaction. It is the only part of the demo with an error measure, so it must be seen once. **Two scopes, never mixed (clever-heron, 07:30 UTC):** the score measures the station model (`stations.rs`, a baseline predictor of boardings from homes and jobs). The MUT result at 1:10 comes from the site accounting (`site.rs`, observed minus base plus scenario) and has no score. The presenter must not say or imply that the 47% validates the MUT number. Scene v2.5 has the "Puntaje" view (keen-panda); it must print the layer label next to the number. If the humans find 47% weak on stage, cut the sentence, not the number. |
| 1:10 | **Wow moment.** Camera back to Tobalaba. The MUT block turns red. One control: "Sacar el MUT (4.679 empleos, modelado). Poner una torre de [298] viviendas." Spoken: "Saquemos el MUT. Pongamos una torre. Doscientas noventa y ocho departamentos, el promedio real de una torre de quince pisos en Santiago." Press "Correr". Two results appear under the day change (−513 subidas al día): "+83 salen en auto al día" and "+51 en transporte público al día", both from the engine's `daily_home_departures` by mode, labelled "supuesto de transferencia". On the station: the columns move by one pixel. Spoken: "El metro ni se entera: quinientas subidas menos en ochenta mil. Pero de los vecinos nuevos, ochenta salen en auto cada mañana y cincuenta en transporte público. Esto no es un video. Díganme un número." All three numbers on screen are daily flows, one unit; the stock (206 commuters, 105 by car) stays in the notes. A juror says a number. The presenter types it. Run again. | **Needs the site route.** `POST /api/site-scenario` (contract PR #26, module merged PR #30, route not wired). Plus one small addition to the response: `resident_commuters_by_mode[]` for the site comuna from `mode_share.csv` (car, public transport, walk, bike) so the scene draws the cars without computing. Measured run time on screen. |
| 1:50 | Case 2. The camera flies west to Estación Central. Spoken: "Esto ya pasó una vez. En Estación Central se aprobaron cincuenta y siete torres de más de veinte pisos, cuatrocientas setenta y siete departamentos cada una. Nadie lo simuló." The control says "57 torres de 477 viviendas en Estación Central". Run. The arcs from the comuna get thicker: +3,926 commuters to Santiago, +2,743 to Las Condes, +5,523 inside Estación Central. Spoken: "Veintiséis mil viajes más salen de Estación Central cada mañana. Casi tres mil llegan acá, a Las Condes." Say deltas, never the new totals (15,332 to Santiago is the new total, not the change). | `POST /api/simulate` with `zone_id` 13106, `housing_multiplier` 1.3578. **Exists; measured today** (bright-heron confirmed): +62,646 persons, +26,080 commuters, 0.09 ms. The scene draws the flow deltas. Honest limit: the comuna engine conserves the 7,400,741 persons, so the new residents come from the other comunas (Santiago loses 3,808). Say "se mueven", never "crecen". Second limit (bright-otter): new residents keep the own-comuna work share, so more housing in a comuna also adds jobs there (+5,523 inside Estación Central). Say it as an assumption: "suponemos que trabajan donde trabaja hoy su comuna". Buildings outside Tobalaba: `blocks.csv` points and simple boxes for the towers. |
| 2:20 | Case 3. Spoken: "Y la pregunta que se hace la U hace diez años: ¿dónde ponemos el estadio?" A stadium marker: 40,000 persons one night. Drag it from Ñuñoa to another comuna. Run. Night columns rise at the stations next to the new place. Label: "origen de hinchas: supuesto, proporcional a población". | **Proposed connection between the scored station model and an editable condition** (the decision clever-heron asked for at 07:30 UTC): this is the one case where `stations.rs` is the engine. It already predicts boardings at all 126 stations from home and job masses; a stadium adds one visitor mass at a point, with an exit profile 21:00 to 23:00 and explicit shares, and the model redistributes it to the stations within the radius. Observed boardings now exist for all 126 stations (PR #35), so the columns can show observed and scenario. The training scale is not refit per intervention. Cases 1 and 2 keep their own engines (site accounting, comuna engine). Not built; needs an owner and a frozen formula before any score. If not built by rehearsal, cut this case. |
| 2:45 | Camera lifts over all Santiago. Spoken: "Un motor. Tres preguntas. La próxima vez que alguien decida un pedazo de Santiago, que lo vea primero." Team name and one line. | Nothing new. |

### Fallback: Variant A on the comuna engine only

Use this if `POST /api/site-scenario` is not wired and checked in the browser by rehearsal time. Same opening (0:00 to 0:50). Then:

| Time | What the audience sees | What must work behind it |
| --- | --- | --- |
| 1:10 | **Wow moment moves to Estación Central.** Spoken: "Esto ya pasó. Cincuenta y siete torres, nadie lo simuló. Hagámoslo ahora." Run. 26,080 more commuters leave the comuna as thicker arcs; the destinations show the change (+3,926 Santiago, +2,743 Las Condes), not the totals. "Díganme una comuna y cuántas torres." A juror says "Ñuñoa, veinte torres". The presenter picks the comuna, types the dwellings, Run. The arcs change. | `POST /api/simulate` only. A comuna selector (52 comunas) and one dwellings input; the scene converts dwellings to `housing_multiplier` = (dwellings + added) / dwellings using the `dwellings` field of `GET /api/world`. That division is the only arithmetic the scene does. Keep multipliers in 0.1 to 2.0 (bright-otter: outside that range the engine returns negative flows). |
| 1:50 | MUT case, shorter. Camera to Tobalaba. The MUT block turns red, the tower rises (scene v1.2 already does this). Spoken: "Y acá, al lado de la estación más grande de Santiago: si sacamos el MUT y ponemos una torre de 298 departamentos, el Censo dice que la mitad de sus vecinos iría al trabajo en auto. Unos ochenta en auto cada mañana, cincuenta en transporte público." The Tobalaba columns stay observed and keep the label "observado, abril 2026". | No engine call. Two numbers from `mode_share.csv` and `communes.csv`, printed by the scene as data, labelled "Censo 2024, si se comportan como Las Condes". Honest: the columns do not change, and we say so. |
| 2:20 | Close as above. Cut the stadium: the comuna engine has no event term and no night. | Nothing new. |

What the fallback loses: no live station change, no stadium. What it keeps: a juror controls a real engine and the city answers at once, with real Census flows.

## Variant B: "El error de Estación Central"

Emotional core: regret and a second chance. We already built the vertical ghettos without a simulation. The engine, run on 2017 data, shows that it would have seen 2024 coming.

| Time | What the audience sees | What must work behind it |
| --- | --- | --- |
| 0:00 | Estación Central in 3D, 2017. Low buildings. Spoken: "Año 2017, Estación Central. Un barrio de casas de dos pisos." Towers rise one by one, 57 of them, 477 dwellings each. "En siete años, cincuenta y siete torres. Nadie preguntó qué pasaría." | Census 2017 block layer and INE permits 2010 to 2018 for Estación Central (both in `construction-pipeline.md`), animated by permit year. |
| 0:40 | **Wow moment.** Spoken: "Le dimos al motor los datos de 2017 y las torres. Y le preguntamos qué pasaría en 2024." Two columns per block: engine 2017 plus towers, and Census 2024 real. "El motor lo vio. Siete años antes." | Engine backtest against Census 2024 blocks. Nobody owns this. The comuna engine has no 2017 state. |
| 1:30 | MUT at Tobalaba, as in Variant A at 1:10, shorter. | Same as A. |
| 2:15 | Stadium, as in Variant A. | Same as A. |
| 2:45 | Close: "Esta vez, miremos antes." | Nothing new. |

Not buildable tonight. Keep only if the humans want it and accept that A is the fallback.

## Variant C: "La torre que se va en auto"

Emotional core: the inversion. We put the tallest towers next to the Metro so that people take the Metro. In Las Condes, the Census says most of them go to work by car.

The check, on the files that exist:

| Dwellings at the MUT site | Commuters (× 0.9251 × 2.0847 × 0.3577, apartment pair, `site_inputs.csv`) | Travel by car (51.1%) | By public transport (31.4%) |
| --- | --- | --- | --- |
| 298 (RM tower average) | 206 | 105 | 65 |
| 477 (Estación Central tower average) | 329 | 168 | 103 |
| 1,000 | 690 | 352 | 217 |

It holds at every size and with every basis (swift-fox checked four: 100 to 127 car commuters for 298 dwellings), because it is a ratio of the comuna: 1.6 car commuters for each public transport rider. On stage say "unas cien personas", not a precise count. Sensitivity, not headline (swift-fox, clever-heron): the Las Condes blocks that are almost all flats show a car share of 0.4684 against 0.5192 for the comuna, on the Census block-table basis. That universe includes people with no fixed place of work and people who work abroad, so it is not the commuter cohort of `mode_share.csv`; a ratio-adjusted chain on one basis gives 221 commuters and 102 by car. It is an association, not a tower effect: most Las Condes flat blocks sit on the Apoquindo and Metro corridor, so part of the gap is the place. Every version rounds to "unas cien personas". The headline stays the comuna share 51.1%, the one measured commuter cohort. The Metro part of the 78 is smaller still, and unknown.

Does it beat the station columns as a surprise? Yes for the MUT case: the column change is about 1% (invisible), the number of people who go by car is a sentence the room repeats. No as a whole pitch: a Census ratio is not a simulation, and the decision `20261121-0100` requires that the change shows in the 3D scene. So C is not a separate pitch. **C is the surprise inside A's wow moment**: cars on the street, one-pixel columns, "el Metro ni se entera".

What the scene must show for C: from the new tower, about 105 small red figures leave by car and about 65 blue figures walk to the station entrance and the bus stops, over 3 seconds, with the two large numbers from the engine beside them and the label "Censo 2024, Las Condes, supuesto: los vecinos nuevos se comportan como la comuna". The weak point to say out loud: residents 100 m from a station probably use the Metro more than the comuna average. We do not have that number (swift-fox's 0.76 Metro share of nearby boardings is an intensity, not a mode probability). The label carries it.

What the engine adds for C (witty-llama owns it, contract posted 06:57 UTC): `resident_commuters_by_mode[]` in the site response, per Census code: `mode_code`, `mode_name`, `share`, `resident_commuters` (stock, before attendance) and `daily_home_departures` (flow, after attendance), with a source block and a `transfer_assumption` label. Two numbers the stage must not mix (clever-heron): the 298-dwelling tower has about 206 resident commuters, of which about 105 go by car (stock); with attendance 0.8 that is about 164 daily departures, about 84 by car (flow). The screen prints the flow, next to the day change in the same unit (keen-panda v2.4: "+83 salen en auto al día"); the spoken line says the same number, "ochenta salen en auto cada mañana", with the public transport flow beside it (51), so the inversion is visible. The scene computes nothing.

## What "looks like the mock" means

- The scene opens at Tobalaba with real building footprints, the MUT towers and the Metro station. Nothing on screen is smaller than a thumb at 10 m from the projector.
- 37 columns, one per observed half hour, on Tobalaba; the 18:00 column reads 7,516 and is the highest. Null bins draw nothing.
- One zoom from Tobalaba to Gran Santiago in one camera move, with 52,646 block points sized by population.
- One control with one number (dwellings) and one button (Correr). A comuna selector only in case 2. No menus.
- After Correr, the result is visible from the back of the room: arcs that change thickness, figures that leave by car, or columns that rise. A one-pixel change gets a label that says it is one pixel.
- Every assumed number on screen carries the word "supuesto". Every real number carries its year.

## The exact interaction and output the pitch needs (for the engine owners)

Full version: one call, three uses. `POST /api/site-scenario`, contract PR #26, module PR #30. The presenter changes only `dwellings_added` (MUT case), the comuna and dwellings (towers case), `visitors` and the exit window (stadium case). Response per the contract (`observed[48]`, `scenario[48]`, `site_base[48]`, `site_scenario[48]`, `flows[]`, `elapsed_ms`, `assumptions[]`) plus `resident_commuters_by_mode[]` for Variant C. The scene draws; it never computes.

Fallback: `POST /api/simulate` with `zone_id` and `housing_multiplier`. Already exists. The scene needs the `dwellings` field of `GET /api/world` to turn a dwellings count into a multiplier.

## What the pitch does NOT need (do not build it)

- Migration flows (`od_migration_rm.csv`): no moment shows them.
- DTPM trips and stages archives: the pitch shows boardings only.
- Saturday and Sunday boardings: all moments are a weekday.
- Jobs at Costanera Center or any block other than the MUT: no moment names them. `jobs.csv` is used for one number, 4,679.
- A Metro share of boardings near the site as a "mode probability": clever-heron already said it is not one. The pitch says "transporte público" and labels the Metro share "supuesto".
- Buildings outside the Tobalaba frame: case 2 uses block points and boxes.
- Schools: no moment says "colegio" now. `schools.csv` has no moment unless the humans want the 08:00 school flows.
- More Metro station files: Tobalaba, Los Leones and El Golf are enough for every moment except the stadium, which uses scenario-only columns.

## Research: what wins a 3-minute hackathon pitch (quick level, 3 searches)

1. **One moment the judges understand at once beats a tour of features.** https://blog.jetbrains.com/ai/2026/06/how-to-win-a-hackathon-notes-from-the-judging-table/
2. **Open with a frustration the room has felt, then build everything toward the moment the product resolves it.** https://musichackspace.org/blog/hackathon-presentation-tips and https://intercom.help/builderbase/en/articles/14836128-builder-s-guide-ch-14-crafting-a-demo-that-lands
3. **Live demos need a recorded fallback, and the pitch leads with the problem, not the stack.** https://developer.salesforce.com/blogs/2014/07/deliver-winning-pitch-hackathons , https://www.df.cl/df-mas/punto-de-partida/36-horas-de-programacion-nonstop-asi-fue-la-primera-platanus-hack

Applied here: the hook is a feeling (18:00 at Tobalaba), the wow is one contrast that a juror controls, and we record a 3-minute video of the final run as a fallback.

## Questions for the humans

Each one is taste or direction. Agents cannot decide them. Answer in `#agent-chat` or in `context/pitch.md`.

1. **Which comuna does the story name for the MUT?** The site is in Las Condes (`13114`), the station is 26 m inside Providencia. Recommendation: say "Las Condes, al lado de Tobalaba" and use the Las Condes numbers (51.1% van en auto, 31.4% en transporte público). Never mix the two comunas in one sentence.
2. **"Transporte público" or "Metro" on stage?** The Census gives only the combined share. Recommendation: say "transporte público" for every Census number; say "Metro" only for DTPM boardings (7,516, 81,613) and for the labelled assumption in the engine.
3. **Which surprise for the MUT case?** (a) station columns that move 1%, (b) the new residents who go to work by car and "el Metro ni se entera", (c) both. Recommendation: (c), Variant A with C inside. It is honest and it is the only version where the MUT result is visible from the back of the room.
4. **Second and third use case?** Options: Estación Central towers (data exists, engine exists today), stadium (needs the station model to accept a visitor mass; observed boardings exist for all 126 stations), MUT at another comuna, school flows at 08:00. Recommendation: Estación Central as case 2 in every version; stadium as case 3 only if an owner connects the station model to it before rehearsal; otherwise two cases and a stronger close.
5. **Variant A or B?** Recommendation: A. B needs a backtest that nobody owns and a 2017 layer that is not downloaded.
