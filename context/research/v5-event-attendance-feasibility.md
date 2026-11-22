# V5 eval feasibility: how many people go to an event, and from where

Question: can the #498 attend/alternative/stay rule, applied to the whole synthetic population, be calibrated on one real Santiago event and validated on another? All sources were read on 2026-11-21. "Unverified" marks a fact with one press source or no source that I opened.

## Verdict

- **Origin shares (attendance fixed): candidate; feasibility pending the excess/noise and coverage checks.** Four Estadio Nacional events fall inside DTPM sample weeks that have per-day trip tables. Each trip row has `comuna_inicio_viaje`, `comuna_fin_viaje`, the first and last stop, half-hour and `proposito`. Excess trips that end at the access stations before the show, minus a control day, are a proxy for incremental trips by comuna of the first boarding stop. They are not identified attendees.
- **Attendance count: does not meet the bar.** Dua Lipa, Cris MJ and Karol G were all sold out. Their counts measure capacity, not interest, so they cannot calibrate or validate "how many go". Chile–Venezuela (31,906 of about 40,000, weak sales) is the only demand-limited event found. No second demand-limited event of the same class exists to validate against.
- **Signal size: unverified.** I did not measure the excess trips. One day member of the trip ZIP is 280–290 MB, and the DTPM server gave 10 MB in 70 s (about 34 min for each day).

## DTPM weeks with per-day trip tables

The [catalog](https://www.dtpm.cl/index.php/documentos/matrices-de-viaje) lists these weeks from 2023. I read the ZIP central directories with HTTP Range requests only.

| Week | Trip ZIP | Day members |
| --- | --- | --- |
| 17–23 Apr 2023, 7–13 Aug 2023, 6–12 Nov 2023 | `viajes_042023_17al23_transparencia.zip`, `viajes_082023_7al13-.zip`, `viajes_112023_6al12.zip` | One CSV for the full week, 1.7–1.9 GB. Not practical. |
| 15–21 Apr 2024 | `viajes202404_transparencia_15al21.zip` | One gzip for each day. Sat 181 MB, Sun 97 MB. |
| 18–24 Nov 2024 | `Tabla-de-Viajes-Nov-24.zip` | One gzip for each day. Tue 19 Nov 363 MB. |
| 21–27 Apr 2025 | `Tabla-de-viajes-011025.zip` | One gzip for each day. |
| 8–14 Nov 2025 | `Tablas_viajes_NOV_2025.zip` | One gzip for each day, stored (no zip compression). The 2025-11-11 member starts at byte 498,900,430 and has 289,654,298 bytes. |
| 11–17 Apr 2026 | `Viajes_2026-04.zip` | One gzip for each day. This is the week of the engine packs ([trip_purpose README](../../data/validation/trip_purpose_2026_04/README.md)). |

The trip table fields are in the [Nov 2024 structure file](https://www.dtpm.cl/descargas/modelos_y_matrices/Estructura-de-Tabla-de-Viajes-%20Nov24.txt). The monthly `Subidas_Paradero_Estacion` workbooks give day-type averages, not dates. Do not use them for a single event. `tools/pack/trip_purpose.py fetch` already extracts one day member by HTTP Range.

## Candidate events

| Event | Date and time | Attendance and source | DTPM week | Control days | Notes |
| --- | --- | --- | --- | --- | --- |
| **Dua Lipa, night 1** | Tue 11 Nov 2025, show 21:10–23:10 | About 61,000 reported by [CNN Chile](https://www.cnnchile.com/cultura/un-optimismo-radical-dua-lipa-conquista-el-estadio-nacional-con-emotivo-homenaje-a-mon-laferte-y-conexion-intima-con-el-publico-chileno_20251112/); counting method and promoter attribution unverified. Sold out in under one hour ([The Clinic](https://www.theclinic.cl/2025/04/10/dua-lipa-agota-todas-las-entradas-para-su-show-en-el-estadio-nacional-y-anuncian-segunda-fecha-para-el-12-de-noviembre/)). Times: [setlist.fm](https://www.setlist.fm/setlist/dua-lipa/2025/estadio-nacional-santiago-chile-2b59ac9e.html). | 8–14 Nov 2025 | Mon 10, Thu 13. Exclude Wed 12. | Metro open to 00:30. Entry only at L6 Estadio Nacional, Ñuble, Ñuñoa. After 23:00 only L3 and L6 ran ([Metro on X](https://x.com/metrodesantiago/status/1987592433689862245), [El Desconcierto](https://eldesconcierto.cl/2025/11/11/extension-horaria-de-metro-este-martes-11-y-miercoles-12-por-concierto-de-dua-lipa-revisa-en-que-estaciones)). |
| Dua Lipa, night 2 | Wed 12 Nov 2025, 21:00 (unverified start) | No count published that I found. | 8–14 Nov 2025 | Mon 10, Thu 13 | Same audience type. Use it for a reproducibility ceiling, not as validation. |
| **Cris MJ** | Fri 17 Apr 2026, 21:00 | Sold out ([The Clinic](https://www.theclinic.cl/2026/04/17/cris-mj-estadio-nacional-sold-out-records-spotify-hito-urbano-chileno/)). "60 mil personas" ([Radio Carolina, 18 Apr](https://www.carolina.cl/noticias/2026/04/18/noche-historica-para-el-genero-urbano-cris-mj-conquista-el-nacional-con-invitados-de-lujo-y-60-mil-personas.html)). The same outlet later wrote "more than 50,000". **Unverified: press only, no promoter or official count.** | 11–17 Apr 2026 | No other Friday in the week. Use Mon–Thu 13–16 Apr scaled by a Friday ratio from another week (for example Fri 14 Nov 2025). | Metro kept some L6 and L3 stations open to 00:30 ([Los 40](https://los40.cl/2026/04/16/cris-mj-en-estadio-nacional-2026-metro-de-santiago-anuncia-extension-de-su-servicio-por-el-concierto/)). This is the engine week, so pack routes match the network. Red Bull Batalla at Movistar Arena on Sat 11 Apr is the existing comparison ([event-attendance-april-2026.md](event-attendance-april-2026.md)). |
| Chile–Venezuela (football) | Tue 19 Nov 2024, 21:00 | 31,906 controlled spectators ([Partidos de la Roja](https://www.partidosdelaroja.com/2024/11/19112024-chile-venezuela-42.html), [ESPN](https://espndeportes.espn.com/futbol/partido/_/juegoId/684636/venezuela-chile)). Capacity 40,000. Weak sales ([T13](https://www.t13.cl/noticia/deportes13/bajo-interes-por-el-evento-revisa-las-entradas-para-el-chile-vs-venezuela-que)). | 18–24 Nov 2024 | Mon 18, Wed 20, Thu 21 | The only demand-limited event. 4,900–7,900 away tickets ([Emol](https://www.emol.com/noticias/Deportes/2024/11/19/1148935/entradas-chile-venezuela-clasificatorias.html)) put part of the audience in comunas where the Venezuelan diaspora lives. A distance-only model will miss that. Use it for a count check only. |
| Karol G, 3 nights | Fri 19, Sat 20, Sun 21 Apr 2024 | More than 180,000 in total, promoter ([Bizarro](https://bizarro.cl/mas-de-180-mil-personas-vivieron-la-experiencia-del-manana-sera-bonito-latam-tour-junto-a-karol-g-en-el-estadio-nacional/)). Night 1: 55,400 by Carabineros ([T13](https://www.t13.cl/amp/noticia/nacional/karol-g-su-primer-estadio-nacional-carabineros-entrega-positivo-balance-pese-de-20-4-2024)) against about 60,000 by the promoter. **Per-night split unverified.** | 15–21 Apr 2024. **This week covers 19 Apr 2024.** | Fri: Mon–Thu 15–18. Sat and Sun have no control in the week. | L6 Estadio Nacional station was closed 12:00–18:00 on 19 Apr ([Los 40](https://los40.cl/2024/metro-de-santiago-anuncia-cierre-de-estacion-estadio-nacional-por-concierto-de-karol-g-2024-181771.html)). Sum over a station set, not one station. Sold out. |

Venues with no event found inside a sample week: Club Hípico, Espacio Riesco, Teatro Caupolicán (Pineapple Thief, Sat 26 Apr 2025, no count), Movistar Arena (Chyno & Nacho, Sat 26 Apr 2025, no count), Monumental concerts. Football at the Monumental is covered in [monumental-match-boardings.md](monumental-match-boardings.md). I did not search Santander Arena or Parque O'Higgins events in the 2024–2025 weeks in depth.

### Events outside DTPM weeks (no mobility signal)

- Lollapalooza 2014, Parque O'Higgins: an Ipsos survey for Lotus reports 140,000 attendees in two days, 32% from outside Santiago and 9% from abroad. No comuna table and no sample size ([La Tercera](https://www.latercera.com/diario-impreso/el-impacto-economico-de-lollapalooza-los-millones-que-deja-el-festival-a-santiago/)). Lead from Arrayán.
- Lollapalooza 2019: 250,000 in three days, no origins ([v1-parque-ohiggins-event.md](v1-parque-ohiggins-event.md)).
- Caveat from Arrayán: a published total can check a frozen turnout prediction only if the total was never a model input. If the published shares become the `audience_profile` input, the check is circular.

## What can be measured

For each event day D and control day C, use the per-day trip table:

1. **Access set S (freeze before you measure).** L6 Estadio Nacional, L6 Ñuble, L3/L6 Ñuñoa, possibly L3 Irarrázaval, and the bus stops on Av. Grecia and Av. Pedro de Valdivia within 800 m of the venue centroid (OSM way 28092600, already in `data/mvp/event_venues.json`). Bus stop codes: unverified, take them from the DTPM workbook and GTFS as `work_arrivals_comunas.py` does.
2. **Arrivals.** Expanded trips with `paradero_fin_viaje` in S and trip end from gate opening to show start, frozen per event before extraction (Karol G night 1: gates 15:00, and L6 Estadio Nacional closed 12:00–18:00, so add alternative access stops). Use the same clock and stops on the control days. Keep the earliest half-hours as separate bins.
3. **Excess by comuna.** `X_c = arrivals_D(c) − k · arrivals_C(c)`, where c is `comuna_inicio_viaje`. k scales C to D with a set of peer stations far from the venue (the same method as the [extra-event contract](extra-event-contract.md)). Clip negative values to 0. Pool comunas with fewer than 1% of the excess into one "other" bin.
4. **Coverage ratio.** `sum_c X_c / attendance`. This is a proxy for partial transit coverage, not an observed mode share: X counts expanded trips to a partial set of destinations and hours, and the denominator counts all attendees. Clipping negative residuals to 0 can inflate it with noise. Gates can open before 16:00 (Karol G night 1 opened at 15:00, [T13](https://www.t13.cl/amp/noticia/nacional/karol-g-su-primer-estadio-nacional-carabineros-entrega-positivo-balance-pese-de-20-4-2024)), so set the window per event.

Do not use post-event trips for origins. After 23:00 only L3 and L6 ran, with entry-only and exit-only stations, so `comuna_fin_viaje` is bent. `comuna_inicio_viaje` is the comuna of the first boarding stop, not the home. On a weekday many trips start at work. Split by `proposito` (estimated by DTPM) to check how large this effect is. Fare evasion is not in the data.

## Smallest scorable eval

**Pair.** Calibrate on Dua Lipa night 1 (Tue 11 Nov 2025). It has two same-week weekday controls; its attendance figure is press-reported (see above). Validate on Cris MJ (Fri 17 Apr 2026). It has an independent audience and the engine's own network week. Use Chile–Venezuela only for a count check.

**Parameter.** One number: the travel-time weight relative to interest (β in `U = interest − β · time − γ · price`), with γ and the interest by activity type fixed from #498. Attendance stays pinned at the published count, because the concerts were capacity-limited. The model picks the top N attendees by utility, or scales probabilities to sum to N. β then sets how far the audience spreads.

**Metric.** Total variation distance over pooled comunas: `TVD = ½ · Σ_c |p_model(c) − p_obs(c)|`. p_obs is `X_c / Σ X`. p_model is the share of modeled transit attendees by the comuna where their trip starts. Report it next to two references:
- Baseline: shares proportional to the comuna population aged 15+. The model must beat it.
- Noise floor: the TVD between the two control days (Mon 10 against Thu 13 Nov 2025, as fake events). Ceiling: the TVD between Dua Lipa night 1 and night 2.

Secondary, descriptive only: modeled transit share next to the observed coverage ratio `Σ X / attendance`.

**What the engine needs.**
- Apply the #498 rule to every synthetic person aged 15+ for one evening, with the venue, ticket price and start time as inputs.
- Route time from each person's location at the decision time (home or work) to the venue, from the existing April 2026 packs. Use the same packs for the November 2025 event and record that as an approximation.
- A mode choice for attendees, or at least a transit/non-transit split, so p_model compares to transit-only observations.
- Output: attendees by comuna of trip start, and the transit count.

**Data work before the engine.** Extract 4 day members: 2025-11-10, 2025-11-11, 2025-11-13 and 2026-04-17 (plus one Friday control). Each takes about 30 min at the measured server speed. Compute X_c and freeze p_obs before the engine runs.

## Blockers

1. Count cannot be validated. No second demand-limited event exists in a DTPM week.
2. Signal size is unverified. If excess arrivals at S are small next to the day-to-day noise, the origin eval fails too. Check the noise floor first.
3. `comuna_inicio_viaje` is a boarding-stop comuna, not a home comuna. Weekday trips from work mix in.
4. The Cris MJ count is press only. Ticket price, which #498 needs, is not collected here for any event.
5. Transit attendees only. Car, ride-hail and walk attendees are not observed.
