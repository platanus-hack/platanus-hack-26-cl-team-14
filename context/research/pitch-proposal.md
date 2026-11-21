# Pitch proposal: "Santiago en un segundo"

- Status: Proposed by brave-gecko (claude-fable-5-1, subagent of gentle-fox), 2026-11-21 06:13 UTC. Updated 06:25 UTC after gentle-tapir's critic round 2b: schools 3,220, MUT jobs 7,124 modeled, buildings file exists, stadium moment has no observed stations, engine contract is PR #26. Humans accept, change, or reject it in `context/pitch.md`.
- Structure follows `context/pitch.md`. Spoken lines are in Chilean Spanish. All other text is simple English.
- Every number has the research file that holds it. `claim` marks a press or publisher number that nobody recomputed.

## Problem (one sentence)

Santiago is decided in pieces (a mall here, a tower there, a stadium somewhere) and nobody can see what each piece does to the 7.4 million people who live with the result.

## The numbers we can say on stage

| Number | What it is | File |
| --- | --- | --- |
| 7,400,741 | Persons with a home comuna in the RM, Census 2024 | `sources/od-flows-admin.md` |
| 52,646 | Census 2024 manzanas of the RM, each with persons and dwellings | `santiago-data-catalog.md`, `data/mvp/zones.csv` |
| 2,740,454 | RM residents who commute to a known comuna (9,128 pairs; 51,640 of them to a comuna outside the RM). The older 2,748,949 included 8,495 persons on region-only codes | brave-condor's CUT check in the thread, 2026-11-21 06:14 UTC; `data/mvp/work_od.csv` |
| 208,799 | Workers who enter Providencia from other comunas each day. Top origins: Santiago 25,134, Ñuñoa 17,455, Puente Alto 15,142, La Florida 13,414 | computed from `data/raw/censo2024-personas/od_work_rm.csv` (codes 2 and 3, regenerated 03:10 UTC). Region-only codes do not touch Providencia |
| 81,613 | Metro boardings at Tobalaba on an average weekday, April 13 to 17, 2026 (sum of 48 half hours) | computed from `data/raw/dtpm/subidas_2026_04.xlsx`, `Modo=Metro`, `Tipo_dia=LABORAL`, `Paradero=TOBALABA`. clever-heron got 81,612.8 independently: `data/mvp/tobalaba_observations.json` (PR #23), station `TB-TOB`, Saturday 46,461, Sunday 27,151 |
| 7,516 | Boardings at Tobalaba in the busiest half hour, 18:00 to 18:30. Next: 18:30 with 6,704 and 17:30 with 6,244 | same workbook |
| 2,469,425 and 2,512,170 | Weekday Metro and bus boardings in Santiago, publisher summary, `claim` | `santiago-transport-verified.md` |
| 298 | Average dwellings in an RM residential tower of 15 floors or more (361 permits, median 238, max 1,161, max 38 floors) | `sources/construction-pipeline.md` |
| 477 | Average dwellings in an Estación Central tower of 20 floors or more (57 permits) | `sources/construction-pipeline.md` |
| 2,156 | Persons in the Pila del Ganso manzana, Estación Central, Census 2017: 1,019 dwellings, about 2.1 persons per dwelling | `sources/construction-pipeline.md` |
| 3,220 | Operating RM schools with coordinates and enrolment, MINEDUC 2026 (3,227 operate; 7 have no coordinates) | `data/mvp/schools.csv`, `sources/verify-people-work.md` |
| 35,345 and 40,167 | Attendance at two U. de Chile home matches at Estadio Nacional, 2025; season average 29,564 | `sources/events-attendance.md` |
| about 48,000 | Capacity of Estadio Nacional, `claim` | `sources/events-attendance.md` |
| 53.5 million | Metro passengers in July 2026, INE press series, `claim` | `sources/score-targets.md` |
| 39.7% | Providencia commuters who go to work by public transport (bus or Metro); 33.7% car, 14.9% walking. Las Condes 31.4%, Santiago 61.7%. Not a Metro-entry share | `data/mvp/mode_share.csv`, `data/mvp/README.md` |
| 7,124 | Modeled jobs on the MUT manzana `13114131009033` (permit 4,643 + 2012 addresses 1,469 + landmark 1,012). Say "modelado". Press claims, not measured: 4,000 office workers, more than 200 shops, 45,765 m² of offices | `data/mvp/README.md`, section "The MUT" |
| 2,085 | OSM building footprints in the Tobalaba frame (bbox -70.6105,-33.4255 to -70.5915,-33.4105). Height from OSM tags, else 9 m flagged `h_src=default` | `web/data/buildings.geojson`, `web/osm_to_scene.py` |
| 1.75 s | 7 million destination draws over 866 zones, numpy, one core, toy test. Not a simulation time | `sources/prior-art-simulators.md` |

Numbers we do NOT have: measured MUT workers or visitors (two press claims on visits disagree: about 12,000 and about 21,000 per day), the Metro-entry share of Providencia residents (only the bus+Metro share exists), where stadium fans live, observed boardings at any Metro station other than Tobalaba in git (49 stops within 1 km of the MUT are in `web/data/run.json` as observations), and the real run time of the Rust engine. Do not say them until a file holds them.

## Variant A: "Las seis de la tarde en Tobalaba" (recommended)

Emotional core: the daily commuter. Every person in the room has stood on the Tobalaba platform at 18:00. The city decided that for them, and nobody simulated it. Now a juror decides, live, and sees the answer in a second.

| Time | What the audience sees | What must work behind it |
| --- | --- | --- |
| 0:00 | Dark screen. One line: "Son las seis de la tarde en Tobalaba." The 3D isometric scene of Tobalaba appears: real buildings, the station. A column grows on the station for each half hour of the day; at 18:00 it hits 7,516. Spoken: "Siete mil quinientas personas suben al metro aquí, en media hora, todas las tardes. Nadie lo decidió. Simplemente pasó." | Scene (browser or native, Juan permits both) with `web/data/buildings.geojson` (2,085 OSM footprints, exists, calm-puma) and the 48 weekday half hours from `data/mvp/tobalaba_observations.json` (merged, PR #23). Camera starts at street level and lifts. |
| 0:25 | Zoom out from Tobalaba to all Santiago. 52,646 manzanas light up with real people. Spoken: "Santiago se decide a pedazos. Un mall acá, una torre allá, un estadio donde salga más barato. Y nadie puede ver qué le pasa a los siete millones que viven las consecuencias." | `data/mvp/zones.csv` as a point layer: 52,646 rows, size by persons. One camera move. |
| 0:50 | Three large numbers appear over the city: 7,4 M personas, 2,74 M viajes al trabajo, 3.220 colegios. Thin flow lines appear from homes to work comunas. Spoken: "Hicimos un motor. Una persona por cada habitante, con su casa, su trabajo y su colegio. Datos reales: Censo 2024, DTPM, MINEDUC. Todo en Rust, para que Santiago entero corra en segundos." | `data/mvp/work_od.csv` flows drawn as arcs (9,128 pairs); `schools.csv` only if the morning (08:00) flows are shown. The Rust engine (`engine/`, HTTP on :8787) is loaded with the pack. |
| 1:10 | **Wow moment.** Camera back to Tobalaba. The MUT block turns red. A large control: "Sacar el MUT (7.124 empleos, modelado). Poner una torre de [298] viviendas." Spoken: "Saquemos el MUT. Pongamos una torre. Doscientas noventa y ocho viviendas, el promedio real de una torre de quince pisos en Santiago." Press "Correr". A timer shows the measured time. The columns at Tobalaba morph from observed to scenario; new flow lines leave the tower to the comunas where the new residents work. Spoken: "Esto no es un video. Díganme un número." A juror says a number. The presenter types it. Run again. | Rust `POST /api/site-scenario`, the contract merged in PR #26 (`context/research/tobalaba-scenario-contract.md`): `scenario[t] = observed[t] − site_base[t] + site_scenario[t]`. Remove 7,124 modeled jobs, add N dwellings, residents' work destinations from the Providencia row of `work_od.csv`, workers' return departures in the evening, home departures in the morning, nearest Metro entrance → parent TB-TOB. Every parameter is explicit and labelled (occupied share, persons per dwelling, Metro-entry share, access share, time windows); the presenter enters 298 as a user assumption. **Owner of the Rust implementation: none yet.** v1 (`/api/simulate`, comuna multipliers) cannot move a station column. Measured run time under 2 s on the laptop. The scene redraws from the response without reload. |
| 1:50 | Case 2. The camera flies west to Estación Central. Spoken: "Esto ya pasó una vez. En Estación Central se aprobaron cincuenta y siete torres de más de veinte pisos, cuatrocientas setenta y siete viviendas cada una, en promedio. Nadie lo simuló." The control now says "57 torres de 477 viviendas en [comuna]". The presenter picks another comuna. Run. Flows and station columns change there. | Same engine call, other point, other N. Comuna selector with 4 options. `buildings.geojson` covers only Tobalaba; the other comuna shows manzana points and the new towers as simple boxes. |
| 2:20 | Case 3. Spoken: "Y la pregunta que se hace la U hace diez años: ¿dónde ponemos el estadio?" A stadium marker: 40,000 persons one night. Drag it from Ñuñoa to another comuna. Run. Night columns rise at the stations next to the new place. A label says "origen de hinchas: supuesto, proporcional a población". | Same `site-scenario` call with an event term: 40,000 visitors **exit** after the match (exits generate the local boardings, not arrivals, per the PR #26 contract), explicit attendance, Metro and access shares, exit profile 21:00 to 23:00. Observed boardings exist in git only for Tobalaba and 49 stops near the MUT, so this moment shows scenario-only columns labelled "supuesto" at the stations near the stadium, or needs a compact all-Metro-stations layer from clever-heron in the format of `tobalaba_observations.json`. |
| 2:45 | Camera lifts over all Santiago. Spoken: "Un motor. Tres preguntas. La próxima vez que alguien decida un pedazo de Santiago, que lo vea primero." Team name and one line. | Nothing new. |

## Variant B: "El error de Estación Central"

Emotional core: regret and a second chance. We already built the vertical ghettos without a simulation. The engine, run on 2017 data, shows that it would have seen 2024 coming.

| Time | What the audience sees | What must work behind it |
| --- | --- | --- |
| 0:00 | Estación Central in 3D, 2017. Low buildings. Spoken: "Año 2017, Estación Central. Un barrio de casas de dos pisos." Towers rise one by one, 57 of them, 477 dwellings each. "En siete años, cincuenta y siete torres. Nadie preguntó qué pasaría." | Census 2017 manzana layer and INE permits 2010 to 2018 for Estación Central (both in `construction-pipeline.md`), animated by permit year. |
| 0:40 | **Wow moment.** Spoken: "Le dimos al motor los datos de 2017 y las torres. Y le preguntamos qué pasaría en 2024." Two columns per manzana: engine 2017 plus towers, and Census 2024 real. They match within a visible margin. "El motor lo vio. Siete años antes." | Engine backtest: 2017 population plus permitted dwellings times persons per dwelling, compared with Census 2024 manzana counts. Score on screen. Nobody owns this today. |
| 1:30 | Case 2: MUT at Tobalaba, as in Variant A at 1:10, shorter. | Same as A. |
| 2:15 | Case 3: stadium, as in Variant A. | Same as A. |
| 2:45 | Close: "Esta vez, miremos antes." | Nothing new. |

### Why I recommend A

- A is buildable tonight with what exists: `zones.csv`, the work matrix, the DTPM boardings, the buildings near Tobalaba. B needs a backtest against Census 2024 manzanas that nobody has started, and a Census 2017 layer that is catalogued but not downloaded.
- A puts the surprise in the hands of a juror. B puts it in a chart that the jury must trust. Hackathon judges reward the moment they understand at once (see findings below).
- A's hook (18:00 at Tobalaba) is a feeling that every person in Santiago has. B's hook is a political topic that can split the room.
- B's story is stronger if the backtest works. If the humans want B, the engine owner must start the backtest at once, and A is the fallback.

## The wow moment

A juror says a number. The presenter types it. Seven million people re-route, and the Tobalaba columns change shape in under two seconds, live, on a projector. The surprise is not the result. The surprise is that the city answers at once.

## What "looks like the mock" means

- The scene opens at Tobalaba with real building footprints and the Metro station. Nothing on screen is smaller than a thumb at 10 m from the projector.
- 48 columns, one per half hour, on Tobalaba; the 18:00 column reads 7,516 and is the highest.
- One zoom from Tobalaba to all Santiago in one camera move, with 52,646 manzana points sized by persons.
- One control with one number (dwellings) and one button (Correr). No menus.
- After Correr, the columns show two colors (observed, scenario) and the measured run time, and new flow lines leave the tower. The change is visible from the back of the room.
- The same control works for three points: Tobalaba, a comuna for towers, a stadium. Nothing else is clickable.
- Every assumed number on screen carries the word "supuesto". Every real number carries its year.

## The exact interaction and output the pitch needs (for the engine owner)

One call, three uses. The presenter changes only the numbers in bold.

The contract is `context/research/tobalaba-scenario-contract.md` (PR #26, merged). This section says only what the presenter touches. Input: a point (`lon`, `lat`), **`jobs_removed`** (MUT case: 7,124 modeled), **`dwellings_added`** (tower cases: the presenter types 298; the contract allows no hidden default), **`visitors`** and an exit window (stadium case: 40,000, 21:00 to 23:00), and a comuna selector that moves the point.

Output, per the contract: `observed[48]` (April 2026, never changes; absent bins are null, not zero), `base[48]`, `scenario[48]`, `site_base[48]`, `site_scenario[48]`, `flows[]` from the point to work comunas with expected commuters, `elapsed_ms`, and `assumptions[]` as short strings that the scene prints with the word "supuesto".

What the scene does with it: columns per half hour in two colors (observed, scenario), arcs for the flows, the timer, and the assumptions list in a corner. The scene never computes; it draws the response.

The comuna-scale v1 (`jobs_multiplier`, `housing_multiplier`) is the integration milestone. It cannot change a station column, so it is not the demo.

## What the pitch does NOT need (do not build it)

- Migration flows (`od_migration_rm.csv`): no moment shows them.
- DTPM trips and stages archives: the pitch shows boardings only.
- Saturday and Sunday boardings: all moments are a weekday.
- A comuna-level multiplier API: the wow moment needs a point (the MUT) and stop columns, not a comuna multiplier. If the engine stays at comuna scale, the Tobalaba columns cannot change and the wow moment is lost.
- Buildings outside the Tobalaba camera frame.
- Schools: only if the 0:50 moment shows the 08:00 flows. If the humans cut the word "colegio", `schools.csv` has no moment.

## Research: what wins a 3-minute hackathon pitch (quick level, 3 searches)

1. **One moment the judges understand at once beats a tour of features.** A JetBrains judge writes that a strong project with a confusing demo loses to a simpler project that the judges understand, and that the pitch needs one clear "this is possible now" contrast between the old frustration and the new version. https://blog.jetbrains.com/ai/2026/06/how-to-win-a-hackathon-notes-from-the-judging-table/
2. **Open with a frustration the room has felt, then build everything toward the moment the product resolves it.** Setup before, payoff after. Teams with better presentations regularly beat teams with stronger tech. https://musichackspace.org/blog/hackathon-presentation-tips and https://intercom.help/builderbase/en/articles/14836128-builder-s-guide-ch-14-crafting-a-demo-that-lands
3. **Live demos need a recorded fallback, and the pitch must lead with the problem, not the stack.** Judges care about the product; spend 15 to 20 seconds on who is affected, keep a recording ready, rehearse with people like the judges. For Platanus specifically, DF quotes Platanus on the first edition: the product "más que un negocio, debe ser una solución a un problema". https://developer.salesforce.com/blogs/2014/07/deliver-winning-pitch-hackathons , https://blogs.reskilll.com/hackathon-demo-presentation-tips-pitch-3-minutes-win-2026/ , https://www.df.cl/df-mas/punto-de-partida/36-horas-de-programacion-nonstop-asi-fue-la-primera-platanus-hack

Applied here: the hook is a feeling (18:00 at Tobalaba), the wow is one contrast (observed vs scenario) that a juror controls, and we record a 3-minute video of the final run as a fallback.

## The one question for the humans

**¿Variante A (el viajero de las seis de la tarde, construible esta noche) o Variante B (el error de Estación Central, más fuerte pero necesita un backtest que nadie ha empezado)?**
