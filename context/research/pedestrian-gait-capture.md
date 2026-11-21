# Pedestrian gait in the opening shot

The pitch proposal follows one traveler at street zoom. Before #396, the viewer moved and turned one fixed person mesh along the route. Walking, waiting, and station entry figures used that pose. #396 adds walking frames and keeps a separate standing pose. [Original mover draw](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/f078db050f29286771e01e78e672c6d83232d6f1/viz/src/movers.rs) · [Integrated gait](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/396) [Pitch proposal](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/354) · [Person mesh and draw](../../viz/src/movers.rs) · [Rider states](../../viz/src/movers/riders.rs) · [Station entry figures](../../viz/src/movers/street_people.rs)

## Motion reference

During the planted part of a step, the foot should stay at the same point on the sidewalk. Distance traveled along the route can drive gait phase without changing the route or its arrival time. Keep waiting figures still. Kovar, Schreiner, and Gleicher describe stationary foot contacts and the sliding that appears when an animation breaks them. [Footskate cleanup](https://graphics.cs.wisc.edu/Papers/2002/KSG02/)

The default replay advances eight city seconds per screen second. A study of adults aged 21–40 uses about 100 steps per minute as a moderate walking reference. At eight times speed, that cadence would show about 13 steps per screen second. Start at one times speed to inspect contact. Check the final shot again at its actual playback rate. The study cadence is a reference for the check. [Playback rate](../../viz/src/movers.rs) · [CADENCE-Adults study](https://pmc.ncbi.nlm.nih.gov/articles/PMC6337834/)

## Capture

- Tagua’s run of follow PR #384 selected group 258, copy 3. The log confirms a 140 m camera. The 07:31 still shows people on the street, but the still does not identify the followed figure.
- For foot contact, use the same baseline result and start time. Leave follow off. Set `SANTIAGO_LOCK_CAMERA=1` and `SANTIAGO_PLAYBACK_RATE=1`. Aim a street camera at the sidewalk and record one visible walker. Compare the old and new pose at the same fixed camera. Look for left and right steps and a foot that stays on the pavement during contact. This check uses the figure in the frame.
- For the pitch shot, use a separate run with `SANTIAGO_FOLLOW=1` and an unlocked camera at the presentation rate. Record the group and copy from the follow log. Open the 140 m shot and check that the audience can identify the followed person and see them walking. If the person reaches a waiting state, check that the stepping stops.

The gait clip and the final-rate check remain open. [Follow capture and log](https://discord.com/channels/1573514576400752640/1573809836699820163/1573823823596228669)
