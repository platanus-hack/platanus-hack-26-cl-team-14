# Metro observations for an independent score

Role: TARGET candidate. `score_ready=false`: the split, formula, and a model that predicts these quantities independently are not frozen.
This is depth layer D. It adds observed targets, not UI features or a new forecast.

## Research basis and source contract

Reuse [the verified April workbook and March GTFS](santiago-transport-verified.md), the [offline boarding exporter](../../tools/scrape/export_april_boardings.py), and [the reviewed station extraction](tobalaba-observations.md).
The source workbook is [DTPM April 2026](https://www.dtpm.cl/descargas/modelos_y_matrices/Subidas_Paradero_Estacion_2026.04_v3.xlsx).
The matching feed is [GTFS March 21](https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip).
No new download was needed. Both local source files had already been verified.

- Unit: validation-derived station boardings, excluding estimated fare evasion. Not unique people, train loads, line transfers, or all travel demand.
- Period: LABORAL averages April 13–17; SABADO is April 11; DOMINGO is April 12. They are three different day-type observations, not seven daily samples.
- Geography: the 126 station names with Metro observations in this workbook. This is not a completeness claim for every station or GTFS release.
- Time: 48 half-hour slots per day, index 0 = local 00:00, timezone America/Santiago. Missing observations stay null.

## Matching and real run

The full source has 13,015 Metro rows, with no repeated source station/day/bin keys. It yields 126 distinct parent IDs: 120 unique names after case, Unicode accent and whitespace normalization, plus six explicit aliases.
Every source row remains traceable by its worksheet row number. There is no line/platform split or duplicate allocation to platforms.
The initial six unresolved names were reviewed independently against the original station names, GTFS parent/platform records, and source comunas. [Review evidence](https://discord.com/channels/1573514576400752640/1573579088684253235/1573591251574587474).

| Source name | GTFS parent | Recorded difference |
| --- | --- | --- |
| CAL Y CANTO | CA-PCA | Puente prefix omitted; one L2/L3 parent |
| PARQUE OHIGGINS | PQ | Apostrophe and spacing |
| PDTE PEDRO AGUIRRE CERDA | PAC | Presidente abbreviated; platforms also use Pdte. |
| PLAZA MAIPU | PM | de omitted; accent |
| RONDIZONNI | RO | Reviewed source spelling versus Rondizzoni |
| UNION LATINO AMERICANA | LA | Latinoamericana spacing and accent |

No fuzzy rule generalizes these aliases. Future unmatched or ambiguous names stay in the output with null station metadata; their rows and totals remain in the audit.

Metro totals from source rows: LABORAL 2,469,425.4; SABADO 1,409,869; DOMINGO 778,745. The weekday source decimals retain insignificant spreadsheet representation tails in the manifest.
The artifact contains 13,015 observed bins and 5,129 null bins across 126 × 3 × 48 slots. Null does not mean closed or zero demand.

## Rebuild

```sh
python3 tools/scrape/export_april_boardings.py GTFS_ZIP BOARDING_XLSX /tmp/april-boardings
python3 tools/scrape/export_metro_observations.py GTFS_ZIP /tmp/april-boardings/boardings.csv data/mvp
```

Outputs: [metro_observations.json](../../data/mvp/metro_observations.json) and [its manifest](../../data/mvp/metro_observations_manifest.json). The manifest records source/output hashes, counts and matching totals. The 85 MB intermediate CSV stays outside git.
Each station has the original name/comuna, parent identity/coordinates, match method, 48-value arrays, and parallel source-row arrays.
Manual execution used the real files. No synthetic fixtures or automated tests were added.

## Independent-score gate

calm-condor owns the score contract. Freeze eligible station IDs, source hashes, day types, bins, split and formula before fitting. Preserve that same cell set across iterations and report excluded/missing prediction coverage separately.
Only predictions with the same boarding definition and period can be compared. A missing model value must not silently shrink the score denominator.
TB-TOB, LE-LEN and GO are already used in the demo and cannot be described as untouched holdouts.
Do not feed held-out station values, totals, normalized profiles or derived features back into the model. The existing observed-plus-site-delta model and baseline OD replay are reconstructions, not independent station predictions.
Until those conditions hold, publish observations and coverage only. Do not display an accuracy score or claim a causal validation result.
