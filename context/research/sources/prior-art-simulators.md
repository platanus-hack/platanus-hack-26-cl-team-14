# Prior art: simulators and synthetic populations of Santiago

Owner: calm-otter (claude-fable-5-2). Date of all checks: 2026-11-21.

## Verdict

- **One open agent simulation of Santiago exists: the MATSim Santiago scenario (2016).** We have its files: 665,201 agents with full day plans and coordinates, a road network, a transit schedule, and 36 count stations. All of it is from 2012-2013.
- **There is no real synthetic population of Santiago.** MATSim Santiago makes clones of the EOD 2012 respondents. No public project fits a population to the Census (IPF or equal). No eqasim Santiago exists. No SUMO Santiago scenario exists.
- **The government models are closed.** ESTRAUS and MUSSA have only empty library records. DTPM publishes its transit model, but in a binary EMME format that needs a paid licence.
- **No prior art models housing, work, and leisure together.** Each model is transport only (MATSim, ESTRAUS, DTPM) or land use only (MUSSA). This is our space.

## Can 7 million agents run again in seconds?

**Yes, if each agent makes choices from zone tables. No, if we simulate vehicles in queues.**

| Evidence | Result | Check |
| --- | --- | --- |
| Our test: 7,000,000 agents, one logit destination choice each, 866 zones (the EOD zone count), numpy, one core, Apple M3 | **1.75 s** | Measured here. The cost matrix is random numbers, not real data. |
| MATSim Santiago v2b: 10% sample, 100 iterations, 6 threads, 8 GB heap | Run time not stated | Config file read. We did not run it. |
| LPSim (GPU vehicle queues): 2.82 million trips | 6.28 minutes on 1 GPU; 9.01 million trips in 21.16 minutes on 2 GPUs | Abstract opened (arXiv 2406.08496). Self-reported. |
| MOSS (GPU microscopic traffic): 2.46 million vehicles | 84 steps per second. One day of 1 s steps is about 17 minutes. | UNVERIFIED: from a search summary. The abstract that we opened does not give the number. |
| MATSim Open Berlin 10%: one iteration | more than 4 hours | UNVERIFIED: from a search summary of a MUM 2025 abstract. |

What this means for the engine:

- Keep 7 million agents as arrays (home zone, work zone, income group, age group). Memory is not a problem: 4 int32 columns are 112 MB.
- Calculate choice probabilities for each (group, zone) pair, not for each agent. Then draw for each agent. This is the 1.75 s path.
- Do not simulate each vehicle. Use link or station loads from sums of agent choices. An equilibrium loop (MATSim does 100 to 200 iterations) makes "seconds" impossible.
- For "what happens at Tobalaba station": draw again only the agents that the change affects, then sum loads.

**Samples that other teams use:** MATSim Santiago v1 0.65% (42,459 agents), v2 10% (665,201). eqasim São Paulo 1%. MATSim Berlin 1% and 10% (search summary). Nobody runs 100% of a city of this size with vehicle queues on a laptop. A 100% run is only practical with zone-table choices.

## Sources

### MATSim Santiago open scenario v2b (data files)
- Link: https://svn.vsp.tu-berlin.de/repos/public-svn/matsim/scenarios/countries/cl/santiago/ (files `v1.zip`, `v2a.zip`, `v2b.zip`, and one readme PDF for each)
- Contents: `randomized_expanded_plans.xml.gz`: one row is one agent with a full day plan. Each activity has a type with duration (`home16.0H`, `shop0.5H`), x and y in EPSG:32719, and start and end times. Each leg has a mode. `expandedAgentAttributes.xml`: car availability for each agent. `network_merged_cl.xml.gz`: 37,834 links and 23,330 nodes (OSM roads plus a separate transit network). `transitschedule_simplified.xml`: 371 lines and 26,124 stops. `counts_merged_VEH_C01.xml`: 36 count stations, hourly car volumes. `gantries.xml`: toll gantries.
- Granularity: person and point. Time step: second.
- Year: demand July 2012 to November 2013 (EOD 2012, working days). GTFS of 2015. The counts file says `year="2013"`, name `BD_FLUJOS_CONTINUAS-14Oct13`. The paper says July 2011. One of them is wrong.
- Access: direct download. `v2b.zip` 125,607,974 bytes (170 MB open, 147 MB are Java jars). Script: `tools/scrape/matsim_santiago.py`.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 125,607,974 bytes. Counted 665,201 `<person>` rows. Legs: walk 527,377; bus 497,296; car 290,251; metro 234,098; other 155,061; ride 132,142; colectivo 64,440; bike 60,879; taxi 29,218; train 636. `v1.zip` (54,811,644 bytes) and `v2a.zip` (85,924,440 bytes): VERIFIED-BYTES, HEAD 200 only.
- Role: INPUT (plans, network) and TARGET (36 count stations)
- Limits: The agents are clones of 42,459 EOD respondents (IDs `10002001`, `10002001_1`, ...). The number of clones follows the EOD expansion factors (Contreras 2015). Clone locations are random points inside SII land use. Thus the population has no new households and no income, age, or sex columns in these files. It is before Metro lines 3 and 6 and before the pandemic. Working days only. No licence file: the paper says "no data restrictions" and asks for a citation. The EOD link in the readme is dead (404).
- Use in the simulation: INPUT. Fast start for day plans (activity chains, durations, modes) at point level. TARGET: the 36 count stations, but they are 13 years old. Do not use it as the population. Build the population from Census 2024 and use these plans only as activity-chain templates.

### matsim-scenarios/matsim-santiago (code)
- Link: https://github.com/matsim-scenarios/matsim-santiago
- Contents: Java code that makes the scenario. `prepare/population/` (EOD CSV to plans, cloning, `RandomizeEndTimes`), `prepare/landuse/` (`BuildingsByBlock`, `RandomizeCoordinates`: SII land use to activity points), `prepare/counts/`, `prepare/gtfs/`, `prepare/network/` (tolls), `colectivos/`, `avcongestion/`.
- Granularity: not data.
- Year: created 2019-10-03, last push 2024-05-02. Default branch `santiago-v2b_matsim-v13.0`.
- Access: direct download (git). 283 KB, 199 files.
- Check: VERIFIED-ROWS 2026-11-21, GitHub API HTTP 200, tree of 199 files listed. Licence GPL-2.0. The README is the MATSim example template. It says nothing about Santiago.
- Role: INPUT (method only)
- Limits: The old path in the paper and the readme (`matsim-org/matsim/playgrounds/santiago`) is dead (API 404). GPL-2.0: if we copy code, our code must be GPL. Java, MATSim 13.
- Use in the simulation: INPUT as a recipe. Read `RandomizeCoordinates` and `ActivityClassifier` before we write our own EOD reader. Do not run MATSim.

### Kickhöfer, Hosse, Turner, Tirachini (2016), VSP Working Paper 16-02
- Link: https://svn.vsp.tu-berlin.de/repos/public-svn/publications/vspwp/2016/16-02/KickhoeferEtAl2016MatsimSantiago.pdf
- Contents: the method of scenario v1. Inputs: (1) EOD 2012: 18,000 households, 60,054 persons, 113,591 trips, 866 zones, 45 comunas; expanded totals 2,051,310 households and 6,651,654 persons. (2) OSM roads, down to secondary (tertiary inside Gran Santiago), free speed cut by 0.50 or 0.75 on small streets. (3) DTPM GTFS, 2015. Behaviour parameters from Munizaga et al. 2008. Car cost 248 CLP/km. Fares 640, 660, 720 CLP.
- Granularity: not data.
- Year: 2016 (v1). v2a and v2b have no paper, only a readme of one page.
- Access: direct download. PDF, about 6.2 MB.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, pages 1 to 20 read.
- Role: INPUT (method only)
- Limits: Calibration is weak. They adjusted 3 mode constants (car +1.100, PT -0.883, walk 0) until the mode shares equalled the shares of the input population itself. Validation: 36 of 49 count stations, 15-minute bins; the simulation has too few cars at most stations. Cars and buses run on separate networks, thus buses do not get congestion. Only car, PT, and walk can change; the other modes are fixed. The paper gives no run time and no hardware. v1 used 42,459 agents (0.65%) with flow capacity 0.0064.
- Use in the simulation: INPUT as method and as a warning. Their score (mode share that equals the input) is circular. Our score must use data that the model did not see.

### DTPM transit assignment model, March 2020 (EMME)
- Link: https://www.dtpm.cl/index.php/documentos/modelos-de-demanda
- Contents: the real model that DTPM uses for buses, Metro, and Metrotren. Three periods: morning peak (PMA), off-peak (FPU), evening peak (PTA). One hour of demand each. Supply: operating programme of March 2020. Demand: bip! transactions of one working week of August 2019, adjusted to passenger counts on buses and Metro of September and October 2019.
- Granularity: zone (OD matrix), link and line (network). One hour for each period.
- Year: 2019 demand, 2020 supply. An older model (July 2018 supply, August 2017 demand) is on the same page as `.rar` files on `dtpm.gob.cl`; we did not open them.
- Access: direct download. `Modelo_PMA_DTPM.zip` 499,450,085 bytes; `Modelo_FPU_DTPM.zip` 68,345,154 bytes; `Modelo_PTA_DTPM.zip` 258,825,867 bytes; description PDF 114,875 bytes.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200. The model records are not readable. FPU archive: 40 files, 190 MB open. PDF read (2 pages). But the network and the matrices are in `emmebank.zip` and `emmemat.zip`, the binary format of EMME. The SQLite file `Database/13.db` has tables `nodes` and `links` with 0 rows. `data_tables.db` has only empty SpatiaLite tables. Readable text: the fare macro (bus 700, Metro 720, values in CLP) and the transfer penalties for Metro nodes.
- Role: INPUT, not usable now
- Limits: EMME is paid software (Bentley, before INRO). We found no open reader and did not search hard for one. DTPM says the east zone (Lo Barnechea, Vitacura, Las Condes, La Reina, Peñalolén) has almost no bus counts, and the evening peak can be too low there because of fare evasion. Before the pandemic. No licence text.
- Use in the simulation: NONE now. It becomes an INPUT (calibrated transit OD matrix for 3 periods of 2019) only if a person opens it with EMME and exports text. The DTPM trip tables give the same information without EMME.

### ADATRAP trip and stage tables (DTPM)
- Link: https://www.dtpm.cl/index.php/documentos/matrices-de-viaje
- Contents: output of ADATRAP, the software that estimates where each bip! card user gets off. One row is one trip (or one stage) with estimated boarding and alighting stop. Also boardings for each stop and half hour, and the zone file `Zona777_ADATRAP.rar`.
- Granularity: person-trip, stop, zone (777 zones). One week for each release.
- Year: 16 releases on the page, from March 2020 to April 2026. Daily estimates from April 2017.
- Access: direct download. ZIP of CSV. `Zona777_ADATRAP.rar` is 1,660,725 bytes.
- Check: VERIFIED-BYTES 2026-11-21. Page HTTP 200, 40 file links read. Zone file: HEAD 200 only. We did not download the trip files. Agent clever-koala does that in `context/research/santiago-transport-verified.md`.
- Role: TARGET (boardings) and INPUT (transit OD)
- Limits: DTPM says the tables do not include fare evasion and that the method changes between versions. The alighting stop is an estimate, not a measurement. Only public transport.
- Use in the simulation: TARGET (boardings for each station and half hour) and INPUT (transit OD). This is the modern replacement for the 2013 counts in MATSim Santiago.

### ADATRAP code (GitHub organisation SmartcitySantiagoChile)
- Link: https://github.com/SmartcitySantiagoChile
- Contents: 16 repositories. Visualisation server (`fondefVizServer`), upload scripts, validators. Last pushes 2017 to 2024.
- Granularity: not data.
- Year: 2017 to 2024.
- Access: direct download (git).
- Check: VERIFIED-ROWS 2026-11-21, GitHub API HTTP 200, 16 repositories listed. Most have no licence.
- Role: none
- Limits: The engine that estimates the alighting stop is not in the organisation. Only tools around it are public.
- Use in the simulation: NONE.

### EOD 2012 Santiago (SECTRA): the base of all the models
- Link: https://www.sectra.gob.cl/info-territorial/region-metropolitana-de-santiago/
- Contents: household travel survey. Tables for households, persons, trips, and stages. See the paper entry above for the totals.
- Granularity: person and trip, with coordinates. 866 zones.
- Year: July 2012 to November 2013.
- Access: 3 Google Drive links on the page.
- Check: VERIFIED-BYTES 2026-11-21. Page HTTP 200, 3 Drive links found. We did not download the files. Another agent has `tools/scrape/eod2012.py`. The old links are dead: `sectra.gob.cl/biblioteca/detalle1.asp?mfn=3253` and `sectra.gob.cl/encuestas_movilidad/encuestas_movilidad.htm` both return 404.
- Role: INPUT
- Limits: 14 years old. MATSim Santiago, ESTRAUS, and the academic models all use this one survey. Thus they have the same errors. No newer EOD of Santiago was found in this research.
- Use in the simulation: INPUT for activity chains and mode choice. Not a TARGET: the prior models already fit it.

### ESTRAUS (SECTRA strategic transport model)
- Link: https://www.sectra.gob.cl/documento/actualizacion-del-modelo-estraus-con-informacion-de-la-eod-2012/
- Contents: the 4-step equilibrium model that the State uses to evaluate Metro lines and highways. The SECTRA library has 8 or more records about it (update with EOD 2012, evening peak, parking, bus corridors).
- Granularity: zone. From search summaries, not opened: 264 zones in the first version and 410 zones in a later one.
- Year: record dated 2024-11-25 (date of the web migration, not of the study).
- Access: form or request. No file.
- Check: UNVERIFIED. The record returns HTTP 200, but the body is empty: no text, no file link (checked through the site API, field `content` is empty). The same is true for the EOD 2012 record and the MUSSA records.
- Role: none
- Limits: closed model, closed networks, closed matrices. The software is the property of the State.
- Use in the simulation: NONE.

### MUSSA and Cube Land (land use model, Martínez and Donoso)
- Link: https://www.sectra.gob.cl/guias-manuales/analisis-de-equilibrio-economico-en-el-mercado-inmobiliario-mussa/mussa-analisis-del-equilibrio-economico-en-el-mercado-inmobiliario/
- Contents: bid-rent equilibrium model of where households and firms locate in Santiago. From search summaries, not opened: 65 household groups, 264 zones, 6 dwelling types, 5 firm types.
- Granularity: zone.
- Year: 2002 to 2010.
- Access: paid. Cube Land is commercial software (Bentley). The SECTRA page is a guide, with no data.
- Check: UNVERIFIED. Page HTTP 200, no data files. The record `analisis-y-desarrollo-de-bases-de-datos-mussa` is empty.
- Role: INPUT (method only)
- Limits: no public inputs, no public calibration data.
- Use in the simulation: NONE as data. The method is the best prior art for the housing mechanism: each dwelling goes to the household group with the highest bid. Use it for "put a vertical ghetto here: who moves in?".

### eqasim São Paulo (the template for a real synthetic population)
- Link: https://github.com/eqasim-org/eqasim-sao-paulo
- Contents: open pipeline that makes a synthetic population from a census, a household travel survey, and OSM. It makes households and persons with attributes, then gives each person an activity chain from a similar survey person (statistical matching), then gives locations.
- Granularity: person and point.
- Year: last push 2023-07-06. Paper: Sallard, Balac, Hörl 2021.
- Access: direct download (git). Ready scenario files are on ETH polybox (not opened).
- Check: VERIFIED-ROWS 2026-11-21, GitHub API HTTP 200, README read. Licence GPL-2.0. The published sample is 1%.
- Role: INPUT (method only)
- Limits: it is for Brazil. No Santiago or Chile version exists: GitHub search `eqasim chile` gives 0 results, and the eqasim organisation has no Chile repository (search summary).
- Use in the simulation: INPUT as method. The 3 steps (census persons, then survey chains by matching, then locations) are the correct replacement for the MATSim cloning. Write ours in Python; do not adapt the Java and Python pipeline.

### SUMO scenario of Santiago
- Link: https://api.github.com/search/repositories?q=santiago+sumo
- Check: UNVERIFIED (negative result) 2026-11-21. 1 result, not related. Web search also found none.
- Use in the simulation: NONE. The hint was wrong.

### Synthetic population of Santiago from EOD plus Census (IPF or equal)
- Link: https://api.github.com/search/repositories?q=santiago+synthetic+population
- Check: UNVERIFIED (negative result) 2026-11-21. 0 results. Two web searches (one extended) found only the MATSim cloning. arXiv 2610.21784 (Spain, France, Italy) and arXiv 2608.12768 (United States) do not cover Chile; abstracts opened.
- Use in the simulation: NONE. We must build it. This is also a pitch point: it does not exist.

### Agent simulator of COVID-19 in Santiago (arXiv 2005.09556)
- Link: https://arxiv.org/abs/2005.09556
- Contents: an agent simulator that makes infection data for each district. From a search summary: 40 districts, census densities, commuting flows.
- Granularity: comuna.
- Year: 2020.
- Access: paper only. No code link found.
- Check: UNVERIFIED. Abstract opened, HTTP 200. No data and no code found.
- Role: none
- Use in the simulation: NONE.

### Optimizing city-scale traffic (arXiv 1906.05093)
- Link: https://arxiv.org/abs/1906.05093
- Check: UNVERIFIED. Abstract opened, HTTP 200. From a search summary: it uses a Santiago network with about 665 thousand vehicles. That number equals the MATSim v2b agent count, thus it is the same scenario and not new data.
- Use in the simulation: NONE.

### Metro de Santiago demand model
- Link: none found.
- Check: UNVERIFIED. One web search found only ESTRAUS reports and a 2026 SECTRA study of new lines. No public Metro model, no files.
- Use in the simulation: NONE. The DTPM trip tables are the public replacement.

### LPSim (speed evidence)
- Link: https://arxiv.org/abs/2406.08496
- Contents: GPU traffic simulator, San Francisco Bay Area. 2.82 million trips in 6.28 minutes on 1 GPU. 9.01 million trips in 21.16 minutes on 2 GPUs. About 113 times faster than CPU methods on 2 A100 GPUs.
- Check: VERIFIED-BYTES 2026-11-21. Abstract opened, HTTP 200. Self-reported numbers. Code not opened.
- Role: none (speed evidence)
- Use in the simulation: evidence only. Vehicle-level simulation of a city is minutes on a GPU, not seconds.

### MOSS (speed evidence)
- Link: https://github.com/tsinghua-fib-lab/moss
- Contents: GPU microscopic traffic simulator. Licence MIT. Last push 2025-08-26.
- Check: VERIFIED-BYTES 2026-11-21, GitHub API HTTP 200. Speed numbers (2.46 million vehicles at 84 steps per second) UNVERIFIED: from a search summary of arXiv 2406.10661.
- Role: none (speed evidence)
- Use in the simulation: evidence only. Needs an NVIDIA GPU; our laptops are Apple.

### AgentTorch (large population models)
- Link: https://github.com/AgentTorch/AgentTorch
- Contents: PyTorch framework for agent models with millions of agents. Licence AGPL-3.0. Last push 2026-11-15. 658 stars.
- Check: VERIFIED-BYTES 2026-11-21, GitHub API HTTP 200. The hint "8.4 million agents of New York" is UNVERIFIED: the README text that we read does not give the number.
- Role: none
- Limits: AGPL-3.0. A new dependency.
- Use in the simulation: NONE. numpy does the 7 million draws in 1.75 s. We do not need a framework.

## Hints that were wrong

- "MATSim Santiago is in matsim-org on GitHub": the playground path is dead. The code is in `matsim-scenarios/matsim-santiago`. The data is on the TU Berlin SVN.
- "eqasim" for Santiago: does not exist.
- "SUMO scenarios" for Santiago: none found.
- "Papers that build a synthetic population of Santiago from EOD and Census": none found. MATSim uses EOD only, with clones.
- "ADATRAP" is not a simulator. It is an estimator of alighting stops. Its output is public; its engine is not.

## Critique

1. **The speed test is too easy.** It measures one destination draw for each agent with a random cost matrix. It has no chain of activities, no mode choice, no capacity feedback, and no real data. A real step can cost 10 to 50 times more. "Seconds" is probable but not proved.
2. **Half of the speed table is not verified.** The MOSS, MATSim Berlin, and AgentTorch numbers come from search summaries or from memory. We did not run MATSim Santiago, thus we have no run time for the only Santiago simulator.
3. **The closed models are documented from summaries.** For ESTRAUS and MUSSA, the zone counts and group counts come from search summaries of MOP reports that we did not open. We did not ask SECTRA for the files (a request under the transparency law is possible but slow).

Not checked: the content of `v1.zip` and `v2a.zip`; the DTPM PMA and PTA archives; the 2018 DTPM model; if an open reader for `emmebank` exists; the EOD Drive files; theses of U. de Chile and PUC that used MATSim Santiago after 2016; Fosco and Zurita 2021 (PLoS ONE, lockdown agent model); models inside private firms (Citymovil, Steer, Telefónica).
