# Open the pitch in the Santiago app

- Status: Accepted
- Date: 2026-11-21
- Source: [Mac’s revised pitch sequence](https://discord.com/channels/1573514576400752640/1573815827617288294/1573846888401342465)

## Decision

Start on the santiaguito.cl welcome screen. Enter opens the full city and the model terminal. A teammate types the MUT question live. Tell the stadium story in the city. Then play the Mapocho and Metro videos together, show measured evaluation evidence, and end with a QR code to santiaguito.cl.

The MUT stays the main case and the other two cases show what the same engine can simulate. [Earlier direction](20261121-2026-mut-main-pitch.md)

## Proposed rehearsal

| Pitch time | Screen and action | Suggested narration |
| --- | --- | --- |
| 0:00–0:20 | Welcome screen. Press Enter, show all of Santiago, and open the terminal. A teammate types «¿Qué pasaría si intercambiamos de lugar el MUT y el Estadio Monumental de Colo-Colo?». | «Hay cambios en Santiago que no podemos probar en la calle. Algunos cuestan demasiado. Otros solo podemos imaginarlos. ¿Y si cambiáramos de lugar el MUT y el Monumental?». |
| 0:20–0:35 | The agent shows the assumptions and opens the saved baseline. Confirm it is visible before playing the experiment. | «El motor simula cómo los cambios afectan los viajes. En esta comparación, el único edificio que se traslada es el estadio. También intercambiamos los empleos de ambos sitios». |
| 0:35–1:20 | Hold the view on the MUT site as the saved comparison shows the cleared site, the stadium in place and the schematic arrival flow. | «Para que entre el Monumental, despejamos 28 edificios y cerramos seis tramos dentro del recinto. Seguimos a la misma hinchada. Miremos cómo llegaría». |
| 1:20–1:55 | Show the fan-mode result after the city view. Use the rebuilt saved pair. | «Con el partido en el MUT, el modelo estima unos 4.100 hinchas más llegando en transporte público». |
| 1:55–2:25 | Play the Mapocho and Metro videos side by side on one slide. Show the ferry boarding and the Metro service disappearing. | «El mismo motor responde otras preguntas. En una, el río se vuelve una ruta de transporte. En la otra, cerramos el Metro durante la mañana». |
| 2:25–2:45 | Show verified evaluation evidence. If the event scores are still pending, show the measured weekday Metro baseline and label it as such. | «Comparamos las entradas diarias al Metro con los registros de 31 estaciones. La suma de los errores absolutos por estación fue el 29,68 % de las entradas observadas. Los puntajes de eventos siguen pendientes». |
| 2:45–2:55 | Show the QR code and the written address. Hold them through the close. | «Escanea el código y prueba tu propia pregunta sobre Santiago». |
| 2:55–3:00 | Allow five seconds for transitions. | |

## Evidence and rehearsal checks

- Check the welcome screen, Enter, the exact live question, and visible playback in one take. The demo chat looks up prepared results and plays the saved baseline and experiment. [Chat integration](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/372)
- The checked pair has 39,222 attendees in both cases. Estimated public-transport arrivals rise from 23,687.5 to 27,789.9. Individual figures illustrate the flow. Show which buildings move on screen before saying both venues moved. [Checked cache](https://discord.com/channels/1573514576400752640/1573809606277337280/1573844937198411918) · [Stadium clip](https://discord.com/channels/1573514576400752640/1573561159955648553/1573846581080498256)
- The 29.68% measure uses default L9, April 13–17, 2026, and daily totals for 31 stations. Use V4 event scores only when the team has matched predictions and observations. [Metro baseline evidence](../research/v1-fixed-score.md)
- Point the QR code to [santiaguito.cl](https://santiaguito.cl/). Scan the final slide at presentation size and time the complete sequence with narration.
