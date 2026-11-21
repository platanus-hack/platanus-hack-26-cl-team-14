# Time use and travel behaviour

Checked 2026-11-21 by swift-falcon. Scope: activity duration, work participation, trip generation, and mode choice.

Use ENUT 2023 for daily activity budgets. Use EOD 2012 as the candidate for ordered trips, pending record parsing. Update work patterns with ENE and CASEN. These surveys do not share person identifiers. Statistical matching is a model assumption, not a measured person-level join.

The smallest useful model samples age/sex/income groups and weekday/weekend activity durations. An hourly sequence still needs a diary or explicit scheduling assumptions. Do not claim that daily duration is an observed start time.

Reproduce INE downloads with `python3 tools/scrape/time_use_behaviour.py enut2023` (also `enut2015`, `ene2026`, `epf-personas`, `epf-gastos`). The script also accepts `eod2012` and `santiago-rancagua2026`; wait at least 10 seconds between Zenodo runs. Archives are complete; the three plain CSV files are 65,536-byte samples. Files and column lists are in `data/raw/time-use-behaviour/`. INE robots returned a 404 page; requests were sequential, at most one per second during discovery. No credentials used.

### II ENUT 2023
- Link: https://www.ine.gob.cl/docs/default-source/uso-del-tiempo-tiempo-libre/bbdd/ii-enut/250403-ii-enut-bdd-csv-v2.zip?sfvrsn=ed041caf_7
- Contents: Person rows; `id_hog`, `id_persona`, `edad`, `sexo`, `NSE`, `glosa_region`, `region_ord`, `tiempo`, `fe_ch`, `fe_cut`, `varstrat`, `varunit`; activity participation and duration pairs such as `to1_p_ds`, `to3_t_ds`, `to5_t_ds`, and `_fds` variants. Income questions include `y2_imp`, `y4_imp`, `y9b_imp`.
- Granularity: person; published region, weekday/weekend activity duration. No comuna field observed.
- Year: 2023; archive revision April 2025.
- Access: direct ZIP containing CSV; 10,540,861 bytes.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200; read 48,020 rows and 1,089 columns. One RM row has age 43, sex 1, `region_ord=7`, `to3_t_ds=0.333333333333333`, `to5_t_ds=10`.
- Role: INPUT.
- Limits: RM is code 7, not CUT 13. Duration values are decimal hours, not HH.MM. Not every household member answered time-use questions; filter the eligible population and use the appropriate weight. CSV has no value labels. NSE is not automatically an income quintile.
- Use in the simulation: distributions of daily paid work, care, domestic work, study and leisure duration by demographic group. No observed hourly schedule.

Interpretation source: [INE manual](https://www.ine.gob.cl/docs/default-source/uso-del-tiempo-tiempo-libre/manuales/ii-enut/manual-de-uso-de-base-de-datos-ii-enut-2023.pdf), opened as a 190-page PDF. Sections 2–3 define survey populations and weights; section 5.6 derives income quintiles. Annex 7.1 identifies work travel duration as `to3_t_ds + to7_t_ds`. Simultaneous activities require separate treatment before building a 24-hour budget.

### I ENUT 2015
- Link: https://www.ine.gob.cl/docs/default-source/uso-del-tiempo-tiempo-libre/bbdd/documentos/base_datos_enut_csv.zip?sfvrsn=b399edf0_5
- Contents: Person rows; `id_vivienda`, `id_hogar`, `id_persona`, `region`, `dia_semana`, `dia_fin_semana`, `tiempo`, `edad`, `ing_mon`, `ing_mon_pc`, `quintil`, `wgtH`, `wgt1`, `wgt2`; activity fields and derived `trab_scn_ds`, `tdnr_ds`, `cgt_ds` and weekend variants.
- Granularity: person; region and day type.
- Year: 2015.
- Access: direct ZIP/semicolon CSV; 5,388,940 bytes; decimal commas.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200; read 34,575 rows and 710 columns. An RM household member has `region=13`, age 3, `ing_mon_pc=94000`, `quintil=1`.
- Role: INPUT.
- Limits: Includes children in the household roster; do not treat every row as a completed time-use interview. Old schema differs from 2023. The sex field coding was not decoded in this pass.
- Use in the simulation: historical duration baseline and income stratification; do not pool editions without harmonization.

### SECTRA EOD Santiago 2012
- Link: https://www.sectra.gob.cl/info-territorial/region-metropolitana-de-santiago/
- Contents: Official EOD database archive; contains `base_datos_eodStgo_2012.accdb`. Table columns and records not read in this pass.
- Granularity: intended household/person/trip/zone source; actual table keys still unverified here.
- Year: 2012.
- Access: [direct Drive ZIP](https://drive.google.com/uc?export=download&id=193675NwL6QNDs6VJFHU0mXiX5-6SdBPJ), 16,324,719 bytes, downloaded without login.
- Check: VERIFIED-BYTES 2026-11-21, HTTP 200; ZIP opened and ACCDB member listed. Local `eod-2012.zip`.
- Role: INPUT.
- Limits: Access reader unavailable in this pass. Trip start/end times, distances, mode codes, weights and person joins remain unverified. Pre-pandemic travel cannot represent current telework directly.
- Use in the simulation: candidate ordered trip chains, trip frequency and mode by person group; parse records before adoption.

### ENE July–September 2026
- Link: https://www.ine.gob.cl/docs/default-source/ocupacion-y-desocupacion/bbdd/2026/csv/ene-2026-08-jas.csv?sfvrsn=11d31bf4_4&download=true
- Contents: Person rows; `ano_trimestre`, `mes_central`, `ano_encuesta`, `mes_encuesta`, `region`, `r_p_c`, `sexo`, `edad`, `b18_region`, `b18_codigo`, `habituales`, `efectivas` and survey design fields.
- Granularity: person; residence/work comuna codes. Survey precision is not guaranteed at comuna level. Weekly work hours within a rolling quarter.
- Year: 2026, July–September.
- Access: direct semicolon CSV, full advertised size 37,971,063 bytes; sampled 65,536 bytes.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 206; 222 columns and first records read. First record: year 2026, central month 8, interview month 7, region 2, residence code 2101, age 44, sex 2. This sample is not an RM extract.
- Role: INPUT.
- Limits: Weekly hours do not show work arrival/departure times. Geographic codes do not make estimates representative at that geography. No individual linkage to ENUT. Telework field semantics not checked.
- Use in the simulation: work participation and weekly-hour constraints; candidate home/work comuna allocation after weighted RM extraction.

[ENE codebook](https://www.ine.gob.cl/docs/default-source/ocupacion-y-desocupacion/bbdd/libro-de-codigos/codigos-ene-2020.pdf) downloaded HTTP 200, 2,467,678 bytes and text read. It identifies `b18_codigo` as work comuna (or foreign country); 88888/99999 are missing-value categories. `habituales` and `efectivas` use 888/999 for unknown/nonresponse, not real hours.

### IX EPF person and household income
- Link: https://www.ine.gob.cl/docs/default-source/encuesta-de-presupuestos-familiares/bbdd/ix-epf-(octubre-2021---septiembre-2022)/base-personas-ix-epf-(formato-csv).csv?sfvrsn=8138b0c8_8&download=true
- Contents: Person rows; `macrozona`, `folio_v`, `folio`, `persona`, `sexo`, `ident_genero`, `edad`, `npersonas`, `fe`, `cse`, `ing_total_hogar_hd`, `ing_disp_hog_hd_pc` and other income components.
- Granularity: person/household; public macrozone, not comuna.
- Year: October 2021–September 2022.
- Access: direct semicolon CSV, 120,870,170 bytes advertised; 65,536-byte sample. Decimal commas and UTF-8 BOM.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 206; 314 columns and records read. First record: `macrozona=1`, `folio=44-1`, person 1, age 47, sex 1, household size 2. Macrozone labels not decoded here.
- Role: INPUT.
- Limits: Do not assume macrozone 1 is Santiago without the dictionary. No hourly activity or trip sequence. Household income fields repeat across people.
- Use in the simulation: household budget distributions and income constraints after geographic label checks.

### IX EPF expenditure
- Link: https://www.ine.gob.cl/docs/default-source/encuesta-de-presupuestos-familiares/bbdd/ix-epf-(octubre-2021---septiembre-2022)/base-gastos-ix-epf-(formato-csv).csv?sfvrsn=2b58ad51_4&download=true
- Contents: Household/item rows; `macrozona`, `folio_v`, `folio`, `fe`, `estrato_muestreo`, `var_unit`, `ccif`, `d`, `g`, `c`, `sc`, `p`, `gasto`, `glosa_ccif`.
- Granularity: household expenditure category; macrozone. No destination point or trip time.
- Year: October 2021–September 2022.
- Access: direct semicolon CSV, 152,107,750 bytes advertised; 65,536-byte sample; decimal commas.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 206; 14 columns and first records read. First record: `folio=44-1`, `ccif=01.1.1.03.01`, PAN CORRIENTE A GRANEL, `gasto=2,39142857142857e+04`.
- Role: INPUT.
- Limits: Sample proves expenditure schema, not transport spending coverage. Spending is not trip frequency; price and quantity differ. Read the CCIF dictionary before filtering transport.
- Use in the simulation: affordability constraints; join to person file on household folio within this survey only.

### Santiago–Rancagua stated preference, May 2026
- Link: https://zenodo.org/records/21270302
- Contents: Respondent/choice-task rows; `age`, `gender`, `income_gse`, `household_size`, `has_licence`, `n_cars_household`, `destination`, `last_trip_purpose`, `last_mile_mode`, `od_design`, `task_id`, `choice`; mode-specific travel time, cost, headway, crowding, access time and availability.
- Granularity: person and hypothetical choice task; interurban corridor, not all Santiago.
- Year: May 2026.
- Access: [ZIP](https://zenodo.org/records/21270302/files/Transporte-FCFM/Santiago-Rancagua-SP-dataset-v1.0.0.zip?download=1), 601,426 bytes; CSV and dictionary. CC BY 4.0. Respect Zenodo's 10-second crawl delay.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200; 5,460 rows, 99 columns, 420 respondents. First respondent: age 41, gender 2, income group 5, destination 3/Paine.
- Role: INPUT.
- Limits: Task 13 is an attention check; only 5,040 tasks are substantive choices. This is stated preference, not observed trip demand. Example fitted coefficients in the archive are not validated citywide parameters.
- Use in the simulation: estimate corridor-specific sensitivity to time, cost and access conditions; transfer to urban modes is an explicit assumption.

### CEDEUS Encuesta de Movilidad de Santiago 2024
- Link: https://www.cedeus.cl/download/44506/?tmstv=1724265104
- Contents: Report of April–May mobility survey; 3,609 complete responses, 2,217 retained after requiring at least two trips. Microdata columns not found.
- Granularity: aggregate adult travel patterns; public respondent records unavailable in this check.
- Year: 2024.
- Access: report PDF readable by web tool; curl returned 12,028-byte HTML challenge. Report licence CC BY-NC 4.0.
- Check: UNVERIFIED for downloadable data: HTTP 200 body is HTML, not PDF. Report text read through web tool; no microdata rows read.
- Role: TARGET.
- Limits: Candidate comparison only. Adults over 18; excluding people with fewer than two trips prevents estimation of a zero-trip rate. The report is complementary to EOD, not a replacement.
- Use in the simulation: possible recent aggregate comparison after matching the survey population; not a citywide daily trip-frequency distribution.

### CASEN 2024 commute questions
- Link: https://observatorio.ministeriodesarrollosocial.gob.cl/encuesta-casen-2024
- Contents: Questionnaire `o28a` gives usual home-to-main-work travel time, H:MM; `o28c` gives the mode used for the longest distance. Public microdata also have a separate comuna/province file.
- Granularity: employed person; usual commute, not daily diary. Geographic inference must follow survey design.
- Year: 2024 survey edition.
- Access: [questionnaire PDF](https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/casen/2024/Cuestionario_Casen_2024.pdf), HTTP 200, 1,301,618 bytes. [Stata base](https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/casen/2024/casen_2024.dta) range request returned HTTP 500 and 27 bytes. Other formats are listed but not downloaded in this pass.
- Check: UNVERIFIED for microdata rows; questionnaire page 27 read locally. The 27-byte Stata response is an error, not data. Another swarm report, `verify-people-work.md`, reports RData records; that claim was not independently repeated here.
- Role: INPUT.
- Limits: Questionnaire fields are not verified database column names. o28c combines bus, metro and shared taxi as public transport. A 0:00 journey includes work at home or routes that start at home; it is not a clean telework flag. 8:88 means unknown.
- Use in the simulation: commute duration and coarse mode by worker group after importing a working microdata format.

## Critique

1. No checked source here provides a current citywide hourly activity diary. ENUT durations do not determine activity order. EOD is old and its Access records remain unread in this pass.
2. Geography and samples do not align. ENUT2023 RM=7 differs from CUT13; EPF stops at macrozone; the 2026 choice study covers one interurban corridor. No cross-survey person join exists.
3. EPF and ENE checks are bounded samples, not complete RM extracts. CASEN Stata failed, EMS microdata were not found, and EOD fields remain pending. No weights, coefficients, or predictive score were fitted. No named-person data were collected.
