# Destinations for shopping, health care and public services

`activity_points.csv` contains 16,697 mapped OpenStreetMap objects in the Santiago urban view, with comuna codes (CUT) in 41 comunas. The input is the 20 November 2026 Chile snapshot. The rectangle runs from −70.85 to −70.45 longitude and from −33.65 to −33.30 latitude. Mapping coverage varies by neighborhood. This November snapshot can include places that opened after April 2026. [Geofabrik Chile extract](https://download.geofabrik.de/south-america/chile/chile-261120.osm.pbf)

| Kind | Objects | OpenStreetMap tags |
| --- | ---: | --- |
| shop | 14,946 | Other active `shop=*`, including unnamed shops and retail services |
| supermarket | 508 | `shop=supermarket` |
| mall | 151 | `shop=mall` |
| hospital | 99 | `amenity=hospital` or `healthcare=hospital` |
| clinic | 453 | `amenity=clinic` or `healthcare=clinic` |
| public_office | 540 | `office=government` or `amenity=townhall` |

Each row has one kind. Health tags take priority, then public offices, then retail. A public office can house government staff without a public counter. OSM can map a mall, its shops and two representations of the same establishment. The counts describe map objects. Unique establishments, visitor counts and opening hours are unverified. [Shop tags](https://wiki.openstreetmap.org/wiki/Key:shop) · [Health tags](https://wiki.openstreetmap.org/wiki/Key:healthcare) · [Government offices](https://wiki.openstreetmap.org/wiki/Tag:office%3Dgovernment)

Coordinates use WGS84 degrees. For each polygon the builder selects a point inside the mapped area. The point can be far from the entrance. Every exported point intersects a polygon in `communes.geojson`. Simplified boundaries can change the CUT near borders. The OSM ID begins with `n` for a node, `w` for a way or `r` for a relation. OSM supplies a name for 14,451 objects.

For 5,914 polygons, `mapped_area_m2` estimates the mapped horizontal area in square metres using a local longitude and latitude scale. Holes reduce the area. A polygon can enclose a building, a shop unit or a whole site. The other rows have a blank area. The `building_levels` field preserves the source tag for 362 objects. These fields describe geometry. They do not measure customer capacity.

## Rebuild

Use Osmium and Python with Shapely 2. Run from the repository root. If the smart urban extract from the building pack exists, start at the filter command and use that extract as the input.

```sh
mkdir -p data/raw/activities
osmium extract -s smart -b -70.85,-33.65,-70.45,-33.30 data/raw/osm-roads/chile.osm.pbf -o data/raw/activities/urban.pbf
osmium tags-filter data/raw/activities/urban.pbf nwr/shop nwr/amenity=hospital,clinic,townhall nwr/healthcare=hospital,clinic nwr/office=government -o data/raw/activities/places.pbf
osmium export -u type_id --geometry-types=point,polygon -f geojsonseq data/raw/activities/places.pbf -o data/raw/activities/places.jsonseq
uv run --with 'shapely>=2,<3' python tools/pack/activity_points.py data/raw/activities/places.jsonseq
```

Credit © OpenStreetMap contributors. This derived dataset is available under the Open Database License 1.0. [Attribution and license](https://www.openstreetmap.org/copyright)
