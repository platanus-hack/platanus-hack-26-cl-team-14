# MUT 2023 historical boarding records

Official records support a descriptive comparison before and after MUT's public opening. They do not establish its causal effect or validate the closure scenario.

## Sources and extraction

Checked and retrieved 2026-11-21. The [primary project contractor](https://www.fleischmann.cl/ingeniero-en-rob%C3%B3tica) gives the opening date as **July 13, 2023**. This does not establish when offices became occupied or when modeled jobs were present.

The [DTPM archive index](https://www.dtpm.cl/index.php/documentos/matrices-de-viaje) links both files:

| Cut | Official archive | Boarding member |
| --- | --- | --- |
| Pre-opening | [April 2023 ZIP](https://www.dtpm.cl/descargas/modelos_y_matrices/Tablas%20de%20Subidas%20y%20Bajadas.zip) | `Tablas de Subidas y Bajadas/2023.04 Matriz_sub_SS_MH.xlsb` |
| Post-opening | [August 2023 ZIP](https://www.dtpm.cl/descargas/modelos_y_matrices/Tablas%20de%20subidas%20y%20bajadas%20ago23.zip) | `Tablas de subidas y bajadas/2023.08 Matriz_sub_SS_MH.xlsb` |

Read the selected boarding members through bounded HTTP 206 ranges, then decoded them with pyxlsb 1.0.10. No raw card records were requested. The [provenance JSON](../../data/validation/mut_2023/provenance.json) records exact member hashes, source notes, headers, row numbers, and coverage. The [CSV](../../data/validation/mut_2023/tobalaba_half_hours.csv) preserves all 192 Tobalaba half-hour cells, including nulls as empty fields.

Both `FICHA_DATOS` sheets define **weekday-average estimated boardings by stop or Metro station and half-hour, excluding evasion**. April covers April 17–21; August covers August 7–11. August row metadata says `Semana 2 (7-13 agosto 2023)`; its source sheet and worksheet title explicitly restrict the mean to weekdays August 7–11. These are sampled weekday averages, not monthly totals. The archive index lists seven-day cuts; these boarding members contain only `subidas_prom_laboral` and the source sheet.

## Verified line records

| Line | April worksheet row | April weekday average | August worksheet row | August weekday average | Difference | Percent change |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| L1 | 65177 | 89,496.0 | 65945 | 91,977.2 | +2,481.2 | +2.7724% |
| L4 | 65178 | 53,261.6 | 65946 | 64,129.2 | +10,867.6 | +20.4042% |

Row numbers are one-based. Percent change is `100 × (August / April − 1)`. Each value is the sum of recorded numeric half-hours and equals the publisher's `TOTAL` exactly.

Each workbook has exactly two TOBALABA-containing rows: `parada SIMT=TOBALABA`, with `servicio Sonda=L1` or `L4`, each once. `Comuna=PROVIDENCIA`; `paradero` and `servicio usuario` are blank. There is no explicit mode column. The Metro description and line keys identify these records. There is no entrance, platform, direction, or MUT visitor split.

All four rows have the same 48 half-hour labels. Each has 36 numeric cells from 05:30 through 23:00. The 12 remaining cells are null: 00:00 through 05:00, plus 23:30. Preserve nulls; the sums do not estimate missing periods.

April has 57 columns; August has 56. April's `segunda_parada=NO` field is absent in August, and `zona_paga` becomes `zona paga`. Read columns by header name. The station/line keys, sample type, time labels, and numeric coverage agree. Processing-version equivalence beyond the publisher's notes remains unverified.

## Interpretation limits

Keep L1 and L4 separate. The source notes do not define within-Metro interchange counting or line assignment. Their sum is not verified as unique station entries. Do not merge these records with the 2026 single-station TOBALABA layer without resolving that accounting difference.

The same files contain CRISTOBAL COLON/L4, EL GOLF/L1, and LOS LEONES/L1 records. Their row references are in the provenance JSON. This establishes availability for later descriptive comparisons, not valid controls.

April and August differ in season. COVID recovery, service supply, fares, network changes, economic activity, adjacent development, and occupancy ramp can also change boardings. Nearby stations can share or receive displaced demand. Public access, retail, and station access changes were bundled. These records cannot isolate MUT's contribution, validate a modeled closure effect, or turn baseline WAPE into intervention evidence.
