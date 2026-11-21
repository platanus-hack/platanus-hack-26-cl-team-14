# Full Metro closure replay

Draft for the three-minute pitch. Allow 25 seconds after the presenter submits the question. The modeled event is a weekday closure from 07:00 to 10:00, Chile time. The scene begins at 08:00.

## Research brief

Metro announced the full closure at 19:20 on Friday, 18 October 2019. The network stayed closed until Monday, 21 October, when 20 Line 1 stations reopened. The closure covered the weekend. The period also had curfews, street disruptions, and changes to bus service. [DTPM 2019 report, chapter 7](https://www.dtpm.cl/descargas/memoria/InformeGestion_2019_DTPM.pdf#page=109)

Use the full closure dates to show the historical service state. To compare the travel response, get observed counts for a defined closure window and a comparable reference window from V1. Align day type, time, area, mode, and units. October-to-December totals combine full closure, partial service, and recovery. [DTPM 2019 report, tables 33–34](https://www.dtpm.cl/descargas/memoria/InformeGestion_2019_DTPM.pdf#page=110)

## Presenter and camera

Type this question in the terminal.

> ¿Qué pasaría si cerráramos todo el Metro de Santiago entre las 7 y las 10 de la mañana de un día hábil? Muéstrame cómo cambiarían los viajes y dónde aumentaría la congestión.

| Time | What the audience sees |
| --- | --- |
| 0–6 s | The terminal shows the saved assumptions. All active Metro stations and lines close from 07:00 to 10:00. Both runs use the same weekday demand, bus schedules, and road network. |
| 6–12 s | The terminal hides. Show the whole city at 08:00. Fade the Metro lines. Zoom to Baquedano to give the audience a familiar point of reference. |
| 12–21 s | Show the same map extent and time before and after. Chart modeled trips by mode for the closure window. Show the change in road speed or delay where the engine provides it. |
| 21–25 s | Show the 2019 closure dates and any verified comparison. Return to the terminal. |

Use the same scale in both views. Label trip counts with their population and time window. Label the historical panel “Santiago, octubre de 2019”. If the historical window and modeled window differ, show both labels and present the historical service state as context.

## Before capture

- V1 supplies the set of active Metro entities, a time-bounded service change, and the trip and congestion outputs. The current station change removes an access point for the full request. It does not cancel train service or change commute mode choice by itself. [Station change](../../engine/src/scenario_stations.rs) · [Scenario result](../../engine/src/scenario.rs)
- V1 supplies a measured 2019 comparison with the source, date, population, and units. If only the closure dates are verified, the travel response remains unverified.
- Save the question, assumptions, and result through the replay tools. Play the saved result in the city view. Check the 08:00 frame, the final chart, and the terminal return. Time the take as part of the full pitch. [V3 contract](../verticals.md#v3--three-scenario-demo)
