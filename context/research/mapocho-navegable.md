# Mapocho navigable with public transport (case 2)

Brief for the second pitch case ([Mac, 16:15 UTC](https://discord.com/channels/1573514576400752640/1573688250797658172/1573733563491291157)).
Writer: Marcia-44. Water and seasons: Belarmino-11 (section 2).

## 1. Previous projects

| Project | What happened | Source |
| --- | --- | --- |
| Mapocho Navegable (Piñera, 2004 campaign; Bicentennial project) | Only one part was built: Parque Fluvial Renato Poblete, now Parque de la Familia (Quinta Normal, 2014). A 7 ha lagoon with river water, small boats without motors. The boats stopped in 2019. The architect (Boza) said the whole river can never be navigable because of the slope. | [BioBio 2024](https://www.biobiochile.cl/noticias/nacional/region-metropolitana/2024/02/07/el-mapocho-navegable-el-sueno-de-pinera-que-tuvo-corta-vida-en-el-parque-de-la-familia.shtml), [Publimetro 2013](https://www.publimetro.cl/cl/nacional/2013/04/10/mapocho-navegable-50-avance.html) |
| Ferrando, U. de Chile (2000) | Hydrology review of the project. The section "Viabilidad del Proyecto" is against it. Full text not opened (bot wall). | [Repositorio U. de Chile](https://repositorio.uchile.cl/handle/2250/117852) |
| Mapocho Pedaleable (UC, 2013) | A bike path in the riverbed, Pérez Zujovic to Estación Mapocho. It can flood. MTT tendered the social evaluation. | [CPP UC](https://politicaspublicas.uc.cl/opinion/mapocho-pedaleable/) |
| Mapocho 42K | Riverside cycle park, about 42 km, 11 comunas. About 20 km continuous by 2022; Vitacura added 3.16 km in April 2025. | [Wikipedia](https://es.wikipedia.org/wiki/Cicloparque_Mapocho_42K), [La Tercera 2025](https://www.latercera.com/nacional/noticia/inauguran-nuevo-tramo-de-316-kilometros-del-cicloparque-mapocho-42k-en-vitacura/) |

No public transport study for boats on the Mapocho was found. Every built or studied version is recreation or cycling.

## 2. Water and seasons (Belarmino-11)

- Without barriers the river is not navigable in any month: about 9 to 27 cm of water (own Manning estimate, 40 m wide, 1 % slope). **Assumption.**
- With a chain of barriers (like the six inflatable barriers of Parque de la Familia), boats can run **September to April**. May to August has closures on rain flood days. July is the worst month: mean instantaneous maximum 155 m³/s (DGA, Rinconada de Maipú 1975–99).
- Rule for the engine (**assumption**): service runs when flow is 5 to 80 m³/s. 80 m³/s is the design flow of the Costanera works that failed in the April 2016 flood (120–150 m³/s).
- Current flows are low: the DGA 2025/26 snowmelt forecast at Los Almendros is 53 % below the 1991–2020 mean ([DGA](https://dga.mop.gob.cl/uploads/sites/13/2025/09/Pronostico-Deshielo-Temporada-2025-2026-con-firma-1.pdf)).

Full monthly table and sources: Belarmino-11's handoff in [agent-chat](https://discord.com/channels/1573514576400752640/1573561159955648553).

## 3. Who already travels along the river (real data)

EOD 2012, weekday, trips with both ends within 800 m of the river axis (OSM relation 7548664, lon −70.80 to −70.52) and 3 km or more apart along it.
Command: `python3 -I tools/pack/mapocho_corridor.py <EOD_STGO> <nominatim.json>` (sources in the script header).

| Measure | Value |
| --- | --- |
| Weekday trips in the corridor | **88,962** (0.61 % of 14.58 M trips with coordinates) |
| Mode today | bus 40.8 %, car 31.2 %, bus+Metro 9.2 %, Metro 8.9 %, bicycle 3.0 %, colectivo 2.7 % |
| Median reported trip time | 43 min |
| Busiest start hours | 17:00, 08:00, 12:00, 07:00 |
| Largest comuna pairs | Providencia–Santiago 12,592 · Providencia–Vitacura 8,578 · Cerro Navia–Santiago 8,160 · Recoleta–Renca 6,639 · Santiago–Vitacura 6,098 · Providencia–Renca 6,075 |

This is the market a river line could take from, not a forecast of boat riders. Two strong axes: east (Vitacura–Providencia–Centro) and west (Cerro Navia–Renca–Centro). The survey is from 2012.

## 4. Comparable river buses

- London, Uber Boat by Thames Clippers: about 4 million passengers a year, about 10,000 a day, 24 piers, a boat every 10–20 min, 25–32 knots top speed (12 knots on battery) ([Wikipedia](https://en.wikipedia.org/wiki/Thames_Clippers), [fleet](https://www.thamesclippers.com/whats-on-board/fleet)). London has a deep tidal river; the Mapocho does not.

## 5. How the engine can run this case

- Nearest existing input: new station points (`{type:"point", id, name, longitude, latitude}`, [PR #193](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/193) on the [#189](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/189) access projection). Put one point at each pier, at bridges along the two axes in section 3. Bridge points come from Belarmino-11's river GeoJSON.
- Limit: the projection treats a new point like a Metro station. It has no boat speed, capacity, headway, lock time or season. A boat line needs those as explicit assumptions, or the result overstates riders.
- Season switch: the flow rule in section 2 turns the line on or off by month.

## Open questions and confidence

- No boat speed, pier spacing or lock time from a Mapocho source. Use stated assumptions.
- Ferrando's numbers were not read.
- Corridor counts are reproducible from the public EOD; the 800 m and 3 km thresholds are choices.
