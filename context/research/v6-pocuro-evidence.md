# V6 · CicloRecreoVía evidence for Pocuro

V6 starts with one real corridor in Providencia. Use Pocuro between Tobalaba and Antonio Varas to show the street opening for the activity, participants moving and stopping, and the street reopening to cars. The citywide weekday model does not supply a Sunday baseline. [V6 direction](https://discord.com/channels/1573514576400752640/1573903663821226187/1573918209529090092)

## Verified inputs

| Input | Source and scope |
| --- | --- |
| Route and schedule | Providencia lists Pocuro from Tobalaba to Antonio Varas, plus connected sections on Ricardo Lyon, Providencia and Andrés Bello. The published Sunday schedule is 09:00–14:00 local time. People on foot, cyclists, skaters and pets can use it. The 8.4 km figure covers the municipal route. [Municipality of Providencia](https://providencia.cl/provi/vive/movilidad/ciclorecreovias) |
| Map geometry | The repo road pack contains named Avenida Pocuro centerlines. Match the municipal endpoints to the map before setting closure IDs. A centerline gives geometry. The municipal description does not specify each carriageway or crossing rule. [Road pack](../../data/mvp/README.md#roadsgeojson) |
| Event scale | The organizer reports about 30,000 participants per weekend across Santiago. Its page gives no date series, corridor breakdown, or count method. Use it as citywide context. [CicloRecreoVía Santiago](https://ciclorecreovia.cl/santiago/) |
| Pocuro activity mix | Mora et al. interviewed attendees at rest stations in August–September 2016. Table 1 has 81 Pocuro respondents out of 401 across five circuits. The Pocuro mix was 65% cycling, 20% jogging, 6% walking and 9% other. Table 3 reports 107 minutes of intended time at Pocuro. Its samples describe reported activity and intended time, with no pass counts. Table 2 gives a different Pocuro denominator. Use Table 1 for the mix. [Mora et al., 2018, pp. 453–456](https://www.scielo.cl/pdf/rmc/v146n4/0034-9887-rmc-146-04-0451.pdf) |

Across the full sample of 401 respondents on five routes, 68.4% said they usually cycled. The mean reported time in the activity was 113 minutes. [Mora et al., 2018](https://www.scielo.cl/article_plus.php?lng=es&pid=S0034-98872018000400451&tlng=es)

## First comparison

Use the same initial city and a matched window for both runs. Run A without the activity. Run B with Pocuro open to participants during the declared hours. Show **activity participants** crossing each named point per time bin, present at each step and leaving. A zero activity count in A says nothing about ordinary traffic or shoppers on Pocuro. If B includes an assumed shop stop, report visits and dwell for the modeled participants. Report these as scenario outputs.

Reuse the published hours and names. Treat total Pocuro participants, entry times, direction splits, speed, shop stop probability and shop dwell as scenario inputs until there are compatible measurements. A 2016 intended duration can guide a sensitivity check. It does not set a modern arrival rate, speed or shop dwell. Keep pets attached to their participant group if the viewer shows them.

Use a small queue witness before adding capacity assumptions at scale. Feed the same participants into two activity runs. Change one assumed stop duration or entry capacity. Account for everyone at each step. Verify that the later crossing changes, and report the time and segment where it changes.

## How to check real activity flows

Collect crossings at the same fixed points, in the same time bins and by the same mode definitions, on an event Sunday and a comparable Sunday without the activity. Retain date, time, direction and count point. A person can cross several points or cross twice. To estimate unique attendance, a separate estimate of repeat crossings is needed. This distinction follows a published open-street evaluation. [CicLAvia evaluation, Los Angeles](https://pmc.ncbi.nlm.nih.gov/articles/PMC5083970/)

The Pocuro survey and public schedule give inputs and context. I found no public Pocuro event-day count series by point and interval, matched Sunday without the activity, or measured shop visits and sales in the sources above. Compare the modeled activity flows first. Sales effects need observed visits or transactions and a comparable control period.

SUMO provides a reference for carrying position and movement stage between steps. Its detailed pedestrian rules also require sidewalk, crossing and demand data. For this corridor, persistent route, wait and stop states are enough for the first witness. [SUMO update loop](https://github.com/eclipse-sumo/sumo/blob/main/docs/web/docs/Developer/Implementation_Notes/Simulation_Loop.md)
