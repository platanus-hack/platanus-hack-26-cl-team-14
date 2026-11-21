# Mapocho boat: trip-level test

Weekday EOD 2012 trips with both ends within 800 m of two different piers (`piers.json`). The boat wins a trip if walk to the pier (straight line ×1.3, 4.5 km/h) + half the headway + river ride between the piers + 1 min per intermediate pier + walk out is shorter than the time the person reported (`viajes.TiempoViaje`). Script: `tools/pack/mapocho_boat_trips.py`.

## Piers at least 3 km apart along the river (the corridor filter; Eliseo-21 counted 95 records, 24,493 trips)

```
min river km 3.0; pier-to-pier trips: 94 records, 23,941 expanded weekday trips
speed_kmh,headway_min,faster_trips,share,median_boat_min
12,5,8,016,0.335,54
12,10,6,461,0.270,57
12,15,6,461,0.270,59
17,5,12,305,0.514,44
17,10,10,871,0.454,47
17,15,8,558,0.357,49
25,5,13,075,0.546,37
25,10,13,075,0.546,40
25,15,12,988,0.542,42
```

## Any two different piers

```
min river km 0.0; pier-to-pier trips: 180 records, 51,929 expanded weekday trips
speed_kmh,headway_min,faster_trips,share,median_boat_min
12,5,15,498,0.298,42
12,10,11,236,0.216,45
12,15,10,515,0.202,47
17,5,22,576,0.435,37
17,10,19,102,0.368,39
17,15,13,848,0.267,42
25,5,27,580,0.531,33
25,10,23,345,0.450,36
25,15,20,955,0.404,38
```

## Reading

- At comuna level the boat attracts nobody (section 7, 0 of 42 pairs): the walk from the comuna point to a pier is too long.
- At trip level, for people who already live and go within 800 m of a pier, a 17 km/h boat every 10 min beats the reported time for **10,871 of 23,941** weekday trips (45 %). Most of them are trips that today take a long or indirect route.
- Limits: reported times are self-reported and rounded; no fare, no transfer, no crowding, no capacity; 94 sample records; piers are assumed positions every ~2 km; 2012 trips. Not boat passengers: an upper bound on trips that would save time.
