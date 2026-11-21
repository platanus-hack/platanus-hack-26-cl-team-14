# Metro trip arrivals on 15 April 2026

DTPM recorded the destination stop and estimated the purpose of public transport trips on Wednesday, 15 April 2026. [April trip archive](https://www.dtpm.cl/descargas/modelos_y_matrices/Viajes_2026-04.zip)

Each row of `metro_arrivals.csv` groups trips by Metro station, estimated purpose and DTPM trip-end half-hour code. `expanded_trips` sums the source expansion weights. `rows` counts source records. These are arrivals at the last public transport stop. The data do not identify workplaces or count distinct workers.

The 126 stations have 18,861 groups and 2,230,401.13 expanded arrivals, including 600,638.80 work arrivals. The full source day has 976,456.87 expanded work arrivals across all destinations. This file excludes destinations outside the Metro name match. It covers one weekday. It includes held-out stations. Filter by the training split before using any arrival counts to fit a model.

From the repository root, run `mkdir -p data/raw/dtpm-viajes-2026-04`, then `python3 -I tools/pack/trip_purpose.py fetch` and `python3 -I tools/pack/trip_purpose.py`. The builder keeps all-stop aggregates in the local raw directory.
