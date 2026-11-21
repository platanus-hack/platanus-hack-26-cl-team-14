# Data pack

## zones.csv

Home zones of the simulation. One row per Census 2024 block-table row of the Región Metropolitana (urban manzanas, rural village blocks, rural entities, comuna containers). Builder: `python3 -I tools/pack/zones.py`.

| Column | Meaning |
| --- | --- |
| `manzana_id` | INE `MANZENT` code, as text |
| `comuna_cut` | 5-digit INE comuna code |
| `lon`, `lat` | Area-weighted polygon centroid, EPSG:4674 (SIRGAS 2000, equal to WGS84 at this scale) |
| `persons` | `n_per`, persons counted |
| `dwellings` | `n_vp`, private dwellings |
| `workers` | `n_ocupado`, employed persons who live in the zone |
| `students` | `n_asistencia_basica` + `n_asistencia_media`, persons who attend basic or secondary school |

- Source: INE Census 2024, `Base_manzana_entidad_CPV24.csv` and `Cartografia_censo2024_R13.gpkg` (layers `Manzanas_CPV24`, `Entidades_CPV24`, `Comunal_CPV24`). Year 2024.
- Rows: 52,646. Persons: 7,400,741 (equal to the census RM total). Providencia (`13123`): 621 rows, 143,974 persons.
- 53 rows are comuna containers (`CONTENEDOR_COMUNAL=1`, 108,997 persons). They have no polygon by INE design. Their point is the person-weighted mean of the block centroids of their comuna, so it lands where people live (Providencia container: -70.6085, -33.4336). All other 52,593 rows join to a polygon.
- The block table has no 15–64 or 5–17 age bins. `workers` and `students` are direct census counts instead.
- `workers` (3,588,512) includes people who work at home and outside the RM. The Census work matrix keeps only real commutes (about 2.75 M). Use the matrix rows as shares, not as counts against `workers`.
- INE suppresses some values (`*`) for disclosure control. The builder writes them as 0 (2 rows in `workers` and `students`).

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
- `MAT_TOTAL` is the exact sum of `MAT_ENS_1` to `MAT_ENS_8`, so it **does** count pre-school: 104,953 of the 1,309,286 students are in `MAT_ENS_1` (parvularia), spread over the 1,663 schools that offer that level.
- **407 of the 3,220 rows have `enrollment` 0, and they are not closed schools.** All 414 operating RM schools with `MAT_TOTAL=0` declare no education level (`ENS_01=0`), and the names are salas cuna and jardines infantiles (107 "SALA CUNA", 101 "INFANTIL", 77 "JARDIN", 41 of the "VITAMINA" chain). MINEDUC lists them in the directory, but it does not publish their enrolment here; JUNJI and Integra report it separately. This is also why the 7 schools dropped above lose 0 students.
- Consequence for the engine: `enrollment` is the destination weight, so those 407 points can never receive a student. That is the correct behaviour for the school trips that `school_od.csv` counts, because the census and MINEDUC school totals also exclude pre-school. Do not "fix" the 0 with an invented capacity.
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
- **Limit on the 136,153 students who go to a school outside the RM (9.6%).** They are not a daily trip. I counted them by school in the raw enrolment file: they spread over 5,962 schools and no school holds more than 372 of them, so no online school and no single artefact explains the volume. **Probable cause, not verified:** `COD_COM_ALU` is the address of record of the register, and for a family that moved it keeps the old RM address. My check rules out one artefact school; it does not prove the cause. Use only the rows where `destination_cut` starts with `13` for daily school trips. The other rows stay in the file because they are in the source, not because the trip is real.
- `schools.csv` has no point for a school outside the RM, so those rows cannot become a point destination.
- Enrolment is choice plus admission, so it understates demand for a school that turns students away. The home side never goes below the comuna.
- One snapshot, cut 2026-04-30. No time of day.
- **The 3 student totals of the pack are 3 different populations. They must not agree.**

  | Number | Population |
  | --- | --- |
  | `zones.csv` students, 1,033,857 | RM **residents** who attend basic or secondary school. Census 2024. No pre-school, no higher education |
  | `schools.csv` enrolment, 1,309,286 | Students **at an RM school**, any home comuna. MINEDUC 2026. Includes 104,953 in pre-school (`MAT_ENS_1`) |
  | `school_od.csv` students, 1,413,489 | Students with an **RM home comuna** at a school anywhere in Chile. MINEDUC 2026 |

  The two MINEDUC numbers meet on the RM: 1,277,336 of the 1,413,489 go to an RM school, and 1,309,286 − 1,277,336 = 31,950 students at RM schools live outside the RM. The census number is lower because it excludes pre-school and higher education, and because it counts 2024, not 2026.

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

## jobs.csv

Work destinations of the simulation. One row per Census 2024 manzana that has jobs. Builder: `python3 -I tools/pack/jobs.py [PACK_DIR]` (15 s).

| Column | Meaning |
| --- | --- |
| `zone_id` | INE `MANZENT` code of the manzana, as text. Not all are in `zones.csv`: `zones.csv` has no manzanas with 0 residents |
| `comuna_cut` | 5-digit INE comuna code |
| `lon`, `lat` | Centroid of the manzana polygon, EPSG:4674 |
| `jobs` | Jobs in the manzana, 2024, modeled. Integer |
| `method` | Signals that put jobs there, joined with `+` |

- Rows: 11,518. Jobs: 2,688,814. All 52 comunas of the RM.
- **Comuna totals are real.** The jobs of each comuna are equal to the column sum by `work_comuna` of the Census 2024 work matrix (`data/raw/censo2024-personas/od_work_rm.csv`). Providencia 225,827. Las Condes 305,601. Santiago 360,648. This total has only workers who live in the RM and gave a work comuna. The INE dashboard gives more (Santiago 433,378) because it has more answer types.
- **The place inside the comuna is a model.** Weight of a manzana = A + B, then scaled to the comuna total.
  - A, `eod2012`: workers of the EOD 2012 survey at their usual workplace point (`personas.csv`, `DirActividadCoordX/Y`, expanded with `Factor`). 22,703 sampled workers have a point. The point goes to the manzana that contains it, or to the nearest manzana edge within 100 m.
  - A, `eod2012_landmark`: the EOD geocoder put vague answers on a landmark, usually a Metro station (Tobalaba has 104 sampled workers on one point). A point with 10 or more sampled workers is a landmark (148 points, 12% of the EOD workers). Its workers spread over the exact-address workers within 500 m.
  - B, `permits2013_2025`: INE building permits, non-residential, 2013–2025, floor area / 40 m² for each job. This adds the buildings that came after the survey. One manzana and one use text count one time (the largest permit), because a project has many permits.
  - `persons_fallback`: 7 rural comunas have fewer than 50 sampled workers (Paine, Curacaví, Tiltil, San José de Maipo, Alhué, María Pinto, San Pedro). Their jobs spread by resident persons. 50,857 jobs, 1.9%.
- Why these two signals: they are the only ones on disk with coordinates. SII addresses have no coordinates and follow the head office. OSM and Overture places have no worker counts.
- What the method gets wrong:
  - The base is from 2012. A place that lost jobs after 2012 keeps them.
  - The sample is small: one sampled worker is about 120 jobs. Only 11,518 manzanas have jobs; a real small workplace with no sampled worker has 0. Use the file at a scale of about 500 m, not manzana by manzana.
  - 20% of the EOD workers have no usable workplace point (no answer, or coordinates 0,0). They are out of the signal. They are not random: workers with no fixed workplace are in this group.
  - 40 m² for each job is an assumption, equal for offices, shops, and warehouses. Permit area includes parking floors. 29% of the permit area does not count (outside a manzana, or a second permit of the same project).
  - The landmark rule also catches a large employer with one address (the airport: 126 sampled workers).

### The MUT (Mercado Urbano Tobalaba)

- Location: Av. Apoquindo 2730, Las Condes. Manzana `13114131009033`, centroid -70.600974, -33.417357, 15,300 m², 0 residents. Opened in 2023.
- `jobs.csv` gives this manzana **7,124 jobs**: 4,643 from the permit (161,443 m², 13 floors, year 2020), 1,469 from 2012 exact addresses, 1,012 from the Tobalaba landmark. The 2012 part is doubtful: the site was a construction pit then, so those points are the old address of the manzana or the geocoder.
- Real numbers, all claims of the operator or the press, not measured:
  - Offices: 4 towers, 45,765 m² ([La Tercera](https://www.latercera.com/pulso-pm/noticia/proyecto-mut-suma-otro-gran-inquilino-metlife-consolidara-sus-oficinas-corporativas-en-6-pisos-de-la-torre-1/HNS3YVGCAVB7RP45GW2A453NXU/)). "4 mil personas" in the offices, a project estimate ([El Diario Inmobiliario](https://eldiarioinmobiliario.cl/noticias/nacional/mut-relacion-de-centro-comercial-con-sustentabilidad/)).
  - Shops: more than 200 in operation ([CNN Chile](https://www.cnnchile.com/pais/territoria-centros-comerciales-mut-chile/)). No published count of shop workers.
  - Visits: "más de tres millones de visitas en lo que va de año" in September 2025 ([Meganoticias](https://www.meganoticias.cl/nacional/498737-nuevos-mut-en-que-comunas-territoria-mercado-urbano-dcv-pdp-12-9-2025.html)), about 12,000 for each day. `context/research/data-sources-buildings-activity.md` has a different press claim: about 650,000 for each month, about 21,000 for each day. The two claims do not agree.
- For the demo: remove the 7,124 jobs of the manzana and say "modeled". The only published jobs number is the claim of 4,000 office workers.
