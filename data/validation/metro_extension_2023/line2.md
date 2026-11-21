# Line 2 extension to San Bernardo

## Comparison fixed before reading the April 2024 counts

Use mean weekday boardings from the August 2023 and April 2024 DTPM station workbooks. La Cisterna was the old terminal. Sum all Metro line records for this physical station in each cut. List April boardings at El Bosque, Observatorio, Copa Lo Martínez and Hospital El Pino separately. They had no passenger service in August.

Use the same control stations in both cuts. Keep physical stations present in both cuts. Exclude every station served by Line 2 in either cut, including interchanges. Exclude stations served by Line 3, which also had an extension in this period. Sum all line records at each remaining station. Divide total April control boardings by total August control boardings.

Multiply August boardings at La Cisterna by that ratio. Report actual April boardings minus this estimate, in boardings per weekday and as a percentage of the estimate. Report the direct before/after change too. The difference describes two survey weeks. Season, service and other changes can affect it.

## Results

The four stations opened on 27 November 2023. The table gives DTPM weekday averages for 7–11 August 2023 and 15–19 April 2024. [Opening announcement](https://mtt.gob.cl/extension-de-linea-2-comenzo-su-operacion-y-beneficiara-a-618-mil-personas/)

| Station or group | August 2023 | April 2024 |
| --- | ---: | ---: |
| La Cisterna | 155,195.2 | 40,084 |
| El Bosque | 0 | 10,841 |
| Observatorio | 0 | 6,980 |
| Copa Lo Martínez | 0 | 9,765 |
| Hospital El Pino | 0 | 23,039 |
| Four new stations | 0 | 50,625 |
| 78 comparison stations | 1,818,086.8 | 1,484,580 |

The comparison stations fell 18.34%. Applying that rate to La Cisterna gives 126,726.5. April came in 86,642.5 lower (68.37% below this reference). The direct change was −115,111.2 (−74.17%).

**The publisher changed how it reports interchanges.** August lists 93,772.8 under L2 and 61,422.4 under L4A at La Cisterna. April gives one station total and has no line field. The workbooks do not establish how they counted internal transfers across these formats. The adjusted difference compares reported figures and cannot establish a change of 86,642.5 in physical station entries. Use the April figures at the four new stations as boarding targets. Keep the old-terminal contrast descriptive.

## Sensitivity to interchange accounting

The weekday workbook gives just one line for some interchanges. We checked line membership against the March 2026 GTFS. For example, the August workbook lists Los Héroes under L1 alone, but L2 also serves it. The primary comparison excludes Los Héroes under the original rule.

A separate comparison keeps only the 71 single-line Metro stations in the control group. Their combined weekday boardings fell from 1,375,505.8 to 1,248,190 (−9.26%). This implies 140,830.4 boardings at La Cisterna. April was 100,746.4 lower (−71.54%). Changing the controls cannot resolve the accounting change at La Cisterna itself.

## Rebuild

From the repository root, place the three 2023 workbooks in the paths in the builder. Place the April 2024 workbook at `data/raw/dtpm-2023/Subida_Paradero_Estacion_2024.04.xlsb`. Run `python3 -I tools/pack/metro_extension_2023.py`. The file `line2.csv` lists every station used. Zero in August means the new station had no passenger service.

The DTPM archive page currently links an April 2025 workbook under April 2024. The direct 2024 workbook specifies weekdays 15–19 April 2024 in its source notes. [August workbook archive](https://www.dtpm.cl/descargas/modelos_y_matrices/Tablas%20de%20subidas%20y%20bajadas%20ago23.zip) · [April 2024 workbook](https://www.dtpm.cl/descargas/modelos_y_matrices/Subida_Paradero_Estacion_2024.04.xlsb) · [Archive index](https://www.dtpm.cl/index.php/documentos/matrices-de-viaje)

April 2024 workbook SHA-256: `719243bdb12fd8ac4f64df095d938a4ead8993368cf9e0489478b13e303f3cb0`.
