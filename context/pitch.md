# Pitch and demo

Source of truth for scope. If a moment is not here, the team does not build for it.
Humans own this file. Agents can propose changes in Discord.

- Status: accepted sequence; rehearsal checks remain open
- Direction: [20:45 live opening](decisions/20261121-2045-live-pitch-opening.md), with the [Gabriela handoff](https://discord.com/channels/1573514576400752640/1573561159955648553/1573876065225547838).

## Problem (one sentence)

Santiago makes city decisions without seeing how they change the day of each person.

## Story (3 minutes or less)

Use the accepted 20:45 sequence below. Spoken lines are in Spanish. The times are stage targets, not measured completion times.
The complete narrated run, including typing, confirmations, loading, media transitions, and the visible close, must take 3:00 or less. That check remains open.

### Prepare the media

- Live MUT: use the checked-in `mut-monumental-routes-a` baseline, then `mut-monumental-routes-b`. The pair starts at 13:30 with the same camera at 650 m. See [the replay package](../data/replays/README.md) and [its assumptions](research/v3-mut-replay.md).
- Paired video: use `mapocho-metro-movimiento.mp4` from [Gabriela's motion package](https://discord.com/channels/1573514576400752640/1573809606277337280/1573856487351001180). This replaces the earlier montage with Metro stills. Keep the source credits with the media (#509).
- Evaluation and close: use `evaluacion.png` and `santiaguito-qr.png` from [the rehearsal package](https://discord.com/channels/1573514576400752640/1573809606277337280/1573854741971214396). Mac still selects the final evaluation and DTPM contrast. Do not replace the slide with unapproved scores.
- The Mapocho source pair is `mapocho-muelle-base` then `mapocho-muelle-ferry`. The Metro source pair is `metro-window-baseline` then `metro-window-closure`. These are sources for the paired video; the stage sequence does not type two more live questions.

### Run of show

| Time | Screen and operator action | Presenter cue |
| --- | --- | --- |
| 0:00–0:20 | Start on the santiaguito.cl welcome screen. Press Enter. Show the city and terminal. A teammate types the exact MUT question below. | «Hay cambios en Santiago que no podemos probar en la calle. Algunos cuestan demasiado. Otros solo podemos imaginarlos. ¿Y si cambiáramos de lugar el MUT y el Monumental?» |
| 0:20–0:35 | Read the assumptions. Confirm the request to show A in a new message. Wait for A, then confirm that it is visible. The chat can now play B. | «El motor simula cómo los cambios afectan los viajes. En esta comparación, el único edificio que se traslada es el estadio. También intercambiamos los empleos de ambos sitios». |
| 0:35–1:20 | Hold the MUT city view. Show the cleared site, the stadium, and the schematic arrival flow. | «Para que entre el Monumental, despejamos 28 edificios y cerramos seis tramos dentro del recinto. Seguimos a la misma hinchada. Miremos cómo llegaría». |
| 1:20–1:55 | Show the saved fan-mode result after the city view. The 1:20 reveal remains blocked by #537; see below. | «Con el partido en el MUT, el modelo estima unos 4.100 hinchas más llegando en transporte público». |
| 1:55–2:25 | Switch from the live city to `mapocho-metro-movimiento.mp4`. Play its 30 seconds, with Mapocho left and Metro right. | «El mismo motor responde otras preguntas. En una, el río se vuelve una ruta de transporte. En la otra, cerramos el Metro durante la mañana». |
| 2:25–2:45 | Switch to the selected evaluation slide. The 2019 service check belongs here if Mac selects the DTPM contrast. | Give only the evidence on the selected slide. If using the daily baseline, name its 31 stations and the absolute-error sum, 29.68% of observed entries. Do not call it an event score. |
| 2:45–2:55 | Switch to `santiaguito-qr.png`. Keep the QR and `santiaguito.cl` visible through the close. Put jury slides after the QR. | «Escanea el código para descargar la app de escritorio y probar tu propia pregunta sobre Santiago». |
| 2:55–3:00 | Keep the close visible. Reserve these five seconds for transitions. | No new content. |

Exact live question:

> ¿Qué pasaría si intercambiamos de lugar el MUT y el Estadio Monumental de Colo-Colo?

The chat first lists the saved replays and ends its assumptions turn with «¿Muestro el escenario base?».
Confirm in a new message. After A opens, the chat asks «¿Ves el escenario base?».
Confirm again before B. Do not send both playback calls in the initial question turn.
Playback uses saved results. Do not call `create_replay` during the pitch.

### Open timing and display checks

- #537: the checked-in B reveals its chart and arrival label at +18 seconds. This does not meet the 1:20 stage cue. Projection owns the cache change. Do not choose +42 or +45 seconds without the final timed chat take.
- [Pebre's handoff](https://discord.com/channels/1573514576400752640/1573561159955648553/1573875932773752833) reports A visible at 0:32 and B at 0:38 in one technical run. It does not prove a narrated three-minute pitch.
- #516: the QR opens a landing page for a desktop download. It does not open a phone chat. Scan the final slide on the physical projector; this check remains open.
- #508 and #509: closure visibility and media credits remain open. Preserve attribution outside the opening screen.
- Check the full sequence on the final viewer revision, with the terminal beside the city. Keep the stadium, captions, legend, and result readable.

## The main moment and the historical check

The MUT city view at 0:35–1:20 is the main case. The two-panel video shows the other two cases at 1:55.
The October 2019 comparison is a service-availability check, not a separate live Metro slot.
Keep its dates visible in the evaluation material if Mac selects that contrast at 2:25.

Metro announced the closure at 19:20 on 18 October 2019. Zero stations operated on 19–20 October; 20 Line 1 stations reopened on 21 October.
See [the Metro source](research/metro-closure-replay.md). The saved scenario models a weekday morning. The observed closure covered a weekend, with curfews and bus changes.
Do not use the zero-station comparison to validate modeled trips, mode shifts, or congestion.

## What the demo must show

- The welcome screen, Enter, live question, assumptions, and two confirmations precede the MUT experiment.
- A and B use the same frame, clock, and scale. The playback reads saved results without a new simulation.
- Each result shows its unit and time window. Model estimates remain separate from measurements.
- The paired motion video starts at 1:55. Evaluation starts at 2:25. The download QR starts at 2:45.
- No car moves on the river. No boat moves on a street.
- The operator records the narrated duration and the final visible close. The target is 3:00 or less, not a completed check.

## Figures and sources

The checked-in replay package and linked research record the saved inputs. These figures do not replace the final native rehearsal.

| Figure | Source | State |
| --- | --- | --- |
| 39.222 asistentes, Superclásico, 31 Aug 2025 | `context/research/monumental-match-attendance.md` | Input, not a result. Two match reports. No official open series. |
| Modeled attendees by mode, Monumental → MUT with job swap: public 23,687.5 → 27,789.9; car 12,758.9 → 9,397.8; other 2,775.6 → 2,034.3; total 39,222 | [MUT smoke](https://discord.com/channels/1573514576400752640/1573809606277337280/1573814113350582272), [routed MUT pair](research/v3-mut-replay.md), `mut-monumental-routes-a` / `mut-monumental-routes-b` | Smoke check del motor y caché; visor nativo pendiente. |
| 10 piers, 17 km/h, a boat every 10 min | `context/research/mapocho/replay-storyboard.md`, `context/research/mapocho/piers.json` | Assumptions, not results. |
| 10.871 de 23.941 viajes cerca de muelles ahorrarían tiempo (45,4 %), EOD 2012 | `context/research/mapocho/boat_trips.md` | Full weekday, not 07:00–10:00. Keep it out of the replay chart. The comuna-point comparison found a longer boat trip on all 42 pairs. Say this limit if you quote the figure. |
| Closure announced at 19:20, 18 Oct 2019; 20 Line 1 stations reopen on 21 Oct | `context/research/metro-closure-replay.md` (DTPM 2019 report) | Official record. |
| Weekday Metro entries, B − A: Tobalaba −2,598.5; Pedrero +2,249.9; Mirador +1,826.5 | [Routed MUT pair](research/v3-mut-replay.md), `result.stations[].scenario_daily` | Smoke check del motor y caché; visor nativo pendiente. The engine does not link a loss at one station to a gain at another. |
| 0 active stations, 19–20 Oct 2019 | `context/research/metro-closure-replay.md` | Official record. |
| Metro closure, 07:00–10:00: 1,487,455 modeled work departures; public 772,186 → 662,807; car 504,342 → 591,740 | [Final Metro cache](research/metro-closure-replay.md) | Modeled closure-window counts, not observed demand. |
| Mapocho: 3,470.2 eastbound and 932.2 westbound modeled work trips per weekday | [Saved ferry pair](research/mapocho/replay-storyboard.md) | Weekday totals, not people per hour or measured demand. |
| Road delay for the Metro closure | none yet | Unverified. Do not claim a measured delay. |
| Final evaluation slide | [Gabriela handoff](https://discord.com/channels/1573514576400752640/1573561159955648553/1573876065225547838) | Mac must select the evidence. Do not imply that three event scores are ready. |

Pending for all scenes: native playback on the final head, chart rendering, and the timed rehearsal.
