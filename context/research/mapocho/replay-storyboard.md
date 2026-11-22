# V3 · Mapocho replay storyboard

## Current pitch

At 1:55, the presenter plays one 30-second video with Mapocho on the left and Metro on the right. The Mapocho source is Matta’s 8.9-second fixed shot at Pío Nono. It shows the ferry approach, passenger movement at the pier, and the start of departure. The figures illustrate the modeled flow. Keep the source credits with the clip. [Pitch decision](../../decisions/20261121-2045-live-pitch-opening.md) · [Mapocho clip and credits](https://discord.com/channels/1573514576400752640/1573815827617288294/1573843670031929424)

## Saved replay

- Generate the two authoring requests with `python3 tools/pack/mapocho_ferry_replays.py`. The saved IDs are `mapocho-muelle-base` and `mapocho-muelle-ferry`. Both start at 07:00 with the same camera at 155 m. The ferry cues show approach at +0 s, boarding and alighting at +9 s, and the result at +17 s. [Authoring and cached pair](https://discord.com/channels/1573514576400752640/1573809606277337280/1573846221167263784)
- The chart gives 3,470.2 eastbound and 932.2 westbound modeled work trips per weekday. The +17 s caption rounds the eastbound value to 3.470. The unit is a modeled weekday total. For the live paired replay, extract the bundle in an isolated checkout and run `tools/side-by-side.sh mapocho-muelle-ferry metro-window-closure`. The bundle uses two saved results. [Paired replay bundle](https://discord.com/channels/1573514576400752640/1573809606277337280/1573855461839740988)

## Earlier solo rehearsal

The original storyboard allowed 35 seconds for the terminal question, camera flight and a proposed congestion chart. The table and the early evidence checklist below record that plan.

### Question and assumptions

> ¿Qué pasaría si pudiéramos cruzar Santiago en barco por el Mapocho?

Show the terminal response before playback. Compare the same weekday from 07:00 to 10:00 with and without ferry service. Set the start and duration to 07:00 and three hours in Chile time. This is a proposed demo window. Use the ten hypothetical piers from Cerro Navia to Vitacura. Assume a navigable channel, a boat speed of 17 km/h, and departures every 10 minutes. Include the walk to and from the pier, the wait, and intermediate stops. Compare travel time, people flow by mode, and congestion on the same streets. [Pier locations](piers.json) · [Trip-time assumptions](boat_trips.md)

### Screen cues

| Time | Screen | Presenter |
| --- | --- | --- |
| 0–8 s | Show the question and the service assumptions in the terminal. | “¿Y si el río fuera una ruta de transporte?” |
| 8–17 s | Hide the terminal. Frame the full river route, then zoom to one pier and its walking connections. Show a boat on the river when the replay supplies its position. | “El viaje empieza mucho antes de subir al barco.” |
| 17–29 s | Show the two runs at the same simulated time and map scale. Chart people flow in people per hour and congestion as delay in minutes on the same street segments, once the engine supplies both measures. | “Veamos quién cambia de ruta y qué pasa en las calles.” |
| 29–35 s | Hold the final comparison. Return to the terminal. | “La pregunta es para quién conviene el barco.” |

### Evidence before capture

- The 2012 survey supports a separate travel-time comparison. At 17 km/h and a 10-minute service interval, the ferry trip was faster than the reported trip for 10,871 of 23,941 expanded weekday trips (45.4%). The estimate comes from 94 survey records near different hypothetical piers at least 3 km apart along the pier polyline. The sample covers a full weekday. If shown, label it “10.871 de 23.941 viajes cerca de muelles ahorrarían tiempo · EOD 2012”. The separate comuna-point comparison found a longer trip by boat on all 42 pairs. State that limit when quoting the survey figure. Keep this figure out of the 07:00–10:00 simulation charts. [SECTRA survey](https://www.sectra.gob.cl/info-territorial/region-metropolitana-de-santiago/) · [Trip-level comparison](boat_trips.md) · [Comuna-point comparison](../mapocho-navegable.md#7-second-engine-test-the-boat-as-faster-travel-minutes)
- V1 must supply the ferry service properties and a baseline and experiment run for the same clock window. Record the engine revision, inputs, replay IDs, and the output fields for both charts. Confirm how segment delay is defined and what counts as one person in the flow chart. The current travel-time brief leaves fare, capacity, and mode choice open. [Mapocho research](../mapocho-navegable.md)
- Open the cached pair. Check that playback reads cached results and makes zero simulation calls. Check the prompt, timing, route, camera, charts, and terminal return. Check the numbers against the recorded output before capture. The storyboard is ready for review. The cached run and screen cues still need verification.
