# A March articulated bus for the native replay

Use route 210 toward Estación Central to show an articulated Red bus in motion. The operating plan identifies a planned vehicle type. The GTFS pack supplies the route and simulated stop times.

DTPM lists Conecta U19 Annex 4 as effective from 28 February 2026. The March 21 update changes the stop registry. The next timetable update starts on April 25. In `Tabla Horaria`, row 3092 gives regular weekday service 1910, user route 210, direction `Ret`, event `C01`, and vehicle type `C2D`. The planned run leaves C104 at 08:00 and reaches C108 at 09:38. DTPM defines class C as articulated. [U19 index](https://www.dtpm.cl/index.php/programas-de-operaciones/unidad-19-conecta-s-a-po26) · [Annex 4](https://www.dtpm.cl/descargas/pops26/PO%202026(28Feb%20al%2013Nov)%20U19%20-%20Anexo%204.xlsx) · [DTPM 2021 report, section 3.1.1](https://dtpm.cl/descargas/memoria/Informe_Gestion_2021.pdf)

March GTFS trip `210-R-L-B04` uses direction 1 and shape `210R`. Its 08:00–09:30 window has a 284-second headway and `exact_times=0`. Annex 4 provides the exact planned departure. No observed plate or vehicle position is available. [March GTFS](https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip)

## Replay lookup

In `web/data/rider_services.json.gz`, use zero-based template **372** and departure start **28800** seconds. Its pattern and shape indices are both 35. To reproduce the join, sort the GTFS shape by point sequence, round coordinates to five decimal places, and compare the full list with packed shape 35. The opposite direction also has an 08:00 start, so use both template and start. Five packed legs can use this template. Leg 154 uses stops 27–30, with offsets of 2,311 and 2,639 seconds. That gives a possible shot from 08:38:31 to 08:43:59 local time. The viewer still needs a run to find a rendered copy and check the camera. [Pack builder](../../tools/pack/rider_services.py)

Annex 4 SHA-256 is `a221dc6eea0827ac6b4a20520b9948f95285eee2dafeac69bdcca6a4478057b2`. The GTFS SHA-256 is `f10945e80beebc39ea2ac70de997d3d69714897b575de6fbf4e3ca643013b77b`, which matches the pack's source hash.

## Route 205 at a right-angle turn

In the same U19 Annex 4, `Tabla Horaria` row 2401 assigns a C2D bus to the regular weekday route 205 return departure from C104 to C102 at 06:30. March GTFS trip `205-R-L-B03` has service day `L`, direction 1, headsign `Santiago` and shape `205R`. Its frequency window starts at 06:30 and ends at 08:00, with a 490-second headway and `exact_times=0`. The Annex identifies the vehicle class for the exact 06:30 departure. [DTPM operating plan](https://www.dtpm.cl/descargas/pops26/PO%202026(28Feb%20al%2013Nov)%20U19%20-%20Anexo%204.xlsx) · [DTPM March GTFS](https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip)

The pack resolves the trip to template 284, start 23400. At about 06:46:27 Chile time, the replay has four assigned rider figures aboard as the bus turns 89.3° near longitude −70.58167, latitude −33.57926. The angle comes from the GTFS shape. The rider figures come from the baseline replay. The operating plan records the scheduled vehicle class. [Pack builder](../../tools/pack/rider_services.py)
