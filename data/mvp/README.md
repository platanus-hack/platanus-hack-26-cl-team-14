# Data pack

Some sections describe builder outputs that are not on `main` yet: `site_inputs.csv`. They arrive in later PRs.

## blocks.csv

Home blocks of the simulation. One row per Census 2024 block-table row of the Región Metropolitana (urban manzanas, rural village blocks, rural entities, comuna containers). Builder: `python3 -I tools/pack/zones.py`.

| Column | Meaning |
| --- | --- |
| `block_id` | INE `MANZENT` code, as text |
| `cut` | 5-digit INE comuna code, as text |
| `longitude`, `latitude` | Area-weighted polygon centroid, EPSG:4674 (SIRGAS 2000, equal to WGS84 at this scale) |
| `population` | `n_per`, persons counted |
| `households` | `n_hog`, households |
| `dwellings` | `n_vp`, private dwellings |
| `workers` | `n_ocupado`, all employed persons who live in the block |
| `students` | `n_asistencia_basica` + `n_asistencia_media`, persons who attend basic or secondary school |

- Source: INE Census 2024, `Base_manzana_entidad_CPV24.csv` and `Cartografia_censo2024_R13.gpkg` (layers `Manzanas_CPV24`, `Entidades_CPV24`, `Comunal_CPV24`). Year 2024.
- Rows: 52,646. Population 7,400,741 (equal to the census RM total), households 2,599,621, dwellings 2,763,387. Providencia (`13123`): 621 rows, 143,974 persons.
- 239 blocks have 0 persons. They stay in the file because the census table has them.
- 53 rows are comuna containers (`CONTENEDOR_COMUNAL=1`, 108,997 persons). They have no polygon by INE design. Their point is the person-weighted mean of the block centroids of their comuna, so it lands where people live (Providencia container: -70.6085, -33.4336). All other 52,593 rows join to a polygon.
- The block table has no 15–64 or 5–17 age bins. `workers` and `students` are direct census counts instead.
- `workers` is **all employed residents**: 3,588,512. It is not the commuter count. It includes people who work at home and outside the RM. Commuters in `work_od.csv`: 2,740,454. Use the matrix rows as shares, not as counts against `workers`.
- INE suppresses some values (`*`) for disclosure control. The builder writes them as 0. In the RM, `n_per`, `n_hog` and `n_vp` have 0 suppressed values, so population, households and dwellings are complete counts. `n_ocupado`, `n_asistencia_basica` and `n_asistencia_media` have 2 suppressed values each.

## communes.csv and communes.geojson

The 52 comunas of the RM. Same builder as `blocks.csv`: `python3 -I tools/pack/zones.py`.

| Column | Meaning |
| --- | --- |
| `cut` | 5-digit INE comuna code |
| `name` | INE comuna name |
| `population` | sum of `n_per` over the comuna's block-table rows (containers included) |
| `households` | sum of `n_hog` |
| `dwellings` | sum of `n_vp`, private dwellings |
| `longitude`, `latitude` | person-weighted mean of the block centroids of the comuna (`blocks.csv`, containers excluded). Not the polygon centroid: that one is 5.2 km off in Las Condes (-70.5007), 14.9 km in Lo Barnechea and 25.9 km in San José de Maipo, in the Andes. Median gap over the 52 comunas: 1.3 km. All 52 points are inside their own polygon |

- Totals: population 7,400,741, households 2,599,621, dwellings 2,763,387. Providencia `13123`: 143,974 / 69,157 / 76,376.
- Check: all origins of `work_od.csv` are in this file. RM to RM workers in `work_od.csv`: 2,688,814.
- `communes.geojson`: the same 52 polygons (MultiPolygon, properties `cut` as 5-digit text, `name`, `population`). The builder rounds coordinates to 4 decimals (about 10 m) and removes repeated points. 2.0 MB. For the browser only.
- City-scale check of `communes.geojson`: 52 features, each 1 polygon, 0 holes. Area sum about 15,350 km² (approximate flat projection). The rural comunas are most of the area: San José de Maipo alone is 4,975 km². A frame on all 52 shows the city small.
- `communes_urban.geojson`: the 34 comunas of Gran Santiago, for the city-scale view. List: the 32 comunas of Provincia de Santiago (CUT `131xx`) plus Puente Alto `13201` and San Bernardo `13401`. This is the common 34-comuna definition of Gran Santiago (Wikipedia "Gran Santiago"). Population 6,260,194. 0.8 MB. Lo Barnechea `13115` is in the list but is mostly Andes, so its polygon reaches -70.18 lon. Fit the camera to the people, not to this bbox: the population-weighted 0.5–99.5 percentile box of the `blocks.csv` rows of these 34 cuts is lon -70.7991 to -70.5031, lat -33.6324 to -33.3379.

## schools.csv

School destinations. One row for each operating school of the Región Metropolitana that has a coordinate. Builder: `python3 -I tools/pack/schools.py`.

| Column | Meaning |
| --- | --- |
| `rbd` | MINEDUC school ID |
| `name` | `NOM_RBD` |
| `cut` | 5-digit INE comuna code of the school |
| `longitude`, `latitude` | School point, WGS84 |
| `enrollment` | `MAT_TOTAL`, students enrolled at the cut of 2026-04-30 |
| `enrollment_no_preschool` | `MAT_TOTAL` − `MAT_ENS_1`, the same count without parvularia |

- Source: MINEDUC Directorio Oficial de Establecimientos 2026, `data/raw/mineduc-directorio/dir2026.rar`. Year 2026, cut 2026-04-30. Catalog entry: `context/research/sources/verify-people-work.md`.
- Filters: `COD_REG_RBD=13` and `ESTADO_ESTAB=1`. The file holds 4,467 RM rows; 3,227 are operating.
- Rows: 3,220. Enrolment: 1,309,286. Comunas: 52. Providencia (`13123`): 83 schools, 29,641 students.
- Dropped: 7 of the 3,227 operating schools. 6 have an empty coordinate, and RBD 41884 is at latitude -29.22, which is in Coquimbo. All 7 are nurseries with `MAT_TOTAL=0`, so the file loses 0 students. The box used is latitude -34.4 to -32.8 and longitude -71.8 to -69.7.
- `MAT_TOTAL` is the exact sum of `MAT_ENS_1` to `MAT_ENS_8`, so it **does** count pre-school: 104,953 of the 1,309,286 students are in `MAT_ENS_1` (parvularia). 1,663 of the 3,220 schools report enrolment in that level.
- `enrollment_no_preschool` subtracts that level on the same row: **1,204,333 students**, exactly 1,309,286 − 104,953. It is **school enrolment excluding pre-school**: not trips, and no evidence of independent travel (`clever-heron`). Added for `lucky-llama`'s L5 candidate. Checks on the rebuild: 3,220 rows unchanged, every earlier column byte-identical, no row negative or above its `enrollment`, and **74 schools with 3,906 pupils drop to 0** because they teach pre-school only. A model using this column must treat those 74 points as zero mass, not as missing.
- **407 of the 3,220 rows have `enrollment` 0, and they are not closed schools.** `ESTADO_ESTAB` is 1 ("Funcionando") for all of them; the codebook has a separate code 3 for "Cerrado" and 4 for "Autorizado sin matrícula". What the directory says about these 414 operating RM schools is: `MATRICULA=0` ("Establecimiento con matrícula al 30 de abril" = No) for all 414, and `ENS_01=0`, so the file records **no education level** for any of them. The names are nearly all pre-school ("ESCUELA DE PARVULOS", "SALA CUNA", "JARDIN INFANTIL", and 41 of the "VITAMINA" chain). 352 of the 414 are `COD_DEPE2=3`, particular pagado, and 242 have an `RBD` above 40000, so most are privately paid and recently registered. The same reason applies to the 7 schools dropped above, which is why dropping them loses 0 students.
- **The 0 is consistent with the enrolment file, verified.** I counted the rows of `Matricula-Oficial-2026` whose `RBD` is one of these 407: **0 rows**. So no student in `school_od.csv` goes to one of these points. A 0 weight is the right behaviour, not a gap.
- Consequence for the engine: `enrollment` is the destination weight, so those 407 points can never receive a student. Do not "fix" the 0 with an invented capacity; it would send students to schools that report none.
- The codebook calls the coordinates approximate. Enrolment is not daily attendance.

## work_od.csv and school_od.csv

The two origin-destination matrices at comuna scale. Builder: `python3 -I tools/pack/od.py`.

| Column | Meaning |
| --- | --- |
| `origin_cut` | 5-digit INE comuna code of the home. Always a comuna of the RM |
| `destination_cut` | 5-digit INE comuna code of the work place or of the school. Any comuna of Chile |
| `workers` / `students` | Persons on the pair |

Both files keep only a pair where each code is one of the 346 comunas of the MINEDUC directory. A country code, a region-only code such as `13000`, and `998`/`999` cannot pass. Codes become 5 digits: the microdata write regions 1 to 9 with 4 digits, so Valparaíso is `5101` in the raw file and `05101` here.

### work_od.csv

- Source: INE Census 2024 person microdata, through `tools/scrape/censo2024-od.py` to `data/raw/censo2024-personas/od_work_rm.csv`. Home is `comuna`, work place is `p44_lug_trab_esp`.
- Rows: 9,128. Workers: 2,740,454. Inside the RM: 2,688,814. To another region: 51,640.
- Check against the INE visualizer: Santiago (`13101`) to Las Condes (`13114`) is **28,583** workers.
- Only real commutes. `p44_lug_trab` must be 2 (works in this comuna) or 3 (works in another comuna). The excluded answers, over the 7,400,741 RM persons of the census, are:

  | Answer | Persons |
  | --- | --- |
  | Kept, codes 2 and 3 with a known comuna | 2,740,454 |
  | No work-place question (not employed) | 3,812,159 |
  | Code 1, works at home | 517,018 |
  | Code 5, no fixed place | 313,358 |
  | Region known, comuna not (`13000` and the other `NN000`) | 8,495 |
  | Code 4, works in another country | 3,793 |
  | Destination `-99` | 3,016 |
  | `p44_lug_trab` `-99` | 2,448 |

  The 8 rows sum to 7,400,741, the census RM total. `data/raw/censo2024-personas/excluded_destinations.csv` holds the same counts by code.
- Code 1 is not a trip, and its destination repeats the home comuna, so leaving it in would raise the diagonal (835,484 workers) by 62%. Code 4 puts a **country** code in the destination column. Both are out.
- Code 5 is 313,358 people, 8.7% of the 3,588,582 persons with a work-place answer, and it is not spread evenly (12.9% in Peñalolén and La Pintana against 3.2% in Melipilla). Those people travel and no matrix can say where. The model misses their trips.
- `p44` is the usual work place, not a trip on a given day. The destination stops at the comuna, so the matrix cannot put a job inside a comuna. Use `jobs.csv` for that.

### school_od.csv

- Source: MINEDUC Matrícula Oficial 2026, through `tools/scrape/mineduc-school-od.py` to `data/raw/mineduc-od/od_school_comuna_rm.csv`. Home is `COD_COM_ALU`, school is `COD_COM_RBD`, filtered on `COD_REG_ALU=13`.
- Rows: 10,704. Students: 1,413,489. Inside the RM: 1,277,336. To another region: 136,153.
- **Limit on the 136,153 students who go to a school outside the RM (9.6%).** They are not verified as daily trips. I counted them by school in the raw enrolment file: they spread over 5,962 schools and no school holds more than 372 of them, so no online school and no single artefact explains the volume. **Probable cause, not verified:** `COD_COM_ALU` is the address of record of the register, and for a family that moved it keeps the old RM address. My check rules out one single large artefact school; it does not prove the cause and it does not rule out online or distance schools spread over many RBD. Use only the rows where `destination_cut` starts with `13` for daily school trips. The other rows stay in the file because they are in the source, not because the trip is verified.
- `schools.csv` has no point for a school outside the RM, so those rows cannot become a point destination.
- Enrolment is choice plus admission, so it understates demand for a school that turns students away. The home side never goes below the comuna.
- One snapshot, cut 2026-04-30. No time of day.
- **The 3 student totals of the pack are 3 different populations. They must not agree.**

  | Number | Population |
  | --- | --- |
  | `blocks.csv` students, 1,033,857 | RM **residents** who attend basic or secondary school. Census 2024. No pre-school, no higher education |
  | `schools.csv` enrolment, 1,309,286 | Students **at an RM school**, any home comuna. MINEDUC 2026. Includes 104,953 in pre-school (`MAT_ENS_1`) |
  | `school_od.csv` students, 1,413,489 | Students with an **RM home comuna** at a school anywhere in Chile. MINEDUC 2026 |

  **The two MINEDUC numbers reconcile to 4 students, measured in the enrolment file, not by subtraction.** Of the students at an RM school, 1,277,336 live in the RM and **31,946** live outside it, which is 1,309,282 against the 1,309,286 of `schools.csv`. The gap of 4 is the 7 schools that `schools.csv` drops for a bad coordinate plus rounding of the two publication cuts. The census number is lower because it excludes pre-school and counts 2024, not 2026.

## mode_share.csv

How the commuters of `work_od.csv` travel to work, by comuna of residence. Built for the pitch moment that sends new residents to a station. Builder: `python3 -I tools/pack/mode_share.py` (one full pass over the person file, about 5 minutes).

| Column | Meaning |
| --- | --- |
| `cut` | 5-digit INE comuna code of the home. The 52 comunas of the RM |
| `mode` | `p45_medio_transporte` code, see the table below |
| `commuters` | Persons |

- Source: INE Census 2024 person microdata, `p45_medio_transporte`. Year 2024.
- Rows: 410. Commuters: **2,740,454**, exactly equal to the total of `work_od.csv`. The same filter builds both (`p44_lug_trab` 2 or 3 with a known comuna), so a share of one comuna divides its `work_od.csv` rows with no residual.
- Codes, with the official labels from the microdata manual, and the RM share:

  | `mode` | Label | Commuters | RM share |
  | --- | --- | --- | --- |
  | 2 | Transporte público (bus, micro, metro, tren, taxi, colectivo) | 1,435,056 | 52.4% |
  | 1 | Auto particular | 814,204 | 29.7% |
  | 3 | Caminando | 244,652 | 8.9% |
  | 4 | Bicicleta (incluye scooter) | 102,801 | 3.8% |
  | 7 | Otro | 97,432 | 3.6% |
  | 5 | Motocicleta | 45,635 | 1.7% |
  | 6 | Caballo, lancha o bote | 376 | 0.0% |
  | -99 | No respuesta | 298 | 0.0% |

- Comunas of the story: Providencia (`13123`) 64,203 commuters, **39.7% public transport**, 33.7% car, 14.9% walking. Las Condes (`13114`) 31.4% public transport against 51.1% car. Santiago (`13101`) 61.7% public transport. La Pintana (`13112`) 65.4%.
- **Limit that matters for the station layer: code 2 puts metro, bus, train, taxi and colectivo in one answer.** The census does not separate them, so this file cannot give a metro share. Any split of code 2 between metro and bus is an assumption, and the page must label it.
- The universe is a person who works away from the dwelling, so "works at home" is `NA` and is not in this file. It is the main mode, not every leg of the trip, and it is the usual mode, not a trip on a given day.

### work_departures.csv

This file gives 48 local half-hour departure shares for each of five travel modes in
the 2012 Santiago origin-destination survey. Bin 0 starts at midnight. Each row contains
`mode,half_hour,expanded_trips,share`. The 48 shares for each mode sum to one.
Use the shares to assign times to trips. The expanded totals describe Santiago in 2012.

The builder selects trips to work (`Proposito=1`) on normal weekdays, with a recorded
`HoraIni`. This includes all trips to work during the day. The survey can count a worker
again after a lunch break. Three eligible records lacked the departure clock.
The 13,437 remaining trips expand to 2,989,363 weekday trips. Each trip weight equals
`viajes.FactorLaboralNormal × personas.Factor_LaboralNormal` for the same respondent.
[Survey and files](https://www.sectra.gob.cl/info-territorial/region-metropolitana-de-santiago/) · [INE expansion method](https://www.ine.gob.cl/docs/default-source/sistema-de-indicadores-y-estandares-de-desarrollo-urbano/manuales/actualizaci%C3%B3n-2019/manual-procedimientos-siedu-act2019.pdf)

| Group | EOD `ModoAgregado` | Sample trips | Expanded trips | Peak start and share |
|---|---|---:|---:|---:|
| car | 1 | 3,387 | 924,788 | 07:00 · 19.33% |
| public | 2–7, 9–12 | 6,798 | 1,244,381 | 07:00 · 17.07% |
| walk | 17 | 1,580 | 482,964 | 14:00 · 16.90% |
| bicycle | 18 | 717 | 151,907 | 07:30 · 18.12% |
| other | 8, 13–16 | 955 | 185,324 | 07:00 · 16.70% |

The public group covers bus, Metro, taxi, colectivo and their combinations.
The other group includes car with Metro and trips that the EOD labels as other.
Applying those timing profiles to 2024 Census mode groups is a modeling assumption.
The EOD table has no separate profiles for scooters, motorcycles or an unknown mode.

After extracting the survey with its original filenames, run:

```sh
python3 -I tools/pack/work_departures.py --survey /path/to/EOD_STGO
```

### mode_share_by_workplace.csv

`cut,work_place,mode,commuters`. The same counts split by `p44_lug_trab`: **2** works in
this comuna, **3** works in another comuna. 781 rows, 2,740,454 commuters. Same builder.
**Summing this file over `work_place` reproduces `mode_share.csv` exactly**, all 410 rows,
so it is a strict refinement and nothing that reads `mode_share.csv` needs to change.

**Why it exists: one share for each comuna blends two populations that travel very
differently.** Over the RM:

| | public transport | car | walking |
| --- | --- | --- | --- |
| works in own comuna (835,484) | 0.3218 | 0.2866 | **0.2663** |
| works in another comuna (1,904,970) | **0.6122** | 0.3017 | 0.0116 |

A person who stays inside the comuna walks 23 times more often than one who leaves it.
So the `work_place` 3 share is a **candidate** for a cross-comuna flow in place of the
comuna average. It is a refinement to preregister and score, not a rule that this file
imposes on an existing fit.

- Effect on the number the engine uses today. The inbound public-transport factor for
  Las Condes, `Σ_c F_c,13114 · m_c / Σ_c F_c,13114`, is **0.4900** with the blended
  shares and **0.5387** with workplace-consistent shares (code 3 for the other origins,
  code 2 for Las Condes itself). That is 10% higher, and it would move the station
  model's ΔJ for 4,679 jobs from −2,292.5 to −2,520.5.
- **0.5387 is a more specific transfer estimate, not a measured correction.** Both
  numbers transfer a home-comuna share to one destination. The split removes **one
  aggregation assumption** — that a commuter who stays inside their comuna travels like
  one who leaves it — and nothing more. It does not reduce destination-specific
  uncertainty, and it is not evidence that 0.5387 is nearer the truth than 0.4900.
- Las Condes residents: **0.2504** public transport when they work in Las Condes,
  **0.3607** when they work in another comuna. The single 0.3141 of `mode_share.csv` is
  the blend of the two.
- Largest origins, blended against cross-comuna: Santiago 0.617 → 0.755, Puente Alto
  0.600 → 0.680, La Florida 0.585 → 0.643, Ñuñoa 0.500 → 0.531.
- **This file is still a transfer, and it does not have to be.** A person row carries
  `p44_lug_trab_esp` and `p45_medio_transporte` together, so the mode of a single
  origin-destination pair can be aggregated in a separate export. This file does not
  contain those pair-specific counts. Use this split for an outbound question about
  a comuna's residents; review a joint mode/OD export before using it for inbound flows.
  Either way, replacing a coefficient inside a fitted model is a refinement to register
  and review before it is scored, not a drop-in fix.

### mode_od.csv

`origin_cut,destination_cut,mode,commuters`. **23,467 rows, 2,740,454 commuters.** The
mode of each origin-destination pair, from the same pass and the same filter. A person row
carries `p44_lug_trab_esp` and `p45_medio_transporte` together, so this needs no transfer:
it observes how the RM-resident known-destination commuters of one pair travel.

**Three reconciliations, each verified by a real run, each exact:**

1. Summing over `mode` gives **all 9,128 pairs of `work_od.csv`**, identical counts.
2. Summing over `destination_cut` gives **all 410 rows of `mode_share.csv`**, identical.
3. Summing over `origin_cut` gives the mode of the RM residents who work in each comuna,
   1,603 rows.

- **Las Condes `13114` as a workplace: 305,601 RM-resident known-destination commuters,
  public-transport share 185,775 / 305,601 = 0.607900.** Two earlier estimates of that
  same quantity transferred the origin comunas' shares and gave 0.4900 and 0.5387. **The
  observed share is 24% higher than the first and 13% higher than the second**; stated
  the other way, those estimates fall 19.4% and 11.4% below the observed value. An
  inbound question needs no transfer now.
- Other workplaces, same cohort: Providencia 0.6664, Santiago 0.6398, Ñuñoa 0.6206,
  Maipú 0.4839.
- `mode` `-99`, "no respuesta", is kept (23 commuters at Las Condes), so every total stays
  equal to `work_od.csv`. Destinations outside the RM are kept for the same reason.
- Limit: the person file covers RM residents, so every number here describes **RM
  residents with a known work comuna**, never everyone who works in a comuna. A worker
  who lives outside the RM is absent. `work_od.csv` has the same universe.
- Applying a comuna's inbound share to one site, such as the modeled MUT jobs, still
  transfers a comuna average to that site.
- The limits of `mode_share.csv` still apply: code 2 joins metro, bus, train, taxi and
  colectivo, and it is the main and usual mode, not a trip on a given day.
- Replacing a coefficient inside a fitted model with a number from this file is still a
  refinement to register and review before it is scored.

## site_inputs.csv

The comuna inputs that the site model of `context/research/tobalaba-scenario-contract.md`
needs and has no number for. One row for each of the 52 comunas of the RM.
Builder: `python3 -I tools/pack/site_inputs.py pass` (25 s; `pass` does the one full pass
over the person file and caches the tallies, then the join runs in 2 s).

| Column | Meaning | Kind |
| --- | --- | --- |
| `cut`, `name` | 5-digit INE comuna code and INE name | |
| `dwellings` | `n_vp`, private dwellings, vacant included | observed |
| `dwellings_occupied` | `n_vp_ocupada` | observed |
| `occupied_share` | `n_vp_ocupada / n_vp` | observed |
| `persons` | persons in private dwellings | observed |
| `persons_per_occupied_dwelling` | `persons / dwellings_occupied` | observed |
| `persons_per_dwelling_with_residents_present` | `persons / dwellings_with_residents_present` | observed |
| `commuters_rm` | persons in private dwellings with `p44_lug_trab` 2 or 3 and a work comuna in the RM | observed |
| `commuters_all` | the same, with a work comuna anywhere in Chile | observed |
| `commuters_rm_per_resident`, `commuters_all_per_resident` | that count / `persons` | observed |
| `apartment_share_of_dwellings` | `n_tipo_viv_depto / n_vp` | observed |
| `persons_per_occupied_dwelling_apartment_blocks` | block-table `n_per` (all dwelling kinds) / `n_vp_ocupada`, over blocks where 90% or more of the private dwellings are apartments and `n_vp` is 10 or more. Its numerator differs from the private-person microdata numerator of the comuna ratio | observed, subset |
| `occupied_share_apartment_blocks` | `n_vp_ocupada / n_vp` over that same subset. Use it together with the column above, so the chain stays closed inside the subset | observed, subset |
| `car_share_block_table`, `public_share_block_table` | `n_transporte_auto` and `n_transporte_publico` over the 7 travel columns of the comuna | observed |
| `car_share_apartment_blocks`, `public_share_apartment_blocks` | the same two shares over the apartment-block subset | observed, subset |
| `travellers_per_dwelling_ratio_apartment_blocks` | (travellers / `n_vp`) of the subset ÷ the same of the comuna. Multiply the comuna chain by this to get the subset value, instead of mixing two universes | observed, subset |
| `car_travellers_per_dwelling_ratio_apartment_blocks` | the same ratio for `n_transporte_auto` alone | observed, subset |
| `apartment_blocks`, `apartment_block_persons` | size of that subset | |
| `dwellings_with_residents_present` | dwellings with at least one person record | observed |
| `persons_all_dwelling_kinds` | `n_per`: private **plus** collective dwellings and the street operative | observed |

- Sources, both INE Census 2024: the person microdata
  (`data/raw/censo2024-personas/personas_censo2024.zip`) for persons, dwellings with
  residents and commuters; the block table (`data/raw/verify-people-work/census-blocks.zip`)
  for `n_vp`, `n_vp_ocupada` and `n_tipo_viv_depto`. No value of these columns is
  suppressed in the RM, so they are complete counts.
- RM row: dwellings 2,763,387, occupied 2,619,563, `occupied_share` 0.9480,
  persons 7,345,647, `persons_per_occupied_dwelling` 2.8041,
  `commuters_rm_per_resident` 0.3658, `commuters_all_per_resident` 0.3728.
- Las Condes `13114` (the MUT site): `occupied_share` **0.9405**,
  `persons_per_occupied_dwelling` **2.4185**, `commuters_rm_per_resident` **0.3577**,
  `commuters_all_per_resident` 0.3670, apartment share 0.7288,
  `persons_per_occupied_dwelling_apartment_blocks` **2.0847** with
  `occupied_share_apartment_blocks` **0.9251**, over 320 blocks and 160,457 persons.
- **Do not compare `commuters_rm` with `work_od.csv`.** The two count different
  universes, so they differ and both are correct. For Las Condes, `commuters_rm` is
  **104,918**, over persons in private dwellings only (`tipo_operativo` 2). The Las Condes
  origin rows of `work_od.csv` with a destination in region 13 sum to **104,980**, over all
  dwelling kinds. The gap of 62 persons is the RM commuters who live in collective
  dwellings or were counted by the street operative. A ratio must keep one universe on
  both sides: 104,918 / `persons` 293,272 = 0.3577 (private dwellings), or 104,980 /
  `persons_all_dwelling_kinds` 296,134 = 0.3545 (all dwelling kinds). The site request of
  the demo uses 0.3577, because the occupancy inputs beside it are apartment-block
  aggregates, which are a subset of private dwellings.
- **The mode mix beside it is a third universe, and it changes nothing.** Las Condes in
  `mode_share.csv` totals **107,707**, every known national work destination, equal to the
  all-destination origin total of `work_od.csv`. Restricted to destinations in region 13
  with `mode_od.csv` (104,980 commuters), the car share is 0.510802 against 0.510728 and
  the public share 0.318308 against 0.314130. On the 298-dwelling scenario both versions
  give the same two whole numbers, 84 by car and 52 by public transport a day.
- **Observed travel shares in the apartment-dominated blocks.** In Las Condes they use
  the car less than the comuna: `car_share_apartment_blocks` **0.4684** against
  `car_share_block_table` **0.5192**, and public transport **0.3219** against 0.3077.
  Walking rises from 9.2% to 12.0%. In Providencia the gap is small (0.3371 against
  0.3469), and in Santiago the subset is almost the whole comuna (0.1286 both).
- **Do not build a chain out of one column of the subset and one of the comuna.** A
  flat holds fewer persons but its persons travel more: in Las Condes the subset has
  0.8384 travellers for each dwelling against 0.9208 in the comuna, and 0.4347
  travellers for each person against 0.4009. Scale the comuna chain by the two ratio
  columns instead. Las Condes, 298 dwellings: the comuna chain gives 242.5 commuters
  and 123.8 of them travel by car; × 0.9105 and × 0.8213 gives **220.8 commuters and
  101.7 travelling by car**. Call the result "ratio-adjusted, block-table subset".
- Limit of the subset: most Las Condes flat blocks are on the Apoquindo and Metro
  corridor, so part of the lower car share comes from the place, not from the dwelling
  type. For a tower 122 m from Tobalaba that bias points the right way, but it is a
  bias.
- **The 4 travel columns are a wider universe than `mode_share.csv` and the two must not
  be mixed.** The block table counts every person with a `p45` answer: 3,068,499 in the
  RM, against 2,740,454 in `mode_share.csv`. **Measured decomposition** of the
  3,068,579 RM persons whose `p45` answer is 1 to 7: 2,740,156 are the commuters of
  `mode_share.csv` without its 298 `-99` answers, 313,258 have no fixed place of work
  (`p44_lug_trab` 5), 11,376 answered 2 or 3 but gave a comuna the matrix cannot use,
  and 3,789 work in another country (code 4). The remaining 80 against the 3,068,499 of
  the block table are the suppressed cells. **A person who works at home is not in this
  universe**: `p45` is `NA` for `p44_lug_trab` 1, so its 517,018 people are out. That
  is
  why the comuna car share here is 0.5192 and `mode_share.csv` gives 0.5107 for the
  same comuna. Use `mode_share.csv` for a commuter count, and these 4 columns only to
  compare a block subset against its comuna on one basis. 45 cells of the 7 travel
  columns are suppressed in the RM and count as 0.
- **No released file joins the two.** A mode share for the commuter cohort below the
  comuna does not exist: `mode_share.csv` has the correct cohort but stops at the
  comuna, because the finest geography of the person microdata is comuna × urban/rural;
  the block table has the block but a wider cohort, and INE publishes no cross-tab of
  `p45` by `p44`. So **any apartment-block refinement of a commuter mode share is a
  transfer assumption**, never a measured commuter count. Label it that way, or keep
  the comuna share of `mode_share.csv`.
- **The apartment pair with a comuna commuter rate is one transfer assumption, and the
  ratio method is another. Neither is a bound on a real tower.** The pair carries the
  smaller flat (2.0847 against 2.4185) but not the higher travel of the people in it.
  Measured with the rate 0.3545: the pair gives 0.6837 commuters for each dwelling and
  the ratio method gives 0.7342, which is 7.4% higher. The difference is between two
  models, not an error against a measurement, so **do not call either one conservative
  or a lower bound**. Label the one you send "supuesto de transferencia" and name it.
  The demo sends the apartment pair with 298 **physical** dwellings; do not fold a
  ratio into `dwellings_added`, which is a count of dwellings.
- The two pairs do not give the same residents for each dwelling built: the comuna pair
  gives 0.9405 × 2.4185 = 2.2746 and the apartment-block pair gives
  0.9251 × 2.0847 = 1.9286, which is 15% lower. Always use a pair, never one column of
  one pair and one column of the other.
- **`persons` is not `communes.csv` `population`.** `tipo_operativo` splits the
  7,400,741 RM persons into 7,345,647 in private dwellings (code 2), 46,636 in
  collective dwellings (code 3) and 8,458 in the street operative (code 1). The labels
  are the official ones of the microdata manual, page with Table 4.2
  (`data/raw/verify-people-work/census-manual.pdf`): "1 = Personas en situación de
  calle, 2 = Viviendas particulares, 3 = Viviendas colectivas". A new tower is a private dwelling, so the site model
  uses the 7,345,647 universe. `communes.csv` `population` is the 7,400,741 total.
- **Check, the chain of the contract closes.** `dwellings × occupied_share ×
  persons_per_occupied_dwelling` reproduces `persons` with an error under 15 persons for
  every comuna; the error is only the 4 decimals of `occupied_share`.
  `dwellings_occupied × persons_per_occupied_dwelling` closes to 5.2 persons or less.
- **Check against the published files.** The commuters of the 52 rows sum to 2,740,454
  with the collective and street rows added, the exact total of `work_od.csv`, and to
  2,688,814 inside the RM. Las Condes sums to 107,707, the denominator of the 31.4%
  public-transport share of `mode_share.csv`.
- **Which rate the site model must take: `commuters_rm_per_resident`, not
  `commuters_all_per_resident`.** `site.rs` receives only the RM-internal OD row of Las
  Condes and normalizes it. If it gets the all-destination rate, the 2,715 Las Condes
  commuters who work in another region are pushed into RM destinations.
- **Why `commuters_rm` is below the origin sum of `work_od.csv` in 49 of the 52
  comunas** (Las Condes 104,918 against 104,980, a difference of 62; Providencia 62,630
  against 62,694; Santiago 211,193 against 211,526). `work_od.csv` counts every person
  of the census with a known work comuna, in any dwelling kind. This file counts only
  the persons in private dwellings, because a new tower is a private dwelling. The 62
  Las Condes commuters that are out live in a collective dwelling (53) or are in the
  street operative (9). Both files are correct; they are two universes.
  **Name the universe in each `site.rs` request.** 104,918 / 293,272 = 0.3577 (private
  dwellings) and 104,980 / 296,134 = 0.3545 (every dwelling kind) are both real, and two
  runs of "the same" scenario disagree by 0.9% if the request does not say which one.
- Occupied dwellings include dwellings that are occupied with the residents absent:
  121,263 against 113,948 with a person record in Las Condes. That is why
  `persons_per_occupied_dwelling` (2.4185) is below `persons_per_dwelling_with_residents_present`
  (2.5737). Use the first one, because `occupied_share` has the same denominator.
- **Limits.**
  - These are the comuna values of 2024. For a new tower in the year it fills, they are
    a proxy, not a measurement of that tower. The contract still needs
    `attendance_home`, `attendance_work`, `filled_job_share`, the Metro-entry share and
    the access share as stated assumptions; this file supplies no value for them.
  - `persons_per_occupied_dwelling_apartment_blocks` comes from the block table, which
    counts persons of every dwelling kind, so the 2.0847 cannot remove a collective
    dwelling inside those blocks. It is 0.97% of the persons of the RM, so the effect
    is small, but it is not zero.
  - The 90% apartment filter selects the existing apartment stock of the comuna, which
    is older and smaller than a new tower.
  - `commuters_rm_per_resident` counts the usual work place, not a trip on a given day,
    and it misses the 313,358 RM people with no fixed place of work (`p44_lug_trab` 5).

### Metro share of the boardings near the site

Command: `python3 -I tools/pack/site_inputs.py metro` (2 s). It writes no file; it prints
the number. The contract needs a Metro share inside public transport and
`mode_share.csv` cannot give one, because Census code 2 joins metro, bus, train, taxi
and colectivo in one answer.

**Average weekday of April 2026, the stops within 1 km of the MUT manzana centroid
(-70.600974, -33.417357): Metro 151,406.4 boardings of 199,360.6, a Metro share of
75.95%.** Metro: Tobalaba `TB-TOB` 81,612.8 at 122 m, Los Leones `LE-LEN` 54,840.0 at
814 m, El Golf `GO` 14,953.6 at 451 m. Bus: 47,954.2 over 44 stops, and all 44 GTFS
stops in the radius have a weekday record.

**What this number is and is not.** It is an observed intensity of the boardings near
the site: of each 100 validated boardings in that circle, 76 are at a Metro turnstile.
It is **not** the probability that a traveller chooses Metro, and
`metro_entry_share_home` and `metro_entry_share_work` of the contract stay user
assumptions. Three facts break the equivalence: a boarding counts a transfer, so one
person can be counted two times; bus and Metro do not have the same validation
coverage; and the circle holds every rider of three stations, not the residents of one
block. Show the 75.95% as evidence that this place is Metro-first, not as a coefficient.

- Sources: DTPM `Subidas_Paradero_Estacion_2026.04_v3.xlsx`
  (`data/raw/dtpm/subidas_2026_04.xlsx`, sheet 3, `Tipo_dia=LABORAL`) and
  `GTFS_20260321_v3.zip` for the stop points
  (`data/raw/dtpm-gtfs-march/`, sha256 `f10945e8…b77b`, equal to the hash that
  `tobalaba_observations.json` publishes; it is the release that `context/mvp.md`
  names). Bus rows join on `Paradero Usuario` to `stop_id`. Metro rows use only the 3
  reviewed joins of PR #23 and PR #29.
- **Unmatched totals, both directions.** All 44 bus stops of the circle have a weekday
  record. In the other direction, 790.4 of the 2,512,170.4 RM weekday bus boardings
  (0.03%) carry a user code that the March GTFS does not hold at all, so they have no
  point; none of them is in Las Condes or Providencia. The November 2026 release
  (`gtfs_20261114.zip`) gives the same 44 stops in the circle and the same 0.7595, but
  it leaves 4,006.6 boardings with no point instead of 790.4. The two largest of **that
  November unmatched set** are `PC275` (Providencia, 685.0) and `PC727` (Las Condes,
  95.2); the March release does hold both, at 2,849 m and 4,689 m, far outside the
  circle. So no unplaced boarding can change the 75.95%, and the builder uses the March
  release because it places more of them.
- The 1 km radius needs no unreviewed join: the next Metro parents are Alcántara at
  1,073 m, Cristóbal Colón at 1,328 m, Pedro de Valdivia at 1,477 m and Escuela Militar
  at 1,678 m, all outside. No Tren stop is in the radius.
- **Limits.**
  - A boarding counts a transfer. Tobalaba is a bus-to-Metro hub, so a rider who
    arrives by bus and enters the Metro is counted two times, one time in each mode.
    The share is a station measure, not the main mode of a person.
  - DTPM measures the validated bus and Metro system. Taxi and colectivo are inside
    Census code 2 and have no boarding record, so 75.95% is not "Metro inside code 2".
    It is the Metro share of the validated boardings.
  - The radius crosses into Providencia. Los Leones is 814 m away and has its own
    catchment. Without Los Leones the share is 66.82%.
  - It is the whole weekday, every rider, not the residents of the site.
    It is newer than EOD 2012 and local, and that is why it is worth more than a 2012
    weighted share, but it does not measure how a new resident of the tower travels.

## jobs.csv

Work destinations of the simulation. One row per Census 2024 manzana that has jobs, plus one row for the airport. Builder: `python3 -I tools/pack/jobs.py [PACK_DIR]` (15 s).

| Column | Meaning |
| --- | --- |
| `block_id` | INE `MANZENT` code of the manzana, as text. 2,489 are not in `blocks.csv` (916,985 jobs): most manzanas with 0 residents are not there. `13124_airport` is not a manzana. Do not join by id; use `longitude`, `latitude` |
| `cut` | 5-digit INE comuna code |
| `longitude`, `latitude` | Centroid of the manzana polygon, EPSG:4674 |
| `jobs` | Jobs in the manzana, 2024, modeled. Integer |
| `method` | Signals that put jobs there, joined with `+` |
| `quality` | `ok`; `known_low`: the real number is higher (1 row, Costanera Center); `known_high`: the number is probably too high (3 rows, see the limits below). The value of a `known_high` row is not corrected |

- Rows: 11,519. Jobs: 2,688,814. All 52 comunas of the RM.
- 55 rows (25,273 jobs) have the centroid 2 to 73 m outside the polygon of their own comuna (check by lucky-llama). `cut` is the INE code and is correct. Use `cut` for the comuna, not a point-in-polygon test.
- **Comuna totals are real.** The jobs of each comuna are equal to the column sum by `work_comuna` of the Census 2024 work matrix (`data/raw/censo2024-personas/od_work_rm.csv`). Providencia 225,827. Las Condes 305,601. Santiago 360,648. This total has only workers who live in the RM and gave a work comuna. The INE dashboard gives more (Santiago 433,378) because it has more answer types.
- **The place inside the comuna is a model.** Weight of a manzana = A + B, then scaled to the comuna total.
  - A, `eod2012`: workers of the EOD 2012 survey at their usual workplace point (`personas.csv`, `DirActividadCoordX/Y`, expanded with `Factor`). 22,703 sampled workers have a point. The point goes to the manzana that contains it, or to the nearest manzana edge within 100 m.
  - A, `eod2012_landmark`: the EOD geocoder put vague answers on a landmark, usually a Metro station (Tobalaba has 104 sampled workers on one point). A point with 10 or more sampled workers is a landmark (148 points, 12% of the EOD workers). Its workers spread over the exact-address workers within 500 m.
  - B, `permits2013_2025`: INE building permits, non-residential, 2013–2025, floor area / 40 m² for each job. This adds the buildings that came after the survey. One manzana and one use text count one time (the largest permit), because a project has many permits.
  - `floor_area`: 2 sites of the demo (Costanera Center, the MUT) get a fixed number from published floor area. See the sections below. The other manzanas of the comuna share the rest, so the comuna total does not change.
  - `persons_fallback`: 7 rural comunas have fewer than 50 sampled workers (Paine, Curacaví, Tiltil, San José de Maipo, Alhué, María Pinto, San Pedro). Their jobs spread by resident persons. 50,857 jobs, 1.9%.
- The airport: the terminal point has 126 sampled workers (14,859 expanded) and is rural land with no manzana. The first version dropped it and spread its jobs over the other manzanas of Pudahuel. Now it is the row `13124_airport`, 27,070 jobs, the largest row of the file. No published count was checked against it.
- **Association with Metro boardings on the training stations (not a held-out validation).** Jobs within 500 m of a Metro station against the observed weekday boardings from 16:00 to 21:00 (`metro_observations.json`, April 2026), on the 95 stations that are not in the held-out set of `tools/pack/check_score.py`: Pearson 0.76, Spearman 0.79. Within 800 m: 0.69, 0.79. Jobs of the nearest station with no distance limit: 0.33, 0.50, because the last stations of each line take the far jobs (the airport row goes to Plaza Quilicura, 6.9 km away). Use a radius, not the nearest station. This association did not carry to the held-out score: a 500 m radius for jobs was scored as layer L2b by lucky-llama and was worse (WAPE_bins 0.5202 against 0.4688 with 1,000 m), so the station model keeps 1,000 m. Against the morning boardings the correlation is 0, as expected for work places.
- Why these signals: they are the only ones on disk with coordinates. SII addresses have no coordinates and follow the head office. OSM and Overture places have no worker counts.
- What the method gets wrong:
  - The base is from 2012. A place that lost jobs after 2012 keeps them.
  - A building with a permit before 2010 that opened after the survey is missing: the permit file starts in 2010. Costanera Center was the known case, corrected below. Other cases can exist.
  - The sample is small: one sampled worker is about 120 jobs. Only 11,519 rows have jobs; a real small workplace with no sampled worker has 0. Use the file at a scale of about 1 km, not manzana by manzana.
  - 20% of the EOD workers have no usable workplace point (no answer, or coordinates 0,0). They are out of the signal. They are not random: workers with no fixed workplace are in this group.
  - 40 m² for each job is an assumption, equal for offices, shops, and warehouses. Permit area includes parking floors. 29% of the permit area does not count (outside a manzana, or a second permit of the same project). So a large mall or warehouse permit is probably too high: Mall Barrio Independencia `13108071002002` has 5,330 jobs from a permit of 233,382 m²; Megacentro Buenaventura `13301031001050` has 5,107, most of them from a permit of 99,887 m² of storage. Both are `known_high`.
  - A row with only `eod2012` and a large number is a few sampled workers on one block. `13120111005004` (Leopoldo Urrutia, Ñuñoa, a residential block) has 3,650 jobs and is `known_high`: it gives the station Monseñor Eyzaguirre 29% of its job mass.
- **Known gap: visitors.** `jobs.csv` has workers only. A mall also brings shoppers, and no file here counts them. Checked on 2026-11-21 for a retail-floor-area-by-block input: among the files checked, the INE permits are the only source with use, area and a point (4,708 permits with a retail word, 7.76 million m², 2010–2025), but the use text is mixed and the area is not split (the MUT permit is 161,443 m² for shops, offices and parking), the file does not cover original construction before 2010, including older malls, and a project has several permits. `sii_f2890` is sale transfers and `sii_ebbrrc` is counts by comuna: no use and no area by block. So no retail file and no visitor layer were built. The MUT visitor numbers in this catalog are operator claims (section below).

### Top 20 rows, with the place a person from Santiago knows

Names from OSM Nominatim at the centroid. 17 are known work places. 2 are probably too high (permit limit above). 1 is doubtful.

| Jobs | `block_id` | Place |
| --- | --- | --- |
| 27,070 | `13124_airport` | Airport, terminal |
| 14,510 | `13101021005026` | Center, Bandera (banks) |
| 8,138 | `13123031001002` | Providencia, Av. Nueva Providencia |
| 7,514 | `13101021005003` | Center, Estado with Pasaje Matte, next to Plaza de Armas |
| 7,272 | `13101021005024` | Center, Matías Cousiño |
| 6,580 | `13114131005017` | El Golf, Reyes Lavalle |
| 6,080 | `13123081003003` | Providencia, Barrio Tajamar |
| 5,904 | `13114011003001` | Nueva Las Condes, Cerro El Plomo |
| 5,863 | `13123081002004` | Costanera Center (`floor_area`, `known_low`) |
| 5,665 | `13114131009016` | Sanhattan, Isidora Goyenechea |
| 5,649 | `13123031003001` | Providencia, Barrio Los Leones |
| 5,330 | `13108071002002` | Mall Barrio Independencia (probably too high) |
| 5,320 | `13114011001012` | Av. Kennedy at Parque Araucano |
| 5,313 | `13125021003018` | Quilicura, industrial zone of Lo Echevers |
| 5,240 | `13132011002004` | Vitacura, Lo Castillo next to Vespucio |
| 5,215 | `13114131008007` | El Golf, Av. Apoquindo |
| 5,177 | `13123011001022` | Hospital del Salvador |
| 5,107 | `13301031001050` | Megacentro Buenaventura, warehouses (probably too high) |
| 4,942 | `13101021008002` | Center, Paseo Puente, north of Plaza de Armas |
| 4,910 | `13120101001020` | Ñuñoa west edge, Av. Matta. Doubtful: no known large employer; part of the number is a landmark share |

Ciudad Empresarial is not one row: the rows within 700 m of its center add 18,137 jobs. The MUT (4,679) is row 26.

### Ratios for `floor_area`

- Offices: 10 m² of usable area for each worker. Source: OGUC art. 4.2.4, occupant load of offices (a design capacity, not a count of workers; read from copies of the table, not from the official text). The MUT claim gives the same ratio: 4,000 people in 40,000 m².
- Mall: 46 m² of leasable area for each job. Source: 4.61 million m² of leasable area in Chilean shopping centers ([DF, Q3 2024](https://www.df.cl/empresas/retail/centros-comerciales-crecen-4-6-en-ingresos-y-3-8-en-visitas-a)) / about 100,000 jobs ([CNC and Cámara de Centros Comerciales, 2021](https://cnc.cl/gremios-del-comercio-presentaron-primer-estudio-de-caracterizacion-de-trabajadores-de-centros-comerciales/)). The two numbers are from different years.

### Costanera Center

- Manzana `13123081002004`, Providencia. The 2012 signal gave 1,259 jobs. Now **5,863**, `known_low`.
- Offices: 29,500 m² occupied in March 2023, of 92,000 m² for lease in the two towers ([La Tercera, 11 March 2023](https://www.latercera.com/pulso/noticia/un-tercio-en-ocho-anos-la-dificil-comercializacion-de-la-torre-costanera/KMW6QMV5Z5B53DWMNPJ4YQLOSQ/)). 29,500 / 10 = 2,950 jobs.
- Mall: 134,000 m² of leasable area ([Wikipedia](https://es.wikipedia.org/wiki/Cenco_Costanera), which cites La Tercera, 2012). 134,000 / 46 = 2,913 jobs.
- Why `known_low`: the office number is the occupancy of 2023, and more floors have tenants now. With all 92,000 m² in use the total is 12,113. The hotel (14,000 m²) is not in the number. No operator count of workers was found.
- The 4,604 added jobs come from the other manzanas of Providencia. Each lost about 2%.
- The point of this row is the centroid of the block, like every other row. The centroid is 409 m from Los Leones and 446 m from Tobalaba, and the block edge is 213 m from a Los Leones entrance and 232 m from a Tobalaba entrance (March GTFS). So a nearest-station rule sends all 5,863 jobs to Los Leones, and no measured share between the 2 stations exists. A test that moved the point toward Tobalaba moved 5,850 afternoon boardings of error from one station to the other and did not reduce it; it was reverted. A rule that shares a block between the stations within the radius is the correct mechanism.

### The MUT (Mercado Urbano Tobalaba)

- Location: Av. Apoquindo 2730, Las Condes. Manzana `13114131009033`, centroid -70.600974, -33.417357, 15,300 m², 0 residents. Opened in 2023.
- `site_jobs_baseline`, 3 numbers:

| Number | Value | Source |
| --- | --- | --- |
| First model | 7,124 | Permit 4,643 (161,443 m² / 40, with parking floors) + 1,469 from 2012 addresses + 1,012 from the Tobalaba landmark. The 2012 part is wrong for this site: it was a construction pit in 2012 |
| Operator claim | 4,000 | "4 mil personas" in the offices, a project estimate ([El Diario Inmobiliario](https://eldiarioinmobiliario.cl/noticias/nacional/mut-relacion-de-centro-comercial-con-sustentabilidad/)). Offices only; no shop workers |
| Floor area | **4,679** | Offices 45,765 m² ([La Tercera](https://www.latercera.com/pulso-pm/noticia/proyecto-mut-suma-otro-gran-inquilino-metlife-consolidara-sus-oficinas-corporativas-en-6-pisos-de-la-torre-1/HNS3YVGCAVB7RP45GW2A453NXU/)) × 88% leased / 10 = 4,027. Shops 40,000 m² × 75% leased / 46 = 652. Shop area and leased shares of July 2024 ([DF MAS, copy at Valuaciones](https://valuaciones.cl/mut-saca-cuentas-alegres-en-su-primer-ano-ocupacion-de-locales-comerciales-es-de-75-y-de-oficinas-88/)) |

- **Default of the control "Sacar el MUT": 4,679, with the label "modelado".** It is now the value in `jobs.csv`. Reasons: it has no 2012 signal, it has offices and shops, and it agrees with the operator claim for the offices (4,027 against 4,000). Limits: "leased" is not "open" (in July 2024, 60% of the offices and 35% of the shops were open); with all the area in use the number is 5,446.
- Other operator and press claims, not measured:
  - Shops: more than 200 in operation ([CNN Chile](https://www.cnnchile.com/pais/territoria-centros-comerciales-mut-chile/)). No published count of shop workers.
  - Visits: "cerca de 15.000 personas diarias" in July 2024 (DF MAS, link above). "Más de tres millones de visitas en lo que va de año" in September 2025 ([Meganoticias](https://www.meganoticias.cl/nacional/498737-nuevos-mut-en-que-comunas-territoria-mercado-urbano-dcv-pdp-12-9-2025.html)), about 12,000 for each day. `context/research/data-sources-buildings-activity.md` has a different press claim: about 650,000 for each month, about 21,000 for each day. The claims do not agree.
- Jobs within 500 m of the Tobalaba station in `jobs.csv`: 63,052, modeled.

## buildings.geojson

3D buildings around Metro Tobalaba. Box: lon -70.635 to -70.565, lat -33.445 to -33.395.
31,516 polygons, 9.80 MB. Builder: `tools/pack/buildings.py` (stdlib, 3 s).

OSM puts many tower heights on `building:part` (Overture `building_part`, 223 parts in the box). Each building is drawn at its own height. Then each part that starts at ground level (`min_height` empty) and is taller than its building is added on top, so towers rise from their podiums. 4 parts with `min_height` > 0 (bridges, upper floors) are skipped.

Properties: `height_m`, `height_source`, `name` (if any), `is_mut` (MUT base and its 4 towers).

`height_source`, in priority order:
- `tag`: Overture `height` (from OSM `height`). 210.
- `levels`: Overture `num_floors` (from OSM `building:levels`) × 3.5 m. 8,551.
- `permit`: max `NUM_PISOS` × 3.5 m of INE permit points inside the footprint. 300.
- `exposure`: `stories_de` × 3.5 m of the Santiago residential exposure model (GitHub aleaicv/ExposureModels, release santiago_chile_ch_2025, reference year 2023, residential buildings with 5 or more stories), when the vertex mean of a model polygon is inside the footprint. 1,052. This is a model value, not a measurement. Check against OSM levels on 1,748 footprints with both: median difference 2 floors, 52% within 2 floors.
- `default`: 6 m placeholder (2 floors). 21,403. Footprints under 55 m² with default height are dropped to keep the file under 10 MB.

Non-default height, measured or inferred: 32% of footprints.
Tallest: Gran Torre Costanera 300 m, Titanium La Portada 200 m, Torre Telefónica 143 m.
MUT: base "Mercado Urbano Tobalaba" 17.5 m (5 levels); Torres 1, 2, 3 = 61.2, 64.6, 57.8 m (tags); Torre 4 has no data (6 m default).

Sources: Overture Maps buildings and building parts, release 2026-11-18.0 (OSM, Google Open Buildings, Microsoft footprints); INE permits; [exposure model](https://github.com/aleaicv/ExposureModels/releases/download/santiago_chile_ch_2025/exposure_santiago.zip). Overpass did not answer from our network, so Overture replaced OSM Overpass. The Overture Parquet download is the only non-stdlib step:

```
uv run --with overturemaps overturemaps download --bbox=-70.635,-33.445,-70.565,-33.395 \
  -f geojson --type=building -o data/raw/overture_buildings/tobalaba.geojson
uv run --with overturemaps overturemaps download --bbox=-70.635,-33.445,-70.565,-33.395 \
  -f geojson --type=building_part -o data/raw/overture_buildings/tobalaba_parts.geojson
mkdir -p data/raw/exposure_santiago
curl -L -o data/raw/exposure_santiago/exposure_santiago.zip \
  https://github.com/aleaicv/ExposureModels/releases/download/santiago_chile_ch_2025/exposure_santiago.zip
unzip -d data/raw/exposure_santiago data/raw/exposure_santiago/exposure_santiago.zip
python3 -I tools/scrape/ine_permits.py   # writes data/raw/ine_permits/rm_permits.json
python3 tools/pack/buildings.py
```

## travel_times.csv

Public-transport travel time between the comunas of the RM at 08:00 on a weekday
(depth layer B). Built so a station model can weight a flow by its cost, not only by
its distance. Builder: `python3 -I tools/pack/travel_times.py` (7 s).

| Column | Meaning |
| --- | --- |
| `origin_cut`, `destination_cut` | 5-digit INE comuna codes. **Directed**: A→B and B→A are two rows and they differ |
| `minutes_door_to_door` | walk to the stop + wait + ride + transfers + walk to the destination point |
| `minutes_in_vehicle` | only the time on board. Free of the headway assumption below |
| `transfers` | number of boardings minus 1 |
| `walk_m_access`, `walk_m_egress` | straight-line metres walked at each end |

- Source: DTPM `GTFS_20260321_v3.zip` (`data/raw/dtpm-gtfs-march/`, sha256
  `f10945e8…b77b`, equal to the hash that `tobalaba_observations.json` publishes). It is
  the release that `context/mvp.md` names. Points: the population-weighted comuna points
  of `communes.csv`.
- Rows: **1,190**, every directed pair of the 35 comunas whose point has a stop within
  1,000 m. Median 48.4 minutes, from 12.8 to 123.6.
- Check against trips a person from Santiago knows: Santiago → Las Condes 33.8 min
  (21.1 on board, 1 transfer); Las Condes → Santiago 28.9; Puente Alto → Santiago 48.5
  (30.4 on board, 2 transfers); La Pintana → Santiago 50.1; Maipú → Santiago 41.7. The
  matrix is **not symmetric**, as a directed matrix must not be.
- `minutes_in_vehicle` is never above `minutes_door_to_door`, no value is negative, the
  largest access walk is 992 m and the largest egress walk 857 m, both inside the
  declared 1,000 m.

### What the model is

- Weekday = Wednesday 2026-04-15, inside the period of the April boardings. Services `L`
  (Monday to Friday) and `LJ` (Monday to Thursday) are active; `V` is Friday only; no
  `calendar_dates` exception falls on that date.
- **It is a headway model, not a timetable search, and the feed forces that.** All 14,436
  bus trips (`route_type` 3) are in `frequencies.txt`, so their `stop_times` start at
  `0:00:00` and are offsets from an unknown trip start. 11,528 Metro trips have absolute
  times. A timetable search would have to invent a departure clock for 55.5% of the trips.
- So the 10,601 weekday trips collapse to **891 stop patterns**. The sample trip of a
  pattern is the one whose first departure is nearest 08:00: a frequency trip carries
  offsets, so every trip of its pattern gives the same numbers, but a Metro trip has
  absolute times and the trip near 08:00 is the right sample.
- A rider who stays on board pays the **dwell** at each intermediate stop. The ride state
  of a place in a pattern is split in two, "arrived" and "ready to depart", and staying
  on board costs that stop's `departure − arrival`. Check on the L1 pattern
  `DL_L1_101_113826_V1`: the rides alone are 1,614 s, the intermediate dwells are 637 s,
  and the two together are 2,251 s, which is exactly the end-to-end time of that trip in
  the feed. One boarding costs
  **headway / 2 at 08:00**: from `frequencies.txt` for a bus, and from the count of trips
  that depart between 08:00 and 09:00 for Metro. **702 of the 891 patterns run at 08:00**;
  the other 189 have no service then and are excluded, not given an invented headway.
- Access and egress: straight line to a stop within 1,000 m at 4.8 km/h. A transfer is a
  walk under 300 m, or 0 m between a platform and its `parent_station` (this feed has no
  `transfers.txt`). One Dijkstra for each of the 35 served origins, 86,249 nodes, 245,166 edges.

### Limits

- **The feed covers Gran Santiago, not the whole region. 17 of the 52 comuna points have
  no stop within 1,000 m, and they get no row**: Buin 1.5 km, Paine 2.1, Calera de Tango
  3.9, Pirque 5.3, Peñaflor 6.4, Lampa 6.5, Colina 10.3, Talagante 13.5, San José de
  Maipo 14.4, Isla de Maipo 14.5, Curacaví 20.3, El Monte 21.3, Tiltil 24.9, María Pinto
  24.9, Alhué 36.9, Melipilla 38.5, San Pedro 67.6. Their rural services are not in this
  GTFS. **1,462 of the 2,652 directed pairs are missing for that reason.** An earlier
  version kept those pairs with their nearest stop, which reported a 67 km walk as a
  travel time. That is gone: a pair with no stop inside the limit has no row.
- The headway gives an **expected** wait, not a real 08:00 timetable. A pattern that runs
  every 90 minutes reads as a 45-minute wait. `minutes_in_vehicle` has none of that, which
  is why it is a separate column.
- Walking is a straight line, not a network, so every walk is shorter than the real one.
- 08:00 only. No fare, no crowding, no capacity, no delay, no car, no bicycle, and no
  diagonal (a comuna to itself).
- **A comuna point is not a trip end.** These are times between two population centres.
  No value here is fitted to an observed travel time.

### Check against EOD 2012, the only observed travel times we hold

Command: `python3 -I tools/pack/travel_times.py eod` (it rebuilds the matrix, then
compares, so the comparison always runs against the current file).

**This is descriptive, not a measure of forecast accuracy.** The two differ in period
(2012 against 2026), in cohort (surveyed trips against a point-to-point search) and in
trip ends (a real door against a comuna centroid). What it can say is whether the matrix
orders the comuna pairs the way the survey did.

I compared the matrix with the real trip durations of EOD 2012
(`data/raw/eod2012/.../viajes.csv`, `TiempoViaje`). Filter: weekday factor present,
`Periodo` 2 (the 07:31 to 09:00 peak, which brackets the 08:00 departure),
`ModoAgregado` in {2, 3, 4, 7, 9, 10} (bus and Metro and their pure combinations, no
taxi, no colectivo, no car), a reported duration above 0 and not above 300 minutes,
origin comuna different from destination comuna, and the pair present in this file. That gives **3,184 sampled trips on 620 pairs**;
I kept the **80 pairs with 10 or more sampled trips** and took the **unweighted** median
of the sampled durations of each pair. `FactorLaboralNormal` is used only to select
weekday trips, never as a weight, so a pair median is a median over sampled trips and
not over the expanded population.

| | result |
| --- | --- |
| EOD 2012 observed median | 53.8 min |
| this matrix | 35.5 min |
| ratio mine / EOD, median | **0.651** (p10 0.452, p90 0.867) |
| mean absolute error | 20.9 min, WAPE 36.4% |
| Pearson r | **0.783** |
| Spearman | **0.734** |

**Spearman uses the average rank inside a group of equal values.** The survey medians
include ties because EOD reports round minutes, so the
no-ties shortcut `1 − 6Σd²/(n³−n)` is not Spearman here and its answer depends on the
order of the pairs. The number above is the Pearson correlation of the tie-corrected
ranks; the shortcut gave 0.744.

Before the dwell was charged, the same check gave 34.1 min, ratio 0.608, WAPE 40.6% and
Pearson 0.739. Charging the dwell moved every one of those numbers toward the
observation. That is an independent sign that the fix was right.

**Read it this way: the ranking is good and the level is 35% too fast against this
survey.** Use the file as a relative cost between pairs, not as a minute count a person
would recognise.
A model that puts these minutes in a decay function with one freely fitted coefficient
absorbs a **common multiplicative** factor, so a uniform level bias would not reach its
score. That is only part of the story here: the ratio runs from 0.45 at p10 to 0.87 at
p90, so the bias is not uniform, and the pair-by-pair part does reach any such score.
What survives is the ordering: Spearman 0.734.

Four causes of the gap, and they do not all point the same way:
1. A comuna point is not a trip end. An EOD trip runs door to door anywhere inside the
   comuna; this matrix runs between two population-weighted centroids, which is shorter
   than an average real pair. This is probably the largest remaining term.
2. Walking is a straight line, so every walk here is shorter than the real one.
3. `headway / 2` assumes a passenger arrives at the stop at random.
4. The network changed after the survey: L6 opened in 2017 and L3 in 2019. The pairs
   where this matrix is fastest against EOD are pairs those lines serve — Quilicura →
   Las Condes, Maipú → Las Condes and Renca → Las Condes each read about half of their
   EOD time. **This check cannot attribute the gap**, because causes 1 to 3 act on those
   pairs too. The pattern is consistent with a real improvement; it does not measure
   one, and it does not establish that EOD is the wrong number there.

I apply **no correction factor**. The 1.54 that would close the level is a fit to EOD,
and a layer must freeze its formula before it is scored.

## Direct bus access to Metro stations

`bus_station_access.csv` gives the shortest scheduled access time from a bus stop to each
Metro parent that riders can reach on one bus. The file has 71,857 stop-parent pairs,
9,476 origin stops and 126 Metro parents. Each row gives the origin stop ID, longitude,
latitude, Metro parent ID and seconds. The scenario adds the walk to the origin stop.

The builder uses the March 2026 DTPM feed and the existing `bus_metro_300m.csv` transfer
links. It uses the same Wednesday and 08:00 service window as `travel_times.csv`.
All 666 active bus trips specify headways. For each downstream stop, the cost includes
half the headway, the time from departure at the origin to arrival at that stop, and a
straight walk to the parent at 4.8 km/h. Each pair keeps its lowest cost, in seconds
rounded to three decimal places. [DTPM feed](https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip) · [GTFS frequencies](https://gtfs.org/documentation/schedule/reference/#frequenciestxt)

For example, PA1 to AN-NA via PA298 takes 878.090 seconds on trip pattern 505-I-L-B04.
The wait is 207.5 seconds, the ride takes 534 seconds, and the transfer walk is 182.12 metres.

To model a station closure, exclude its parent ID before selecting a path. Of 247
origin stops whose fastest path ends at FER, LCR or PQU, 144 switch to LIB when those
three close, 34 switch to other stations and 69 lose all direct access by bus.
These counts describe scheduled access in the 2026 bus network. The table has no
passenger counts, Metro ride times or historical bus routes.

```sh
python3 -I tools/pack/bus_station_access.py --gtfs /path/to/GTFS_20260321_v3.zip


### Walking and cycling by distance

`active_mode_distance.json` estimates the probability that a 2012 Santiago trip to work
used a bicycle, conditional on the traveler walking or cycling. For straight-line distance
`d` in km, the estimated log odds of cycling are `intercept + slope_log1p_km × ln(1 + d)`.
Use this distribution to assign walking and cycling within an existing active-mode total.
The fit describes mode choices at different distances in 2012. A forecast of a cycleway's
effect needs evidence from a change in the network.

The builder uses the EOD fields `Proposito=1` and `ModoAgregado=17,18`. It keeps positive
normal-weekday expansion weights and positive, non-imputed `DistEuclidiana` in metres.
It multiplies the trip and person normal-weekday weights. It excludes 845 imputed distances
and 113 nonpositive distances. The fit covers 780 walking and 560 cycling observations from
1,118 households, equivalent to 210,874.42 walking and 122,876.47 cycling trips.
[Survey](https://www.sectra.gob.cl/info-territorial/region-metropolitana-de-santiago/) · [INE expansion method](https://www.ine.gob.cl/docs/default-source/sistema-de-indicadores-y-estandares-de-desarrollo-urbano/manuales/actualizaci%C3%B3n-2019/manual-procedimientos-siedu-act2019.pdf)

Five folds keep households together. Each household's group is the first eight bytes of
its SHA-256 digest as a big-endian integer, modulo five, using the UTF-8 source ID.
Each fit uses four folds. Its reference uses the weighted bicycle share of those same
four folds. Pooled weighted log loss in the held-out folds fell from 0.660953 to 0.536681.
One fold had higher loss with distance, 0.663965 versus 0.645002. The full-sample model
uses all five folds after the pooled comparison met its acceptance rule.
[Preregistration](https://discord.com/channels/1573514576400752640/1573579088684253235/1573676569355092079)

```sh
python3 -I tools/pack/active_mode_distance.py --survey /path/to/EOD_STGO
```

## jobs_2026.csv

Experimental destination weights for each job point, in the order of `jobs.csv`. The `jobs` column contains a relative weight. Keep the census totals for each comuna when allocating workers. The weights reflect transit use as well as job density.

Run `python3 -I tools/pack/jobs_2026.py` with the [April 15 arrival groups](../validation/trip_purpose_2026_04/README.md). The builder assigns each point to its nearest training station within 500 m. For each of the 95 training stations it divides the station's share of expanded work arrivals by its share of assigned job mass. That ratio multiplies the point's original weight. Points outside the radius keep their weights. The builder uses no boardings or held-out arrivals.

The frozen L4b run used the candidate file as its jobs input in an isolated directory. Across 31 held-out stations and 1,123 observed half-hours, absolute error fell from 254,347.44 to 252,057.91 boardings. Half-hour WAPE fell from 0.4459345830 to 0.4419204606. Tobalaba, a training station, rose from 41,702.00 to 56,543.75 predicted boardings across the observed bins. The observed sum was 81,612.80.

## transit_paths.json

The city view loads `web/data/transit_paths.json`. The pack links to the same file.
Run `python3 -I tools/pack/transit_paths.py` from the repo root to rebuild it.
Source: [March 2026 Santiago GTFS](https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip).

Each Metro record gives the line, GTFS color, direction (0 or 1), ordered parent station IDs, and a path of longitude/latitude points.
The 48 values in `trips` count departures by half hour from 00:00 Santiago time on a regular Monday (service IDs L and LJ).
A train contributes once, at the first stop of its scheduled trip. Short-turn and express trips contribute to their direction's total.
The path shows the full line. It does not describe the stops of each train.

The 14 paths cover 7 lines. One direction per line gives 143 station entries, which matches the [Ministry of Transport network count](https://mtt.gob.cl/extension-de-linea-2-comenzo-su-operacion-y-beneficiara-a-618-mil-personas/).
Interchange lines can share a parent station. The feed has 126 distinct parent IDs across these paths.

Each bus record represents one GTFS route. Its path uses the shape with the most expected departures that day.
The builder keeps every third shape point and the last point. The route's `trips` values cover all directions and variants.
For each overlapping headway interval the builder divides its duration in the half hour by the published headway.
All bus intervals use `exact_times=0`. Fractional values are expected departures, rounded to three decimal places.
These counts describe scheduled service. They do not measure passenger demand or vehicle positions. [GTFS frequencies reference](https://gtfs.org/documentation/schedule/reference/#frequenciestxt)

The file contains 878 Metro path points and 68,715 bus path points across 416 bus routes. Size: 1,635,778 bytes.

## travel_times_blocks.csv

Run `python3 -I tools/pack/travel_times_blocks.py` with the March 2026 GTFS feed and Census 2024 blocks. The table gives modeled public transport minutes at 08:00 on a weekday. It contains 1,602 directed pairs across 41 of the 52 comunas. The builder samples up to six blocks per comuna from blocks with employed residents and a GTFS stop within 1 km. Trips within a comuna use different residential blocks at each end. Worker counts affect both the sample selection and the averaging weights. At 08:00 the other 39 origins reach both Buin and Paine. Buin reaches itself and Paine. Paine reaches only itself.

`origin_access_share` and `destination_access_share` give the fraction of resident workers within 1 km of a stop at each end. The minutes describe travel between the sampled blocks with access. `worker_pairs` counts routed block pairs.

Eleven comunas have no worker block within 1 km of a stop in this feed. These are San José de Maipo, Tiltil, Melipilla, Alhué, Curacaví, María Pinto, San Pedro, Talagante, El Monte, Isla de Maipo, and Peñaflor. They account for 211,532 workers. A time for these comunas requires a separate assumption. The table contains no car times. The reported median ratio to EOD 2012 survey times is about 0.65 and serves only as a descriptive comparison.

## roads.geojson

Major roads in the Santiago city view. Each GeoJSON LineString has the OSM way ID in `id`, the road class in `kind`, and the mapped `name` when available. Link roads use their parent class. A way that leaves and re-enters the view has several features with the same ID.

The view spans longitude −70.85 to −70.45 and latitude −33.65 to −33.30. It contains 18,649 ways and 18,651 paths, with 2,781 km of mapped carriageways before simplification. Opposing carriageways count separately. Paths use an 8 m simplification tolerance and five decimal places per coordinate. File size is 3,578,472 bytes.

Map credit: [© OpenStreetMap contributors](https://www.openstreetmap.org/copyright). Road data and derived geometry are available under the [Open Database License 1.0](https://opendatacommons.org/licenses/odbl/1-0/). The [Geofabrik Chile extract](https://download.geofabrik.de/south-america/chile/chile-261120.osm.pbf) records the OSM state on 2026-11-20 at 21:21:21 UTC. Its MD5 is `fef09ad7ee6f28c7afc925ef68c737a4`.

Place the extract at `data/raw/osm-roads/chile.osm.pbf`. Run the following commands from the repository root. The larger extraction box includes all 52 comuna reference points.

```sh
cd data/raw/osm-roads
osmium extract -b -71.59,-34.12,-70.23,-32.88 chile.osm.pbf -o region.osm.pbf
osmium tags-filter region.osm.pbf 'w/highway=motorway,trunk,primary,secondary,motorway_link,trunk_link,primary_link,secondary_link' -o santiago.osm
cd ../../..
python3 -I tools/pack/roads.py
```

## car_paths.json and car_minutes.csv

The two files cover all 2,652 directed pairs between **different** comunas in the RM. Each JSON record contains the string keys `origin_cut` and `destination_cut` and a `path` array of longitude and latitude pairs. Paths follow the OSM major-road network described above. They contain 147,558 points after 20 m display simplification. The CSV uses the same keys and gives free-flow `car_minutes`, road `distance_km`, and the two straight-line gaps to the road in `origin_snap_m` and `destination_snap_m`. Travel time and distance use the full geometry.

Each comuna uses its population-weighted reference point from `communes.csv`. We use the nearest primary or secondary road node to enter and leave the network. The median gap is 135 m. The largest is 2,976 m at Tiltil. The model omits local access time and same-comuna travel.

The route minimizes driving time. A road's assumed free-flow speed follows the [OSRM car profile](https://github.com/Project-OSRM/osrm-backend/blob/master/profiles/car.lua): 90, 85, 65 and 55 km/h on motorways, trunk, primary and secondary roads. Corresponding link speeds are 45, 40, 30 and 25 km/h. Lower numeric OSM speed limits cap each direction. The graph respects static one-way roads, roundabouts, and mapped car-access bans. It excludes roads tagged reversible or alternating. The model omits turn restrictions, conditional traffic rules, signals, queues and congestion. Peak trips can take longer.

Maipú to Las Condes takes **28.03 modeled minutes over 28.276 km**. The route passes through Avenida Los Pajaritos, Ruta 78, the Alameda, and Costanera Norte. The gaps to the major roads are 21 m in Maipú and 131 m in Las Condes. These are estimates for the two reference points.

Build both files from the same regional OSM extract with `python3 tools/pack/car_paths.py`. Display the map credit **© OpenStreetMap contributors** with a link to [the OSM copyright page](https://www.openstreetmap.org/copyright). The derived network data use ODbL 1.0.

The view reads `web/data/car_routes.json`. Each key joins the two CUT codes with a hyphen, for example `13119-13114` for Maipú to Las Condes. Each value contains the existing car path as an array of longitude and latitude pairs. The export keeps between-comuna routes with at least one worker in `work_od.csv`. Rebuild it with `python3 -I tools/pack/car_routes_view.py`.

## cycleways.geojson

The urban display box contains 2,157 paths and 643.5 km of mapped ways. The file is 488,457 bytes. Each feature gives its OSM way ID, name, kind and five-decimal WGS84 coordinates. A lane on both sides of one street contributes one street length. Separately mapped ways contribute one length each. Map credit: © OpenStreetMap contributors. [Geofabrik Chile extract, 2026-11-20 21:21 UTC](https://download.geofabrik.de/south-america/chile.html) · [Open Database License](https://www.openstreetmap.org/copyright)

For comparison, the July 2026 SECTRA inventory reports 565.414 km in 442 existing segments across the 34 Gran Santiago comunas. The query sums the KM field for ETAPA = existentes and CUT_COM in 13101–13132, 13201 and 13401. SECTRA uses comuna boundaries. The display uses a rectangle and includes tagged lanes as well as separate cycleways. The two inventories have different coverage and counting rules. [SECTRA inventory](https://services6.arcgis.com/feQ9HId8vmgonvvD/ArcGIS/rest/services/CICLOV_validVisor_WFL1/FeatureServer/0)

Rebuild with the regional PBF used for roads:

```sh
mkdir -p data/raw/osm
osmium extract --bbox=-70.85,-33.65,-70.45,-33.30 data/raw/osm-roads/region.osm.pbf -o data/raw/osm/urban.pbf
osmium tags-filter data/raw/osm/urban.pbf w/highway=cycleway w/cycleway=lane,track w/cycleway:right=lane,track w/cycleway:left=lane,track w/cycleway:both=lane,track -o data/raw/osm/cycleways.osm
python3 tools/pack/cycleways.py
```

## public_routes.json

The map loads `web/data/public_routes.json`. The pack links to the same file. Run `python3 tools/pack/public_routes.py` to rebuild it from the March 2026 GTFS.

Each key is a directed `origin_cut-destination_cut` pair from `travel_times.csv`. Each value contains a path of longitude/latitude points and the GTFS route IDs used, listed in first-boarding order within each mode. The keys are `metro_lines`, `bus_routes`, and `rail_routes` when the path uses EFE. The map has 1,190 pairs between 35 comuna centres. The other 17 centres have no GTFS stop within 1 km. Same-comuna paths are outside this matrix.

The builder traces the winning path of the existing weekday 08:00 router. It includes the initial walk, each transfer, each ride, and the final walk. Walks use straight segments at 4.8 km/h, up to 1 km at each end and 300 m per transfer edge. A platform and its parent station have zero transfer cost. Rides use a GTFS shape between the boarding and exit stops, or the ordered stop positions when a shape is absent. The display simplifies each ride at 25 m tolerance and rounds coordinates to five decimal places.

The router assigns one wait to a stop pattern. When several route IDs share one pattern, the display uses the active trip with the shortest headway, or the first scheduled departure in the 08:00 hour. These paths connect population-weighted comuna centres and show modeled travel on a representative weekday. They have no passenger-level endpoints.

Example: Quilicura → Santiago (`13125-13101`) uses bus 425 and Metro L3. The corresponding CSV row gives 37.4 minutes door to door, 25.6 minutes on board, and one transfer.

## Bicycle routes

`web/data/bicycle_routes.json` maps each directed key `originCut-destinationCut` to a path of WGS84 `[longitude, latitude]` points. Endpoints are the nearest routable bicycle locations to the population-weighted comuna points in `communes.csv`. The paths represent possible routes between these points. They do not measure cyclists or represent every street used by residents.

The cutoff uses great-circle distance between the original comuna points, strictly below 15 km. The file contains all 822 eligible directed pairs among 46 of the 52 comunas. The other six have no other comuna point inside that radius. Road distance ranges from 2.53 to 27.50 km. Of the 822 routes, 309 exceed 15 km on the road. Display paths use a 4 m simplification tolerance. Together they contain 63,065 coordinates and take 1,512,826 bytes.

`bicycle_route_lengths.csv` gives the direct distance, routed distance, and the gap from each comuna point to its route endpoint, in metres. Routed distance excludes both endpoint gaps. The median origin gap across directed routes is 11.2 m. Lampa has the largest gap, 716.9 m.

For example, Santiago → Providencia (`13101-13123`) is 4,499.8 m direct and 5,268.3 m by bicycle. The route uses the Antonio Varas, Bilbao and Pocuro cycleways. Endpoint gaps are 9.6 and 25.6 m. The return trip is 5,294.7 m. All 29,827 route instructions across the export have OSRM mode `cycling`.

Use the map credit [© OpenStreetMap contributors](https://www.openstreetmap.org/copyright). The route geometry and distance table are available under the [Open Database License 1.0](https://opendatacommons.org/licenses/odbl/1-0/). Source: [Geofabrik Chile, 20 November 2026](https://download.geofabrik.de/south-america/chile/chile-261120.osm.pbf).

### Rebuild

Use OSRM 26.11.0 with its bicycle profile and the project wrapper. The profile uses mapped bicycle access, barriers, one-way rules and cycle lanes. Its `cyclability` weight penalizes roads with less suitable conditions. The wrapper excludes motorways, trunks, steps and motor roads. It excludes footways, pedestrian streets and paths unless bicycle access is explicit. It removes directions that require pushing a bike. Temporary closures and missing access signs remain unknown. [OSRM bicycle profile](https://github.com/Project-OSRM/osrm-backend/blob/v26.11.0/profiles/bicycle.lua)

Set `OSRM_PROFILES` to the OSRM directory that contains `bicycle.lua` and `lib/`. From the repository root, run:

```sh
mkdir -p data/raw/bicycle
osmium extract -b -71.59,-34.12,-70.23,-32.88 data/raw/osm-roads/chile.osm.pbf -o data/raw/bicycle/region.osm.pbf
osrm-extract -t 2 -p tools/pack/bicycle_profile.lua -o data/raw/bicycle/region.osrm data/raw/bicycle/region.osm.pbf
osrm-partition -t 2 data/raw/bicycle/region.osrm
osrm-customize -t 2 data/raw/bicycle/region.osrm
osrm-routed -i 127.0.0.1 -p 5196 -t 2 -a mld data/raw/bicycle/region.osrm
```

From a second terminal, run `python3 -I tools/pack/bicycle_routes.py`.

## City block volumes

`web/data/city_blocks.json` contains one display volume for every row of `blocks.csv`, in the same order. Each row has six fields: `[longitude, latitude, side_m, height_m, cut, block_id]`. Coordinates use decimal degrees. Side and height use display metres. Both IDs are strings. The file has 52,646 rows, 52 comunas and 3,087,045 bytes.

For 52,593 rows, area comes from the 2024 INE polygon with the same `MANZENT` in `Manzanas_CPV24` or `Entidades_CPV24`. Its `SHAPE_Area` is in square degrees (EPSG:4674). The builder approximates square metres as `SHAPE_Area × (π × 6371008.8 / 180)² × cos(latitude)`, with latitude in radians. The square is centred on the point in `blocks.csv`. [INE RM cartography](https://storage.googleapis.com/bktdescargascenso2024/Cartografia/GPKG/Cartografia_censo2024_R13.zip) · [INE Census block table](https://storage.googleapis.com/bktdescargascenso2024/Datos_agregados/Base_manzana_entidad_CPV24.zip)

The remaining 53 rows are Census comuna containers. INE withholds their exact location. The volume uses the population-weighted point from `blocks.csv`, a fixed assumed area of 625 m² and a display side of 20 m. In this file these are exactly the block IDs ending in `999999`. They account for 42,018 dwellings. The tallest volume (78.3 m) is the Puente Alto container. Its location and size are illustrative. [INE cartography manual](https://storage.googleapis.com/bktdescargascenso2024/Datos_agregados/Manual_uso_cartografia_CPV2024.pdf)

The graphic represents activity density. Its dimensions are stylized. With area `A` in m², Census dwellings `D` and allocated job count `J`, the display rules are:

- `side_m = min(120, 0.8 × sqrt(A))`. The inset leaves space between neighbouring blocks. The cap limits large rural squares. It applies to 7,604 rows.
- `height_m = 1 + 5 × log2(1 + (D + 0.5 × J) × 10000 / A)`. A doubling of one plus weighted activity per hectare adds 5 display metres. Empty blocks have a 1 m base. Both dimensions round to 0.1 m. The median height is 30.2 m.
- `J` is the modeled weight at the same block ID in `jobs.csv`, or zero if absent. The match covers 1,771,829 jobs. Another 916,985 jobs on 2,489 rows have their own coordinates outside the home-block list. They remain in `jobs.csv`, including the airport and the 4,679 modeled jobs at MUT. Use the job layer for those locations.

To rebuild, run `python3 -I tools/pack/city_blocks.py` from the repository root. The default input is `data/raw/verify-people-work/cartography-rm.gpkg`. An optional argument selects another path to the same INE file.

## Central Santiago building footprints

The city view can load `web/data/city_footprints.json`. It contains **22,451 OSM building polygons** in a continuous 32.86 km² central area. The rectangle extends from −70.663 to −70.583 longitude and −33.448 to −33.408 latitude, from Santiago Centro through Providencia to MUT. Every exterior ring lies inside the rectangle. Mapped polygons cover 6.719 km² in total after subtracting courtyard holes. This sum can include overlapping buildings. OSM mapping coverage varies.

The whole urban view has about 279,000 mapped polygons. The 4,723,922-byte export keeps the central area to meet the view's 5 MB limit. Osmium assembles multipolygon relations before export. The pack includes 244 polygon components from 232 relations and retains 325 holes. The builder excludes footprints crossing the rectangle, untagged structures, and standalone parts with only a `building:part` tag. One outline lost its area at five-decimal precision and was omitted. Read each `buildings[].polygon` as an array of closed rings with `[longitude, latitude]` vertices at five decimal places. The first ring is the exterior.

Use `height_m` where present (178 records). An additional 5,959 polygons have tagged `levels` only. Estimate their display height as `levels * metres_per_level` (3.5 m). The remaining 16,314 polygons use `default_height_m` (6 m) for display. These estimates describe styling. The tallest tagged height in the pack is 300 m.

Credit the map as [© OpenStreetMap contributors](https://www.openstreetmap.org/copyright). The footprint data use the [Open Database License 1.0](https://opendatacommons.org/licenses/odbl/1-0/). Source: [Geofabrik Chile, OSM snapshot 2026-11-20 21:21 UTC](https://download.geofabrik.de/south-america/chile/chile-261120.osm.pbf).

To rebuild, place that extract at `data/raw/osm-roads/chile.osm.pbf` and run the following commands.

```sh
mkdir -p data/raw/osm-buildings-city
cd data/raw/osm-buildings-city
osmium extract -s smart -b -70.85,-33.65,-70.45,-33.30 ../osm-roads/chile.osm.pbf -o urban.osm.pbf
osmium tags-filter urban.osm.pbf 'wr/building' -o buildings.osm.pbf
osmium export -u type_id --geometry-types=polygon -f geojsonseq buildings.osm.pbf -o polygons.geojsonseq
cd ../../..
python3 -I tools/pack/city_footprints.py
```

## City tree points

`web/data/city_trees.bin.gz` contains one point per mapped `natural=tree` node. Inflate the gzip stream, then read four bytes per tree. The first two bytes encode unsigned little-endian x, and the next two encode unsigned little-endian y. There is no header. Longitude is `(x - 7085000) / 100000`, latitude is `(y - 3365000) / 100000`. This is the coordinate grid of the city building tiles.

All 77,296 tree nodes in the urban extract fall inside the inclusive box [-70.85, -33.65, -70.45, -33.30]. The 77,296 records occupy 309,184 bytes before compression. Quantization gives 77,266 distinct positions. Separate nodes can share the same rounded position. The map is an incomplete inventory of Santiago's trees.

Of these nodes, 3,185 (4.12%) have a `height` tag. Of the 3,185 values, 3,184 give a numeric height, with or without a metre suffix, and one says `single`. The binary contains positions only. The source also has 491 `natural=tree_row` ways. They represent lines of trees and do not supply an individual point count. The builder excludes them. [OSM tree tags](https://wiki.openstreetmap.org/wiki/Tag:natural%3Dtree) · [OSM tree row tags](https://wiki.openstreetmap.org/wiki/Tag:natural%3Dtree_row)

Run `python3 -I tools/pack/city_trees.py data/raw/osm-buildings-city/urban.osm.pbf` with osmium-tool installed. The input is the same urban extract used for the building mesh. Its SHA-256 is `4a1cfdb3ffdfdb58cbc6247ab36cac38ad2dc35509b5c1e102a7e53461234cb3`.

Credit the map as [© OpenStreetMap contributors](https://www.openstreetmap.org/copyright). The point data use the [Open Database License 1.0](https://opendatacommons.org/licenses/odbl/1-0/). [Geofabrik Chile snapshot, November 20, 2026](https://download.geofabrik.de/south-america/chile/chile-261120.osm.pbf)
