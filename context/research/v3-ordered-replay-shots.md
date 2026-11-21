# Ordered shots for the cached demo

Use one saved result per shot. The viewer takes the newest queued result each frame. Sending the baseline and the experiment together can skip the baseline. The relay receipt confirms publication. The runner confirms what appeared on screen. [Viewer input](../../viz/src/net.rs) · [Replay relay PR](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/350)

## Metro cue

| Shot | Saved result | Runner check |
| --- | --- | --- |
| Baseline | Normal weekday service at Tobalaba, 08:00 Chile time | Open service appears. The camera reaches the comparison frame. |
| Closure | Full Metro closure, same camera and 08:00 frame | Closed service appears. The clock restarts at 08:00. The final chart uses the saved closure result. |

Show the experiment assumptions before the baseline. Hide the terminal while the shots play. Hold the baseline until the audience can read it. Publish the closure after that hold. Return to the terminal after the chart.

Give each publication a new result revision. Give a camera cue a new scene revision when the camera must move. The scene half-hour sets the initial viewer clock for each result. For Tobalaba, use half-hour 16 in both shots. Use the 07:00–10:00 service window in the experiment inputs when V1 provides it. [Camera](../../viz/src/focus.rs) · [Clock](../../viz/src/movers.rs)

## Rehearsal evidence

Record each replay ID, publication order, and visible frame. Open one capture from each shot and check the camera, clock, and service state. Rehearse with the relay running and verify zero simulation calls during playback. A Bevy window capture completes asynchronously. Use the returned image to confirm a captured frame. [Bevy window capture](https://docs.rs/bevy/latest/bevy/render/view/window/screenshot/struct.Screenshot.html)

The current relay publishes one snapshot per call. The cue order, terminal visibility, and final chart still need a complete playback run.
