# Census 2002: raw RM workplace aggregate

Checked 2026-11-21. Full raw cross-tab for the specified employed/habitual-home cohort.
Independent review passed for raw artifact fidelity only. This is not an approved or harmonized 2002/2024 evaluation target.

- `raw_od.csv`: `origin,destination_raw,workers`; 52 habitual RM origins × 403 raw destination codes = 20,956 dense cells, totaling 2,242,438 workers.
- `query.spc`: byte-for-byte copy of the submitted readable Redatam program.
- `manifest.json`: public source endpoint, cohort, source/artifact hashes, dimensions, review scope and unresolved classification.

Counts aggregate employed people, not daily trips. Zero cells are zeros in the returned raw table, not missing observations filled with assumed demand.
Read `origin` and `destination_raw` as code text and `workers` as an integer count.
`destination_raw` preserves source codes exactly. Do not zero-pad this column or interpret every value as a current domestic CUT.
Raw code `99999` contains 202,489 workers; its exact category semantics remain unverified.
Foreign and unusual codes remain unclassified. Domestic destinations outside RM remain present; they are not RM-internal flows.
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
The viewer covers 48 origins with three selected destinations each; matching those references does not classify every raw code.
The temporary home recode preserves the 52 original CUT labels one-to-one, without normalization.

Destination classification, official geographic concordance and comparable 2024 exclusions remain pending.
Do not silently drop raw codes, normalize a discrepancy, infer foreign status from a numeric prefix, or compare gross cohort totals to a different denominator.
Employment/habitual-residence selection does not resolve INE's assumption that employed students reported work rather than study.
Raw consistency establishes source fidelity, not predictive accuracy or a causal effect of housing/jobs changes.
See the [validation synthesis](../../../context/research/santiago-redistribution-validation.md) before defining an experiment.
