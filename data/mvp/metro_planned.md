# Planned Metro network

The GeoJSON contains 54 line-specific station records and five route features. Isidora Goyenechea occurs on both L6 and L7. Its two records link to each other. Station names follow the project plans and can change before service begins.

| Corridor | Station records | Projected opening | Source geometry |
| --- | ---: | --- | --- |
| L7 | 19 | 2028 | January 2021 SEA design. Map label points and engineering axis. Baquedano uses the existing access label. Estoril uses its northern label. |
| L8 | 14 | 2032 and 2033 | February 2026 SEA design. Station footprints and main tunnel axis. |
| L9 | 19 | 2030, 2032 or 2033 depending on stage and source | March 2025 and June 2026 SEA designs. The southern path follows one edge of the tunnel footprint. |
| L6 west | 1 | 2028 | Centre of the Lo Errázuriz footprint and midpoints of the tunnel footprint. |
| L6 east | 1 | 2028 | Isidora Goyenechea label and tunnel axis. |

Coordinates are WGS84 longitude and latitude, in degrees. The `precision` field identifies how the builder obtained each geometry. Metro does not publish survey accuracy for these points. A map label can be offset from an entrance or platform. The L6 extensions and L7 include terminal tails. MultiLineString pieces retain source breaks and can overlap. For travel order, use `neighbor_ids`.

A station's `id` identifies its line-specific record. The `corridor` distinguishes both L6 extensions. The `existing_neighbor_id` joins an extension to its present terminal. The `gtfs_parent_station_id` identifies transfers at existing stations. Planned interchanges use `planned_interchange_id`. Both Cal y Canto records share one existing GTFS parent. Published alignments and present GTFS IDs identify transfers. [L7 environmental filing](https://infofirma.sea.gob.cl/DocumentosSEA/MostrarDocumento?docId=dc%2Fc5%2F633d6ad5981d5e2f70c9002441a05c2f251b) · [L8 environmental filing](https://infofirma.sea.gob.cl/DocumentosSEA/MostrarDocumento?docId=2024%2F07%2F19%2Ff085-758d-4dd5-b87c-58e20cd71536) · [L9 and the former L4 proposal](https://www.transporteinforma.cl/noticias/metro-presenta-trazado-de-la-futura-linea-9/)

The L9 northern stage carries both published years. Keep the selected year explicit in a scenario. The projected 37 minutes for L7 applies to the entire terminal-to-terminal ride. Segment travel times, transfer delays and passenger savings are unmeasured. [Regional delegation](https://dprmetropolitana.dpr.gob.cl/2026/03/19/delegado-presidencial-participo-en-importante-encuentro-de-tuneles-de-la-linea-7-del-metro/)

Run `python3 -B tools/pack/metro_planned.py` from the repository root. It uses Python 3, curl and bsdtar. It downloads design files into ignored directories under `data/raw/`. Each feature links to its source. Verification status: not rebuilt by another laptop yet.
