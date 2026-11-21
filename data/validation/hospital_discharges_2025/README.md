# Public hospital discharges in Santiago

DEIS REM20 reports hospital discharges by establishment, functional area and month. This pack sums calendar year 2025 by the CUT of the hospital's comuna. It keeps the comunas in [the model's population table](../../mvp/communes.csv). Each row counts discharge events and reporting establishments. A discharge includes transfers to another hospital and deaths. Transfers between units within a hospital are a separate field.

Cut 5 can compare each comuna's share of public hospital discharges with its share of modeled health destinations. Compare the shares only where the hospital data has coverage. The counts cover inpatient discharges. They give no measure of outpatient demand, origin, mode, travel time, Metro entries, or trip totals.

## Rebuild

Download these source snapshots outside the repo. Both source catalogs declare [CC0](https://creativecommons.org/publicdomain/zero/1.0/).

| Input | Snapshot | Publisher |
| --- | --- | --- |
| [indicadores_rem20_20261027.csv](https://datos.gob.cl/dataset/a756323f-85ba-4759-87dc-f5d5e63868cc/resource/657cc933-eac8-4bfc-b004-c4d6dcd988a8/download/indicadores_rem20_20261027.csv) | 2026-10-27 | [DEIS REM20](https://datos.gob.cl/api/3/action/package_show?id=a756323f-85ba-4759-87dc-f5d5e63868cc) |
| [establecimientos_20261117.csv](https://datos.gob.cl/dataset/3bf4cf7c-f638-4735-9a01-f65faae4beca/resource/2c44d782-3365-44e3-aefb-2c8b8363a1bc/download/establecimientos_20261117.csv) | 2026-11-17 | [DEIS establishment registry](https://datos.gob.cl/api/3/action/package_show?id=3bf4cf7c-f638-4735-9a01-f65faae4beca) |

Run from the repository root with Python 3. Replace the two input paths with your local copies.

```sh
python3 data/validation/hospital_discharges_2025/pack.py \
  /tmp/indicadores_rem20_20261027.csv /tmp/establecimientos_20261117.csv \
  /tmp/hospital_discharges.csv
shasum -a 256 /tmp/indicadores_rem20_20261027.csv \
  /tmp/establecimientos_20261117.csv /tmp/hospital_discharges.csv
```

The [DEIS dictionary](https://datos.gob.cl/dataset/a756323f-85ba-4759-87dc-f5d5e63868cc/resource/6c7c8a9d-204d-4947-9c8e-82a893fc3f2e/download/diccionario-rem20.xlsx) defines the source fields. The pack selects year 2025 and `TIPO_PERTENENCIA=1` for the National Health Services System. It joins `CODIGO_ESTABLECIMIENTO` to the later registry's `EstablecimientoCodigo` and uses `ComunaCodigo` as `cut`. It adds `NUMERO_EGRESOS` across functional areas. It retains reported zeros and skips blank counts. An absent comuna-month means the joined source supplied no count. The script prints unmatched establishment codes and the area codes used so the reviewer can check for missing sites and subtotal rows.

The first external rebuild will pin the input and output checksums, confirm the area codes, and record coverage.
