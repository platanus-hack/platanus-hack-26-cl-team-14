# Metro trip arrivals on 15 April 2026

DTPM recorded the destination stop and estimated the purpose of public transport trips on Wednesday, 15 April 2026. [April trip archive](https://www.dtpm.cl/descargas/modelos_y_matrices/Viajes_2026-04.zip)

Each row of `metro_arrivals.csv` groups trips by Metro station, estimated purpose and DTPM trip-end half-hour code. `expanded_trips` sums the source expansion weights. `rows` counts source records. These are arrivals at the last public transport stop. The data do not identify workplaces or count distinct workers.

The 126 stations have 18,861 groups and 2,230,401.13 expanded arrivals, including 600,638.80 work arrivals. The full source day has 976,456.87 expanded work arrivals across all destinations. This file excludes destinations outside the Metro name match. It covers one weekday. It includes held-out stations. Filter by the training split before using any arrival counts to fit a model.

From the repository root, run `mkdir -p data/raw/dtpm-viajes-2026-04`, then `python3 -I tools/pack/trip_purpose.py fetch` and `python3 -I tools/pack/trip_purpose.py`. The builder keeps all-stop aggregates in the local raw directory.

## Work arrivals by comuna

`work_arrivals_by_comuna.csv` groups expanded TRABAJO trips by the comuna and mode of the final stop. Each row gives the INE comuna code, the destination mode, the count of distinct source stop codes and the expanded trip count. Use it to check transit destination choice. A workplace can lie across the comuna boundary from the final stop.

Run `python3 -I tools/pack/work_arrivals_comunas.py` after building the all-stop aggregate. For bus stops the builder joins the internal code to the public ID through the April workbook, then joins that ID to March GTFS coordinates. For Metro it uses the GTFS parent station. It locates bus and Metro points in the rounded INE comuna polygons in `data/mvp/communes.geojson`. Coordinates near a boundary are sensitive to rounding and the choice of a station point. For rail it uses the comuna named for the station in the workbook. [DTPM April stop lookup](https://www.dtpm.cl/descargas/modelos_y_matrices/Subidas_Paradero_Estacion_2026.04_v3.xlsx) · [DTPM March GTFS](https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip)

The file places 975,829.4283 of 976,456.8729 expanded work arrivals across 39 comunas. It places 368,251.9572 at 9,817 bus codes, 600,638.7959 at 126 Metro stations and 6,938.6752 at 10 train stations. Another 61 codes have 627.4446 arrivals and no mode in the lookup. The file keeps these under `UNPLACED`. One comuna has source records with zero expansion weight.

The GTFS point and the April workbook give different comunas for 2,292.04 expanded bus arrivals (0.62%) and 26,759.25 Metro arrivals (4.46%). For example, the Plaza Egaña point falls in Ñuñoa and the workbook assigns it to La Reina (7,260.35 work arrivals). This target uses the point consistently for Metro and bus.
