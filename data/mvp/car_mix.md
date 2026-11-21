# Motorized permits by issuing comuna

`car_mix.csv` gives the share of 2024 motorized permits in each INE class. Use the 52 five-digit CUTs for comuna distributions. CUT `13` gives the published regional distribution as a fallback. It uses the regional counts directly. Each of the 53 distributions has 15 rows. The source key is `ine_permisos`.

On 21 November 2026, the latest workbook in INE's *Cuadros estadísticos → Resultados* was 2024. INE published the provisional figures on 14 August 2025. The issuing municipality can differ from the vehicle's home or observed road. [INE annual series](https://www.ine.gob.cl/estadisticas-por-tema/comercio-y-servicios/permisos-de-circulacion) · [Release](https://www.ine.gob.cl/sala-de-prensa/prensa/general/noticia/2025/08/14/permisos-de-circulaci%C3%B3n-crecieron-un-2-9-en-2024)

## Classes and denominator

The numerator is the class count in table 4. The denominator is the sum of its 15 motorized classes in the same issuing area. A dash in the workbook means zero permits. The CSV includes zero shares. [INE workbook, tables 3 and 4](https://www.ine.gob.cl/docs/default-source/parque-de-vehiculos/cuadros-estadisticos/resultados/2024.xlsx?sfvrsn=5801b9b3_4)

| INE use | Table 4 columns | Classes in column order | Regional motorized permits |
|---|---|---|---:|
| Private and other | E–J | Automobile/station wagon/all-terrain combined, van, private minibus, pickup, motorcycle and similar, other motorized | 2,292,850 |
| Public and collective | L–P | Basic taxi, collective taxi, tourism taxi, collective minibus, collective bus | 71,417 |
| Freight | Q–T | Truck, tractor truck, agricultural tractor, other motorized freight | 78,270 |
| **All motorized** | | | **2,442,537** |

INE combines passenger cars, station wagons and all-terrain vehicles in column E. A sedan/SUV split requires another source. The notes include hearses in E and F and ambulances in F. Collective classes include school and worker services. A visual that shows a subset of classes must divide each selected share by their sum and label the subset.

Table 4 reports 2,476,153 RM permits. Removing 11,654 other nonmotorized permits (K) and 21,962 trailers and semitrailers (U) gives 2,442,537. All 52 comunas and the regional row agree exactly with the motorized total in table 3, column E. Each of the 15 comuna class sums equals the published regional count. The 52 column-E counts match [the existing private-car pack](../../context/research/vehicle-fleet.md). Shares have 12 decimal places.

## Vitacura and La Pintana

| Issuing area | Motorized permits | Automobile/station wagon/all-terrain | Pickups | Motorcycles and similar | Collective buses |
|---|---:|---:|---:|---:|---:|
| Vitacura (13132) | 94,812 | 73.08% | 17.66% | 4.20% | 0.06% |
| La Pintana (13112) | 26,155 | 67.44% | 12.82% | 5.07% | 2.55% |
| RM (13) | 2,442,537 | 68.46% | 14.27% | 5.67% | 0.75% |

For private cars, vans, private minibuses and pickups alone (E–H), the denominators are 89,961 in Vitacura and 22,524 in La Pintana. Pickups account for 18.61% and 14.89% of those subsets.

## Rebuild

Run `python3 -BI tools/pack/car_mix.py data/raw/vehicle-fleet/2024.xlsx` from the repository root with the linked workbook. Its size is 193,510 bytes and SHA-256 is `75947066cffee7efac99d178d7db7af645f307c4bf0c39ccff4a93a29b451ebe`. The builder reuses the repository workbook reader and joins INE names to Census CUTs.
