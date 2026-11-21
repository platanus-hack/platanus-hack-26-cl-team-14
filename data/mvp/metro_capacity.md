# Scheduled Metro departures and train capacity

[metro_capacity.csv](metro_capacity.csv) gives mean departures per weekday for April 13–17, 2026. Each row covers one line, direction and half hour. A train counts once at the first scheduled departure, including trips that start at an intermediate station. For station boarding supply, use the departures at that station.

Rebuild from the repository root with the downloaded [DTPM March 21 feed](https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip).

```sh
python3 -I tools/pack/metro_capacity.py data/raw/dtpm-gtfs-march/GTFS_20260321_v3.zip
```

The feed SHA-256 is `f10945e80beebc39ea2ac70de997d3d69714897b575de6fbf4e3ca643013b77b`. The calendar uses service L five days, LJ four days and V on Friday. No calendar exceptions applied. Each direction has all 48 half hours in Santiago local time. Zero trains means the timetable has no trip starts in that interval.

The schedule gives 3,934 trip starts on Monday through Thursday and 3,874 on Friday, a weekday mean of 3,922. The Monday counts match all 672 Metro cells in [transit_paths.json](../../web/data/transit_paths.json). Friday differs from Monday in 116 cells.

## Places per train

L3 and L6 use the CAF AS-2014 five-car train. CAF gives a total nominal capacity of 1,299 people at six passengers per square metre. Metro independently reported 1,299 places per L3 train to the Chamber of Deputies. Its L3/L6 project presentation gives the rounded figure of 1,300 and identifies the five-car configuration. [CAF technical sheet, archived June 2024](https://web.archive.org/web/20240620111436/https://www.caf.net/es/soluciones/proyectos/proyecto-detalle.php?p=267) · [Metro D/18/2023](https://www.camara.cl/verdoc.aspx?prmID=123568&prmTIPO=OFICIO_FISCALIZACION_RESPUESTA) · [Metro project presentation](https://alamys.org/wp-content/files_mf/1452868259p63jul2015acr.pdf)

The two capacity columns contain the train rating and its product with mean departures. A full train still contributes its nominal places. The file gives no count of free places or passengers on board.

L1, L2, L4, L4A and L5 have blank capacity columns. We could not verify one rating or a train assignment by departure for these lines in April 2026. For example, Subtrans reports about 1,341 places for the seven-car NS16 on L2 and L5. The publication omits standing density, and the timetable does not identify individual train types. [Subtrans NS16 fleet announcement](https://www.subtrans.gob.cl/modernizacion-de-metro-subsecretario-de-transportes-inspecciona-primeros-trenes-armados-en-nuestro-pais/)

Table 5 in SECTRA's 2018 and 2020 studies gives historical train capacities for their morning transport models. The values differ across the two studies even on L6. Both tables omit the standing density and the vehicle assignment for each trip. [2018 study](https://www.dtpm.cl/descargas/cevias2018/IMPACTOS%20DE%20NUEVA%20LICITACION%20SOBRE%20EL%20SISTEMA%20DE%20BUSES%20DE%20SANTIAGO.pdf) · [2020 study, with Metro S.A. 2019 as a source](https://www.dtpm.cl/descargas/licitacion21/Informe%20SECTRA_Licitacion.pdf)

| Model year | L1 | L2 | L3 | L4 | L4A | L5 | L6 |
|---|---:|---:|---:|---:|---:|---:|---:|
| 2018 | 1,469 | 1,431 | — | 1,610 | 805 | 1,255 | 1,299 |
| 2020 | 1,490 | 1,230 | 1,299 | 1,580 | 805 | 1,170 | 1,210 |
