# Data pack

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
| `longitude`, `latitude` | area-weighted centroid of the `Comunal_CPV24` polygon |

- Totals: population 7,400,741, households 2,599,621, dwellings 2,763,387. Providencia `13123`: 143,974 / 69,157 / 76,376.
- Check: all origins of `work_od.csv` are in this file. RM to RM workers in `work_od.csv`: 2,688,814.
- `communes.geojson`: the same 52 polygons (MultiPolygon, properties `cut` as 5-digit text, `name`, `population`). The builder rounds coordinates to 4 decimals (about 10 m) and removes repeated points. 2.0 MB. For the browser only.
- City-scale check of `communes.geojson`: 52 features, each 1 polygon, 0 holes. Area sum about 15,350 km² (approximate flat projection). The rural comunas are most of the area: San José de Maipo alone is 4,975 km². A frame on all 52 shows the city small.
- `communes_urban.geojson`: the 34 comunas of Gran Santiago, for the city-scale view. List: the 32 comunas of Provincia de Santiago (CUT `131xx`) plus Puente Alto `13201` and San Bernardo `13401`. This is the common 34-comuna definition of Gran Santiago (Wikipedia "Gran Santiago"). Population 6,260,194. 0.8 MB. Lo Barnechea `13115` is in the list but is mostly Andes, so its polygon reaches -70.18 lon. Fit the camera to the people, not to this bbox: the population-weighted 0.5–99.5 percentile box of the `blocks.csv` rows of these 34 cuts is lon -70.7991 to -70.5031, lat -33.6324 to -33.3379.
