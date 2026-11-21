# Freeze one presentation revision for rehearsal

- Status: Accepted — agent operational decision; not human approval of pitch taste or direction.
- Date: 2026-11-21
- Source: [Agreed freeze and owners](https://discord.com/channels/1573514576400752640/1573579088684253235/1573620304184483963).

## Decision

At **2026-11-21 12:30 UTC**, freeze one reviewed serving commit together with its bound `data/mvp/station_score_run.json` for rehearsal. No commit is selected by this decision, and the freeze does not start now.

## Why

Accepted layers change the score, station baseline and scenario response. Humans need stable numbers to rehearse. The code, artifact, labels and spoken numbers must describe the same revision.

## What it means for agents

- **Bright-otter:** at the checkpoint, record the full reviewed commit SHA and artifact SHA-256. Create a detached checkout at `/tmp/demo-freeze` and run `./run.sh` there on 8787 and 5173. Stop only the existing servers you own before replacing them; do not kill other port owners. Record the actual serving revision and command.
- **Calm-condor:** verify that artifact model/input/target/scorer hashes match the frozen checkout. Pass its `{parent_id: predicted[48]}` mapping to `tools/pack/check_score.py`; the scorer does not accept the full score artifact directly. Post the actual output and retain the development-validation and non-intervention limits.
- **Keen-panda:** check the frozen browser/API baseline, change and reset; confirm the score/layer and Tobalaba-only values match the response, including its baseline gap, assignment rule and missing-bin coverage. Post **1–3 key screenshots**, as AGENTS.md requires, rather than the four requested in the source message.
- **Keen-koala:** tag spoken numbers with the frozen layer and serving revision. Keep scenario assumptions and model limits visible. Human-owned `context/pitch.md` remains unchanged by this operational decision.
- **Orchestrators/reviewers:** record these checks in one release checkpoint. A pending or failed browser check is not a completed presentation release.

Later experiments can be scored, reviewed and merged on other worktrees. They must not move, rebuild from another revision, or replace the artifact in `/tmp/demo-freeze`. Keep that presentation fixed until humans authorize a change; then record a new serving commit/artifact pair and repeat the affected checks.
