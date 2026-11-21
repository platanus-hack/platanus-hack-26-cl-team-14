# shopping_huff.csv

Where MUT shoppers come from, with the MUT at its site and at the Monumental site. A Huff model fitted on EOD 2012 weekday shopping trips (brief: `context/research/destination-choice.md`).

`site,origin_cut,origin_name,share_of_mut_visitors,mut_share_of_origin_shopping`. Two sites × 52 origin comunas.

## Model

P(d | o) ∝ (shops_d + 1)^α × exp(−β × minutes_od), by comuna.
- Trips: `viajes.Proposito` 11 (De compras), weight `FactorLaboralNormal × Factor_LaboralNormal`, home comuna from `Hogares`, destination `ComunaDestino`. 5,782 sample trips, 1,588,690 expanded weekday trips.
- shops_d: shop, mall and supermarket points in `activity_points.csv` (OSM, Nov 2026). The +1 keeps rural comunas outside the OSM extract.
- minutes: public-transport door-to-door (`travel_times.csv`), with the straight-line fit of `choice.rs` for missing pairs.
- Maximum likelihood: **α = 0.586, β = 0.2047 per minute**. Total variation of destination shares, model vs survey: 0.180.
- The MUT is added as one more destination in the comuna of each site. Its size uses the 26 mapped shop, mall, and supermarket points in the podium bounding box. [MUT footprint count](../../context/research/v1-mut-shop-membership.md)

## Result

| Site | Modeled weekday shopping trips | Top origins |
| --- | --- | --- |
| MUT, Las Condes | 5,745 | Las Condes 39.7 %, Vitacura 20.0 %, Providencia 13.6 % |
| Monumental, Macul | 6,410 | Macul 25.3 %, Ñuñoa 24.0 %, Peñalolén 13.7 % |

## Limits

- Weekday only; no Saturday fit. Trips, not distinct people.
- Comuna level: the MUT's minutes are the comuna-pair minutes of its site, so its location inside the comuna does not count, and Tobalaba station access does not count.
- Shop counts are OSM objects, not floor area or sales. 2012 trips with 2026 shops.
- The operator reports about 800,000 monthly visits (about 26,000 a day, all purposes; #201). The model's 5,745 weekday shopping trips are one purpose only. Use the operator total only as a sensitivity case with a stated shopping share.

Rebuild: `python3 -I tools/pack/shopping_huff.py` after the EOD extraction (`data/mvp/eod_car_mode.md`).
