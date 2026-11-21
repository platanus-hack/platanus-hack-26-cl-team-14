# Joining Santiago data sources

Checked: 2026-11-21. Scope: prevent incorrect joins between the existing catalog sources.
This is a bounded compatibility audit, not a complete crosswalk or an ingestion pipeline.
The pitch is still empty. This research supports source selection for the urban simulation.

## Evidence and proposed join contracts

### 1. A commune code needs its source namespace

Three actual records from the [INE Censo 2024 commune layer][ine-query] contain `CUT`, `COD_REGION`, and `COMUNA`.
All three have `COD_REGION=13`. The [SII real-estate lookup][sii] publishes different codes for two of the same names:

| Commune | INE `CUT`, actual feature | SII commune lookup | SII Tesorería code |
| --- | --- | --- | --- |
| Santiago | 13101 | 13101 | 070 |
| Providencia | 13123 | 15103 | 072 |
| Renca | 13128 | 14113 | 077 |

**Proposed contract:** keep the code namespace with the value. Use the source's documented correspondence before joining to INE `CUT`.
Preserve source strings such as Tesorería `070`; integer conversion discards the published format.
One matching example, Santiago, does not establish compatibility for all communes.
This lookup does not establish the coding scheme of every SII company or property export.
The three name comparisons above are evidence of a mismatch, not a complete or current crosswalk for all SII records.

### 2. The viewer's CRS does not establish the download's CRS

The [INE viewer][viewer] says its data use WGS84, EPSG:4326.
The [underlying commune service][ine-layer] advertises native spatial reference `4674`.
Both statements were opened independently; they describe different access surfaces.

The [native query][ine-query] returned three polygon records with response `spatialReference.wkid=4674`.
The same query with [`outSR=4326`][ine-query-4326] returned `wkid=4326`.
Its first ring coordinate for Santiago was `[-70.66533, -33.42822]`, rounded to five decimal places.
The other two features were Providencia and Renca. Both requests returned HTTP 200 and actual feature geometry.

**Proposed contract:** record each extract's returned CRS. Request a common output CRS or transform from the declared source CRS before spatial joins.
Do not change a CRS label and treat that as a transformation.
The rounded coordinate agreement between these two samples does not establish datum equivalence or positional accuracy.
No distances, areas, boundary intersections, or transformation accuracy were measured.

### 3. Census microdata cannot locate a person in a census block

The [INE microdata release][microdata] describes person, household, and dwelling records at **commune-area** level.
The [viewer][viewer] also supplies finer aggregate geographies, including census blocks and rural entities.
The viewer states that commune boundaries serve census operations and do not define official administrative boundaries.

**Proposed contract:** retain each dataset's geographic level and boundary edition.
A commune-area match can constrain totals; it does not identify a person's block or a building.
Allocating released people to finer polygons is a model assumption, not an observed record match.
Use the relevant census geometry for census totals. Do not silently replace it with another provider's administrative polygons.
Microdata files and block records were not downloaded in this audit. No person-to-block crosswalk was verified.

### 4. A shared place name does not align survey zones or observation years

The [SECTRA Santiago page][sectra] identifies the EOD survey year as **2012** and links its database and zoning separately.
It also describes Santiago's **2012 orthophotomosaic** as WGS84, UTM zone 19 south.
That statement concerns the orthophotomosaic; it does not establish the CRS of the unopened EOD zoning archive.

**Proposed contract:** retain the EOD year and zone identifiers with its own zoning file.
Do not equate an EOD zone with an INE commune or census block from its numeric value or name.
A 2012 behavioral distribution combined with 2024 census totals remains a mixed-year model assumption, not a 2024 observation.
The EOD database, zone geometry, zone-to-commune crosswalk, and current-year representativeness remain unverified here.
DTPM's page and format PDF timed out in this audit; existing transport sampling remains with its owner.

## Verification boundary

Verified: three INE commune feature records and geometry in two declared output CRSs; three SII lookup rows; primary publication statements.
The INE samples report `n_per` values of 438856 (Santiago), 143974 (Providencia), and 143622 (Renca).
Those are sampled service values, not an independent validation of census counts.
No full archive, complete crosswalk, join cardinality, missing-key rate, polygon overlap, or time-aligned calibration dataset was verified.
No runtime code or tests were added.

[sii]: https://zeus.sii.cl/avalu_cgi/br/brch10.sh
[viewer]: https://visualizador.ine.gob.cl/
[ine-layer]: https://services5.arcgis.com/hUyD8u3TeZLKPe4T/ArcGIS/rest/services/Censo2024_v2/FeatureServer/11
[ine-query]: https://services5.arcgis.com/hUyD8u3TeZLKPe4T/ArcGIS/rest/services/Censo2024_v2/FeatureServer/11/query?where=CUT%20IN%20%2813101%2C13123%2C13128%29&outFields=CUT%2CCOD_REGION%2CCOMUNA%2Cn_per&returnGeometry=true&f=json&resultRecordCount=3&geometryPrecision=5
[ine-query-4326]: https://services5.arcgis.com/hUyD8u3TeZLKPe4T/ArcGIS/rest/services/Censo2024_v2/FeatureServer/11/query?where=CUT%20IN%20%2813101%2C13123%2C13128%29&outFields=CUT%2CCOD_REGION%2CCOMUNA%2Cn_per&returnGeometry=true&f=json&resultRecordCount=3&geometryPrecision=5&outSR=4326
[microdata]: https://www.ine.gob.cl/sala-de-prensa/prensa/general/noticia/2025/12/04/ine-publica-base-de-datos-del-censo-de-poblaci%C3%B3n-y-vivienda-2024
[sectra]: https://www.sectra.gob.cl/info-territorial/region-metropolitana-de-santiago/
