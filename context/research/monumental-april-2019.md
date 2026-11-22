# Metro entries after Colo-Colo vs O’Higgins in 2019

Colo-Colo played O’Higgins at the Monumental on Saturday 13 April 2019. The reported kickoff was 15:30 and the reported attendance was 22,217. The 17:00 half-hour can include the final whistle. This is a candidate for a third comparison of stadium departures. [Match report](https://chile.as.com/chile/2019/04/13/futbol/1555180379_555588.html) · [Attendance record](https://www.historiadecolocolo.com/nacional/2019/ohiggins2019-1.html)

| Station | 17:00 | 17:30 | 18:00 | 18:30 |
| --- | ---: | ---: | ---: | ---: |
| Pedrero | 363 | 2,164 | 1,146 | 348 |
| Mirador | 560 | 1,241 | 977 | 766 |

The table counts dated Metro boarding records for all riders. All selected rows have service expansion factor 1. The two stations recorded 3,405 entries from 17:30 to 18:00. These counts do not identify spectators. I selected the stations and time bins before reading the counts. [Selection record](https://discord.com/channels/1573514576400752640/1573863187672207411/1573881929718042626)

To reproduce the table, extract `etapas_19/etapas20190413_transparencia.rar` from the April stages ZIP. The RAR SHA-256 is `8a900dc6c1869996262314b210fe054b643f71ebc57b44cb23a5aee548615707`. Read the semicolon-separated CSV. Select `tipo_transporte = METRO`, `tiempo_subida` on 13 April, and `par_subida` of `PEDRERO` or `MIRADOR`. Count rows for each `mh_subida` above. The aggregate boarding CSV has no date field. [April stages](https://www.dtpm.cl/descargas/tablas/etapas_19.zip) · [Field definitions](https://www.dtpm.cl/descargas/tablas/etapas_19.txt)

The August weekend archive has timestamps, so 10 August 2019 is a possible same-year reference. Colo-Colo played in Rancagua the next day. Universidad de Chile hosted Antofagasta at Estadio Nacional at 15:00 on 10 August. Audax Italiano hosted Huachipato in La Florida at 20:00. Both fixtures can affect local travel. DTPM GTFS uses the same Line 5 terminal departure headways in both date windows. In each direction they are 250 and 280 seconds before 18:00, then 341 and 377 seconds. This is the schedule at the terminals. Background events and actual service on 10 August are unverified. Use the comparison as descriptive evidence. The two dates are in different months and the August date has other matches. [August stages](https://www.dtpm.cl/descargas/tablas/tabla-etapas.zip) · [2019 GTFS archive](https://www.dtpm.cl/descargas/gtfs/2019.zip) · [Fixture records](https://chile.as.com/chile/2019/08/09/futbol/1565351833_019300.html)

| Reference station | 10 Aug 17:00 | 17:30 | 18:00 | 18:30 |
| --- | ---: | ---: | ---: | ---: |
| Pedrero | 207 | 173 | 162 | 190 |
| Mirador | 500 | 577 | 581 | 612 |

The observed total in the eight cells is 7,565 entries on 13 April and 3,002 on 10 August. The largest half-hour difference is 2,655 entries from 17:30 to 18:00 across both stations. This is a difference in all-rider records. The comparison has no engine score.

To reproduce the reference table, extract `tabla etapas/etapas201908_sab_dom_transparencia.zip` from the August stages archive. The inner ZIP SHA-256 is `ce9c2f6283d427b34d5d9fc0f19cfb923af87f78503365c3f6db25ecd87e3786`. Read its semicolon-separated CSV with the same station and half-hour filters. Select 10 August in `tiempo_subida`. All selected rows have service expansion factor 1.
