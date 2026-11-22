# Full Metro closure replay

The three-minute pitch plays the Metro and Mapocho videos side by side for 30 seconds. The Metro clip starts at 08:00 and shows a modeled weekday closure from 07:00 to 10:00, Chile time. [Current pitch decision](../decisions/20261121-2045-live-pitch-opening.md)

## Final cache and video

The final pair is `metro-window-baseline` and `metro-window-closure`. Both use Tobalaba at 08:00 with the Metro service layer. The cache comes from `main` at `93d8d33e26c7ec7d21f8a0ef72d484b8ea591da3`. Each playback made one relay request and zero simulation requests. [Final caches and provenance](https://discord.com/channels/1573514576400752640/1573809606277337280/1573855157962407936)

Each run counts 1,487,455 modeled work departures in the closure window. Public transport falls from 772,186 to 662,807. Car rises from 504,342 to 591,740. Every station has zero entries during the closure. At 10:00 Tobalaba recovers 144.7 modeled entries and its baseline access origins. [Timed engine input](metro_closure_window.json)

Maitén’s 14-second native clip shows trains moving in the baseline, then empty tracks and a closure sign in the same frame. The baseline clock reaches 08:01. The closure starts at 08:00. The clip uses the same saved results as the final pair, apart from creation and compute times. The paired edit and projector rehearsal remain open. [Metro source clip](https://discord.com/channels/1573514576400752640/1573809606277337280/1573855396974563428)

## Research brief

Metro announced the full closure at 19:20 on Friday, 18 October 2019. The network stayed closed until Monday, 21 October, when 20 Line 1 stations reopened. The closure covered the weekend. The period also had curfews, street disruptions, and changes to bus service. [DTPM 2019 report, chapter 7](https://www.dtpm.cl/descargas/memoria/InformeGestion_2019_DTPM.pdf#page=56)

Use 19–20 October to check that the full closure shows zero active stations. The DTPM public trip catalog has April and August samples for 2019. Its crisis totals cover 16 October to 31 December. A comparison of changes in travel needs counts for the closure and a comparable reference window. [V1 evidence review](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/343)

## Earlier standalone rehearsal

The former standalone sequence allowed 25 seconds after question submission.

Type this question in the terminal.

> ¿Qué pasaría si cerráramos todo el Metro de Santiago entre las 7 y las 10 de la mañana de un día hábil? Muéstrame cómo cambiarían los viajes y dónde aumentaría la congestión.

| Time | What the audience sees |
| --- | --- |
| 0–6 s | The terminal shows the saved assumptions. All active Metro stations and lines close from 07:00 to 10:00. Both runs use the same weekday demand, bus schedules, and road network. |
| 6–12 s | The terminal hides. Show the whole city at 08:00. Fade the Metro lines. Zoom to Tobalaba to give the audience a familiar point of reference. |
| 12–21 s | Show the same map extent and time before and after. Chart modeled trips by mode for the closure window. Show the change in road speed or delay where the engine provides it. |
| 21–25 s | Show the 2019 closure dates and compare Metro service availability. Return to the terminal. |

Use the same scale in both views. Label trip counts with their population and time window. Label the historical panel “Santiago, octubre de 2019”. Label the weekday replay and the observed weekend separately. Present the observed closure as a check on service availability. Keep mode shifts and congestion labeled as model estimates.

## Timed preview evidence

Both runs model 1,487,455 work departures from 07:00 to 10:00. Public transport falls from 772,186 to 663,173. Car rises from 504,342 to 591,475. The cached chart reads the closure result's `closure_window` counts. Both shots used only the replay endpoint during playback. [Timed A/B bundle](https://discord.com/channels/1573514576400752640/1573809606277337280/1573820133032591372) · [Independent numerical check](https://discord.com/channels/1573514576400752640/1573809606277337280/1573820992717856799)

All 126 modeled stations show zero entries in the six closure bins. Outside the window, every station bin and 894 viewer trip groups match the baseline. The preview has no road speed or delay measure. [Engine and cache check](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/371#issuecomment-7069861666)

The runner opened the chart at 1280 × 720 and read its units and counts. The chart appeared between 53.98 and 54.23 seconds after the closure arrived. The saved delay was 12 seconds. The viewer measured 1 FPS in warm-up. [Native timing check](https://discord.com/channels/1573514576400752640/1573809606277337280/1573821785055567944)

## Checks for the paired rehearsal

- Keep the station label, closure sign, and caption visible in the paired video. Time the full 30-second slot on the projector.
- Show road speed or delay only when the final cache provides that measure.
- Confirm zero active stations and zero boardings during the saved closure. The official record establishes the expected service state. A check of travel changes still needs matched observed counts.
- Check the cut from the MUT replay to the paired video, then to the evaluation slide, during the full pitch. [V3 contract](../verticals.md#v3--three-scenario-demo)
