# Building permits

`building_permits.csv` has one row per comuna and calendar year. It covers all 52 comunas of the Región Metropolitana for 2023–2025. The November 2026 release includes January–September 2026 only.

| Field | Meaning |
| --- | --- |
| cut, comuna | Five-digit INE comuna code and name from `communes.csv` |
| year | Calendar year of the permit statistics |
| dwellings_authorized | Number of dwellings covered by approvals for new housing and regularization of new housing |
| residential_m2_authorized | Approved residential floor area in square metres for the same works |
| minvu_status | MINVU marks 2023 and 2024 final and 2025 preliminary |
| ine_mapped_m2_all_uses | Sum of the INE map’s floor area for all uses within the comuna and year. Blank means the query returned no group. |

The MINVU housing figures cover authorization. A permit gives no completion or occupancy date. The MINVU series excludes extensions and nonresidential area. The data cannot locate an individual building.

MINVU publishes annual comuna totals from the monthly Formulario Único de Estadística de Edificación. The source sheets are `número_total` and `m2_total`. [MINVU catalog](https://centrodeestudios.minvu.gob.cl/repositorio/categoria/permisos-de-edificacion/) · [November 2026 workbook](https://catalogo.minvu.cl/cgi-bin/koha/opac-retrieve-file.pl?id=d40a41546b26b2a7fe5f1440af6e6ff2)

| Year | Dwellings | Residential m² | Regional comparison |
| --- | ---: | ---: | --- |
| 2023 | 29,678 | 2,396,777 | Matches INE annual table 2.2 |
| 2024 | 25,602 | 2,091,583 | Matches INE annual table 2.2 |
| 2025 | 18,912 | 1,638,041 | Preliminary MINVU comuna series |

INE’s September 2026 release reports 22,211 dwellings and 1,913,788 residential m² across its twelve monthly RM totals for 2025. MINVU flags the difference in its workbook. This CSV preserves the MINVU comuna values. INE’s annual tables and monthly release give regional totals for comparison. [INE 2023](https://www.ine.gob.cl/docs/default-source/permisos-de-edificacion/cuadros-estadisticos/anuarios/informe-anual-2023.xls?sfvrsn=2b647df3_9) · [INE 2024](https://www.ine.gob.cl/docs/default-source/permisos-de-edificacion/cuadros-estadisticos/anuarios/informe-anual-2024.xls?sfvrsn=f6593447_5) · [INE monthly tables 3 and 4](https://www.ine.gob.cl/docs/default-source/permisos-de-edificacion/cuadros-estadisticos/series-mensuales/series-mensuales-a-septiembre-2026.xls?sfvrsn=426bcbd7_6)

INE’s public map groups floor area across residential, mixed, and nonresidential permits. The query requests only sums by comuna and year. Coverage varies within and between comunas. Keep these mapped areas separate from the MINVU housing series. The sources overlap. [INE map layer](https://services5.arcgis.com/hUyD8u3TeZLKPe4T/arcgis/rest/services/CRF_PE_v2/FeatureServer/1) · [INE geographic data catalog](https://www.ine.gob.cl/herramientas/portal-de-mapas/geodatos-abiertos)

| Year | Comunas with mapped area | Mapped m² | INE regional m², new works and extensions |
| --- | ---: | ---: | ---: |
| 2023 | 44 | 2,386,083 | 4,038,116 |
| 2024 | 44 | 2,570,377 | 3,663,225 |
| 2025 | 43 | 2,327,190 | 3,582,002 |

Regional comparisons use annual table 1.2 for 2023–24 and monthly tables 7 and 8 for January–December 2025. The full regional area has no verified municipal breakdown in those tables. [INE permit statistics](https://www.ine.gob.cl/estadisticas-por-tema/industria-energia-y-construccion/permisos-de-edificacion)

Las Condes also publishes monthly permit lists. Providencia offers a paid report by address or property tax ID. The CSV contains only municipal aggregates. [Las Condes catalog](https://transparencia.lascondes.cl/transparencia/actos_permisos_edificacion.html) · [Providencia report](https://providencia.cl/provi/municipalidad/servicios/obras-municipales/informe-permisos-de-edificacion)

Run `uv run --no-project tools/pack/building_permits.py` from the repository root. The live INE map can change. The MINVU source workbook has SHA-256 `786f0d9f1da88862dc267516b9cbdd26cb10a224cb8d091254379d426646e5dc`.

Verification status: not rebuilt by another laptop yet.
