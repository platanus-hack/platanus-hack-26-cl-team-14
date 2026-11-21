# EOD 2012 sectors for Santiago destinations and home blocks

Join each table to the input with the same name in the parent directory. Use `block_id` for homes, `rbd` for schools, and `osm_id` for activity points. For a higher-education campus use the four fields `institution`, `campus`, `cut`, and `year` together. Each source row has one output row. Blank sectors identify unmatched inputs. A sector can join directly to the tour profiles’ `home_sector` or `destination_sector`.

| Source rows | Sector assigned | Outside survey zones | Missing coordinates | Conflicting sector |
| --- | ---: | ---: | ---: | ---: |
| 137 higher-education campuses | 68 | 0 | 69 | 0 |
| 16,697 activity points | 16,665 | 7 | 0 | 25 |
| 52,646 home blocks | 50,699 | 1,901 | 0 | 46 |
| 3,220 schools | 3,105 | 112 | 0 | 3 |

The assigned campus rows contain 346,778 in-person registrations. Another 176,824 lack coordinates. Blocks outside the survey polygons contain 198,904 residents. The unresolved zone contains 5,201 residents. Schools outside contain 38,038 enrollments and the unresolved zone contains 2,202. School enrollment includes preschool.

## Survey geometry and sector evidence

The local EOD 2012 archive contains 866 survey zones in 45 comunas. Its projection file specifies WGS84 / UTM 19S (EPSG:32719). The builder projects longitude and latitude from EPSG:4326 into that CRS before intersecting survey polygons. It repairs five self-intersecting geometries with GEOS. It keeps overlapping zone IDs in ascending order, separated by a vertical bar. Eight blocks, two schools and one activity point hit multiple polygons with the same sector. These rows have status `shared_sector`. The national survey archive and the local university mirror have not been compared byte for byte. [Local survey inventory and provenance](../../../context/research/sources/verify-transport.md)

The polygon attribute `Zona` is the EOD survey zone number. `Com` is the polygon's comuna CUT. Sector is absent from the polygon attributes. The survey dictionary gives 1 Norte, 2 Poniente, 3 Oriente, 4 Centro, 5 Sur, 6 Sur-Oriente and 7 Extensión Sur-Poniente. Code 9 means exterior a RM. It has no polygon in this file. Code 0 has no sector label. A point outside the polygons can still lie inside the RM. The builder leaves its sector blank.

`zones.csv` records the sector evidence for each polygon. For 849 zones, direct household or trip evidence agrees with the household sector for the polygon's comuna. For 16 zones with no direct observation, the builder uses the comuna's household sector. All 45 comunas have household evidence and each maps to a single household sector. Zone 471 in Peñalolén has conflicting evidence. Fifteen households, 168 trip origins and 172 trip destinations give sector 6. One trip origin gives sector 3. Its sector stays blank. Counts refer to unweighted source records. These assignments use survey geography and do not depend on station arrivals or boardings.

The geographic join can place a point in a different comuna from the input CUT. This happens for 304 blocks, 71 activity points, 15 schools and one campus. For example, SIES reports AIEP Bellavista under Santiago (13101). Its campus point falls inside EOD zone 516 in Providencia (13123). The output keeps the SIES key and the geographic sector. Differences can reflect boundaries, dates, address quality or the source's reporting convention.

## Rebuild

From the repository root, pass the path to the extracted `EOD_STGO` folder used by the other EOD builders. This command requires local copies of PyShp, pyproj and Shapely 2.1 or newer. It reads the zone geometry and its projection, the zone attribute table, `Hogares.csv` and `viajes.csv`. It exports only aggregate zone evidence and geographic joins.

```sh
uv run --offline --with pyshp --with pyproj --with 'shapely>=2.1,<3' python tools/pack/eod_point_sectors.py /path/to/EOD_STGO
```
