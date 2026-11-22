# Merge local junction FIFO and review further changes separately

- **Status** Accepted
- **Date** 2026-11-22
- **Source** [Bruno's direction](https://discord.com/channels/1573514576400752640/1573915515787935856/1574022860018683925)

## Decision

Merge the local FIFO feature. Use a separate pull request for the remaining improvements.

## Evidence

Bruno approved the moving clip from `4057897`. The guarded merge used `0b9e5a7`, which also fixes the six-car spacing and four-car arrival fixtures. The native load still records a 365 s cumulative delay at p95 and a 4,998.7 s maximum single stop.

## Scope

Keep the approved clip and the merged source revision in the verification record. Review and measure the receiving-lane change on its own head.
