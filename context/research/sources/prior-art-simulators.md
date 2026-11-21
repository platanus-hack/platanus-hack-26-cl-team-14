# Prior art: simulators and synthetic populations of Santiago

Owner: calm-otter (claude-fable-5-2). Date of all checks: 2026-11-21.

## Verdict

These are search results with limits. An empty search does not prove that a thing does not exist.

- **We found one open agent simulation of Santiago: the MATSim Santiago scenario (2016).** We have its files: 665,201 agents with full day plans and coordinates, a road network, a transit schedule, and 36 count stations. All of it is from 2012-2013.
- **We found no public synthetic population of Santiago that is fitted to the Census.** MATSim Santiago makes clones of the EOD 2012 respondents. Searches done: 5 GitHub searches and 3 web searches (list in the entries). We did not search theses or private firms.
- **We could not get the government models.** ESTRAUS and MUSSA have only empty library records. DTPM publishes its transit model: we read its OD matrix, but not its network.
- **Each model that we found is transport only (MATSim, ESTRAUS, DTPM) or land use only (MUSSA).** We found none that puts housing, work, and leisure together.

## Can 7 million agents run again in seconds?

**Not proved.** We have one toy measurement and some numbers from other simulators.

| Evidence | Result | Check |
| --- | --- | --- |
| Toy test here: 7,000,000 agents, one logit destination draw each, 866 zones, numpy, one core, Apple M3 | 1.75 s | Measured by one agent, one run. Random cost matrix. No activity chain, no mode choice, no capacity. It is a lower limit, not a simulation time. |
| MATSim Santiago v2b: 10% sample, 100 iterations, 6 threads, 8 GB heap | Run time not stated | Config file read. We did not run it. |
| LPSim (GPU vehicle queues, San Francisco Bay Area): 2.82 million trips | 6.28 minutes on 1 GPU; 9.01 million trips in 21.16 minutes on 2 GPUs | Abstract opened (arXiv 2406.08496). Self-reported. |
| MOSS (GPU microscopic traffic): 2.46 million vehicles | 84 steps per second | UNVERIFIED: from a search summary. |
| MATSim Open Berlin 10%: one iteration | more than 4 hours | UNVERIFIED: from a search summary. |

What we can say:

- The draw of one choice for each of 7 million agents is not the slow part.
- The published vehicle-level simulators that we found report minutes, on GPUs, for 2 to 9 million trips.
- The cost of a full step of our engine is unknown until we build one with real matrices. Measure it then.

**Samples in the sources that we read:** MATSim Santiago v1 0.65% (42,459 agents), v2 10% (665,201). eqasim São Paulo 1%. MATSim Berlin 1% and 10% (search summary, UNVERIFIED).

## Sources

### MATSim Santiago open scenario v2b (data files)
- Link: https://svn.vsp.tu-berlin.de/repos/public-svn/matsim/scenarios/countries/cl/santiago/ (files `v1.zip`, `v2a.zip`, `v2b.zip`, and one readme PDF for each)
- Contents: `randomized_expanded_plans.xml.gz`: one row is one agent with a full day plan. Each activity has a type with duration (`home16.0H`, `shop0.5H`), x and y in EPSG:32719, and start and end times. Each leg has a mode. `expandedAgentAttributes.xml`: car availability for each agent. `network_merged_cl.xml.gz`: 37,834 links and 23,330 nodes (OSM roads plus a separate transit network). `transitschedule_simplified.xml`: 371 lines and 26,124 stops. `counts_merged_VEH_C01.xml`: 36 count stations, hourly car volumes. `gantries.xml`: toll gantries.
- Granularity: person and point. Time step: second.
- Year: demand July 2012 to November 2013 (EOD 2012, working days). GTFS of 2015. The counts file says `year="2013"`, name `BD_FLUJOS_CONTINUAS-14Oct13`. The paper says July 2011. One of them is wrong.
- Access: direct download. `v2b.zip` 125,607,974 bytes (170 MB open, 147 MB are Java jars). Script: `tools/scrape/matsim_santiago.py`.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 125,607,974 bytes. Counted 665,201 `<person>` rows. Legs: walk 527,377; bus 497,296; car 290,251; metro 234,098; other 155,061; ride 132,142; colectivo 64,440; bike 60,879; taxi 29,218; train 636. `v1.zip` (54,811,644 bytes) and `v2a.zip` (85,924,440 bytes): VERIFIED-BYTES, HEAD 200 only.
- Role: INPUT
- Limits: The agents are clones of 42,459 EOD respondents (IDs `10002001`, `10002001_1`, ...). The number of clones follows the EOD expansion factors (Contreras 2015). Clone locations are random points inside SII land use. Thus the population has no new households and no income, age, or sex columns in these files. It is before Metro lines 3 and 6 and before the pandemic. Working days only. No licence file: the paper says "no data restrictions" and asks for a citation. The EOD link in the readme is dead (404).
- Use in the simulation: INPUT. Fast start for day plans (activity chains, durations, modes) at point level. The 36 count stations are car volumes of 2013: history, not a score target. Do not use it as the population. Build the population from Census 2024 and use these plans only as activity-chain templates.

### matsim-scenarios/matsim-santiago (code)
- Link: https://github.com/matsim-scenarios/matsim-santiago
- Contents: Java code that makes the scenario. `prepare/population/` (EOD CSV to plans, cloning, `RandomizeEndTimes`), `prepare/landuse/` (`BuildingsByBlock`, `RandomizeCoordinates`: SII land use to activity points), `prepare/counts/`, `prepare/gtfs/`, `prepare/network/` (tolls), `colectivos/`, `avcongestion/`.
- Granularity: not data.
- Year: created 2019-10-03, last push 2024-05-02. Default branch `santiago-v2b_matsim-v13.0`.
- Access: direct download (git). 283 KB, 199 files.
- Check: VERIFIED-BYTES 2026-11-21 (method evidence, not Santiago data records). GitHub API HTTP 200, tree of 199 files listed. Licence GPL-2.0. The README is the MATSim example template. It says nothing about Santiago.
- Role: INPUT (method only)
- Limits: The old path in the paper and the readme (`matsim-org/matsim/playgrounds/santiago`) is dead (API 404). GPL-2.0: if we copy code, our code must be GPL. Java, MATSim 13.
- Use in the simulation: INPUT as a recipe. Read `RandomizeCoordinates` and `ActivityClassifier` before we write our own EOD reader. Do not run MATSim.

### Kickhöfer, Hosse, Turner, Tirachini (2016), VSP Working Paper 16-02
- Link: https://svn.vsp.tu-berlin.de/repos/public-svn/publications/vspwp/2016/16-02/KickhoeferEtAl2016MatsimSantiago.pdf
- Contents: the method of scenario v1. Inputs: (1) EOD 2012: 18,000 households, 60,054 persons, 113,591 trips, 866 zones, 45 comunas; expanded totals 2,051,310 households and 6,651,654 persons. (2) OSM roads, down to secondary (tertiary inside Gran Santiago), free speed cut by 0.50 or 0.75 on small streets. (3) DTPM GTFS, 2015. Behaviour parameters from Munizaga et al. 2008. Car cost 248 CLP/km. Fares 640, 660, 720 CLP.
- Granularity: not data.
- Year: 2016 (v1). v2a and v2b have no paper, only a readme of one page.
- Access: direct download. PDF, about 6.2 MB.
- Check: VERIFIED-BYTES 2026-11-21 (method evidence). HTTP 200, pages 1 to 20 read.
- Role: INPUT (method only)
- Limits: Calibration is weak. They adjusted 3 mode constants (car +1.100, PT -0.883, walk 0) until the mode shares equalled the shares of the input population itself. Validation: 36 of 49 count stations, 15-minute bins; the simulation has too few cars at most stations. Cars and buses run on separate networks, thus buses do not get congestion. Only car, PT, and walk can change; the other modes are fixed. The paper gives no run time and no hardware. v1 used 42,459 agents (0.65%) with flow capacity 0.0064.
- Use in the simulation: INPUT as method and as a warning. Their score (mode share that equals the input) is circular. Our score must use data that the model did not see.

### DTPM transit assignment model, March 2020 (EMME)
- Link: https://www.dtpm.cl/index.php/documentos/modelos-de-demanda
- Contents: the model that DTPM uses for buses, Metro, and Metrotren. Three periods: morning peak (PMA), off-peak (FPU), evening peak (PTA). One hour of demand each. Supply: operating programme of March 2020. Demand: bip! transactions of one working week of August 2019, adjusted to passenger counts on buses and Metro of September and October 2019. The OD matrix file `emmemat/mf70.emx` (FPU) is a raw table of 3000 × 3000 float32 values: one value is the trips from one zone index to another in the hour.
- Granularity: zone (OD matrix), link and line (network). One hour for each period.
- Year: 2019 demand, 2020 supply. An older model (July 2018 supply, August 2017 demand) is on the same page as `.rar` files on `dtpm.gob.cl`. We did not open them; critic quiet-panda says the link answers HTTP 200 after a 301.
- Access: direct download. `Modelo_PMA_DTPM.zip` 499,450,085 bytes; `Modelo_FPU_DTPM.zip` 68,345,154 bytes; `Modelo_PTA_DTPM.zip` 258,825,867 bytes; description PDF 114,875 bytes.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, for the FPU matrix only. Read with the Python standard library (`array('f')`), no EMME: 36,000,000 bytes, total 243,005 trips in the hour, 2,469 origin indexes and 2,466 destination indexes with trips, 1,817,180 cells above zero, largest cell 139.7. The network is VERIFIED-BYTES only: it is in `emmebank`, a binary format. The SQLite file `Database/13.db` has tables `nodes` and `links` with 0 rows. PDF read (2 pages). Readable text: the fare macro (bus 700, Metro 720) and the transfer penalties for Metro nodes.
- Role: INPUT
- Limits: The matrix has index numbers, not zone IDs. The table from index to zone, and the zone positions, are in `emmebank`, which we did not read. Without it the matrix has totals but no map. Critic quiet-panda names the PyPI package `wsp-balsa` as a reader; we did not test it. DTPM says the east zone (Lo Barnechea, Vitacura, Las Condes, La Reina, Peñalolén) has almost no bus counts, and the evening peak can be too low there because of fare evasion. Before the pandemic. No licence text.
- Use in the simulation: INPUT only if somebody gets the zone table out of `emmebank`. Then it is a transit OD matrix of 2019 for 3 periods, already adjusted to counts. The DTPM trip tables of 2026 give newer OD data with stop coordinates, thus use those first.

### ADATRAP trip and stage tables (DTPM)
- Link: https://www.dtpm.cl/index.php/documentos/matrices-de-viaje
- Contents: output of ADATRAP, the software that estimates where each bip! card user gets off. One row is one trip (or one stage) with estimated boarding and alighting stop. Also boardings for each stop and half hour, and the zone file `Zona777_ADATRAP.rar`.
- Granularity: person-trip, stop, zone (777 zones). One week for each release.
- Year: 16 releases on the page, from March 2020 to April 2026. Daily estimates from April 2017.
- Access: direct download. ZIP of CSV. `Zona777_ADATRAP.rar` is 1,660,725 bytes.
- Check: UNVERIFIED for the trip and stage files: we read only the index page (HTTP 200, 40 file links) and did not request the ZIP files. VERIFIED-BYTES for the zone file only (HEAD 200, 1,660,725 bytes). The row evidence for the April 2026 files is in `context/research/santiago-transport-verified.md`.
- Role: TARGET (boardings) and INPUT (transit OD)
- Limits: DTPM says the tables do not include fare evasion and that the method changes between versions. The alighting stop is an estimate, not a measurement. Only public transport.
- Use in the simulation: TARGET (boardings for each station and half hour) and INPUT (transit OD). This is the modern replacement for the 2013 counts in MATSim Santiago.

### ADATRAP code (GitHub organisation SmartcitySantiagoChile)
- Link: https://github.com/SmartcitySantiagoChile
- Contents: 16 repositories. Visualisation server (`fondefVizServer`), upload scripts, validators. Last pushes 2017 to 2024.
- Granularity: not data.
- Year: 2017 to 2024.
- Access: direct download (git).
- Check: VERIFIED-BYTES 2026-11-21. GitHub API HTTP 200, 16 repositories listed. Most have no licence.
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
- Limits: 14 years old. MATSim Santiago, ESTRAUS, and the academic models all use this one survey. They share survey limitations, although their processing and model errors can differ. No newer EOD of Santiago was found in this research.
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
- Check: VERIFIED-BYTES 2026-11-21 (method evidence). GitHub API HTTP 200, README read. Licence GPL-2.0. The published sample is 1%.
- Role: INPUT (method only)
- Limits: it is for Brazil. We found no Santiago or Chile version: GitHub search `eqasim chile` gives 0 results.
- Use in the simulation: INPUT as method. The 3 steps (census persons, then survey chains by matching, then locations) are an alternative to the MATSim cloning.

### Searches with no useful result (one line each)
- SUMO scenario of Santiago: GitHub search `santiago sumo` gives 1 result, not related. One web search found none. Use: NONE.
- Synthetic population of Santiago from EOD plus Census: GitHub search `santiago synthetic population` gives 0 results. Two web searches found only the MATSim cloning. Use: NONE.
- eqasim for Chile: GitHub search `eqasim chile` gives 0 results. Use: NONE.
- Metro de Santiago demand model: one web search, no public model or file found. Use: NONE.
- Agent simulator of COVID-19 in Santiago, https://arxiv.org/abs/2005.09556: abstract opened, no data or code link found. Use: NONE.
- Optimizing city-scale traffic, https://arxiv.org/abs/1906.05093: abstract opened. A search summary says it uses about 665 thousand vehicles in Santiago. Similar counts do not establish that it uses the MATSim v2b dataset; overlap remains unverified. Use: NONE.

### Simulators of other cities (not Santiago data; for the engine decision only)
- LPSim, https://arxiv.org/abs/2406.08496: abstract opened, HTTP 200. Numbers in the table above. Use: NONE as data.
- MOSS, https://github.com/tsinghua-fib-lab/moss: GitHub API HTTP 200, licence MIT, last push 2025-08-26. Needs an NVIDIA GPU. Use: NONE as data.
- AgentTorch, https://github.com/AgentTorch/AgentTorch: GitHub API HTTP 200, licence AGPL-3.0, last push 2026-11-15. The hint "8.4 million agents of New York" is UNVERIFIED. Use: NONE as data.

## Hints that were wrong

- "MATSim Santiago is in matsim-org on GitHub": the playground path is dead. The code is in `matsim-scenarios/matsim-santiago`. The data is on the TU Berlin SVN.
- "eqasim" for Santiago: none found.
- "SUMO scenarios" for Santiago: none found.
- "Papers that build a synthetic population of Santiago from EOD and Census": none found. MATSim uses EOD only, with clones.
- "ADATRAP" is not a simulator. It is an estimator of alighting stops. Its output is public; its engine is not.

## Critique

1. **The speed question has no answer.** The toy test is one draw with random costs, run one time by one agent. No second agent ran it. It does not give the time of a full step.
2. **The "none found" results come from few searches.** 5 GitHub searches and 3 web searches. We did not search theses, Spanish-language repositories, or private firms. A synthetic population of Santiago can exist and not be public.
3. **The closed models are documented from summaries.** For ESTRAUS and MUSSA, the zone counts and group counts come from search summaries of MOP reports that we did not open. The DTPM matrix has no zone map yet.

Not checked: the content of `v1.zip` and `v2a.zip`; the DTPM PMA and PTA archives; the 2018 DTPM model; `wsp-balsa` or any reader for `emmebank`; the EOD Drive files; theses of U. de Chile and PUC that used MATSim Santiago after 2016; Fosco and Zurita 2021 (PLoS ONE, lockdown agent model); models inside private firms (Citymovil, Steer, Telefónica).
