# Santiago transport source verification

Checked 2026-11-21 against [the transport catalog](../santiago-transport.md) at commit `5722e3b`. Sizes below are bytes. VERIFIED-ROWS means we opened data records. The April DTPM trip, stage, boarding, and matching timetable files belong to the other verifiers. The [boarding workbook check](score-targets.md) currently records VERIFIED-BYTES for that workbook. Its script inspected metadata and shared strings.

### SECTRA Santiago EOD database and zones — OK as a historical input

The [SECTRA index](https://www.sectra.gob.cl/info-territorial/region-metropolitana-de-santiago/) returned HTTP 200 (121,970 bytes). Its [database](https://drive.google.com/file/d/193675NwL6QNDs6VJFHU0mXiX5-6SdBPJ/view) and [zoning](https://drive.google.com/file/d/1SM2C5b3jhbmYCbAkmtg2yWRU2dMQkGRl/view) preview pages returned HTTP 200 (86,675 and 87,106 bytes). The previews list ZIP sizes of 16,324,719 and 336,780 bytes. The download host bars automated retrieval. The official archive contents and their equivalence to the mirror remain UNVERIFIED.

**VERIFIED-ROWS** from the [Universidad de Chile mirror](https://users.dcc.uchile.cl/~egraells/gds-data/eod2012.tgz), HTTP 200, 9,445,461 bytes. The compressed archive contains household, person, trip, stage, vehicle, lookup, and zone records. The survey name is “2012.” Across the household table, `Fecha` runs from 2012-06-26 to 2013-11-25. One trip starts at 22:30 and ends at 23:40. It links zones 400 and 407, with survey comuna 94 (Maipú) at both ends, purpose 7 (return home), and normal-weekday weight 1. Both zone polygons carry comuna CUT 13119. The zoning DBF has 866 records. Zones 103 and 104 give comuna 13105, El Bosque. The projection file specifies WGS 84 / UTM zone 19S.

Of 227,182 trip endpoints, 222,670 match a survey polygon. The remaining 4,512 carry zone 0 or 999. Keep those endpoints in a separate unknown-location group.

**INPUT.** Use the trip's survey weight for its day type and season when estimating mode, purpose, or OD shares. The counts below are unweighted records. Join the survey's comuna and zone codes through their dictionaries and geometry before comparing with current administrative units or DTPM zones.

Run the [standard-library scraper](../../../tools/scrape/eod2012.py) from the repository root with `/usr/bin/python3 tools/scrape/eod2012.py` on this Mac. It saves the archive and a table inventory with columns, byte sizes, and data-row counts under `data/raw/eod2012/`. The mirror archive SHA-256 is `6dce8fb939fafc7b236654ecd511caec9069773fede6692abb579a9e7360d726`. The count excludes headers and blank rows. Each table below is a CSV in the mirror. This inventory includes the working copy `Modo - copia.csv`. The [PLUMAS EOD importer](https://github.com/PLUMAS-research/palomatrix-dtpm/blob/main/herramientas/construir_datos_eod.py) also uses the university archive and handles its mixed delimiters and encodings.

**Survey tables (8)**

| Table | Rows | Table | Rows |
| --- | ---: | --- | ---: |
| `DistanciaViaje` | 113,591 | `personas` | 60,054 |
| `Edadpersonas` | 60,054 | `Vehiculo` | 8,887 |
| `Etapas` | 133,444 | `viajes` | 113,591 |
| `Hogares` | 18,264 | `ViajesDifusion` | 113,591 |

**Lookup tables in `Tablas_parametros/` (60)**

| Table | Rows | Table | Rows |
| --- | ---: | --- | ---: |
| `Actividad` | 7 | `ModoAgregado` | 18 |
| `ActividadDestino` | 8 | `ModoDifusion` | 9 |
| `ActividadEmpresa` | 8 | `ModoestacionaBicicleta` | 2 |
| `AdultoMayor` | 2 | `ModoMotor` | 3 |
| `Autopista` | 9 | `ModoPriPub` | 5 |
| `CirculacionBicicleta` | 4 | `NoUsaAutopista` | 4 |
| `CodigoTiempo` | 6 | `NoUsaTransantiago` | 8 |
| `Combustible` | 4 | `NoViaja` | 6 |
| `Comuna` | 106 | `Ocupacion` | 9 |
| `Comunas` | 106 | `PaseEscolar` | 2 |
| `ConoceSantiago` | 2 | `Periodo` | 6 |
| `Discapacidad` | 9 | `Propiedad` | 7 |
| `Donde Estudia` | 9 | `PropiedadBicicleta` | 3 |
| `EdadVehiculo` | 6 | `PropiedadVehiculo` | 4 |
| `Estaciona` | 4 | `Proposito` | 14 |
| `EstacionaBicicleta` | 3 | `PropositoAgregado` | 4 |
| `EstacionMetro` | 108 | `RecorridoTransantiago` | 356 |
| `EstacionMetroCambio` | 16 | `Relacion` | 7 |
| `EstacionTren` | 21 | `Sector` | 8 |
| `Estudios` | 11 | `SelloVerde` | 2 |
| `Formapago` | 8 | `Sexo` | 2 |
| `HorarioMetro` | 3 | `SexoViaje` | 2 |
| `IngresoImputado` | 3 | `Temporada` | 2 |
| `JornadaTrabajo` | 4 | `TiempoMedio` | 9 |
| `LicenciaConducir` | 6 | `TieneIngresos` | 4 |
| `MarcaVehiculo` | 179 | `TipoDia` | 2 |
| `MediosUsados` | 292 | `TipoVeh` | 9 |
| `MedioViajeRestriccion` | 11 | `TramoIngreso` | 7 |
| `Modo - copia` | 18 | `UsaCiclovia` | 3 |
| `Modo` | 18 | `UsoHabitualBicicleta` | 2 |

### DTPM November static GTFS, including Metro — OK

**VERIFIED-ROWS · INPUT.** The [index](https://www.dtpm.cl/index.php/gtfs-vigente) returned HTTP 200 (38,148 bytes). The [November feed](https://www.dtpm.cl/descargas/gtfs/GTFS_20261114_v2.zip) returned HTTP 200 (12,829,198 bytes). Its six service calendars run from 2026-11-14 to 2026-12-31. Counts include 425 routes, 26,137 trips, 1,098,717 stop times, and 18,486 records in `stops.txt`. That last table includes stations, platforms, and other location types.

In `routes.txt`, Metro has agency ID `M` and seven route IDs (L1, L2, L3, L4, L4A, L5, L6). In `stops.txt`, Tobalaba is `stop_id=TB-TOB`, `location_type=1`, at −33.418120, −70.601929. Nearby bus stop PC155 also contains “Tobalaba” in its name. Use station IDs and location type to select Metro stations. These rows describe scheduled supply in November and December.

### Metro annual ridership — OK at network scale

**VERIFIED-ROWS · TARGET.** Metro S.A. supplies table 42 on printed page 194 of the [DTPM 2025 management report](https://www.dtpm.cl/descargas/memoria/Informe%20de%20Gestion%20DTPM2025.pdf). The PDF returned HTTP 200 (84,595,457 bytes). The table covers 2012–2025, with one column per year. It reports 640 million annual entries in 2024 and 661 million in 2025, rounded. The 2025 mean weekday count was 2.212 million. Line 1 accounted for 33.85% of annual entries according to the accompanying text. The table has no station identifier. Use the annual totals to check the simulated Santiago Metro network at the same scale.

### Geofabrik OpenStreetMap roads — OK

**VERIFIED-ROWS · INPUT.** The [Chile index](https://download.geofabrik.de/south-america/chile/index.html) lists the [2026-11-20 shapefile ZIP](https://download.geofabrik.de/south-america/chile-261120-free.shp.zip). HEAD returned HTTP 200, 764,445,166 bytes. HTTP 206 range reads retrieved 8,738,228 bytes for the compressed road attribute member and 65,536 bytes from the geometry member. The DBF header lists 1,016,395 national road records. We matched central Santiago geometry and attributes by record position.

OSM way 7981313 is Teatinos (`fclass=secondary`, `code=5114`, `oneway=F`, `maxspeed=50`). Its bounding box is −70.655841, −33.432434 to −70.655553, −33.432367. Way 7981317 is San Antonio (`primary`, 5113). Other fields include `ref`, `bridge`, `tunnel`, and `layer`. Use geometry for the comuna join. The snapshot carries ODbL terms. Speed limits and road classes describe the network. Travel speed and traffic volume need observations.

### SECTRA cycleways — OK

**VERIFIED-ROWS · INPUT.** The [catalog page](https://geoportal.cl/index.php/geoportal/catalog/36392/ciclovias) returned HTTP 200 (38,701 bytes). The July 2026 [layer metadata](https://services6.arcgis.com/feQ9HId8vmgonvvD/ArcGIS/rest/services/CICLOV_validVisor_WFL1/FeatureServer/0) returned HTTP 200 (24,364 bytes). A [three-feature query](https://services6.arcgis.com/feQ9HId8vmgonvvD/ArcGIS/rest/services/CICLOV_validVisor_WFL1/FeatureServer/0/query?where=CUT_COM%3D13101%20AND%20ETAPA%3D%27existentes%27&resultRecordCount=3&outFields=IDENTIFICA%2CCUT_COM%2CEJE_VIA%2CCOMUNA%2CKM%2CETAPA%2CYEAR_EJECUCION%2COBJECTID&outSR=4326&f=geojson) returned HTTP 200 (3,731 bytes).

In Santiago comuna 13101, Tarapacá–Curicó has ID 131010001, length 2.25 km, and execution year 2014. Santa Isabel (131010003) has 2.38 km, 2014. Victoria Subercaseaux (131010004) has 0.23 km, 2017. Querying `CUT_REG=13` and `ETAPA='existentes'` returned 606 regional segments. Keep the stage and year filters when building a historical network. These lines describe infrastructure.

### MINVU bicycle counters — OK for observed site-days

**VERIFIED-ROWS · TARGET.** The [MINVU source page](https://www.minvu.gob.cl/ciclovias-minvu/) returned HTTP 200 (47,985 bytes). Its legacy [public counter page](http://www.eco-public.com/ParcPublic/?id=5737) returned HTTP 200 (12,194 bytes) over HTTP. The HTTPS version had an expired certificate. The [public HTTPS inventory](https://www.eco-visio.net/api/aladdin/1.0.0/pbl/publicwebpageplus/5737?withNull=true) returned HTTP 200 (142,072 bytes), with 172 sites. Fields include `idPdc`, `nom`, `lat`, `lon`, and flow IDs in `pratique`.

At **Alameda con Portugal**, site 300062559 (−33.439979, −70.640171), the [daily series](https://www.eco-visio.net/api/aladdin/1.0.0/pbl/publicwebpageplus/data/300062559?idOrganisme=5737&idPdc=300062559&debut=11%2F04%2F2026&fin=18%2F04%2F2026&interval=4&flowIds=353658456%3B353658457%3B353658458%3B353658459) returned HTTP 200 (155 bytes). These date/count pairs use all four published flows. The request uses day/month/year and excludes the end date. The response uses month/day/year.

The inventory has 14 central Santiago sites with “Alameda” in the name. All returned seven daily bicycle totals (98 site-days, HTTP 200, 155 bytes per site). Twelve sites list only bicycles. At the two TÓTEM sites, the queries selected mode 2 (bicycle) and excluded mode 13 (scooter). The [portal legend](http://www.eco-public.com/ParcPublic/styles/eco-font.css) defines these codes.

| Site (daily bicycle passages, April 2026) | 11 Sat | 12 Sun | 13 Mon | 14 Tue | 15 Wed | 16 Thu | 17 Fri |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| [Alameda con Portugal](https://www.eco-visio.net/api/aladdin/1.0.0/pbl/publicwebpageplus/data/300062559?idOrganisme=5737&idPdc=300062559&debut=11%2F04%2F2026&fin=18%2F04%2F2026&interval=4&flowIds=353658456%3B353658457%3B353658458%3B353658459) | 2,330 | 3,326 | 3,022 | 3,136 | 3,620 | 3,277 | 3,223 |
| [Alameda Norte-Abate Molina](https://www.eco-visio.net/api/aladdin/1.0.0/pbl/publicwebpageplus/data/300062560?idOrganisme=5737&idPdc=300062560&debut=11%2F04%2F2026&fin=18%2F04%2F2026&interval=4&flowIds=353658467%3B353658468) | 1,321 | 1,531 | 1,816 | 1,799 | 1,953 | 1,838 | 1,784 |
| [Alameda Norte-Brasil](https://www.eco-visio.net/api/aladdin/1.0.0/pbl/publicwebpageplus/data/300062561?idOrganisme=5737&idPdc=300062561&debut=11%2F04%2F2026&fin=18%2F04%2F2026&interval=4&flowIds=353658471%3B353658472) | 1,577 | 1,820 | 2,187 | 2,187 | 2,382 | 2,257 | 2,162 |
| [Alameda Norte-Paseo Tenderini](https://www.eco-visio.net/api/aladdin/1.0.0/pbl/publicwebpageplus/data/300062562?idOrganisme=5737&idPdc=300062562&debut=11%2F04%2F2026&fin=18%2F04%2F2026&interval=4&flowIds=353658475%3B353658476) | 1,767 | 2,343 | 2,485 | 2,547 | 2,828 | 2,603 | 2,493 |
| [Alameda Norte-República](https://www.eco-visio.net/api/aladdin/1.0.0/pbl/publicwebpageplus/data/300062563?idOrganisme=5737&idPdc=300062563&debut=11%2F04%2F2026&fin=18%2F04%2F2026&interval=4&flowIds=353658479%3B353658480) | 1,362 | 1,597 | 1,904 | 1,820 | 2,084 | 1,951 | 1,887 |
| [Alameda Sur-Abate Molina](https://www.eco-visio.net/api/aladdin/1.0.0/pbl/publicwebpageplus/data/300062564?idOrganisme=5737&idPdc=300062564&debut=11%2F04%2F2026&fin=18%2F04%2F2026&interval=4&flowIds=353658483%3B353658484) | 1,269 | 1,412 | 1,680 | 1,700 | 1,833 | 1,719 | 1,697 |
| [Alameda Sur-Dieciocho](https://www.eco-visio.net/api/aladdin/1.0.0/pbl/publicwebpageplus/data/300062565?idOrganisme=5737&idPdc=300062565&debut=11%2F04%2F2026&fin=18%2F04%2F2026&interval=4&flowIds=353658487%3B353658488) | 1,704 | 1,908 | 2,426 | 2,427 | 2,660 | 2,425 | 2,404 |
| [Alameda Sur-Latorre](https://www.eco-visio.net/api/aladdin/1.0.0/pbl/publicwebpageplus/data/300062566?idOrganisme=5737&idPdc=300062566&debut=11%2F04%2F2026&fin=18%2F04%2F2026&interval=4&flowIds=353658491%3B353658492) | 1,551 | 1,713 | 2,150 | 2,144 | 2,319 | 2,167 | 2,100 |
| [Alameda Sur-Pasaje Tenderini](https://www.eco-visio.net/api/aladdin/1.0.0/pbl/publicwebpageplus/data/300062567?idOrganisme=5737&idPdc=300062567&debut=11%2F04%2F2026&fin=18%2F04%2F2026&interval=4&flowIds=353658495%3B353658496) | 1,734 | 2,584 | 2,902 | 2,968 | 3,276 | 2,996 | 3,033 |
| [Alameda Sur-República](https://www.eco-visio.net/api/aladdin/1.0.0/pbl/publicwebpageplus/data/300062568?idOrganisme=5737&idPdc=300062568&debut=11%2F04%2F2026&fin=18%2F04%2F2026&interval=4&flowIds=353658499%3B353658500) | 1,146 | 1,304 | 1,509 | 1,562 | 1,694 | 1,600 | 1,564 |
| [Alameda-Tucapel](https://www.eco-visio.net/api/aladdin/1.0.0/pbl/publicwebpageplus/data/300062569?idOrganisme=5737&idPdc=300062569&debut=11%2F04%2F2026&fin=18%2F04%2F2026&interval=4&flowIds=353658503%3B353658504) | 1,357 | 2,107 | 2,691 | 2,600 | 2,967 | 2,701 | 2,648 |
| [Alameda-Vicuña Mackenna](https://www.eco-visio.net/api/aladdin/1.0.0/pbl/publicwebpageplus/data/300062570?idOrganisme=5737&idPdc=300062570&debut=11%2F04%2F2026&fin=18%2F04%2F2026&interval=4&flowIds=353658507%3B353658508%3B353658509%3B353658510) | 2,574 | 3,854 | 3,336 | 3,569 | 3,954 | 3,483 | 3,435 |
| [TÓTEM ALAMEDA SUR](https://www.eco-visio.net/api/aladdin/1.0.0/pbl/publicwebpageplus/data/300062720?idOrganisme=5737&idPdc=300062720&debut=11%2F04%2F2026&fin=18%2F04%2F2026&interval=4&flowIds=353660023%3B353660024) | 1,092 | 2,070 | 2,325 | 2,398 | 2,657 | 2,337 | 2,397 |
| [TÓTEM ALAMEDA NORTE](https://www.eco-visio.net/api/aladdin/1.0.0/pbl/publicwebpageplus/data/300062721?idOrganisme=5737&idPdc=300062721&debut=11%2F04%2F2026&fin=18%2F04%2F2026&interval=4&flowIds=353660037%3B353660038) | 1,641 | 2,051 | 2,316 | 2,375 | 2,664 | 2,385 | 2,314 |

Compare simulated passages at each counter with its daily total. One rider can pass more than one counter along Alameda. Coverage elsewhere in Santiago and counter accuracy during outages remain unverified.

### DTPM operating statistics — WRONG as a general operated-kilometer target

**VERIFIED-ROWS.** The [statistics index](https://www.dtpm.cl/index.php/noticias/estadisticas) returned HTTP 200 (58,097 bytes). The [August service-unit workbook](https://www.dtpm.cl/descargas/kilometros/08%20Kms%20Agosto2026.xls) returned HTTP 200 (2,725,376 bytes). Of its 2,163 August 1–31 rows, 2,046 have type PO. On August 1, unit U8, Saturday, A2D has 5,042.65 in `Kilómetros Día` and blank executed fields. On August 7, U10 has 49.44 in `Kilómetros POE Ejecutados` for an event at Claro Arena. **TARGET** only for a defined executed category with compatible simulated service. Adding daily and executed fields can count the same event twice.

The [September operator workbook](https://www.dtpm.cl/descargas/kilometros/17%20Kms%201%20Quincena%20Septiembre%202026.xls) returned HTTP 200 (2,666,496 bytes). It contains 552 rows for September 1–15 and units U2–U5. The two workbooks cover different operator groups. Neither sample contains a comuna or coordinates.

The [historical speed workbook](https://www.dtpm.cl/descargas/archivos/Resumen%20velocidades%20cada%20media%20hora%20vigente%20desde%2027-02-2010.xls) returned HTTP 200 (1,439,744 bytes). For T1, service 101, outbound weekday speed is 28.7363 km/h at 00:00 and 29.4237 at 00:30. The index gives an effective date of 2010-02-27. Use for historical reference only.

### CONASET crash map and district report — USELESS for the travel-demand score

**VERIFIED-ROWS · NONE.** The [portal](https://mapas-conaset.opendata.arcgis.com/) returned HTTP 200 (35,341 bytes). The [feature layer](https://services3.arcgis.com/vaJl1B5HEzZj7154/arcgis/rest/services/Base_SINIESTROS_2020_2025/FeatureServer/0) returned three Providencia points in a 1,180-byte HTTP 200 GeoJSON sample. Record 1582952 gives 2024-01-05 at 11:00, Providencia, Santiago province, Metropolitana de Santiago region, side collision, zero deaths and injuries, at −70.621437, −33.439396. Fields include `ID`, `FECHA`, `Hora`, `COMUNA`, `TIPO_SINIE`, `FALLECIDOS`, and `TOTAL_LESI`. The 2024 web map filters to urban crashes. The base layer allows an all-area query.

The [official district report](https://www.conaset.cl/wp-content/uploads/2025/09/Distritos-Circunscripciones-2024.pdf) returned HTTP 200 (2,099,465 bytes). Its 2024 district 10 row lists 1,152 crashes in Providencia, four deaths, and 274 injuries. The [base-layer count for all of Providencia](https://services3.arcgis.com/vaJl1B5HEzZj7154/arcgis/rest/services/Base_SINIESTROS_2020_2025/FeatureServer/0/query?where=A%C3%91O%3D2024%20AND%20COMUNA%3D%27Providencia%27&returnCountOnly=true&f=json) returned 1,142 crashes that year (HTTP 200, 16 bytes). The reason for the ten-record difference remains unverified. Both sources describe safety outcomes.

## Critique

1. We verified the university EOD mirror. Direct comparison with the official ZIPs and validation of the survey weights against the official expanded totals remain open.
2. Bicycle observations cover one week on a single corridor. We checked three cycleways and two street geometries. Network completeness and counter availability require wider checks.
3. The GTFS dates, Metro annual totals, and bus kilometer workbooks cover different periods and geographic units. None provides observed station crowding or current street-level car volumes.
