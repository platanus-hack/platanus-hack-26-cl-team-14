# Pedestrian gait in the opening shot

The pitch proposal follows one traveler at street zoom. The current viewer moves and turns the person mesh along the route. The arm and leg poses stay fixed. The draw path uses the same mesh for a routed person walking and waiting. A separate station-entry draw path also uses this mesh. [Pitch proposal](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/354) · [Person mesh and draw](../../viz/src/movers.rs) · [Rider states](../../viz/src/movers/riders.rs) · [Station entry figures](../../viz/src/movers/street_people.rs)

## Motion reference

During the planted part of a step, the foot should stay at the same point on the sidewalk. Distance traveled along the route can drive gait phase without changing the route or its arrival time. Keep waiting figures still. Kovar, Schreiner, and Gleicher describe stationary foot contacts and the sliding that appears when an animation breaks them. [Footskate cleanup](https://graphics.cs.wisc.edu/Papers/2002/KSG02/)

The default replay advances eight city seconds per screen second. A study of adults aged 21–40 uses about 100 steps per minute as a moderate walking reference. At eight times speed, that cadence would show about 13 steps per screen second. Start at one times speed to inspect contact. Check the final shot again at its actual playback rate. The study cadence is a reference for the check. [Playback rate](../../viz/src/movers.rs) · [CADENCE-Adults study](https://pmc.ncbi.nlm.nih.gov/articles/PMC6337834/)

## Capture

- Tagua’s run of follow PR #384 selected group 258, copy 3. The log confirms a 140 m camera. The 07:31 still shows people on the street, but the still does not identify the followed figure.
- For a gait check, use the same baseline result and start time. Record one walking segment with `SANTIAGO_PLAYBACK_RATE=1`. Use a fixed street camera to see foot contact against the pavement. Confirm the group and copy in the log again. Compare with the original pose at the same camera and time.
- Read a left step, a right step, and a planted foot on the sidewalk. If the selected person reaches a waiting state in the clip, check that the feet stop stepping. Then open the final 140 m follow shot at the presentation rate. Confirm the audience can identify the followed person and read that they are walking.

The gait clip and the final-rate check remain open. [Follow capture and log](https://discord.com/channels/1573514576400752640/1573809836699820163/1573823823596228669)
