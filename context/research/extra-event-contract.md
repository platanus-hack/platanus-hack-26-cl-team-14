# Extra hypothetical event: proposed accounting contract

**Review proposal only; no build authorization.** The L4b weekday model and
its development score stay unchanged. This case adds an assumed event to a
fixed observed weekend day. It does not move or remove an observed event.

## Evidence gate and scope

The [primary ticket page](https://www.puntoticket.com/red-bull-batalla-internacional-2026),
opened on 2026-11-21, lists Red Bull Batalla at Movistar Arena on April 11,
2026, at 17:00. It provides no end time or measured attendance. The reported
Parque O'Higgins 20:30 contrast is a same-date evening boarding shape, not
identified event exits or evidence for any scenario parameter.

Before implementation, retain an auditable record of all six predeclared
station/day comparisons, source hashes, line membership, peer counts,
per-bin missingness and the exact normalization rule. Root must review that
record and this contract. No parameter is fitted to the observed peak.

Use `metro_observations.json`: SABADO is **2026-04-11**, DOMINGO is
**2026-04-12**, in America/Santiago. These are individual dates, not a typical
weekend. Values are validation-derived boardings excluding estimated fare
evasion, not unique people, alightings or train loads. Null means no source row.

## Proposed request: all numeric assumptions explicit

| Field | Meaning |
| --- | --- |
| `site_id` | One coordinator-approved venue point, with reviewed WGS84 coordinates and source. No arbitrary citywide placement in the first implementation. |
| `day_type` | `SABADO` or `DOMINGO`; response states its actual 2026 date. |
| `attendance` | Assumed persons attending this **additional** event, not capacity or ticket sales. |
| `public_transport_share` | Fraction of attendees departing by public transport. |
| `metro_entry_share_given_public` | Fraction of those public-transport departures producing a local, validation-covered Metro entry. Public transport is not synonymous with Metro. |
| `local_station_share` | Fraction of those Metro entries using the modeled nearby parents. The remainder is explicitly outside this local result. |
| `exit_start_bin`, `exit_end_bin_exclusive` | Half-hour indices within this same date, with a nonempty window in 0–48. No midnight wrap. |
| `assumption_sources` | Value provenance for attendance, each share and exit window: user assumption or cited source, with description. |

No hidden numeric defaults. Attendance is nonnegative; shares are in [0,1].
The declared first timing model is **uniform departures within the supplied
window**. This is an assumption, not a profile learned from the observed peak.
The proposal's 40,000 attendees and two hours are illustrative, not measured
defaults. One departure entry per selected attendee; do not double round trips.

## Calculation and response

Use the unchanged L4b flat-distance function and verified Metro parent points.
Eligible parents lie within 1,000 m; weights are normalized
`1 / max(distance_m, 50)`. This is a geometric access proxy, not walking paths,
capacity, service availability or station choice learned from data.

```text
entries = attendance * public_transport_share
          * metro_entry_share_given_public * local_station_share
event_delta[s,t] = entries * station_weight[s] / window_bin_count  (inside window)
event_delta[s,t] = 0                                               (outside window)
scenario[s,t] = observed[s,t] + event_delta[s,t]                    (observed bin)
scenario[s,t] = null                                              (missing bin)
```

Preserve all eligible parents and their weights. Do not renormalize onto
stations or bins with observations. If no parent is eligible, report the
entries as spatially unassigned; do not guess a nearest station outside 1 km.
An assumed departure during missing coverage remains a modeled contribution,
not an observed zero or proof that the station is open.

Return parent IDs/coordinates/distances/weights, source date/hash, `observed[48]`,
`event_delta[48]`, and nullable `scenario[48]`. Report modeled entries over all
bins separately from observed-mask baseline/scenario/delta totals. Zero
attendance preserves the observed series and its null mask exactly.

Label: **extra hypothetical event on the observed day; accounting sensitivity,
not causal forecast or validation**. Existing events remain in that baseline;
never subtract their presumed contribution. The weekday L4b score must not be
attached to this result. No new routing, commuter OD, capacity or crowd model.

## Internal decisions required before build

Coordinator supplies the bounded venue-point list and reviewed coordinate
sources for the third pitch case; the comparison venues are not automatically
scenario sites. Confirm which weekend date the UI shows and display all
assumptions. Review the evidence artifact, additive API shape and demonstration
scope before assigning implementation. Work not reviewed by 12:15 UTC stays
out of the presentation freeze; human story choice remains separate.
