# Extra hypothetical event: boarding-profile contract

**Accepted contract after review of [evidence PR96](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/96)
and [venue PR97](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/97).**
Implementation may proceed under the staged review gates below. L4b and its weekday
score stay unchanged. Add a scaled observed contrast to a fixed Saturday
baseline. Do not remove the source event or label this operation “move”.

## Evidence and declared scenario choice

The [primary ticket page](https://www.puntoticket.com/red-bull-batalla-internacional-2026),
opened on 2026-11-21, lists Red Bull Batalla at Movistar Arena on April 11,
2026, at 17:00. No end time or measured attendance is supplied. Parque
O'Higgins (PQ) has a same-date evening boarding contrast. Neither fact proves
that the contrast is event exits, or validates transferring it elsewhere.

Before build, review the data owner's artifact: all six predeclared station/day
comparisons, exact line membership, source hashes, peer counts, masks and
original observed-day normalization. A common-mask analysis is a separately
reported sensitivity, not a replacement gate or a selection of the best result.

This contract selects **one** scenario: copy the positive evening contrast,
with a visible multiplier. Attendance, public share and Metro share are not
estimated or added as controls. Attributing and transferring the contrast is
an explicit assumption. Positive clipping and the window below are scenario
choices made after observing the contrast, not a validated event model.

## Fixed template

Use `metro_observations.json`, `SABADO` = **2026-04-11**, America/Santiago.
Units are validation-derived boardings excluding estimated fare evasion;
not unique persons, alightings or train loads. Each index is local k×30 minutes.

```text
D_s = sum of all non-null Saturday bins for station s
m_t = median(peer_t / D_peer) over the original 25 L2 peers observed at t
E_t = max(0, PQ_t - D_PQ * m_t)       for t = 38..47, when both terms exist
E_t = null                           within that window when either is missing
E_t = 0                              outside the window, by scenario assumption
```

PQ is excluded from its peers. Preserve the original day totals even when
coverage differs; do not normalize again across the window or available peers.
The window is **19:00–23:30 on April 11**, not a wrap to that morning.
PQ has 9 observed bins in this ten-bin window; 23:30 is null. The known
contrast mass is `sum(non-null E_t)`, not a complete event attendance or total.
Derive from full-precision source observations and frozen peer IDs, not the
six-decimal comparison CSV. The reviewed template must preserve that precision. Use peer IDs from
`data/validation/event_shape_2026_04/manifest.json`. Expected known mass is
3399.865062907704 boardings. A small separately labelled scenario-profile
artifact derived from those exact inputs is permitted; rounded CSV is not.

## Exact proposed interface and geography

Implement separately in `event.rs`; leave source-bound `stations.rs` and the
L4b fit untouched. The existing API must still return its bound L4b score.
Add `POST /api/event-scenario` with request `{"site_id":"<approved id>",
"multiplier":1}`. Saturday and the template revision are fixed, not request
parameters. Multiplier is explicit, finite and in **[0,3]** for this demo: 1 copies the known
contrast; 0 means no extra contribution. No hidden parameter defaults.

Use only the two entries in reviewed `data/mvp/event_venues.json`:

| `site_id` | Longitude, latitude | Coordinate source |
| --- | --- | --- |
| `estadio_nacional` | -70.6106404880707, -33.4644707146861 | OSM venue way 28092600, version 27 |
| `estadio_monumental` | -70.60596628713479, -33.50653677377368 | OSM venue way 297163434, version 7 |

These are venue-polygon centroids, not entrances or station coordinates.
The table records primary URLs, source hashes, retrieval date and centroid method.
Its review is a **pre-implementation gate**. Accept only these IDs, not arbitrary
coordinates. The audited geometry expects ESN alone for Nacional, and PE plus
MA for Monumental; calculate distances and weights from the pinned parent data.

Use L4b's unchanged flat distance and parent points. Parents within 1,000 m
receive weights proportional to `1/max(distance_m,50)`. This is geometric
access, not walking routes, service/capacity or measured station choice. Do not
renormalize based on observation coverage. With no eligible parent, report the
known contrast mass as unassigned; do not choose a more distant station.

## Output, missingness and accounting

For each eligible parent, `event_delta[t] = multiplier * weight * E_t`.
For a positive multiplier, a null E remains null. `scenario[t]` is
`observed[t] + event_delta[t]` only when both exist; otherwise null. An observed
null remains null even when the modeled contribution is known. Outside the
window the added contribution is zero by assumption, not evidence of no trips.
**At multiplier 0**, every delta is zero and scenario equals observed exactly,
including its null mask; missing template bins cannot change a no-op baseline.

Return the actual source date/hash/template revision; site and parent IDs,
coordinates, distances and weights; `observed[48]`, nullable `event_delta[48]`,
nullable `scenario[48]`, and source/target coverage masks. Report separately:
(1) known template mass × multiplier, (2) known assigned and unassigned mass,
and (3) baseline/scenario/delta sums over their common known target bins with
that bin count. Do not compare totals with different masks. Always retain the
full observed-day total as a separate value, not the scenario comparator.

Label **“additional hypothetical event; contrast attribution and transfer are
assumptions”**. The source-day activity stays in every baseline, including PQ.
No causal forecast, event validation, attendance inference, weekday score,
refit, commuter OD or capacity model. Root must review the evidence artifact,
site table and this contract before implementation is pushed or merged. UTC
gates: data/contract 10:50, module 11:15, route 11:35, scene 11:50, browser
12:10, freeze eligibility 12:15. Human story choice remains separate.
