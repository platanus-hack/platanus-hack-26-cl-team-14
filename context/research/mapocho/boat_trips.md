# Mapocho boat: trip-level test

Weekday EOD 2012 trips with both ends within 800 m of two different piers (`piers.json`). The boat wins a trip if walk to the pier (straight line ×1.3, 4.5 km/h) + half the headway + river ride between the piers + 1 min per intermediate pier + walk out is shorter than the time the person reported (`viajes.TiempoViaje`). Script: `tools/pack/mapocho_boat_trips.py`.

## Piers at least 3 km apart along the pier polyline

Not the same filter as `mapocho_corridor.py` (east–west separation after projection onto the river). Eliseo-21 crossed the survey keys: 92 records in both, 3 only in the corridor filter, 2 only in this one.

```
min river km 3.0; pier-to-pier trips: 94 records, 23,941 expanded weekday trips
speed_kmh,headway_min,faster_trips,share,median_boat_min
12,5,8016,0.335,54
12,10,6461,0.270,57
12,15,6461,0.270,59
17,5,12305,0.514,44
17,10,10871,0.454,47
17,15,8558,0.357,49
25,5,13075,0.546,37
25,10,13075,0.546,40
25,15,12988,0.542,42
```

## Any two different piers

```
min river km 0.0; pier-to-pier trips: 180 records, 51,929 expanded weekday trips
speed_kmh,headway_min,faster_trips,share,median_boat_min
12,5,15498,0.298,42
12,10,11236,0.216,45
12,15,10515,0.202,47
17,5,22576,0.435,37
17,10,19102,0.368,39
17,15,13848,0.267,42
25,5,27580,0.531,33
25,10,23345,0.450,36
25,15,20955,0.404,38
```

## Reading

- Across the 42 directed pairs of comuna reference points, the assumed boat trip always took longer than the public transport reference time.
- At 17 km/h with departures every 10 minutes, the modeled door-to-door trip was faster than the reported time for **10,871 of 23,941** expanded weekday trips (45.4%). The 94 survey records in this comparison started and ended within 800 m of different hypothetical piers at least 3 km apart along the pier polyline. The median modeled duration for this group was 47 minutes.
- The 2012 survey provides trip endpoints and rounded, self-reported durations. Pier locations are assumptions. The comparison omits fares, transfers, crowding and capacity. It measures a travel-time advantage. Passenger demand would require a separate mode-choice estimate.
