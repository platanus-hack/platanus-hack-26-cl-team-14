# Building-use tag coverage in Santiago

The population of interest is the 279,385 mapped footprint components in the native city view. The view covers −70.85 to −70.45 longitude and −33.65 to −33.30 latitude. A comuna’s denominator counts only its mapped footprints inside this rectangle. The measurement excludes the generated buildings. The 43 footprints without a Census block assignment remain in the city total. [Geometry and comuna assignment](../city_buildings/README.md)

An OSM activity tag can describe the use of a building or a business inside it. A building type such as `building=school` can describe the original design even if the use has changed. A point inside a footprint can identify one tenant in a mixed-use building. These signals measure what the map records. They do not establish how many people work there or visit it. [OSM building types](https://wiki.openstreetmap.org/wiki/Key:building) · [OSM building use](https://wiki.openstreetmap.org/wiki/Key:building:use) · [OSM amenities](https://wiki.openstreetmap.org/wiki/Key:amenity)

The three camera samples count footprints intersecting circles of radius 500 m in UTM zone 19S. Their centres and footprint denominators match the geometry report. Tobalaba includes 460 footprints, Puente Alto 337, and Quilicura 319. [Camera definitions](../city_buildings/README.md#three-camera-centres)

## Coverage

Of 279,385 mapped footprints, 211,032 (75.53%) have enough recorded information to assign a class. Building type supplies 195,709 assignments. An activity tag on the footprint supplies 10,628. A point inside the footprint supplies the remaining 4,695. Housing accounts for 189,119 assigned footprints. These figures do not measure current occupancy. [Aggregate counts](coverage.csv)

| Area | Assigned | Mapped | Share |
| --- | ---: | ---: | ---: |
| La Reina, highest comuna share | 11,462 | 12,161 | 94.25% |
| Recoleta, lowest comuna share | 990 | 3,123 | 31.70% |
| Tobalaba, 500 m | 420 | 460 | 91.30% |
| Puente Alto, 500 m | 273 | 337 | 81.01% |
| Quilicura, 500 m | 133 | 319 | 41.69% |

The extremes cover the 42 comunas with at least one assigned mapped footprint in the view. The table includes all 52 comuna rows. An empty share means the denominator is zero.

## Classification

Each footprint receives at most one class. First use recognized activity tags on the polygon, including explicit building use. Then use a recognized architectural type. Finally, look for tagged OSM nodes strictly inside the polygon, excluding its holes. A node on a shared boundary does not count. At the selected step, conflicting classes count once as `other`. Evidence counts report the step that supplied the assignment. The values `building=yes` and `building=roof` alone leave the use unknown. The builder lists the accepted values for housing, office, commerce, education, health, industry, public and other. [Classification rules](../../../tools/pack/osm_building_use.py)

## Reproduction

Use Python with Shapely 2.1 or later and pyproj, plus the osmium CLI. Run from the repository root. The urban PBF and original polygon export come from the [city footprint preparation](../../mvp/README.md). The [city building geometry pipeline](../city_buildings/README.md#rebuild) supplies the rounded footprints and their Census assignments.

```sh
urban=data/raw/osm-buildings-city
fill=data/raw/building-fill
work=$(mktemp -d)
osmium tags-filter -R "$urban/urban.osm.pbf" n/amenity n/shop n/office n/leisure n/healthcare n/tourism -o "$work/points.pbf"
osmium export --geometry-types=point -f geojsonseq "$work/points.pbf" -o "$work/points.geojsonseq"
python3 tools/pack/osm_building_use.py "$urban/polygons.geojsonseq" "$fill" "$work/points.geojsonseq" data/validation/osm_building_use/coverage.csv
```

| Input | SHA256 |
| --- | --- |
| urban.osm.pbf | 4a1cfdb3ffdfdb58cbc6247ab36cac38ad2dc35509b5c1e102a7e53461234cb3 |
| polygons.geojsonseq | e07515f22b14db4c1a7bcb2b9871681f006f6c6a58379dc17095321f216212fd |
| real_rendered.geojsonseq | b5fa1b6c8eb3aa434d714217cc2d2f0060a7c47721fe680a3a76bfbaeaaa5162 |
| real_blocks.csv | 355896837e3e9f77182334abbfb97408e8fdedb79fb727fafcbc396ceeee98d8 |
