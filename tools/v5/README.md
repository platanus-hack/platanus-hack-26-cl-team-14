# V5 evaluations

From the repository root, run `python3 tools/v5/run_evals.py`. Cargo builds the release engine as needed. To compare two revisions without rebuilding, run the command once for each binary with `--scenario /path/to/scenario`. Use the same packed data and record both engine commits. Leave `SANTIAGO_STATION_JOB_WEIGHTS` unset to reproduce the default L9 layer.

Run `python3 tools/v5/run_evals.py --metro` to include both Metro checks. To use existing binaries, add `--scenario /path/to/scenario --population /path/to/population`. Use binaries and packed data from the same revision. The April 2026 check scores 31 held-out stations. The April 2024 Line 2 check adds four stations to the 2026 network and compares them with observed entries. [Metro inputs, scores, and limits](../../context/research/v5-final-metro-evals.md)

The command compares the saved Line 1 property change with an empty request. It reports the original four cells and the expanded eight cells. In each panel it sums absolute errors in entry changes and divides by Thursday reference entries. The no-change score uses the same cells and denominator. The command prints the five closed stations for inspection. Their expected zeros stay outside both scores. These are development scores for one event under 2026 morning routing. The engine omits the historical shuttle. [DTPM cells and source hashes](../../context/research/v4-line1-interruption.md) · [Engine assumptions](../../context/research/v4-line1-model.md)

The command also scores the two Monumental matches. April is a fitted comparison. November is a descriptive check against a Saturday reference. The Mapocho case still needs matched observations. [Monumental inputs and limits](../../context/research/monumental-egress-eval.md)

Compare the predicted sign and size of changes with findings from the same sites and periods. Use separate observations when claiming validation beyond the cases used for development. [FHWA comparison guidance](https://ops.fhwa.dot.gov/publications/fhwahop13015/sec8.htm)
