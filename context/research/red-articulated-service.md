# A scheduled articulated bus for the native view

Show the joint on a Red bus that has an articulated type in an official schedule. The shape and schedule come from DTPM. A vehicle in this example is a planned logical bus. Its actual plate and movement were not observed.

## Verified departure

DTPM's 14 November 2026 operating program gives a weekday departure for Conecta U19 on route 210, toward Puente Alto. In Annex 4, sheet `Tabla Horaria`, row 3875 gives logical bus 152, event 5, service TS code 1910, user code 210, direction `Ida`, and a commercial run (`C01`) from 08:03 to 09:52. The type is `C2D`, class C1/C2 in the workbook dictionary. DTPM defines class C as articulated. [Operating program](https://www.dtpm.cl/descargas/poss26/PO-vigente_14Nov2026.zip) · [Classification, DTPM 2021 report §3.1.1](https://dtpm.cl/descargas/memoria/Informe_Gestion_2021.pdf)

I counted 567 commercial `C2D` rows in this U19 annex. All are in the weekday schedule. Route 210 has 338 rows, 205 has 191, and 210e has 38. These are planned service departures. The count does not describe distinct buses or operational fleet shares for other routes.

## Link to the map

The matching November GTFS template is `210-I-L-B04`, route ID 210, direction 0, shape `210I`. Its weekday frequency window is 08:00–09:30 with a 337-second headway. `exact_times=0` means GTFS provides a frequency, so take the exact 08:03 departure and its type from Annex 4. The November feed covers Monday 23 November 2026 for that weekday calendar. [November GTFS](https://www.dtpm.cl/descargas/gtfs/GTFS_20261114_v2.zip) · [DTPM GTFS description](https://www.dtpm.cl/descargas/gtfs/Descripcion%20GTFS%20estatico%20DTPM_20260210.pdf)

The checked-in rider pack uses the March feed and has six route-210 boarding/alighting entries. Its bus templates carry shape, stop offsets, departure seconds and GTFS route type. They carry no vehicle type. Use November inputs together for this example. The November assignment does not establish the type of an existing March replay copy. [Current pack builder](../../tools/pack/rider_services.py)

For another route-level witness, Red announced that Metbus began putting electric articulated buses on 506 in December 2025. U5 Annex 8 for July–December 2026 gives vehicle codes by departure, including weekday 506 departures marked `C2X` and `B2L`. That source supports variation within the route. Use a per-departure type when available. [Red Movilidad announcement](https://www.red.cl/red-comunica/comienza-a-operar-la-primera-flota-de-buses-electricos-articulados-para-red-movilidad/)

## Reproduce the source lookup

In the operating ZIP, open `PO vigente/U19 - Conecta/PO 2026-2027(14Nov26 al 12Nov27) U19 - Anexo 4_rectificado.xlsx`. SHA-256 `53138885dcf671b548e120b353154c763a98b6bffd4395fbd85836397560cb58`. Filter the schedule to `TIPO_EVENTO=C01` and `TIPO_BUS=C2D`. Confirm row 3875 and check `Diccionario` for both codes.

In the November feed, match route 210, trip `210-I-L-B04` and its frequency row. SHA-256 `ff0cda445cc4652013e159a133fcf863278ec55f6146e48c5a2416bf41c0b155`. The first visual check is a single matched November departure. Motion and native framing remain unverified.
