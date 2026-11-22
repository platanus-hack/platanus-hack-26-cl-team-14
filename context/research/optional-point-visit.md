# A point-level optional shopping visit in Maipú

The same synthetic worker keeps the same home and job in both cases. At 18:00 they can walk from home to either supermarket. The 45-minute limit includes the round trip and 20 assumed minutes in store. With CLP 16,000, the worker's fixed preference selects Jumbo. With CLP 14,000, only Tottus fits. At CLP 14,000 and 39 minutes, the worker skips. Both stores are in comuna 13119. A result by comuna alone cannot show this change.

| Place | Packed OSM ID | Round-trip walk | Walk plus shopping | CLP per basket |
|---|---|---:|---:|---:|
| Jumbo Pajaritos, 3302 | `n4463786186` | 1.10 km, 14.7 min | 35 min | 15,270 |
| Tottus Pajaritos, 2356 | `n11976918097` | 1.47 km, 19.6 min | 40 min | 12,900 |

The [fixture](optional-point-visit-maipu.json) uses worker index 1163263, home block 13119101001048 and job block 13119141011012. It uses the same nine SERNAC pantry specifications at both chains. The chain and comuna quotes were retrieved on 21 November 2026. Store prices and stock at checkout may differ. The [route](https://routing.openstreetmap.de/about.html) uses the mapped pedestrian network from the model's home point. The preference, time in store, stock, and budgets are case inputs. The only change here is available cash. [Price data](../../data/mvp/supermarket_prices/sernac_pantry9_by_comuna.csv) · [Jumbo address](https://assets.jumbo.cl/uploads/2025/11/ECUSA-REVISIO%CC%81N-BASES-KIDD-VOODOO-CENCOSUD-20-nov-202524658095.pdf) · [Tottus address](https://snifa.sma.gob.cl/General/Descargar/1104355051)

## Request and result

Submit the fixture as the body of `POST /api/scenario` or as the arguments of the Santiago MCP `simulate_city` tool. For the CLI, pass the fixture's `changes` array. Each option can supply a different scenario quote. If omitted, it keeps the baseline time, cost and availability. Use one preference per option and give the source or assumption for each quote. The result reports the two choices and the worker's fixed block anchors. A null visit means the worker skips. A request with a different anchor or a population edit is reported in `not_modeled`.
