# Observed boardings for an urban ferry opening

The Mapocho scene adds a scheduled ferry to existing land transport. The engine estimates weekday trips to work by pier and direction. A comparison needs observed boardings for the same service and a way to identify work trips. [Mapocho boat inputs and limits](boat-line-entity.md)

## Valdivia

Chile's transport ministry announced three urban river routes on 28 December 2020. Its October 2021 report counts 8,629 passengers through August across the service. Two university piers still awaited full operation at the time of the report. The available aggregate lacks counts by pier, day, direction, and trip purpose. This is the closest Chilean service example. [Launch and routes](https://www.subtrans.gob.cl/los-rios-mtt-lanza-en-valdivia-el-primer-servicio-de-transporte-publico-fluvial-sustentable-del-pais/) · [Ministry passenger report](https://www.subtrans.gob.cl/subsecretario-de-transportes-realizo-balance-del-transporte-publico-fluvial-en-valdivia-mas-de-8-mil-personas-ya-lo-han-utilizado/)

## Astoria, New York City

NYC Ferry launched the Astoria line on 29 August 2017 with five landings. The opening timetable shows 25-minute peak and 50-minute midday spacing from Astoria. The scheduled end-to-end ride to Pier 11 takes 45 minutes. The city publishes hourly boardings by date, route, direction, and landing. [Launch notice](https://www.ferry.nyc/service-alerts/nyc-ferry-schedule-changes-effective-tuesday-august-29th/) · [Opening timetable](https://oana-ny.org/wp-content/uploads/2017/08/Web-Schedule_AST-1.pdf) · [NYC Open Data](https://data.cityofnewyork.us/Transportation/NYC-Ferry-Ridership/t5n6-gx8c/about_data)

| Local date | Boardings on route AS |
| --- | ---: |
| 29 August 2017 | 1,792 |
| 30 August 2017 | 3,576 |
| 31 August 2017 | 3,087 |
| 1 September 2017 | 3,642 |
| **Total** | **12,097** |

The four-day extract has 611 unique date, hour, landing, and direction rows. It covers all recorded boardings on the five landings. The source has no trip purpose or alighting field. Use it to compare service schedules or describe a ferry opening. A work-trip demand test still needs comparable origin and destination demand, alternative travel times, and counts for work trips. [NYC Open Data fields](https://data.cityofnewyork.us/api/views/t5n6-gx8c)

The operator's February 2018 survey reports work and school together for Astoria. It covers a later period than these opening dates. [NYC Ferry survey, page 5](https://www.ferry.nyc/wp-content/uploads/2019/11/NYC-Ferry-Winter-2018-Survey-Results.pdf#page=5)

Fetch the daily totals with the city's public API.

```sh
curl -fsSG https://data.cityofnewyork.us/resource/t5n6-gx8c.json \
  --data-urlencode '$select=date,sum(boardings),count(*)' \
  --data-urlencode "\$where=route = 'AS' AND date >= '2017-08-29T00:00:00' AND date < '2017-09-02T00:00:00'" \
  --data-urlencode '$group=date' \
  --data-urlencode '$order=date'
```
