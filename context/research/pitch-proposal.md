# Pitch proposal: "Santiago en un segundo"

- Status: Proposed by brave-gecko, 2026-11-21 06:13 UTC. Rewritten by keen-koala (claude-fable-5-1, replaces brave-gecko) at 06:50 UTC, numbers checked by bright-heron and calm-condor at 06:40 UTC: the MUT site is Las Condes (`13114`), a fallback wow moment on the comuna engine, Variant C on the Las Condes car share, and questions for the humans. Humans accept, change, or reject it in `context/pitch.md`.
- Structure follows `context/pitch.md`. Spoken lines are in Chilean Spanish. All other text is simple English.
- Every number has the file that holds it. `claim` marks a press or publisher number that nobody recomputed. `thread` marks a number from a real run that an agent posted but no file holds yet: say it on stage only after a file holds it.

## Problem (one sentence)

Santiago is decided in pieces (a mall here, a tower there, a stadium somewhere) and nobody can see what each piece does to the 7.4 million people who live with the result.

## What exists tonight (06:50 UTC)

| Part | State | Who |
| --- | --- | --- |
| Comuna engine, `GET /api/world` and `POST /api/simulate` (`zone_id`, `jobs_multiplier`, `housing_multiplier`) | Merged in `main` (PR #25, #28). Runs on Emerson's laptop on :8787 with the real files. 0.09 ms per run, measured | witty-llama, bright-otter |
| Point-to-station module `engine/src/site.rs` (contract PR #26) | Merged (PR #30). **The route `POST /api/site-scenario` is not wired**; the scene cannot call it yet | witty-llama wires, clever-heron reviews |
| Scene `web/` | v1.2 draws Tobalaba with observed columns; no API call yet. v2 (Santiago/Tobalaba switch, API wiring) in work | keen-panda |
| Data pack `data/mvp/` | `communes.csv`, `work_od.csv`, `mode_share.csv`, `blocks.csv`, `jobs.csv`, `schools.csv`, `buildings.geojson`, `communes_urban.geojson`, 3 Metro station files | lucky-llama, swift-fox, bright-heron |

Two pitches follow from this: the full one (site route wired) and the fallback (comuna engine only). Both are written below. The humans choose the story; the agents choose which one ships by the state of the route at rehearsal time.

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
| 0.9405, 2.4185, 0.3577 | Las Condes measured site inputs: occupied share of dwellings, persons per occupied dwelling, RM commuters per resident (private-dwelling basis). So a tower of 298 dwellings holds about 242 commuters: 124 by car, 76 by public transport, if they behave like the comuna. Two other bases exist (0.3545 all persons in clever-heron's example, 0.367 all destinations); the engine request must name one, and the stage number follows the engine | `data/mvp/site_inputs.csv` (swift-fox), `mode_share.csv`; see Variant C |
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
| 31,516 | Building footprints with height around Tobalaba (Overture 2026-11-18, OSM + Google + Microsoft). MUT base 17.5 m and 3 towers 58 to 65 m tagged `is_mut`. Gran Torre Costanera 300 m | `data/mvp/buildings.geojson` |
| 0.09 ms | Comuna engine run time on Emerson's laptop, 52 zones, measured by bright-otter and by this agent. Say "menos de un milisegundo", not "7 millones de agentes" | thread 06:31 UTC; `curl` on :8787 |
| −778 per day | `thread`. Manual run of `site.rs` by clever-heron (06:32 UTC) with 7,124 jobs: Tobalaba loses 821 evening boardings and gains 43 morning boardings, under labelled assumptions. With 4,679 jobs the work term scales to −539, net about −496: **0.6% of 81,613** | thread; clever-heron's `tobalaba-wiring-example.json` (06:36 UTC) |

Numbers we do NOT have: measured MUT workers or visitors, the Metro share inside the 31.4%, where stadium fans live, observed boardings at any station other than Tobalaba, Los Leones, El Golf, and the run time of the site route. Do not say them until a file holds them.

## The finding that changes the wow moment

clever-heron's real run of `site.rs` says: the MUT replacement moves Tobalaba by about 1% per day (0.6% with the new 4,679 jobs). On a column of 7,516 boardings, a 1% change is one pixel on a projector. **The station columns cannot be the surprise for the MUT case.** Even 5,000 dwellings add about 720 morning boardings (0.14 per dwelling per day, from the same run), spread over 8 bins: 2% per bin.

Two things in the data are large enough to see from the back of the room:

1. **The street, not the station.** 51.1% of Las Condes commuters drive. A 298-dwelling tower at the MUT puts about 124 cars on Apoquindo and 76 people in buses and the Metro together. The surprise is the inversion: a tower 100 m from the biggest Metro station of Santiago makes more cars than public transport riders, if its residents behave like Las Condes. (Variant C.)
2. **The stadium.** 40,000 persons exit in 2 hours. With a public transport share near 50%, that is about 20,000 boardings over 4 half hours: 5,000 per bin, the size of the Tobalaba evening peak, on stations that normally have far less. This moves columns. It needs scenario-only columns labelled "supuesto" at the stations near the stadium, because we have no observations there.

Recommendation: keep Variant A's story (18:00 at Tobalaba), but the juror's number produces two results on screen: the cars on the street (large, red) and the change at the station (small, honest: "el Metro casi no lo nota"). The columns move for real in the stadium case.

## Variant A: "Las seis de la tarde en Tobalaba" (recommended)

Emotional core: the daily commuter. Every person in the room has stood on the Tobalaba platform at 18:00. The city decided that for them, and nobody simulated it. Now a juror decides, live, and sees the answer in a second.

| Time | What the audience sees | What must work behind it |
| --- | --- | --- |
| 0:00 | Dark screen. One line: "Son las seis de la tarde en Tobalaba." The 3D isometric scene appears: real buildings, the station, the MUT towers. A column grows on the station for each half hour of the day; at 18:00 it hits 7,516. Spoken: "Siete mil quinientas personas suben al metro aquí, en media hora, todas las tardes. Nadie lo decidió. Simplemente pasó." | Scene with `data/mvp/buildings.geojson` and the 48 weekday bins of `tobalaba_observations.json` (null bins draw nothing). Exists in scene v1.2. |
| 0:25 | Zoom out to Gran Santiago. 52,646 blocks light up with real people. Spoken: "Santiago se decide a pedazos. Un mall acá, una torre allá, un estadio donde salga más barato. Y nadie puede ver qué le pasa a los siete millones que viven las consecuencias." | `blocks.csv` as a point layer sized by `population`; camera frame from `communes_urban.geojson` (lon −70.80 to −70.50, lat −33.63 to −33.34). Scene v2, keen-panda. |
| 0:50 | Three large numbers: 7,4 M personas, 2,74 M viajes al trabajo, 52 comunas. Thin arcs from home comunas to work comunas. Spoken: "Hicimos un motor. Casa, trabajo y cómo se mueve cada comuna. Datos reales: Censo 2024, DTPM. Todo en Rust: Santiago entero corre en menos de un milisegundo." | `GET /api/world` flows as arcs (2,627 internal pairs). Exists; the scene must call it. |
| 1:10 | **Wow moment.** Camera back to Tobalaba. The MUT block turns red. One control: "Sacar el MUT (4.679 empleos, modelado). Poner una torre de [298] viviendas." Spoken: "Saquemos el MUT. Pongamos una torre. Doscientas noventa y ocho departamentos, el promedio real de una torre de quince pisos en Santiago." Press "Correr". Two results appear. On the street: about 124 red cars leave the tower onto Apoquindo. On the station: the columns move by one pixel, and a label says "el Metro casi no lo nota: −1%". Spoken: "El metro ni se entera. Apoquindo sí. Esto no es un video. Díganme un número." A juror says a number. The presenter types it. Run again. | **Needs the site route.** `POST /api/site-scenario` (contract PR #26, module merged PR #30, route not wired). Plus one small addition to the response: `resident_commuters_by_mode[]` for the site comuna from `mode_share.csv` (car, public transport, walk, bike) so the scene draws the cars without computing. Measured run time on screen. |
| 1:50 | Case 2. The camera flies west to Estación Central. Spoken: "Esto ya pasó una vez. En Estación Central se aprobaron cincuenta y siete torres de más de veinte pisos, cuatrocientas setenta y siete departamentos cada una. Nadie lo simuló." The control says "57 torres de 477 viviendas en Estación Central". Run. The arcs from the comuna get thicker: +3,926 commuters to Santiago, +2,743 to Las Condes, +5,523 inside Estación Central. Spoken: "Veintiséis mil viajes más salen de Estación Central cada mañana. Casi tres mil llegan acá, a Las Condes." Say deltas, never the new totals (15,332 to Santiago is the new total, not the change). | `POST /api/simulate` with `zone_id` 13106, `housing_multiplier` 1.3578. **Exists; measured today** (bright-heron confirmed): +62,646 persons, +26,080 commuters, 0.09 ms. The scene draws the flow deltas. Honest limit: the comuna engine conserves the 7,400,741 persons, so the new residents come from the other comunas (Santiago loses 3,808). Say "se mueven", never "crecen". Second limit (bright-otter): new residents keep the own-comuna work share, so more housing in a comuna also adds jobs there (+5,523 inside Estación Central). Say it as an assumption: "suponemos que trabajan donde trabaja hoy su comuna". Buildings outside Tobalaba: `blocks.csv` points and simple boxes for the towers. |
| 2:20 | Case 3. Spoken: "Y la pregunta que se hace la U hace diez años: ¿dónde ponemos el estadio?" A stadium marker: 40,000 persons one night. Drag it from Ñuñoa to another comuna. Run. Night columns rise at the stations next to the new place. Label: "origen de hinchas: supuesto, proporcional a población". | Same site route with an event term (40,000 exits, 21:00 to 23:00, explicit shares, per PR #26). Columns are scenario-only, labelled "supuesto": no observed boardings exist for those stations. If the site route is not wired, cut this case. |
| 2:45 | Camera lifts over all Santiago. Spoken: "Un motor. Tres preguntas. La próxima vez que alguien decida un pedazo de Santiago, que lo vea primero." Team name and one line. | Nothing new. |

### Fallback: Variant A on the comuna engine only

Use this if `POST /api/site-scenario` is not wired and checked in the browser by rehearsal time. Same opening (0:00 to 0:50). Then:

| Time | What the audience sees | What must work behind it |
| --- | --- | --- |
| 1:10 | **Wow moment moves to Estación Central.** Spoken: "Esto ya pasó. Cincuenta y siete torres, nadie lo simuló. Hagámoslo ahora." Run. 26,080 more commuters leave the comuna as thicker arcs; the destinations show the change (+3,926 Santiago, +2,743 Las Condes), not the totals. "Díganme una comuna y cuántas torres." A juror says "Ñuñoa, veinte torres". The presenter picks the comuna, types the dwellings, Run. The arcs change. | `POST /api/simulate` only. A comuna selector (52 comunas) and one dwellings input; the scene converts dwellings to `housing_multiplier` = (dwellings + added) / dwellings using the `dwellings` field of `GET /api/world`. That division is the only arithmetic the scene does. Keep multipliers in 0.1 to 2.0 (bright-otter: outside that range the engine returns negative flows). |
| 1:50 | MUT case, shorter. Camera to Tobalaba. The MUT block turns red, the tower rises (scene v1.2 already does this). Spoken: "Y acá, al lado de la estación más grande de Santiago: si sacamos el MUT y ponemos una torre de 298 departamentos, el Censo dice que la mitad de sus vecinos saldría en auto. Ciento veinticuatro autos más en Apoquindo. Setenta y seis personas al transporte público." The Tobalaba columns stay observed and keep the label "observado, abril 2026". | No engine call. Two numbers from `mode_share.csv` and `communes.csv`, printed by the scene as data, labelled "Censo 2024, si se comportan como Las Condes". Honest: the columns do not change, and we say so. |
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

## Variant C: "La torre que llena Apoquindo de autos"

Emotional core: the inversion. We put the tallest towers next to the Metro so that people take the Metro. In Las Condes, the Census says they drive.

The check, on the files that exist:

| Dwellings at the MUT site | Commuters (× 0.9405 × 2.4185 × 0.3577, `site_inputs.csv`) | By car (51.1%) | By public transport (31.4%) |
| --- | --- | --- | --- |
| 298 (RM tower average) | 242 | 124 | 76 |
| 477 (Estación Central tower average) | 388 | 198 | 122 |
| 1,000 | 814 | 416 | 255 |

It holds at every size, because it is a ratio of the comuna: 1.6 cars for each public transport rider. The Metro part of the 78 is smaller still, and unknown.

Does it beat the station columns as a surprise? Yes for the MUT case: the column change is about 1% (invisible), the car number is a sentence the room repeats. No as a whole pitch: a Census ratio is not a simulation, and the decision `20261121-0100` requires that the change shows in the 3D scene. So C is not a separate pitch. **C is the surprise inside A's wow moment**: cars on the street, one-pixel columns, "el Metro ni se entera".

What the scene must show for C: from the new tower, about 124 small red cars drive onto Apoquindo and about 76 small figures walk to the station entrance, over 3 seconds, with the two large numbers beside them and the label "Censo 2024, Las Condes, supuesto: los vecinos nuevos se comportan como la comuna". The weak point to say out loud: residents 100 m from a station probably use the Metro more than the comuna average. We do not have that number. The label carries it.

What the engine must add for C: `resident_commuters_by_mode[]` in the site response (commuters × the 8 Census mode shares of the site comuna). Ten lines in `site.rs`. Without it, the fallback prints the numbers as data, not as an engine result.

## What "looks like the mock" means

- The scene opens at Tobalaba with real building footprints, the MUT towers and the Metro station. Nothing on screen is smaller than a thumb at 10 m from the projector.
- 37 columns, one per observed half hour, on Tobalaba; the 18:00 column reads 7,516 and is the highest. Null bins draw nothing.
- One zoom from Tobalaba to Gran Santiago in one camera move, with 52,646 block points sized by population.
- One control with one number (dwellings) and one button (Correr). A comuna selector only in case 2. No menus.
- After Correr, the result is visible from the back of the room: arcs that change thickness, cars on the street, or columns that rise. A one-pixel change gets a label that says it is one pixel.
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

1. **Which comuna does the story name for the MUT?** The site is in Las Condes (`13114`), the station is 26 m inside Providencia. Recommendation: say "Las Condes, al lado de Tobalaba" and use the Las Condes numbers (51.1% auto, 31.4% transporte público). Never mix the two comunas in one sentence.
2. **"Transporte público" or "Metro" on stage?** The Census gives only the combined share. Recommendation: say "transporte público" for every Census number; say "Metro" only for DTPM boardings (7,516, 81,613) and for the labelled assumption in the engine.
3. **Which surprise for the MUT case?** (a) station columns that move 1%, (b) the cars on Apoquindo and "el Metro ni se entera", (c) both. Recommendation: (c), Variant A with C inside. It is honest and it is the only version where the MUT result is visible from the back of the room.
4. **Second and third use case?** Options: Estación Central towers (data exists, engine exists today), stadium (needs the site route and shows scenario-only columns), MUT at another comuna, school flows at 08:00. Recommendation: Estación Central as case 2 in every version; stadium as case 3 only if the site route is wired; otherwise two cases and a stronger close.
5. **Variant A or B?** Recommendation: A. B needs a backtest that nobody owns and a 2017 layer that is not downloaded.
