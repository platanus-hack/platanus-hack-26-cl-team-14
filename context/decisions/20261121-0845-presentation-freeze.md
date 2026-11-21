# Freeze one presentation revision for rehearsal

- Status: Accepted — agent operational decision; not human approval of pitch taste or direction.
- Date: 2026-11-21
- Source: [Agreed freeze and owners](https://discord.com/channels/1573514576400752640/1573579088684253235/1573620304184483963).
- Follow-up: [Verified demo-host rehearsal](https://discord.com/channels/1573514576400752640/1573579088684253235/1573632599052779590), [12:15 eligibility cutoff](https://discord.com/channels/1573514576400752640/1573579088684253235/1573632877784989779).

## Decision

At **2026-11-21 12:30 UTC**, freeze one reviewed serving commit together with its bound `data/mvp/station_score_run.json` for rehearsal. No commit is selected by this decision, and the freeze does not start now.

Layer eligibility closes at **12:15 UTC**. A layer still under review then stays out of the freeze, even if it merges before 12:30; record it as after the freeze. Choose an eligible full reviewed SHA at the checkpoint. Use the 12:30 main head only if it contains no ineligible layer and its complete presentation passed review. Do not let a moving main ref select the revision implicitly.

## Why

Accepted layers change the score, station baseline and scenario response. Humans need stable numbers to rehearse. The code, artifact, labels and spoken numbers must describe the same revision.

## What it means for agents

- **Bright-otter:** at the checkpoint, record the eligible full reviewed commit SHA and artifact SHA-256. Clone from GitHub or fetch the GitHub `origin` in an existing checkout; never clone the local shared folder, whose local main can be stale. Run `git checkout --detach <reviewed-full-SHA>` in `/tmp/demo-freeze` and confirm `git rev-parse HEAD`. Stop only the existing servers you own on 8787 and 5173, then run `./run.sh` there; do not kill other port owners. Record the actual serving revision and command.
- **Calm-condor:** on the actual demo host, verify that artifact model/input/target/scorer hashes match the frozen checkout. Pass its `{parent_id: predicted[48]}` mapping to `tools/pack/check_score.py`; the scorer does not accept the full score artifact directly. Post the actual output and retain the development-validation and non-intervention limits.
- **Bright-otter and Calm-condor:** check the page and all four API routes on that same serving revision: `GET /api/world`, `POST /api/simulate`, `POST /api/site-scenario`, and `POST /api/station-scenario`. Use the actual UI assumptions with 298 dwellings and 4,679 jobs for both site routes. Compare with the selected revision's reviewed values, record the bound score and TB-TOB observed-bin coverage, and check zero/reset. Do not substitute the earlier 7,124-job smoke request.
- **Keen-panda:** check the frozen browser/API baseline, change and reset; confirm the score/layer and Tobalaba-only values match the response, including its baseline gap, assignment rule and missing-bin coverage. Post **1–3 key screenshots**, as AGENTS.md requires, rather than the four requested in the source message.
- **Keen-koala:** tag spoken numbers with the frozen layer and serving revision. Keep scenario assumptions and model limits visible. Human-owned `context/pitch.md` remains unchanged by this operational decision.
- **Orchestrators/reviewers:** record these checks in one release checkpoint. A pending or failed browser check is not a completed presentation release.

Later experiments can be scored, reviewed and merged on other worktrees. They must not move, rebuild from another revision, or replace the artifact in `/tmp/demo-freeze`. Keep that presentation fixed until humans authorize a change; then record a new serving commit/artifact pair and repeat the affected checks.

## Verified rehearsal, not the freeze

Calm-condor ran the procedure on the demo Mac from a clean GitHub clone at `4583b16`, with a port-shifted launcher on 8795/5195. Page and all four API routes passed. The canonical scorer reproduced L4b WAPE 0.4459 / daily 0.3383; the bound API score was 0.44593458303264427. With the UI scenario, site accounting returned −512.943 and the TB-TOB station response returned −2,781.413 over observed bins. TERM stopped both owned servers; the temporary checkout was removed. The real 8787/5173 ports were not used in that rehearsal. These are historical L4b checks, not fixed expected values for a later selected layer.
