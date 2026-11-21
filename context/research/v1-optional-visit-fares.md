# Santiago fares for an optional visit

Use a cash cost in Chilean pesos for each proposed trip. The figures below cover the standard adult Red fare in the 2026 schedule. The engine's existing mode choice uses travel time, distance and fitted mode terms. It has no fare term. [Current mode choice](../../engine/src/choice.rs) · [Panel of Experts announcement, 20 February 2026](https://www.paneldeexpertostarifas.cl/)

| Standard weekday trip | Fare, CLP per passenger |
| --- | ---: |
| Red bus, including eligible bus transfers | 795 |
| Metro or Tren Nos only, 06:00–06:59 or 20:45–23:00 | 735 |
| Metro or Tren Nos, 09:00–17:59 or 20:00–20:44 | 815 |
| Metro or Tren Nos, 07:00–08:59 or 18:00–19:59 | 895 |

Red allows up to two eligible transfers within 120 minutes from the first validation, in the same direction and with no repeated bus service. The low rail fare rises to 795 if the trip includes a bus. A bus-to-rail transfer in the peak band adds 100 to the initial 795. Check the band at the rail validation. [Red fare rules and examples](https://www.red.cl/tarifas-y-recargas/conoce-las-tarifas/) · [DTPM fare table](https://www.dtpm.cl/index.php/tarjeta-bip/tarifas-vigentes-stpm)

For an optional-visit example after 09:00, budget 815 for an adult rail trip in the valley band or 795 for an eligible bus-only trip. If the choice covers the whole visit, add an independently priced return leg. A short visit does not make a return in the opposite direction an eligible transfer. Use zero direct cash cost for walking as a declared assumption. A match ticket or venue spend is a separate cost.

Student and older-adult discounts depend on the applicable fare product. Age, school attendance and the CASEN household income band do not show which fare the traveler pays. Household income also does not identify a person's spending limit. Record the optional-visit budget as a scenario assumption. This schedule supplies costs for a 2026 example. An October 2019 comparison needs tariffs from the historical date. [Red cards and discounts](https://www.red.cl/tarifas-y-recargas/conoce-aqui-todas-las-formas-de-pago-para-viajar-en-red-movilidad-v4/)
