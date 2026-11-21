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

- Source: MINEDUC Directorio Oficial de Establecimientos 2026, `data/raw/mineduc-directorio/dir2026.rar`. Year 2026, cut 2026-04-30. Catalog entry: `context/research/sources/verify-people-work.md`.
- Filters: `COD_REG_RBD=13` and `ESTADO_ESTAB=1`. The file holds 4,467 RM rows; 3,227 are operating.
- Rows: 3,220. Enrolment: 1,309,286. Comunas: 52. Providencia (`13123`): 83 schools, 29,641 students.
- Dropped: 7 of the 3,227 operating schools. 6 have an empty coordinate, and RBD 41884 is at latitude -29.22, which is in Coquimbo. All 7 are nurseries with `MAT_TOTAL=0`, so the file loses 0 students. The box used is latitude -34.4 to -32.8 and longitude -71.8 to -69.7.
- `MAT_TOTAL` is the exact sum of `MAT_ENS_1` to `MAT_ENS_8`, so it **does** count pre-school: 104,953 of the 1,309,286 students are in `MAT_ENS_1` (parvularia). 1,663 of the 3,220 schools report enrolment in that level.
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
| `persons_per_occupied_dwelling_apartment_blocks` | the same ratio, over the blocks of the comuna where 90% or more of the private dwellings are apartments and `n_vp` is 10 or more | observed, subset |
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
- **The apartment pair with a comuna commuter rate is one transfer assumption, and the
  ratio method is another. Neither is a bound on a real tower.** The pair carries the
  smaller flat (2.0847 against 2.4185) but not the higher travel of the people in it.
  Measured with the rate 0.3545: the pair gives 0.6837 commuters for each dwelling and
  the ratio method gives 0.7342, which is 6.9% higher. The difference is between two
  models, not an error against a measurement, so **do not call either one conservative
  or a lower bound**. Label the one you send "supuesto de transferencia" and name it.
  The demo sends the apartment pair with 298 **physical** dwellings; do not fold a
  ratio into `dwellings_added`, which is a count of dwellings.
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
| `coverage` | `in_feed` when both comuna points have a stop within 1,000 m, else `outside_feed` |

- Source: DTPM `GTFS_20260321_v3.zip` (`data/raw/dtpm-gtfs-march/`, sha256
  `f10945e8…b77b`, equal to the hash that `tobalaba_observations.json` publishes). It is
  the release that `context/mvp.md` names. Points: the population-weighted comuna points
  of `communes.csv`.
- Rows: 2,454 of the 2,652 directed pairs. **1,190 have `coverage` `in_feed`**; use those.
  Median 45.5 minutes, from 12.8 to 114.7, median 58% of the time on board.
  Transfers: 79 pairs need none, 319 need one, 487 need two, 254 three, 51 four or five.
- Check against trips a person from Santiago knows: Santiago → Las Condes 27.4 min
  (14.6 on board, 0 transfers); Las Condes → Santiago 25.1; Providencia → Las Condes
  23.0; Puente Alto → Santiago 44.5 (26.4 on board, 2 transfers); La Pintana → Santiago
  48.9. The matrix is **not symmetric**: the median gap between A→B and B→A is 1.7
  minutes and the largest is 74.7.

### What the model is

- Weekday = Wednesday 2026-04-15, inside the period of the April boardings. Services `L`
  (Monday to Friday) and `LJ` (Monday to Thursday) are active; `V` is Friday only; no
  `calendar_dates` exception falls on that date.
- **It is a headway model, not a timetable search, and the feed forces that.** All 14,436
  bus trips (`route_type` 3) are in `frequencies.txt`, so their `stop_times` start at
  `0:00:00` and are offsets from an unknown trip start. 11,528 Metro trips have absolute
  times. A timetable search would have to invent a departure clock for 55.5% of the trips.
- So the 10,601 weekday trips collapse to **891 stop patterns** (46,668 pattern edges).
  The ride time on an edge is the exact offset difference. One boarding costs
  **headway / 2 at 08:00**: from `frequencies.txt` for a bus, and from the count of trips
  that depart between 08:00 and 09:00 for Metro. **702 of the 891 patterns run at 08:00**;
  the other 189 have no service then and are excluded, not given an invented headway.
- Access and egress: straight line to a stop within 1,000 m at 4.8 km/h. A transfer is a
  walk under 300 m, or 0 m between a platform and its `parent_station` (this feed has no
  `transfers.txt`). One Dijkstra for each of the 52 origins, 49,874 nodes, 208,089 edges.

### Limits

- **The feed covers Gran Santiago, not the whole region. 17 of the 52 comuna points have
  no stop within 1,000 m**: Pirque 5.3 km, San José de Maipo 14.4, Colina 10.3, Lampa
  6.5, Tiltil 24.9, Buin 1.5, Calera de Tango 3.9, Paine 2.1, Melipilla 38.5, Alhué 36.9,
  Curacaví 20.3, María Pinto 24.9, San Pedro 67.6, Talagante 13.5, El Monte 21.3, Isla de
  Maipo 14.5, Peñaflor 6.4. Their rural services are not in this GTFS. Those rows keep
  their nearest stop so nothing is hidden, but the walk becomes the whole trip (San Pedro
  to Alhué reads 1,353 minutes). **Filter on `coverage = in_feed`.** Alhué reaches nothing
  at all and has no row as an origin, which is why there are 2,454 rows and not 2,652.
- The headway gives an **expected** wait, not a real 08:00 timetable. A pattern that runs
  every 90 minutes reads as a 45-minute wait. `minutes_in_vehicle` has none of that, which
  is why it is a separate column.
- Walking is a straight line, not a network, so every walk is shorter than the real one.
- 08:00 only. No fare, no crowding, no capacity, no delay, no car, no bicycle, and no
  diagonal (a comuna to itself).
- **A comuna point is not a trip end.** These are times between two population centres.
  Nothing here is calibrated against an observed travel time; EOD 2012 reported trip
  durations and could later check them.

## jobs.csv

Work destinations of the simulation. One row per Census 2024 manzana that has jobs, plus one row for the airport. Builder: `python3 -I tools/pack/jobs.py [PACK_DIR]` (15 s).

| Column | Meaning |
| --- | --- |
| `block_id` | INE `MANZENT` code of the manzana, as text. 2,489 are not in `blocks.csv` (916,985 jobs): most manzanas with 0 residents are not there. `13124_airport` is not a manzana. Do not join by id; use `longitude`, `latitude` |
| `cut` | 5-digit INE comuna code |
| `longitude`, `latitude` | Centroid of the manzana polygon, EPSG:4674 |
| `jobs` | Jobs in the manzana, 2024, modeled. Integer |
| `method` | Signals that put jobs there, joined with `+` |
| `quality` | `ok`, or `known_low`: the real number is higher. 1 row is `known_low` (Costanera Center) |

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
- **Check against independent data.** Jobs within 500 m of a Metro station against the observed weekday boardings from 16:00 to 21:00 (`metro_observations.json`, April 2026), on the 95 stations that are not in the held-out set of `tools/pack/check_score.py`: Pearson 0.76, Spearman 0.79. Within 800 m: 0.69, 0.79. Jobs of the nearest station with no distance limit: 0.33, 0.50, because the last stations of each line take the far jobs (the airport row goes to Plaza Quilicura, 6.9 km away). Use a radius, not the nearest station. Against the morning boardings the correlation is 0, as expected for work places.
- Why these signals: they are the only ones on disk with coordinates. SII addresses have no coordinates and follow the head office. OSM and Overture places have no worker counts.
- What the method gets wrong:
  - The base is from 2012. A place that lost jobs after 2012 keeps them.
  - A building with a permit before 2010 that opened after the survey is missing: the permit file starts in 2010. Costanera Center was the known case, corrected below. Other cases can exist.
  - The sample is small: one sampled worker is about 120 jobs. Only 11,519 rows have jobs; a real small workplace with no sampled worker has 0. Use the file at a scale of about 500 m, not manzana by manzana.
  - 20% of the EOD workers have no usable workplace point (no answer, or coordinates 0,0). They are out of the signal. They are not random: workers with no fixed workplace are in this group.
  - 40 m² for each job is an assumption, equal for offices, shops, and warehouses. Permit area includes parking floors. 29% of the permit area does not count (outside a manzana, or a second permit of the same project). So a large mall or warehouse permit is probably too high: Mall Barrio Independencia `13108071002002` has 5,330 jobs from a permit of 233,382 m²; Megacentro Buenaventura `13301031001050` has 5,107, most of them from a permit of 99,887 m² of storage.

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
