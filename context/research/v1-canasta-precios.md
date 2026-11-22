# Basic basket and supermarket price levels

Bruno's request: a reference basic-basket price and a price level for each supermarket brand in the OSM data, with low-cost chains compared to mainstream chains. Research date: 21 November 2026. Updated the same day with data from the Discord research thread.

## Canasta Básica de Alimentos (CBA)

- **October 2026: $93.678 per person per month.** Change: +0,9 % in the month, +5,2 % in 12 months, +5,0 % since December 2025. Source: Ministerio de Desarrollo Social y Familia, "Valor de la Canasta Básica de Alimentos y Líneas de Pobreza, informe mensual, octubre de 2026", Cuadro 1. https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/cba/nueva_serie/2026/Informe_CBA_2026_10.pdf (read 21 Nov 2026; press report of 17 Nov 2026: https://radiopaulina.cl/2026/11/17/canasta-basica-de-alimentos-subio-52-en-un-ano-y-llego-a-93-678-por-persona-en-octubre)
- The value is national. The ministry does not publish a Santiago value.
- Poverty lines in the same report (per equivalent person): $257.076 for non-renters, $379.838 for renters. Family of 4: $678.427 non-renters, $1.002.398 renters.
- Method (metodología 2024): spending on food by the first income quintile in the IX EPF (INE, 2021–2022), less 50 % of ultra-processed food, less items that fewer than 10 % of households buy, scaled to 2.000 kcal per person per day. The ministry updates it every month with the INE CPI food division. It is a cost of food, not a price at one chain.
- Earlier 2026 values: January $89.570, March $90.261. https://www.theclinic.cl/2026/04/15/canasta-basica-de-alimentos-sube-a-90-261-en-marzo-y-presiona-las-lineas-de-pobreza-en-chile/

### CBA items (Anexo 2, March 2022 prices, total $66.896 per person per month)

The report lists the items with March 2022 spending. Only the total is updated each month. Main items, CLP per person per month at March 2022 prices:

| Group | Items (spending) |
| --- | --- |
| Bread and cereals | Pan corriente a granel 10.239, arroz 1.061, harina 628, pan especial 618, espaguetis 462, otras pastas 528, galletas and cakes (about 1.700 in total) |
| Meat and fish | Vacuno (posta 1.107, molida 1.803, otros cortes 3.396), trutro de pollo 2.045, pechuga de pollo 1.407, cerdo chuleta 759, cecinas and vienesas (about 2.500), atún 591, jurel 261 |
| Dairy and eggs | Huevos 2.273, queso gouda 1.446, yogur 1.106, leche entera 1.031, leche en polvo 614, other cheese, milk and dairy desserts (about 1.950) |
| Fats | Aceite 1.564, mantequilla 693, margarina 407 |
| Fruit | Palta 1.647, plátano 891, limón 633, manzana 466, naranja 304, durazno 239, uva 193 |
| Vegetables and legumes | Papas 1.916, tomate 1.446, lechuga 779, cebolla 687, zapallo 499, zanahoria 424, porotos 394, lentejas 199, others (about 1.500) |
| Sugar, sauces, drinks | Azúcar 680, refrescos 1.903, agua mineral 933, jugos 703, café 503, té 526, mayonesa 417, salsa de tomate 471 |
| Food away from home | Almuerzos y cenas 2.446, platos de fondo 1.327, sándwiches and empanadas (about 1.260) |

Full list (about 100 rows, with grams and kcal): Anexo 2, pp. 9–11 of the PDF above.

## ODEPA RM supermarket subset

ODEPA consumer prices in Región Metropolitana supermarkets, 16–20 November 2026 (https://datos.odepa.gob.cl). 28 CBA categories at the official CBA quantities give **$47.966 per person per month**. This is a subset of the CBA, not the full basket. ODEPA has no data by chain. Source: other agents in the Discord research thread, 21 Nov 2026 (I did not open the ODEPA file myself).

## Price index by chain (Líder = 100)

**Main index: one basket for all chains.** The same 17 food items (one sales pack each), public online prices 19–21 November 2026. Source: https://lukisi.cl/comparar-supermercados, read by Queltehue on 19–21 Nov 2026. That page changes: on 22 Nov it shows 24 items and a different lowest chain. The dated rows used here are the snapshot `alimentos-seis-cadenas.csv` (item, chain, price, read date), posted by Waldemar in the Discord research thread, message 1573858348745359431. Diuca's data PR adds that file to the repo.

| Chain (owner) | 17-item basket | Index |
| --- | ---: | ---: |
| aCuenta (Walmart) | $30.060 | 89 |
| Tottus (Falabella) | $30.450 | 90 |
| Santa Isabel (Cencosud) | $30.500 | 90 |
| Jumbo (Cencosud) | $32.770 | 97 |
| Líder (Walmart) | $33.710 | 100 |
| Unimarc (SMU) | $36.440 | 108 |
| Chains with no price (median) | $31.635 | 94 |

**Official cross-check: SERNAC cotizador.** Retrieved 21 Nov 2026, 21:31 CLT. The same 9 pantry items with identical descriptions and pack sizes. 7 chains, 20 chain-comuna pairs in Santiago, Maipú and Puente Alto. Values for comuna Santiago:

| Chain | 9 pantry items | Index |
| --- | ---: | ---: |
| Unimarc | $10.610 | 76 |
| Tottus | $11.870 | 85 |
| Alvi | $13.170 | 95 |
| Líder | $13.890 | 100 |
| Santa Isabel | $14.910 | 107 |
| Jumbo | $15.810 | 114 |

- The rows have no observation date, no GTIN and no branch.
- It is a pantry purchase, not the full CBA.
- Alvi needs a merchant account and has an $80.000 minimum for online orders.

**The order changes with the basket.** Unimarc is the cheapest chain in the SERNAC 9 pantry items and the most expensive chain in the 17-item list. Santa Isabel and Jumbo also move. Use the 17-item index as the main value.

Superseded: the first version of this brief used the SERNAC Canasta Dieciochera of 2023 (barbecue basket, national) and a 2026 blog (CashControlly, no method). Those values are removed.

## Price for each OSM supermarket

The per-point price file is owned by Chucao (Bruno's agents, Chucao and Waldemar), in the Discord research thread. Chucao owns the handoff of the tags. That CSV gives a price to all 508 `shop=supermarket` points in `data/mvp/activity_points.csv`: 228 by chain, 280 with the median. This brief documents the sources of that CSV. The name-matching table of the first version of this brief is superseded by that CSV.

Context for chains that the 17-item list does not price:

- **Ekono:** Walmart's format list as of 31 Oct 2026 has no Ekono. The OSM name is probably stale (now Lider Express, or closed). https://corporate.walmart.com/content/corporate/en_us/about/location-facts/chile.html
- **Mayorista 10:** SMU converts these stores to Super 10 (43) and Alvi (15) during 2026–2028. https://chocale.cl/2025/12/smu-plan-de-inversiones-sustituye-formato-mayorista-10-y-reafirma-interes-abrir-supermercado-unimarc-zapallar/
- **Central Mayorista:** Walmart says it sells "exclusively to businesses". Households do not shop there.

## Limits

- **Unverified: branch price and stock.** Both indices use online prices. Nobody checked the price or the stock in a branch.
- The 17-item list comes from a private comparison site (lukisi.cl), not an official source. Another agent checked it. I did not open it myself.
- The SERNAC cotizador rows have no date, GTIN or branch. Only comuna Santiago values are in this brief.
- Both baskets are small (17 and 9 items). Neither is the full CBA.
- Alvi is a wholesale chain with quantity price steps and a merchant account. Its household price is not comparable to the other chains.
- The CBA is national. The ODEPA subset is RM. Neither gives a value by chain.
- Not found: an official 2024–2026 study with basket totals for all chains.
