# Origin-destination flows in administrative data

> Privacy limit (review of PR #15): SAE and PAES rows have a person key (`mrun`) and, for SAE, the home point of a child.
> Use aggregates only. Do not commit person rows. `tools/scrape/mineduc-school-od.py` writes comuna totals only; the point export was removed in review.

Topic: which administrative source gives a real pair (home place, destination place),
and at which granularity. Every entry below says which two columns hold the pair.
Checks were made on 2026-11-21.

## Summary: who has a real OD pair

| Source | Home column | Destination column | Home granularity | Destination granularity |
| --- | --- | --- | --- | --- |
| MINEDUC SAE 2025 | `lat_con_error`, `lon_con_error` | `rbd_admitido` + school `lat`, `lon` | **point** (noise added) | **point** |
| Censo 2024 personas | `comuna` | `p44_lug_trab_esp` | comuna | comuna |
| Censo 2024 personas (migration) | `p24_lug_resid5_esp` | `comuna` | comuna | comuna |
| MINEDUC Matrícula 2026 | `COD_COM_ALU` | `COD_COM_RBD` (+ `RBD`) | comuna | comuna, or point by RBD join |
| DEMRE PAES 2026 | `CODIGO_COMUNA_DOMICILIO` | `SIGLA_UNIVERSIDAD` + `SEDE` | comuna | campus (coarse text) |
| SIES 2026, joined to PAES on `mrun` | `CODIGO_COMUNA_DOMICILIO` | `comuna_sede` | comuna | comuna |
| CASEN 2024 | `comuna` (second base) | `e9com_esc`, `e9com_sup`, `e9rbd_esc` | comuna | comuna, or point by RBD join |
| INE Conmutación CPV2024 | `o` | `d` | comuna | comuna |
| Censo 2002 (Redatam; employed `SLABORAL` 1/2/4, raw export verified; not harmonized) | habitual residence `VIVEHAB` | raw `COMTRAES` | comuna | raw destination code |

No pair: DEIS hospital discharges, DEIS emergency visits, DEIS births, DEIS deaths,
SII worker counts, FONASA, Servel, Censo 2017.

---

## Sources with a real pair

### MINEDUC SAE 2025 (school admission for 2026)
- Link: https://datosabiertos.mineduc.cl/wp-content/uploads/2026/03/SAE_2025.rar
- Contents: one row for each applicant in `B1_Postulantes`, with the home point of the
  applicant: `mrun;cod_nivel;es_mujer;prioritario;alto_rendimiento;lat_con_error;lon_con_error;calidad_georef`.
  `D1_Resultados` gives `rbd_admitido` for the same `mrun`. `A1_Oferta` gives `lat;lon`
  for each `rbd`. `C1_Postulaciones` gives every school that the applicant asked for,
  in preference order. Sample row of the pair (home point -> admitted school point):
  `-33.4054163,-70.6923936 -> rbd 10235 at -33.40723,-70.69227` (0.20 km).
- Granularity: person. Home is a **point with added noise** (`calidad_georef` grades the
  geocode). Destination is a school point. No time step; one admission round per year.
- Year: applications of 2025 for the school year 2026. Files exist for 2016 to 2025.
- Access: direct download, RAR, 30,633,350 bytes, 11 CSV files inside.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 30,633,350 bytes. `B1` and `D1` have
  463,923 rows each, `A1` has 78,411 rows. 430,712 admitted applicants join to both a
  home point and a school point. 141,953 of those homes are inside the Santiago box.
  Straight-line home-to-school distance in Santiago: median 1.53 km, p90 6.32 km.
- Role: INPUT for school choice, TARGET for the trip-length distribution of school trips.
- Limits: only schools inside the SAE system (it excludes most paid private schools).
  The home point carries deliberate noise, so it is good for distributions and wrong for
  one household. `rbd_admitido` is the offer, not the enrolment; use Matrícula for the
  enrolment. `calidad_georef` marks weak geocodes and this check did not filter on it.
  **Privacy.** One row is a child: a home point plus the school that admitted that child.
  Review of PR #15 removed the point export, so this repository holds no script and no
  output for it. I deleted the downloaded person rows on 2026-11-21: `data/raw/mineduc-sae/`
  and `data/raw/mineduc-od/{sae.rar,od_school_points_rm.csv}`. Nothing below the comuna
  goes into a commit or the demo. If anyone rebuilds it, use the pairs for distributions,
  never for one household, and delete the rows afterwards.
- Use in the simulation: the only public administrative source that gives a trip below
  the comuna. It calibrates the distance-decay term of destination choice, which the
  comuna-level sources cannot do.
- Script: none in the repository. The point export was removed in review (person-level data). The numbers in this entry come from a local run that is not kept.

### Censo 2024, person microdata
- Link: https://storage.googleapis.com/bktdescargascenso2024/personas_censo2024.zip
- Contents: one row for each person, 62 columns, `;` separated. Two pairs:
  1. **Work**: `comuna` (home) and `p44_lug_trab_esp` (where the person works).
     `p44_lug_trab` says what the destination means, and `p45_medio_transporte` is the
     mode. The codes, counted on RM residents: `1` works at home (517,018, destination
     repeats the home comuna), `2` in this comuna (835,484, same), `3` in another comuna
     (1,916,481, a real comuna code, 4 digits in regions 1 to 9 and 5 digits in regions
     10 to 16), `4` in another country (3,793, the destination is a 1 to 3 digit country
     code), `5` no fixed place (313,358, destination `998` or `999`).
  2. **Migration**: `p24_lug_resid5_esp` (comuna or country of residence 5 years before)
     and `comuna` (residence in 2024).
  Sample work pair: `comuna=13101 -> p44_lug_trab_esp=13114`, 28,583 people.
  Sample migration pair: `p24_lug_resid5_esp=862 (Venezuela) -> comuna=13101`, 26,556 people.
- Granularity: person. Home and destination both at comuna. `comuna_bajo_umbral` marks
  comunas masked for disclosure. No time step.
- Year: Census 2024, collected March–July 2024; usual residence, not a one-day travel diary.
- Access: direct download from a public Google Cloud Storage bucket. ZIP 355,253,274 bytes,
  CSV inside 2,516,081,907 bytes. The bucket also holds `hogares_`, `viviendas_`,
  `Microdatos_Redatam_Censo2024.zip` and the aggregated manzana bases.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 355,253,274 bytes. 7,400,741 persons with a
  home comuna in region 13. 3,588,582 of them have a work-place answer, and the code
  counts above sum to exactly that. The script `tools/scrape/censo2024-od.py` keeps only
  `p44_lug_trab` 2 or 3 with a comuna code: 2,740,454 commuters in 9,128 work pairs, and 13,234
  migration pairs. It writes the excluded counts to `excluded_destinations.csv`: 517,018 work at
  home, 313,358 have no fixed place, 3,793 work in another country, and 8,495 have a region
  but no comuna (8,250 of them in the RM). `13101 -> 13114` holds 28,583 people, the same value that
  the INE visualizer publishes.
- Role: INPUT and TARGET.
- Limits: the destination stops at the comuna, so it cannot place a job inside a comuna.
  The corrected script keeps categories 2 and 3 and removes missing and region-only
  destinations. It excludes work-at-home and foreign-work categories, with separate counts.
  The [official manual](https://censo2024.ine.gob.cl/wp-content/uploads/2025/12/manual_uso_microdatos_censo2024.pdf),
  pages 42–43, explains the region-only values. The [owner checked the output](https://discord.com/channels/1573514576400752640/1573579088684253235/1573582284097986581)
  against 346 MINEDUC comuna codes: zero unmatched destinations or workers. This review
  did not repeat the full-file run. The 2,740,454 retained workers include 51,640 whose
  destination is outside RM. Preserve those external flows separately for an RM-only engine;
  do not relabel them as internal flows. Migration now also excludes region-only values
  and pads keys; its retained origin codes can still represent a comuna or a foreign place.
  Code 5, "no fixed place", is 313,358 people, 8.7% of all employed RM residents with an
  answer, and it is concentrated: 12.9% in Peñalolén (13122) and La Pintana (13112)
  against 3.2% in Melipilla (13501). Those people travel and the matrix cannot say where.
  `p44` is the usual workplace, not a trip on a given day. Study trips are not asked,
  unlike Censo 2002.
- Use in the simulation: the backbone home-to-work OD matrix for all of Santiago, and the
  residential-move matrix that drives the housing layer.
- Script: `tools/scrape/censo2024-od.py` -> `data/raw/censo2024-personas/od_work_rm.csv`,
  `od_migration_rm.csv`

### MINEDUC Matrícula por estudiante 2026
- Link: https://datosabiertos.mineduc.cl/wp-content/uploads/2026/10/Matricula-Oficial-2026.rar
- Contents: one row for each enrolled student. Columns `COD_COM_ALU;NOM_COM_ALU`
  (comuna where the student lives) and `COD_COM_RBD;NOM_COM_RBD` plus `RBD`
  (the school). `COD_REG_ALU` is the home region. Top RM cross-comuna pairs:
  `13201 PUENTE ALTO -> 13110 LA FLORIDA` 6,919 students;
  `13101 SANTIAGO -> 13201 PUENTE ALTO` 6,335; `13101 SANTIAGO -> 13119 MAIPÚ` 5,502.
- Granularity: person. Home at comuna, school at `RBD`, which becomes a point through the
  Directorio join below. No time step; one snapshot per year.
- Year: 2026, cut at 2026-04-30. One file per year from 2004.
- Access: direct download, RAR 32,861,518 bytes, CSV inside 582,741,121 bytes,
  3,467,024 rows, plus a record description PDF.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 32,861,518 bytes. 1,413,489 students live in
  region 13. 574,481 of them (40.6%) go to school in another comuna. 10,704 distinct
  home-comuna to school-comuna pairs in the RM.
- Role: INPUT and TARGET.
- Limits: the home side is a comuna only, never an address. `COD_COM_ALU` and
  `COD_COM_RBD` are easy to swap; the first is the student, the second the school.
  Enrolment is the result of choice plus admission, so it understates demand for
  oversubscribed schools. The file is sorted by `RBD`, so the first rows are Arica.
- Use in the simulation: school destination choice at mass level, and the daily school
  trip volume that the transport layer must carry.
- Script: `tools/scrape/mineduc-school-od.py` -> `data/raw/mineduc-od/od_school_comuna_rm.csv`

### MINEDUC Directorio oficial de establecimientos 2026
- Link: https://datosabiertos.mineduc.cl/wp-content/uploads/2026/11/Directorio-EE-Oficial-2026.rar
- Contents: one row for each school. `RBD;DGV_RBD;NOM_RBD;...;COD_COM_RBD;NOM_COM_RBD;
  ...;LATITUD;LONGITUD;...;MAT_TOTAL;ESTADO_ESTAB;PAGO_MATRICULA;PAGO_MENSUAL`.
  Sample row: `RBD 1, LICEO POLITECNICO ARICA, LATITUD -18,48720000, LONGITUD -70,295212000`.
- Granularity: point, one per school. One snapshot per year.
- Year: 2026, cut at 2026-04-30.
- Access: direct download, RAR 908,273 bytes.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 908,273 bytes. Header and first rows read;
  `LATITUD` and `LONGITUD` are filled, with a comma as the decimal mark.
- Role: INPUT.
- Limits: it is a joiner, not a flow. It has no student. Coordinates are not graded for
  quality, and this check did not count the empty ones.
- Use in the simulation: it turns the comuna destination of Matrícula into a school point,
  which gives a point destination for 1.4 million RM school trips.

### DEMRE PAES 2026 (home address plus admitted programme)
- Link: https://datosabiertos.mineduc.cl/wp-content/uploads/2026/07/PAES-2026-Socioeconomicos.rar
  and https://datosabiertos.mineduc.cl/wp-content/uploads/2026/07/PAES-2026-Matricula.rar
- Contents: `B_SOCIOECONOMICO_DOMICILIO_PAES_2026_PUB_MRUN.csv` has
  `MRUN;ANYO_PROCESO;CODIGO_REGION_DOMICILIO;CODIGO_PROVINCIA_DOMICILIO;CODIGO_COMUNA_DOMICILIO;NOMBRE_COMUNA_DOMICILIO;...`.
  `D_MATRICULA_PAES_2026_2026_PUB_MRUN.csv` has
  `MRUN;ANYO_PROCESO;CODIGO_CARRERA;NOMBRE_CARRERA;SEDE;SIGLA_UNIVERSIDAD;PREFERENCIA;...`.
  Join on `MRUN`. Top RM pairs: `LAS CONDES -> UC, SANTIAGO` 647 students;
  `LAS CONDES -> UDD, SANTIAGO` 476; `LO BARNECHEA -> UC, SANTIAGO` 476.
- Granularity: person. Home at comuna of the declared address. Destination is a campus
  name, not a code and not a point.
- Year: admission process 2026. Files exist back to 2004 under other names.
- Access: direct download, two RAR files, 3,052,169 and 1,985,185 bytes.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200. 320,087 address rows and 146,873 enrolment
  rows. 141,402 enrolled students join to a home comuna.
- Role: INPUT.
- Limits: only the centralised admission system, so it misses most institutes and
  professional centres. `SEDE` is free text such as `SANTIAGO` or
  `SEDE SANTIAGO (S.C. APOQUINDO)`, so the destination needs manual geocoding before it
  is useful below the comuna. `A_INSCRITOS_PUNTAJES` has `CODIGO_COMUNA_EGRESO`, which is
  the comuna of the **school**, not of the home; do not mistake it for the home comuna.
- Use in the simulation: the university destination-choice layer, and the one source that
  links a home comuna to a named campus.

### CASEN 2024
- Link: https://observatorio.ministeriodesarrollosocial.gob.cl/encuesta-casen-2024
  (base: `storage/docs/casen/2024/casen_2024.RData`, plus
  `casen_2024_provincia_comuna.RData` for the comuna)
- Contents: one row for each person, 877 variables. The destination side is
  `e9com_esc` (comuna of the school), `e9com_sup` (comuna of the higher-education
  institution), `e9rbd_esc` and `e9rbd_sup` (the MINEDUC `RBD`), `e9dir_esc` (address).
  The home comuna is `comuna` in the second base; join on `folio` and `id_persona`.
  Commute is `o28a_hr`, `o28a_min` (time from home to work) and `o28c` (mode).
  Sample pairs: `13114 -> 13130`, `13125 -> 13101`.
- Granularity: person, with survey weights `expc` and `expp`. Home and destination at
  comuna; `e9rbd_esc` raises the school side to a point through the Directorio.
- Year: field work 2024-11 to 2025-01.
- Access: direct download. `.RData` 46,840,844 bytes. The `.dta` and `.sav` links both
  returned HTTP 500 on 2026-11-21, so use `.RData`.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 46,840,844 bytes, 218,367 rows and 877
  columns read with R. `e9com_esc` has 40,228 non-missing values and `e9rbd_esc` 40,744.
  After the merge, 7,364 RM school students have a school comuna and 1,999 of them study
  in another comuna.
- Role: INPUT.
- Limits: **CASEN has no work comuna.** It asks the commute time and the mode, never the
  place. The sample is far too small for an OD matrix: 7,364 RM students against 1.4
  million in Matrícula. Weights are designed for comuna totals, not for comuna pairs.
- Use in the simulation: the commute-time and mode distribution that validates the travel
  cost function. Use Matrícula, not CASEN, for the school flows.

### SIES Matrícula en Educación Superior 2026
- Link: https://datosabiertos.mineduc.cl/wp-content/uploads/2026/09/Matricula-Ed-Superior-2026.rar
- Contents: one row for each enrolled student, 53 columns. It has `region_sede`,
  `provincia_sede`, `comuna_sede` (the campus) and `mrun`, but **no column for where the
  student lives**. Destination only, half a pair.
- Granularity: person; destination at comuna.
- Year: 2026. Series from 2007.
- Access: direct download, RAR 34,275,223 bytes.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 34,275,223 bytes. 424,736 students who
  started their programme in 2026. **The `mrun` join to PAES works**: 196,271 of those
  (46.2%) match a `MRUN` in `B_SOCIOECONOMICO_DOMICILIO_PAES_2026`, which recovers the
  home comuna. Top RM pairs built that way: `PUENTE ALTO -> SANTIAGO` 2,447 students,
  `MAIPU -> SANTIAGO` 2,187, `LA FLORIDA -> SANTIAGO` 1,581.
- Role: INPUT.
- Limits: on its own the file is destination only. The PAES join covers 46% of first-year
  students; the other 54% never registered for PAES, which is most entrants to technical
  programmes, so the recovered pairs are biased towards university entrants. `comuna_sede`
  is the comuna of the campus, not a point. `mrun` here is re-scrambled (`PUBL_MRUN`,
  values start at 19), so it does **not** join to the 8-digit `MRUN` of the school
  Matrícula file.
- Use in the simulation: higher-education destination choice for institutes and technical
  centres, which PAES alone misses.

### INE Conmutación laboral CPV2024 (visualizer)
- Link: https://conmutacion-laboral.ine.gob.cl/ and
  https://storage.googleapis.com/bktdescargascenso2024/Conmutacion_Laboral_cpv2024.pdf
- Contents: the page embeds one `DATA={...}` literal with all its tables. `destino` holds
  OD rows `{"o":13101,"on":"Santiago","d":13114,"dn":"Las Condes","frecuencia":28583,"orden":1}`
  for 2002 and 2024, top 3 destinations for each of the 48 Santiago comunas. `relativas`
  holds entry, exit, balance and an "attractor / emitter" class for each comuna.
  `desajuste` holds `ocupados_n` against `puestos_n` (jobs) for each comuna.
- Granularity: comuna pair, one snapshot for 2002 and one for 2024.
- Year: 2002 and 2024.
- Access: scrape of one HTML page, 14,550,297 bytes, plus a PDF of results.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 14,550,297 bytes. 144 OD rows and 48
  balance rows for Santiago in each year. The value `13101 -> 13114 = 28,583` is
  **exactly** the value computed from the Censo 2024 microdata, so the two agree.
- Role: TARGET.
- Limits: only the top 3 destinations for each origin, so it is not a full matrix; the
  microdata is. It counts only trips to a *known* comuna other than the home comuna.
  INE filters the 2002 work-or-study answer to employed people, assuming employed
  students reported work. Comparison requires habitual residence and harmonized
  exclusions; the question wording alone does not make it invalid (methodology §§3.3–3.6).
- Use in the simulation: a ready, citable score target for commuting, and the
  jobs-against-residents imbalance for each comuna.
- Script: `tools/scrape/ine-conmutacion-censo2024.py` -> `data/raw/ine-conmutacion/`

### Censo 2002, person dictionary and online cross-tab (Redatam)
- Link: https://redatam-ine.ine.cl/redbin/RpWebStats.exe/Dictionary?BASE=CENSO_2002&ITEM=DICPOB&lang=esp
  and [public CmdSet](https://redatam-ine.ine.cl/redbin/RpWebStats.exe/CmdSet?BASE=CENSO_2002&ITEM=PROGRED&lang=ESP).
- Contents: the person dictionary lists `LUGTRAES` "Cual es la comuna de estudio o
  trabajo" (`P33A`) and `COMTRAES` "Código de comuna de estudio o trabajo" (`P33B`),
  with `LUGVIV`/`VIVEHAB` for habitual residence and `SLABORAL` for employment.
  Use habitual residence and an employed-person filter to construct a commuting
  cross-tab. Enumeration geography is not necessarily home: 2002 counted people
  where they slept the previous night. The full raw specified-cohort cross-tab is now retrieved.
- Granularity: habitual-home comuna × raw workplace code; aggregate output only. One snapshot.
- Year: 2002.
- Access: public aggregate query; no person records downloaded. [Raw counts, query and manifest](../../../data/validation/census_2002/README.md).
- Check: VERIFIED-RAW 2026-11-21, HTTP 200. 52 habitual RM origins × 403 raw destination
  codes, 20,956 dense cells and 2,242,438 employed people. Independent raw review matched
  every CSV cell, row/column totals and all 144 published 2002 viewer flows.
- Role: TARGET.
- Limits: 24 years old, before Metro Lines 4, 4A, 3 and 6. INE's employed-person filter
  assumes working students prioritized work. Workplace nonresponse differs: 8.4% in
  2002 versus 0.06% in 2024; the latter's multiple-workplace category is excluded from
  matrix-derived measures. Harmonize these definitions before comparison. See the
  [INE methodology, §§3.3–3.6](https://storage.googleapis.com/bktdescargascenso2024/Conmutacion_Laboral_cpv2024.pdf).
  Raw code `99999` contains 202,489 workers; its exact semantics remain unverified.
  Foreign/unusual-code classification and 2024 harmonization remain pending; domestic
  external destinations are retained. Raw review is not benchmark approval.
- Use in the simulation: a candidate earlier state after destination classification and
  universe harmonization; the current artifact is not an approved evaluation target.

---

## Sources without a pair (negative results, verified)

### Censo 2017, person dictionary (Redatam)
- Link: https://redatam-ine.ine.cl/redbin/RpWebStats.exe/Dictionary?BASE=CENSO_2017&ITEM=DICPOB&lang=esp
- Contents: 33 person variables. There is **no** work or study comuna. It has
  `P11COMUNA` "Comuna de Residencia Anterior" (residence 5 years before) and
  `P12COMUNA` "Comuna de Nacimiento", which give a residential-migration pair with the
  home comuna of the record.
- Granularity: person; output at comuna pair.
- Year: 2017.
- Access: online cross-tab.
- Check: UNVERIFIED for counts. 2026-11-21, HTTP 200, 14,230 bytes, the full person
  variable list was read from the dictionary. A dictionary is the right evidence for
  "the variable is absent", but no cross-tab was run.
- Role: INPUT for migration only.
- Limits: the 2017 census was abridged, which is why INE's own commuting product jumps
  from 2002 to 2024. Do not expect a 2017 commute matrix; it does not exist.
- Use in the simulation: residential migration between the 2002 and 2024 census points.
  For commuting: NONE.

### DEIS Egresos hospitalarios (hospital discharges) 2019 and 2025
- Link: https://repositoriodeis.minsal.cl/DatosAbiertos/EGRESOS/EGRESOS_2025.zip
  (and `EGRESOS_2019.zip`; one file per year from 2001)
- Contents: one row for each discharge:
  `PERTENENCIA_ESTABLECIMIENTO_SALU;SEXO;GRUPO_EDAD;ETNIA;GLOSA_PAIS_ORIGEN;COMUNA_RESIDENCIA;GLOSA_COMUNA_RESIDENCIA;REGION_RESIDENCIA;GLOSA_REGION_RESIDENCIA;PREVISION;GLOSA_PREVISION;ANO_EGRESO;DIAG1;DIAG2;DIAS_ESTADA;CONDICION_EGRESO;ERROR`.
  The hospital is **not** there. `PERTENENCIA_ESTABLECIMIENTO_SALU` only says whether the
  establishment is inside the public network. The dataset title says "por Comuna de
  Residencia (disociados)", and "disociados" means the establishment was removed.
- Granularity: person; home at comuna; no destination.
- Year: 2001 to 2025.
- Access: direct download, ZIP 22,824,807 bytes (2025), CSV inside 302,672,065 bytes.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200. Header and rows read for 2025 and for 2019.
  Both have the same 17 or 18 columns and neither has an establishment code.
- Role: INPUT (health demand by home comuna).
- Limits: the hint that this file holds "home comuna and hospital" is **wrong** for the
  open version. The pair exists only inside MINSAL. BCN also notes that discharges with
  an unknown comuna of residence are dropped from comuna counts.
- Use in the simulation: hospital demand generated by each comuna. For OD: NONE.

### DEIS Atenciones de Urgencia (emergency visits) 2026
- Link: https://repositoriodeis.minsal.cl/SistemaAtencionesUrgencia/AtencionesUrgencia2026.zip
- Contents: one row for each establishment, cause and day:
  `IdEstablecimiento;NEstablecimiento;IdCausa;GlosaCausa;Total;Menores_1;De_1_a_4;De_5_a_14;De_15_a_64;De_65_y_mas;fecha;semana;...;CodigoComuna;NombreComuna`.
  `CodigoComuna` is the comuna of the **establishment**, not of the patient.
- Granularity: establishment (point, by code) x cause x day. Daily time step.
- Year: 2026 to date, updated weekly. Files back to 2008.
- Access: direct download, ZIP 49,873,179 bytes, CSV inside 1,782,738,648 bytes.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 49,873,179 bytes, header and rows read.
- Role: TARGET (destination volume only).
- Limits: no patient origin at all, so it cannot make a pair. It is counts, not persons.
- Use in the simulation: a daily observed arrival count at each health facility, which is
  the best TARGET for a health destination-choice layer. For OD: NONE.

### DEIS Nacimientos 2001-2023
- Link: https://repositoriodeis.minsal.cl/DatosAbiertos/VITALES/NACIMIENTOS/Serie_Nacimientos_2020_2023.zip
- Contents: 29 columns. Geography is `REGION_RESIDENCIA;GLOSA_REGION_RESIDENCIA` only.
  `PARTO_LOCAL` is a code for the type of place, not a comuna or an establishment.
- Granularity: person; home at **region**.
- Year: 1992 to 2023 in three series.
- Access: direct download, ZIP 8,092,780 bytes, CSV inside 83,788,960 bytes.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200. Header and first rows read for 2020-2023 and
  for 2001-2019. Both stop at the region.
- Role: NONE.
- Limits: the open version is coarser than the discharges file. No comuna, so no pair.
- Use in the simulation: NONE.

### DEIS Defunciones 2024-2026
- Link: https://repositoriodeis.minsal.cl/DatosAbiertos/VITALES/DEFUNCIONES_FUENTE_DEIS_2024_2026_17112026.zip
- Contents: `AÑO;FECHA_DEF;SEXO_NOMBRE;EDAD_TIPO;EDAD_CANT;COD_COMUNA;COMUNA;NOMBRE_REGION;DIAG1;...;LUGAR_DEFUNCION`.
  One comuna only (residence). `LUGAR_DEFUNCION` is a category such as
  `Casa habitación`, not a place.
- Granularity: person; home at comuna; daily date.
- Year: 2024 to 2026, updated weekly.
- Access: direct download, ZIP 13,187,304 bytes, CSV inside 110,146,789 bytes.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, header and first row read.
- Role: INPUT for population change.
- Limits: no destination. Not a flow.
- Use in the simulation: population decrement by comuna. For OD: NONE.

### SII Estadísticas de empresas (workers by comuna)
- Link: https://www.sii.cl/sobre_el_sii/estadisticas_de_empresas.html
  (file `https://www.sii.cl/sobre_el_sii/empresas/PUB_COMU_TRTRAB.xlsb`)
- Contents: counts of firms, sales, workers and wages. The geography column is
  `Comuna del domicilio o casa matriz`, with `Provincia del domicilio o casa matriz`,
  `Tramo según trabajadores dependientes informados`,
  `Número de trabajadores dependientes informados` and `Renta neta informada en UF`.
- Granularity: comuna of the firm headquarters x year.
- Year: annual series.
- Access: direct download, XLSB 4,550,113 bytes.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200, 4,550,113 bytes. Column labels were read
  from `xl/sharedStrings.bin`; the BIFF12 cell records were not parsed.
- Role: INPUT (job weights, with a known bias).
- Limits: workers are counted at the **headquarters** comuna, not at the place where they
  work and not at the place where they live. A retail chain with its head office in Las
  Condes puts all its workers in Las Condes. So there is no pair, and even the
  destination side is wrong. This matches the gap that `clever-heron` reported.
- Use in the simulation: a weak prior for job locations. For OD: NONE.

### FONASA, AFC and the Registro Social de Hogares
- Link: https://www.fonasa.cl/sites/fonasa/datos-abiertos (HTTP 403 on 2026-11-21);
  https://datos.gob.cl/api/3/action/package_search?q=fonasa+beneficiarios (2 results,
  both dashboards); same search for `registro social de hogares` (27 results, all
  municipal benefit counts).
- Contents: beneficiary or household counts by comuna of residence. No destination.
- Granularity: comuna.
- Check: UNVERIFIED. The FONASA open-data page returned HTTP 403. The national portal
  returned only Tableau dashboards and municipal aggregates, with no microdata resource.
- Role: INPUT at best.
- Limits: a health insurance or a social register records where a person lives, never
  where that person goes. A FONASA-to-provider pair would need the claims file, which is
  not public.
- Use in the simulation: NONE for OD.

### Servel (electoral roll and polling places)
- Link: https://www.servel.cl/padrones-electorales-definitivos-y-nominas-de-inhabilitados-elecciones-2025/
- Contents: the roll lists named persons with a RUT and an assigned polling place.
- Check: UNVERIFIED **by choice**. The catalog rules forbid personal data of named
  persons, so the file was not downloaded.
- Role: NONE.
- Limits: a polling place is assigned by the registered address, so the "pair" is an
  administrative rule, not an observed trip. Election-day travel happens once every two
  years and does not represent a normal day.
- Use in the simulation: NONE.

---

## Critique

The three weakest points of this catalog:

1. **One point-level source carries the whole sub-comuna story.** Only SAE gives a pair
   below the comuna, and only for school trips of applicants inside the SAE system, with
   noise added to the home point. If the demo needs a sub-comuna *work* trip, nothing here
   supplies it, and the gap must be filled by the transport sources that other agents own
   (DTPM stage data, EOD) or by a model, not by administrative data.
2. **The best work matrix is one snapshot and 8.7% of its workers have no destination.**
   Censo 2024 gives the usual workplace at the time of the interview (field work from March to July 2024), not trips. Code 5, "no
   fixed place", covers 313,358 employed RM residents and it is not spread evenly: 12.9%
   in Peñalolén and La Pintana against 3.2% in Melipilla. So the matrix systematically
   loses the mobile and informal workers of the poorer comunas, which is exactly the group
   a transport layer must not lose. I report the codes but I do not model those people.
3. **Two sources were checked by metadata, not by data.** Censo 2002 `COMTRAES` was read
   from the Redatam dictionary and no cross-tab was run, so the 2002 OD numbers in this
   file come from the INE visualizer, not from Redatam. CASEN `e9dir_esc` (the school
   address) was confirmed to exist but its values were never read, so I cannot say whether
   the public base keeps the address or blanks it.

What I did not check:

- The MINEDUC `Matrícula longitudinal`, `Rendimiento por estudiante` and
  `Asistencia anual por estudiante` files. All three are keyed by `MRUN` and `RBD` and may
  give the same pair over many years, plus attendance as a TARGET.
- Whether the SAE `calidad_georef` grades change the distance distribution, and how many
  home points are unusable.
- The empty-coordinate count in the Directorio, so the real join rate from `RBD` to a
  point is unknown.
- Any FONASA or AFC request route. Both were dismissed on public evidence only.
- The Censo 2024 `Microdatos_Redatam_Censo2024.zip`, which may hold a finer geography than
  the comuna in the CSV, which would change conclusion 1 above.
