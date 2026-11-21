# Census 2002: raw RM workplace aggregate

Checked 2026-11-21. Full raw cross-tab for the specified employed/habitual-home cohort.
Independent raw-artifact review passed; destination classification received a qualified pass. No benchmark or model score is implemented.
The prepared off-diagonal domain does not establish full-matrix cohort equivalence or approve a policy-response evaluation target.

- `raw_od.csv`: `origin,destination_raw,workers`; 52 habitual RM origins × 403 raw destination codes = 20,956 dense cells, totaling 2,242,438 workers.
- `query.spc`: byte-for-byte copy of the submitted readable Redatam program.
- `manifest.json`: public source endpoint, cohort, source/artifact hashes, dimensions and qualified classification scope.
- `destination-classification.csv` and `old_current_cut_crosswalk.csv`: unchanged copies of the audited response classes and derived primary-list crosswalk.
- `intercommunal_2002.csv`, `intercommunal_2024.csv`, `comparison_manifest.json`: reproducible off-diagonal preparation and per-year coverage/provenance.

Counts aggregate employed people, not daily trips. Zero cells are zeros in the returned raw table, not missing observations filled with assumed demand.
Read `origin` and `destination_raw` as code text and `workers` as an integer count.
`destination_raw` preserves source codes exactly. Do not zero-pad this column or interpret every value as a current domestic CUT.
Raw code `99999` contains 202,489 workers: 16,569 reporting another commune without its code and 185,920 ignored workplace answers.
Official response categories classify 462 foreign workers; country names remain unmapped. Domestic external destinations remain in the raw file.
No person records, source HTML, temporary server URLs, credentials or helper code are included.

## Public query reproduction

Open the [official Census 2002 CmdSet form](https://redatam-ine.ine.cl/redbin/RpWebStats.exe/CmdSet?BASE=CENSO_2002&ITEM=PROGRED&lang=ESP)
and submit the full contents of `query.spc` as the program. The public POST used these fields:

```text
MAIN=WebServerMain.inl
BASE=CENSO_2002
LANG=ESP
CODIGO=XXUSUARIOXX
ITEM=PROGRED
MODE=RUN
CMDSET=<full contents of query.spc>
```

`XXUSUARIOXX` is a literal public form value, not a credential. The program selects all enumeration geography,
filters habitual RM residence and employed codes 1/2/4, then cross-tabulates raw workplace codes against 52 explicitly recoded home categories.
The [official questionnaire, PDF page 3](https://redatam-ine.ine.cl/cuestionarios/C2002.pdf) supplies the employment categories.
Inspect the generated program and returned table; follow the result link from that response. Server-generated result links can expire.
The source request, source table and [INE viewer](https://conmutacion-laboral.ine.gob.cl/) are identified by hash in the manifest.
Exact retrieval UTC was not recorded and stays null; only the actual checked date is reported.

## What passed and what remains

Independent review matched every CSV cell to the raw source table, all row/column totals, and all 144 published Santiago 2002 viewer flows.
The viewer covers 48 origins with three selected destinations each; matching those references alone does not classify every raw code.
The temporary home recode preserves the 52 original CUT labels one-to-one, without normalization.

Classification uses a separate official `COMTRAES × LUGTRAES` aggregate for the same cohort, not numeric prefixes.
Its source HTML SHA256 is `771380f1a9186ba2e590640b6d1e7220fa5549dbebc35b2ce83718aeb819d263`.
The classification and crosswalk hashes are recorded in `manifest.json`; both CSVs retain their audited bytes.
The crosswalk joins [Decree 1439's legacy code list](https://www.bcn.cl/leychile/navegar?idNorma=168185&tipoVersion=original)
to the [official Redatam dictionary](https://redatam-ine.ine.cl/redbin/RpWebUtilities.exe/Down_Dictionary.dic?LFN=RpBases%5CCenso2002%5CBaseR%5CCENSO2002.dic).
The legal page includes later amendments despite its `original` parameter; this is not an untouched 2000 snapshot.
[INE ENE Annex 1](https://www.ine.gob.cl/docs/default-source/ocupacion-y-desocupacion/bbdd/libro-de-codigos/codigos-ene-2020.pdf) confirms all 52 RM codes are unchanged.
This derived primary-list join is not an official published concordance or proof of cadastral boundary equivalence.
Four external historical parents require current child aggregation: Iquique/Alto Hospicio, Talcahuano/Hualpén,
Santa Bárbara/Alto Biobío and Nueva Imperial/Cholchol. Historical counts cannot be allocated to those children.
Do not silently drop raw codes, normalize a discrepancy, infer foreign status from a numeric prefix, or compare gross cohort totals to a different denominator.
Employment/habitual-residence selection does not resolve INE's assumption that employed students reported work rather than study.
Raw consistency establishes source fidelity, not predictive accuracy or a causal effect of housing/jobs changes.
See the [validation synthesis](../../../context/research/santiago-redistribution-validation.md) before defining an experiment.

## Reproduce the off-diagonal preparation

From the repository root:

```sh
python3 -I tools/pack/historical_commuting.py
```

This standard-library script reads committed raw/classification files and canonical `data/mvp/communes.csv`/`work_od.csv`.
It writes `origin_cut,destination_cut,workers` over the sorted 52 × 51 RM code pairs with unequal habitual-home/work CUTs.
Each output contains 2,652 dense cells. Missing canonical 2024 pairs become zero only under the existing complete known-workplace count-aggregation assumption.
It preserves all raw categories, excludes external and diagonal mass, and does not allocate or normalize that mass elsewhere.

| Year | Source workers | Internal RM including diagonal | Removed diagonal | Included off-diagonal | Excluded outside internal RM |
| --- | ---: | ---: | ---: | ---: | ---: |
| 2002 | 2,242,438 | 2,009,186 | 646,660 | 1,362,526 | 233,252 |
| 2024 | 2,740,454 | 2,688,814 | 835,484 | 1,853,330 | 51,640 |

The 2002 outside-internal total is 30,301 domestic external + 462 foreign + 202,489 unknown-destination workers.
The 2024 source is already restricted to work-location codes 2/3 and known domestic destinations; its missing/foreign/multiple/home-work categories were excluded upstream.
**Material cohort limit:** 2002 cannot separate home workers from other same-commune workers; 2024 excludes home workers (`p44=1`).
The off-diagonal restriction avoids comparing those mixed diagonal populations, but does not establish full-matrix cohort equivalence.
Follow [INE methodology §§3.3–3.6](https://storage.googleapis.com/bktdescargascenso2024/Conmutacion_Laboral_cpv2024.pdf) for remaining employment, destination and residence assumptions.
`comparison_manifest.json` records actual hashes, dense/positive cells, included/excluded counts and source limits; unknown retrieval UTC remains null.
This prepares data only. Conditional benchmarking still needs a frozen, separately reviewed information boundary before any model score.
