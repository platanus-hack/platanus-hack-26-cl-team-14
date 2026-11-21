# Opening hours for one optional shopping visit

Two mapped bookstores near Tobalaba have published hours. They can illustrate a flexible visit after work. The person, book preference, travel time, and decision in this example are model assumptions.

| Place | Packed ID | Friday hours |
| --- | --- | --- |
| Libro Verde, MUT | `n13772856105` | 10:00–20:00 |
| Librería Antártica, Cenco Costanera | `n10988886667` | 10:00–21:00 |

The venue directories publish the hours. The OpenStreetMap nodes give the locations and tag both shops `shop=books`. The two points are about 467 m apart in a straight line. [MUT store page](https://mut.cl/es/tienda/libro-verde/) · [Cenco store page](https://www.cencomalls.cl/costanera/tiendas/libreria-antartica) · [Libro Verde node](https://www.openstreetmap.org/node/13772856105) · [Antártica node](https://www.openstreetmap.org/node/10988886667)

At 20:15 on a normal Friday, the published hours exclude Libro Verde and leave Antártica open. A model can check whether the traveler has time to get there and complete the visit. Book availability, actual walking time, and the person's willingness to switch still need assumptions or other sources. A missing hours value leaves availability unknown.

[The activity pack](../../data/mvp/activity_points.csv) keeps both IDs and coordinates but groups both under `kind=shop`. [Its builder](../../tools/pack/activity_points.py) discards the original shop type and hours. The two source nodes currently have no `opening_hours` tag. Keep the published hours and bookstore classification with the proposed inputs for this example.
