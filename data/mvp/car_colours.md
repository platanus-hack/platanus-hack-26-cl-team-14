# Passenger car colours on Pudahuel permits

`car_colours.csv` covers permits issued by Pudahuel (CUT 13124) for 2023. Its denominator is 29,670 distinct passenger vehicles among 36,150 eligible payment records. White accounts for 7,096 vehicles (23.9%), grey for 6,630 (22.3%) and silver for 5,763 (19.4%). An additional 250 (0.84%) have unknown or unrecognized colour. The shares include these vehicles and sum to one. Credit Municipalidad de Pudahuel, CC BY. [Municipal permit register](https://datos.gob.cl/dataset/permisos-de-circulacion-2020-2026/resource/3eb7c5f3-dbd2-40f2-9fdf-49fc42e68fd2)

The builder retains permits for 2023 in the 2023 payment file. It includes the municipal classes automóvil, station wagon, SUV, hatchback, sedán, todo terreno, jeep and cabriolet. The denominator excludes pickups, vans, taxis, buses, motorcycles and other classes. Each plate receives one vote across installments. The builder groups the first word of a reported colour into an English colour family. For example, `GRIS PLATA` maps to grey and `PLATEADO PLATA` maps to silver. The first word may omit a secondary colour. If payments for one plate supply different known colour families, the plate goes to unknown. This occurred zero times. Among the eligible plates, 1,111 have labels containing two recognizable colour families. Leading punctuation leaves two otherwise recognizable colours in the unknown category. Only aggregate shares leave the builder.

The issuing municipality does not establish where the owner lives or where the car travels. The other 51 RM comunas have no verified share in this file. The 2024 Pudahuel full-year register has colour but omits the vehicle class needed to select passenger cars. The 2024 file has 44,952 distinct plates. The municipal vehicle code and manufacture year match the 2024 SII valuation tables exactly for 41,072. A further 337 match only on code, and 3,543 have no match. SII classifies vehicle models, while the permit can classify the same model as a taxi or a private car. Later monthly files change schema. [SII vehicle valuations](https://www.sii.cl/servicios_online/1049-2612.html) The 2023 register has both fields in one annual release. [Pudahuel releases](https://datos.gob.cl/dataset/permisos-de-circulacion-2020-2026)

## Other sources checked on 21 November 2026

- A national catalogue search on permit titles returned 233 datasets. The Maipú source groups 2023 payments. The sampled Peñalolén and Huechuraba headers omit colour. La Reina describes permit counts and amounts. [National catalogue query](https://datos.gob.cl/api/3/action/package_search?q=title%3Acirculacion&rows=1000)
- Puente Alto publishes an INE dashboard by vehicle type and year. The Providencia transparency page covers a permit-processing contract. Santiago provides a payment interface. None supplied a verified passenger-car colour distribution. [Puente Alto](https://datos.mpuentealto.cl/dashboards/21823/permisos-de-circulacion/) · [Providencia](https://transparencia.providencia.cl/Carpeta/VerTabla/22338) · [Santiago](https://pagos.munistgo.cl/pagopci/)
- The search also covered the Las Condes open-data portal, Lo Barnechea's geographic portal and Vitacura's municipal service portal. A usable permit register with colour remains unverified for these comunas. [Las Condes](https://datos.lascondes.cl/datos) · [Lo Barnechea](https://opendata-vecinos-sitmlb.hub.arcgis.com/) · [Vitacura](https://portalmivita.vitacura.cl/terminos)

## Passenger body classes in Pudahuel

Use `car_bodies.csv` only after drawing `automovil_station_wagon_todoterreno` in Pudahuel's INE mix. This uses the 2023 municipal composition as an approximation for the INE combined 2024 class. It covers the same 29,670 vehicles as the colour shares. The builder checks every 2023 payment class for each eligible plate. A plate with conflicting classes contributes once to `unknown`. None had a conflict.

| Municipal class | Distinct vehicles |
| --- | ---: |
| Automóvil, body unspecified | 10,330 |
| Station wagon | 5,344 |
| SUV | 5,014 |
| Hatchback | 4,414 |
| Sedán | 4,100 |
| Todo terreno | 356 |
| Jeep | 109 |
| Cabriolet | 3 |

The generic automóvil records stay generic. The municipality supplies each class label. The split applies only to Pudahuel. [Municipal permit register](https://datos.gob.cl/dataset/permisos-de-circulacion-2020-2026/resource/3eb7c5f3-dbd2-40f2-9fdf-49fc42e68fd2)

## Rebuild

From the repository root, download the original CSV into the ignored raw directory. The 12,045,164-byte file has SHA-256 `c6e4657a1bfc1de069c84fad34cb9e9e2874d60192d2fc6b49963b4fe026e0b9`.

```sh
mkdir -p data/raw/car-colours
curl -fsSL 'https://datos.gob.cl/dataset/a00ac17a-c264-443a-8c0f-dbcc24c6851c/resource/3eb7c5f3-dbd2-40f2-9fdf-49fc42e68fd2/download/permisos-de-circulacion-ano-2023.csv' -o data/raw/car-colours/pudahuel-2023.csv
python3 -I tools/pack/car_colours.py data/raw/car-colours/pudahuel-2023.csv
```
