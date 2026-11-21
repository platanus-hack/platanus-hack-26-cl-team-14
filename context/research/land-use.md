# Floor area and nonagricultural properties

The seven area columns in `data/mvp/land_use.csv` cover the comuna of Santiago. The property counts cover all 52 comunas of the Metropolitan Region. Blank cells mean the source did not supply the measure at that scale and date.

| Area type | Year | Key | Source and coverage |
| --- | --- | --- | --- |
| `municipal_block` | 2021 | `santiago2021-` + municipal layer `OBJECTID_1` | 1,458 municipal blocks in CUT 13101. Each row gives floor area in m². The source reports 48 blocks with zero total. |
| `comuna` | 2026 | Five-digit INE CUT | SII second-semester count of all nonagricultural property tax rolls, including exempt properties. Sanity check against the published RM total gives 3,702,224. |

The Municipalidad de Santiago publishes the complete use breakdown. Its web scene labels the unscaled fields as floor area and the fields ending in `_2` as densities per hectare. Housing uses `H`, offices `O`, commerce `C`, industry `I`, education and culture `E`, and health `S`. Other uses include storage, parking and unclassified space. The sum of the seven columns equals the published `TOTAL_M2` in all 1,458 blocks. The total is 29,721,080 m², including 3,818,230 m² of offices. The municipality supplies the floor-area layer. Its item record gives no cadastre semester or method for allocating mixed-use properties. The municipal polygons have their own boundaries and IDs. [Municipal layer](https://services9.arcgis.com/zqDAjElD9y8GPDk4/arcgis/rest/services/TOTAL_DESTINOS_M2_CONSTRUIDOS_2021_gdb/FeatureServer/0) · [Municipal map labels](https://www.arcgis.com/sharing/rest/content/items/5a6d746ddef14a30a119af1466a7543a/data?f=pjson)

The SII publishes three property-count tables at the same URL. The builder reads the nonagricultural table and matches commune names to the Census, including SII's “Til Til” and INE's “Tiltil”. The file header supplies the year. These counts measure tax rolls. A building can contain many rolls. [SII table](https://www.sii.cl/sobre_el_sii/estadisticas/csv/ebbrrc_16.csv)

The SII transparency portal sends the detailed commune download to its tax login before users can select a commune. The public SII dictionary distinguishes parcel land area from construction-line surface, which can use m² or m³. A regional floor-area layer and block property counts from the original roll remain unavailable in this release. [SII access page](https://www.sii.cl/destacados/transparencia/index.html) · [SII file dictionary](https://www.sii.cl/bbrr/descargas/estructura_detalle_catastral.pdf)

The OCUC 2018 layer has area columns by use for 34 urban comunas. We excluded it from the floor-area data because the publisher gives no units or calculation method for those columns. After removing repeated keys, 73,604,638 of 530,305,431 units fall under vacant land. In Santiago alone, OCUC reports 1,071,353 for vacant land. The municipal 2021 file gives zero built m² for that use. [OCUC layer](https://services9.arcgis.com/kKJR3Qt68ohAWuet/arcgis/rest/services/DESTINOS_SII_2018/FeatureServer/0)

To refresh the CSV from both publishers, run `python3 -I tools/pack/land_use.py` from the repository root. SII replaces its public CSV each semester.
