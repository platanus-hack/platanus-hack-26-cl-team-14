# Ferry motion in the cached Mapocho shot

The Mapocho shot zooms from the route to the river near Pío Nono. The native viewer draws the new line. It has no ferry model or motion for that line. The proposal below uses the cached result for the vessel. [Routed cache and preview](https://discord.com/channels/1573514576400752640/1573809606277337280/1573822794423345193) · [Existing line draw](../../viz/src/city/edits.rs)

## Saved data

I opened the baseline and ferry JSON files from the V3 preview at `f6da8dec5b7a72b37d1112a73e278a6b3e6206e4`.

- The baseline has no boat edit. The ferry result has two `city_edits` with `kind: "line"`, `op: "add"`, and `after.properties.mode: "boat"`. Their IDs are MAPOCHO-E and MAPOCHO-W.
- Each `after.geometry.coordinates` has 355 positions in travel order as longitude and latitude. Each `after.properties` has ten ordered `stop_ids`, `new_stops` with point coordinates, nine `segment_minutes`, ten `dwell_minutes`, and a 10-minute `headway_minutes`.
- The saved properties give a headway. I found no first departure, last departure, or phase. V3 needs to confirm this assumption before the viewer can draw a scheduled vessel.

## Proposed draw

Read the boat lines when the result revision changes. Project their stops onto the saved line in travel order. For each active departure, advance by distance along the saved line within each timed leg. Hold at the projected stop during dwell. Use the local line direction for the hull. Use the simulated clock for route motion and the existing presentation clock for camera cues. Clear the vessels when the baseline replaces the ferry result. The hull and wake need a native check against the mapped banks. [Clock inputs](../../viz/src/movers/clock.rs) · [Ferry shape mock](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/347)

## Close-up witness

The eastbound MAP07 vertex is `[-70.63568039,-33.43594133]`, 0.15 m from the planning point. The sum of the first six leg times and preceding dwell is 51.198 minutes. With departures on :00 and :10 and service active before 07:00, an eastbound 06:10 ferry arrives at 07:01:11.88. It holds for one minute. That arrival is about 9 seconds into a 07:00 shot at 8× playback before frame delays. The preview asks for a close camera cue at 4 seconds and a chart at 11 seconds.

Compare baseline and ferry at the same river camera. For a fixed-clock check use `SANTIAGO_FOCUS=-70.637,-33.43559`, `SANTIAGO_DISTANCE=630`, and `SANTIAGO_CLOCK_SECONDS=25272`. Confirm that the baseline has no vessel and that the ferry holds at MAP07 under the agreed schedule. Then watch moving playback. The schedule assumption, native framing, hull placement and cue timing remain unverified.
