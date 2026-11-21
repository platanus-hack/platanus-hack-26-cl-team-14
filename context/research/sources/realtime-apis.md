# Live data feeds for Santiago

Checked on 2026-11-21 with `curl` (browser User-Agent). Server clock showed 2026-11-21 05:03 UTC.
Result: only one true live feed is open with no key (Bike Itaú GBFS). Bus, Metro, traffic, and air have no open live feed that I could reach.

### Bike Itaú Santiago (GBFS)
- Link: https://santiago.publicbikesystem.net/customer/gbfs/v3.0/gbfs.json (listed in MobilityData `systems.csv`, system ID `bike_santiago`; supports GBFS 1.1, 2.3, 3.0)
- Contents: one row is one dock station. `station_information`: id, name, lat, lon, address, capacity. `station_status`: `num_bikes_available`, `num_docks_available`, `status`, `last_reported`.
- Granularity: point (station). Update every 15 to 30 s (`ttl` 15 to 30).
- Year: live (2026-11-21)
- Access: API, JSON, no key. `station_status.json` 109 KB. `station_information.json` 111 KB.
- Check: VERIFIED 2026-11-21, HTTP 200 on gbfs.json (961 B), `v2/en/station_status.json` (109710 B), `v2/en/station_information.json` (111007 B).
  First bytes: `{"last_updated":1795237409,"ttl":19,"data":{"stations":[{"station_id":"4","num_bikes_available":10,"num_bikes_disabled":0,"is_charging_station":false,"status":"IN_SERVICE",...`
  Station: `"name":"P02 - Italia / Marín","lat":-33.443928,"lon":-70.625648,"address":"Italia 980..."`
- Note (critic sunny-koala): the v3.0 `station_status` path listed in `gbfs.json` is also live (about 101 KB, newer field names such as `num_vehicles_available`). Not checked by me.
- Limits: Shows bike stock per station, not trips. To get flows, poll every minute and diff the stock. Docked bikes only. No history.
- Use in the simulation: score for short trips and last-mile demand (net bike flow per station by hour). Needs our own logger to build history.

### DTPM GTFS (static schedule, current)
- Link: https://www.dtpm.cl/descargas/gtfs/GTFS_20261114_v2.zip (found on https://www.dtpm.cl/index.php/gtfs-vigente)
- Contents: standard GTFS: routes, stops, trips, stop_times, shapes for Red buses and Metro.
- Granularity: point (stop), timetable.
- Year: 2026 (file name date 2026-11-14; Last-Modified 2026-11-16)
- Access: direct download, ZIP, 12,829,198 B. Old versions: https://www.dtpm.cl/index.php/programas-de-operaciones/gtfs-historicos
- Check: VERIFIED 2026-11-21, HTTP 200 on HEAD, Content-Length 12829198. I did not unzip it (no rows read).
- Limits: This is a plan, not the real service. No real vehicle positions.
- Use in the simulation: network for bus and Metro routing. (Another topic file may cover it in depth.)

### red.cl REST service (route and service data)
- Link: https://www.red.cl/restservice_v2/rest/getservicios/all and `.../conocerecorrido?codsint=506` and `.../getrecorrido/506`
- Contents: list of bus service codes; per route: operator, schedule by day type, route geometry.
- Granularity: route.
- Year: current.
- Access: API, JSON, no key. `getservicios/all` 3030 B; `getrecorrido/506` 711 KB.
- Check: VERIFIED 2026-11-21, HTTP 200. First bytes: `["101", "102", "103", "104", "105", "105c", ...` and `{"negocio": {"id": 5, "nombre": "Buses Metropolitana S.A.", ...`
- Limits: Static data. Unofficial paths, no documentation. Could change without notice.
- Use in the simulation: route shapes and operating hours. Not live.

### red.cl bus arrival predictor (`predictorPlus`)
- Link: https://www.red.cl/predictorPlus/prediccion?t=<token>&codsimt=PA433&codser=
- Contents: predicted arrival of next buses at a stop (what the Red app shows).
- Granularity: stop and service, minutes.
- Year: live.
- Access: API. Needs a token `t`.
- Check: UNVERIFIED for data. Endpoint exists: HTTP 500, body `Token inválido` (15 B). I could not find how to get a token. The old paths `getpredicciontodos` and `getparadas` return 404 `Ruta no encontrada.`
- Limits: Token is probably created by the web page with JavaScript (not checked). Using it may break the terms of the site.
- Use in the simulation: NONE for now. Even if it worked, it gives waiting time per stop, not flows of people.

### api.xor.cl (unofficial Red wrapper)
- Link: https://api.xor.cl/red/bus-stop/PA433 and https://api.xor.cl/red/metro-network
- Check: UNVERIFIED. HTTP 522 (origin down) on all calls on 2026-11-21. Treat the hint as dead.
- Use in the simulation: NONE.

### Metro de Santiago network status
- Link: https://www.metro.cl/api/estadoRedDetalle.php
- Check: UNVERIFIED. HTTP 403 "Acceso restringido" (HTML, 3931 B) for `curl`. The whole site `metro.cl` gives 403 to `curl`. I did not try to bypass it.
- Limits: Bot blocking. Even with data, it only gives line open/closed state.
- Use in the simulation: NONE (no flow data).

### GTFS-Realtime (vehicle positions) for Red
- Link: none found
- Check: UNVERIFIED. Web search found no GTFS-RT feed for DTPM. The Mobility Database lists only the static GTFS. The DTPM GTFS page has no "realtime" link. I conclude that no public GTFS-RT feed exists (this is an inference).
- Use in the simulation: NONE. Real bus speed and load must come from other sources (topic of another agent).

### GBFS for scooters (Lime, Whoosh, others)
- Link: https://raw.githubusercontent.com/MobilityData/gbfs/master/systems.csv
- Check: VERIFIED 2026-11-21, HTTP 200, 186,712 B. Filter `^CL,` gives one row only: Bike Itaú - Santiago. No scooter operator in Chile is in the registry.
- Use in the simulation: NONE. The hint is wrong for the public registry.

### TomTom Traffic Flow API
- Link: https://api.tomtom.com/traffic/services/4/flowSegmentData/absolute/10/json?point=-33.45,-70.66&key=<KEY>
- Contents: current and free-flow speed for the road segment near a point.
- Granularity: road segment, minutes.
- Access: API, JSON. Needs a key. Free key from developer.tomtom.com (free tier exists; limit not checked).
- Check: UNVERIFIED. Without key: HTTP 401 `You are missing valid authentication credentials`. Endpoint is alive. No key in this session.
- Limits: Terms limit storage of results (not checked). One call per point, so a full map needs many calls.
- Use in the simulation: score for congestion by hour on main roads (compare simulated speed to real speed). Best live traffic option if a key is got.

### HERE Traffic API v7
- Link: https://data.traffic.hereapi.com/v7/flow?in=circle:-33.45,-70.66;r=500&apiKey=<KEY>
- Access: API, JSON. Needs a key from developer.here.com (free plan exists; limit not checked).
- Check: UNVERIFIED. Without key: HTTP 401 `apiKey invalid. apiKey not found.`
- Use in the simulation: same as TomTom. Area queries return many segments in one call, which is better for a map.

### Waze live map / Waze for Cities
- Link: https://www.waze.com/live-map/api/georss?...
- Check: UNVERIFIED. HTTP 403. Waze for Cities data is for approved partners only (partner programme, not checked in this session).
- Use in the simulation: NONE unless a partner account exists.

### UOCT traffic (Santiago traffic control)
- Link: not found. `https://tiempo.mop.gob.cl/` and `https://www.aeropuertosantiago.cl/` did not connect (HTTP 000).
- Check: UNVERIFIED. I found no open UOCT API.
- Use in the simulation: NONE.

### SINCA live air quality (MMA)
- Link: https://sinca.mma.gob.cl/ and `https://sinca.mma.gob.cl/index.php/json/listadomapa2k19/`
- Check: UNVERIFIED. HTTP 403 Forbidden for `curl` on the root and the JSON path (also with a browser User-Agent). Search says SINCA offers PDF, TXT, CSV exports only.
- Use in the simulation: NONE for now (air quality is not a mechanism of the engine).

### WAQI / aqicn.org air API
- Link: https://api.waqi.info/feed/geo:-33.46;-70.66/?token=<TOKEN>
- Check: VERIFIED that the API answers (HTTP 200, 2272 B JSON). But the `demo` token ignores the place: it returns Shanghai (`"aqi":127`, source "Shanghai Environment Monitoring Center"). The Santiago data was not seen. A free token exists (aqicn.org/data-platform/token); not tried.
- Use in the simulation: NONE (see SINCA).

### Open-Meteo (weather and air model)
- Link: https://api.open-meteo.com/v1/forecast?latitude=-33.45&longitude=-70.66&current=temperature_2m,precipitation and https://air-quality-api.open-meteo.com/v1/air-quality?...&current=pm2_5
- Contents: temperature, rain, PM2.5 at a grid cell.
- Granularity: grid (about 11 km; returned point -33.427,-70.643). Hourly.
- Access: API, JSON, no key (free for non-commercial use).
- Check: VERIFIED 2026-11-21, HTTP 200, 327 B and 309 B. `"current":{"time":"2026-11-21T05:00","interval":...`
- Limits: Model data, not station measurements. One value for all of Santiago at this resolution.
- Use in the simulation: weather input to mode choice (rain, heat) in a "what if" scenario. Weak, optional.

### Flights at SCL airport
- Link: https://www.aeropuertosantiago.cl/ and https://www.dgac.gob.cl
- Check: aeropuertosantiago.cl UNVERIFIED (no connection). dgac.gob.cl HTTP 200 (27,837 B HTML shell only); no flight data endpoint found.
- Use in the simulation: NONE.

### datos.gob.cl CKAN search "transantiago"
- Link: https://datos.gob.cl/api/3/action/package_search?q=transantiago&rows=3
- Check: VERIFIED API works (HTTP 200). `"count": 0`: no dataset with that word.
- Use in the simulation: NONE.

## Critique

1. Weakest: no live feed for buses, Metro, or traffic is verified. The three that matter most for movement (red.cl predictor, Metro, TomTom/HERE) are blocked or need a key. A key for TomTom or HERE is the best next step.
2. Many sites (metro.cl, sinca.mma.gob.cl, Waze) block `curl` with 403. A real browser might get through. I did not test it, and I did not try to bypass the blocks.
3. I did not unzip the GTFS file, read the free-tier limits of TomTom/HERE, or search for other live sources (for example the Red GPS bus data on the open-data portal, Metro ridership, or `transit.land` GTFS-RT).
