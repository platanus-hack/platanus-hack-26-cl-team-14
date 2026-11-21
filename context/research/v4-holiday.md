# V4 holiday candidate · Santiago governor runoff

The V4 slide needs an observed result and a prediction for the same event. DTPM records Metro boardings on the Sunday 24 November 2024 election holiday. The same release gives two weekday reference dates. Neither supplies a normal Sunday with comparable service. Keep the holiday candidate out of the fixed score until it has a comparable baseline and the engine can model the event.

## Event and data

The Santiago region held its governor runoff on 24 November. Malls under shared management had to close. Metro and the Nos commuter train charged CLP 0, and passengers still had to validate each entry. Metro announced a 07:00 opening and additional trains, with reinforcement from 09:00 to 17:00. Red announced additional buses from 09:30 to 18:30. [Dirección del Trabajo](https://dt.gob.cl/portal/1627/w3-article-126967.html) · [Transportes Informa](https://www.transporteinforma.cl/noticias/plan-especial-de-transporte-publico-apoyara-traslados-en-segunda-vuelta-de-eleccion-de-gobernadores/) · [Fare resolution](https://www.bcn.cl/leychile/navegar?idNorma=1208670&idVersion=2024-11-23)

| Recorded Metro boarding stages | Mon 18 Nov, Line 1 interruption | Thu 21 Nov, weekday | Sun 24 Nov, election holiday |
| --- | ---: | ---: | ---: |
| All local half-hours | 2,263,585 | 2,307,896 | 962,851 |
| 09:00–17:00, bins 18–33 | 891,917 | 914,446 | 642,037 |

Metro reported seven Line 1 stations without service at 13:14 on Monday, from Salvador to Alcántara. Metro announced full service at 14:34. The interruption falls inside the comparison window. Thursday gives another weekday observation. We found no Thursday interruption in the notices we reviewed. [Metro notices reproduced by Cooperativa](https://cooperativa.cl/noticias/pais/transportes/metro/metro-repone-el-servicio-en-toda-la-red-tras-suspension-de-linea-1/2024-11-18/134444.html) · [Reopening time](https://www.latercera.com/nacional/noticia/metro-anuncia-suspension-de-servicio-en-varias-estaciones-de-linea-1/E4URJOF5QZHMNNHKSAYH57GKFI/)

Salvador, Manuel Montt, Pedro de Valdivia, El Golf, and Alcántara each recorded zero boardings in the 13:30 and 14:00 bins on Monday. The station totals for Los Leones and Tobalaba combine lines and still record entries. These counts can check a partial Line 1 closure. Score the demand response separately using open stations or bus stops and a matched weekday reference.

The rows cover 126 common station names. Count one recorded Metro boarding stage per row. All selected rows have service expansion factor 1. These figures exclude unrecorded entries and count stages, so one person can appear more than once. They give no measure of voting trips or the effect of free fares alone. The dated stage archive contains one Sunday. The companion workbook lists 25–26 November as its weekend dates, but those dates fell on Monday and Tuesday in 2024. Use the dated stages for this comparison. [DTPM catalog and methodology](https://www.dtpm.cl/index.php/documentos/matrices-de-viaje) · [Dated November stages](https://www.dtpm.cl/descargas/modelos_y_matrices/Tabla-de-Etapas-%20Nov24.zip) · [Companion workbook](https://www.dtpm.cl/descargas/modelos_y_matrices/Subida_Paradero_Estacion_2024.11.xlsb)

## Model requirement

Start from a normal Sunday demand and service profile. Change fare and service by mode and time, close affected shops, and add election trips to polling locations. The current engine starts from a usual workday and can change line headway and jobs. It lacks a Sunday base, a fare input, and voting trips. A headway change alone cannot predict these observed counts. If the required inputs become available, compare total Metro boarding stages by station and half-hour for the same date and window. Use a target outside the data used to fit the model.

## Reproduce

In the DTPM ZIP, use the three named GZIP members. The extracted member hashes are:

- `Tabla de Etapas/2024-11-18.etapas.csv.gz` — `5ba47edeb2543594b5b9e86cf32ef62f4aec7a95db6c8bb2e50a212db35995d4`
- `Tabla de Etapas/2024-11-21.etapas.csv.gz` — `4a8fef7747da7b57b6b30e0082a3b01fc923335f26098ae81efb767a3356e3e7`
- `Tabla de Etapas/2024-11-24.etapas.csv.gz` — `b39a4a7ee2a3e6306f5f58f150f003b1388e0a36a1f5b57d0a2061a73da0f808`

For each date, count rows using the boarding timestamp in field 10, mode in field 6 and half-hour in field 13. Field 23 gives the station name. [DTPM file dictionary](https://www.dtpm.cl/descargas/modelos_y_matrices/Estructura-de-Tabla-de-Etapas-%20Nov%2024.txt)

```sh
dtpm_date=2024-11-24
unzip -p dtpm-nov24.zip "Tabla de Etapas/${dtpm_date}.etapas.csv.gz" |
  gzip -dc |
  awk -F '|' -v date="$dtpm_date" '
    $6 == "METRO" && substr($10, 1, 10) == date {
      total++
      stations[$23] = 1
      bin = substr($13, 1, 5)
      if (bin >= "09:00" && bin < "17:00") window++
    }
    END { print total, window, length(stations) }
  '
```
