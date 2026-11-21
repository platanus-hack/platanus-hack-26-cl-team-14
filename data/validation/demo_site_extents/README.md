# Demo site extents

These two OSM outer boundaries fit inside PR #409's 500 m capture boxes.
This check supports the MUT and Monumental demo scenes. It does not check a downloaded scene.

| Site | West clearance | East clearance | North clearance | South clearance | Outer / inner ways |
| --- | ---: | ---: | ---: | ---: | ---: |
| MUT | 193.319 m | 161.993 m | 198.082 m | 196.285 m | 1 / 3 |
| Monumental | 135.241 m | 137.981 m | 110.205 m | 117.314 m | 1 / 1 |

## Sources and reproduction

The full public OSM responses were fetched with AX on 2026-11-21.
Both relations have version 2. The CSV records relation timestamps and raw SHA-256 hashes.
The saved responses also retain each member way and node version.
A relation version alone does not freeze its member geometry.

- MUT: <https://api.openstreetmap.org/api/0.6/relation/18760408/full.json>
- Monumental: <https://api.openstreetmap.org/api/0.6/relation/14111047/full.json>
- Capture formula: [PR #409 source, pinned commit](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/85b9def9fbad92acf9ffe00762b2b415258b6950/tools/demo-tiles-capture/src/main.rs#L13).
- Data: © OpenStreetMap contributors, [ODbL 1.0](https://www.openstreetmap.org/copyright).

From the repository root:

```sh
python3 tools/scrape/demo_site_extents.py --saved
```

Omit `--saved` to fetch current data with AX and replace the saved responses and CSV.
The script requires Python 3 and AX. It uses no other Python packages.

## Method and checks

The script uses the exact preset centres and WGS84 ECEF constants from the pinned PR.
It applies the PR's east/south rotation to outer-way vertices. The half-extent is 250 m.
OSM nodes have no heights. The script sets each node's ellipsoid height to zero.
Anchor height translates only the local up axis, so it cannot change these horizontal clearances.
The approximation concerns node height and map accuracy, not a different capture projection.

For local east x and south z, clearances are:
W = 250 + min(x), E = 250 - max(x), N = 250 + min(z), S = 250 - max(z).
Positive values mean the boundary is inside that edge. Negative values mean it extends outside.
Three decimal places preserve reproducible calculations; they do not imply millimetre survey accuracy.

Raw inspection found one closed outer way per relation: 16 unique MUT nodes and 54 unique Monumental nodes.
The member counts and versions agree with the CSV.
An independent local metres-per-degree calculation agreed within 0.2 m on all four edges.
A second run with `--saved` reproduced the results.

## What this proves

The saved OSM outer vertices, and straight segments between their projected positions, fit inside both boxes.
This is a check of the selected relations, not proof of a complete property boundary.
Monumental relation 14111047 describes a building multipolygon with an inner hole; it is not the football field.
The field geometry and the complete club property are different objects.

Static bounds do not prove Google tile availability, complete buildings, textures, refinement, or correct display orientation.
They do not establish camera framing, height alignment, or successful offline reopening.
The capture selects intersecting tile bounds and keeps whole edge tiles. Its 500 m box is not a mesh clipping boundary.
No viewer, capture, or build ran for this check.
