# Santiago redistribution: what can be validated

Date: 2026-11-21. Status: source synthesis and proposed experiments; no evaluation implemented.

There is no Census 2017 commuting target in the audited person dictionary.
The verified INE viewer exposes only 144 top-three flows from 48 Santiago origins per year, for 2002 and 2024.
It does not supply a complete historical matrix. A full 2002 cross-tab remains unverified.
Do not invent a predictive Santiago score from the current engine's baseline identity.

Retrieval update, 2026-11-21: [official Redatam CrossTab](https://redatam-ine.ine.cl/redbin/RpWebStats.exe/CrossTab?BASE=CENSO_2002&ITEM=CRUCPOB&lang=ESP) returned aggregates beyond the viewer's top three; the source audit verified employed codes `SLABORAL` 1, 2, and 4.
Santiago→Providencia is 8,025 paid workers + 278 temporarily absent workers + 47 unpaid family workers = 8,350, matching INE exactly.
The earlier provisional `SLABORAL <= 3` filter omitted the 47 unpaid family workers; the discrepancy is resolved by the corrected cohort, not normalization.
Only this control pair is verified. A complete, correctly filtered 2002 OD matrix remains unverified and is not an approved evaluation target.
Endpoint trap: a leading-parenthesis `TEXT_FILTER` submission silently lost its filter. Inspect the generated `FOR` clause before trusting an aggregate.

## Observed target coverage

| Source | Verified scope | Limit |
| --- | --- | --- |
| [Census 2017 dictionary](https://redatam-ine.ine.cl/redbin/RpWebStats.exe/Dictionary?BASE=CENSO_2017&ITEM=DICPOB&lang=esp) | 33 person variables; no workplace/study destination | Residence and migration fields are not commuting flows. |
| [INE 2002/2024 viewer](https://conmutacion-laboral.ine.gob.cl/) | 144 ranked cross-comuna pairs and 48 origins per year | Top three only; unlisted pairs are missing, not observed zeros. |
| [Census 2002 dictionary](https://redatam-ine.ine.cl/redbin/RpWebStats.exe/Dictionary?BASE=CENSO_2002&ITEM=DICPOB&lang=esp) | Work/study destination, employment, habitual-residence fields | Dictionary access does not verify a full cross-tab export. |
| [EOD 2012 description](https://www.subtrans.gob.cl/mtt-presenta-resultados-de-la-encuesta-origen-destino-de-santiago/) | 45 communes; collected July 2012–November 2013, expanded to 2012 | Official archive contents were not verified in the source audit. |

The engine covers 52 RM communes; the INE viewer covers 48 and the EOD covers 45.
Use explicit common-area masks, code crosswalks, and external-flow accounting before comparing them.
Usual-work commuters are people, not daily trip legs, Metro boardings, or secondary jobs.

## Required 2002 correction

[INE methodology, §§3.3–3.6](https://storage.googleapis.com/bktdescargascenso2024/Conmutacion_Laboral_cpv2024.pdf)
filters 2002 work/study responses to employed people, assuming employed students reported their workplace.
Use habitual residence, not enumeration geography: 2002 counted people where they slept the previous night.
Workplace nonresponse is 8.4% in 2002 versus 0.06% in 2024.
The 2024 multiple-workplace category has no unique destination and is excluded from matrix-derived measures.
These differences require harmonization; they do not make a comparison inherently impossible.
INE holds metropolitan boundaries fixed and reconciles later commune splits and regional boundaries.

## Current engine: accounting, not predictive accuracy

The [commune core](../../engine/src/main.rs) copies observed OD into its baseline and derives workers/jobs from the same cells.
At multipliers 1, it returns that input unchanged. Missing/zero pairs cannot acquire positive flow.
Coordinates do not enter allocation. One selected commune changes residential and destination weights.
Population, household, and commuter totals remain conserved; local dwelling stock changes separately.
This cannot represent general multi-commune historical change or regional growth without another mechanism.
Conservation and baseline reproduction establish accounting consistency, not accuracy of a response to changed conditions.
[FHWA §8](https://ops.fhwa.dot.gov/publications/fhwahop13015/sec8.htm) distinguishes calibration from comparison with independent observations.

## Minimal honest next step: choose an experiment

1. **Historical conditional allocation:** retrieve and verify a full earlier matrix, harmonize its universe with the later matrix,
   and freeze an earlier→later experiment before fitting. Retain an unchanged-share comparator.
   Observed target-year origin/destination totals may be supplied only as an explicit conditional allocation experiment,
   with the same information and constraints for every model. This does not validate prediction of those totals.
   General historical growth still needs a mechanism that the current one-commune, conserved-total model lacks.
2. **Hidden-cell completion:** preregister the cell mask, source revision, allowed inputs, comparators, and final evaluation boundary.
   Hide cells before deriving OD totals; do not leak withheld counts through totals, shares, or chosen multipliers.
   Add a mechanism that can create positive mass on unseen pairs; the current core cannot recover them.
   Check that absent source cells mean zero before including them as zero targets.
   This measures within-year allocation recovery, not historical change or policy effects.

These are alternatives and proposals, not implemented benchmarks. No split percentages or accuracy thresholds are approved.
A 2024→2002 comparison against the selected viewer flows is only a limited backward-transfer check.
It cannot support a full-matrix score, a forward forecast claim, or a causal intervention claim.
Do not tune against the target and then report its fit as independent validation.

## Metrics and tonight's claims

For a declared evaluation set H, report common part of commuters:
`CPC = 2 × Σ_H min(predicted, observed) / (Σ_H predicted + Σ_H observed)`, plus summed absolute error.
Report target coverage, included commuter mass, geography, missing pairs, and the diagonal/intercommunal breakdown.
Use equal inputs and constraints across models, as in [Lenormand et al. (2016)](https://arxiv.org/html/1506.04889v3).
A score on 144 selected pairs describes that selection only; it is not full-matrix accuracy.
Tonight, report observed baseline reconstruction, source/support limits, conserved totals, and explicit scenario sensitivity.
Retain `validation=accounting_only`. No scored causal or predictive Santiago result follows from the current baseline.
