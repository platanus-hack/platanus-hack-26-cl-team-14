# Public hospital discharges in Santiago

DEIS REM20 reports hospital discharges by establishment, functional area and month. This pack sums calendar year 2025 by the CUT of the hospital's comuna. It keeps the comunas in [the model's population table](../../mvp/communes.csv). Each row counts discharge events and reporting establishments. A discharge includes transfers to another hospital and deaths. Transfers between units within a hospital are a separate field.

Cut 5 can compare each comuna's share of public hospital discharges with its share of modeled health destinations. Compare the shares only where the hospital data has coverage. The counts cover inpatient discharges. They give no measure of outpatient demand, origin, mode, travel time, Metro entries, or trip totals.

## Coverage

The [aggregate](hospital_discharges.csv) contains 323,211 discharges from 35 reporting establishments. It covers all 12 months in 19 of the model's 52 comunas. The other 33 have no joined report. All public establishments with 2025 REM20 rows joined to the national registry. The 28 codes used are named functional areas. The labels contain no subtotal rows, and every included row has a discharge count.

The first aggregation used the publisher's [Datastore API](https://datos.gob.cl/api/3/action/datastore_search?resource_id=657cc933-eac8-4bfc-b004-c4d6dcd988a8&limit=0). It retrieved the seven relevant REM20 fields for 2025 and public establishments, and the establishment and comuna codes from the registry. The [independent rebuild on Francisco’s laptop](https://discord.com/channels/1573514576400752640/1573688249287708682/1573734787066241025) produced a byte-for-byte match from both complete CSV snapshots.

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
cmp /tmp/hospital_discharges.csv data/validation/hospital_discharges_2025/hospital_discharges.csv
```

The [DEIS dictionary](https://datos.gob.cl/dataset/a756323f-85ba-4759-87dc-f5d5e63868cc/resource/6c7c8a9d-204d-4947-9c8e-82a893fc3f2e/download/diccionario-rem20.xlsx) defines the source fields. The pack selects year 2025 and `TIPO_PERTENENCIA=1` for the National Health Services System. It joins `CODIGO_ESTABLECIMIENTO` to the later registry's `EstablecimientoCodigo` and uses `ComunaCodigo` as `cut`. It adds `NUMERO_EGRESOS` across functional areas. It retains reported zeros and skips blank counts. An absent comuna-month means the joined source supplied no count. The script prints unmatched establishment codes and the area codes used so the reviewer can check for missing sites and subtotal rows.

| File | Expected SHA-256 |
| --- | --- |
| REM20 snapshot | `22e795cc7432528cc2464fa186a4245ff212f2b839956f1cbccd7f00b8744420` |
| Establishment registry snapshot | `febff55e3612fb60076df9279f1090240cd02a39e9c7b79b822119dfdef84c8d` |
| Aggregate | `bd0e470f51ad6ac6fd45d0f215d108fec2d51855385889fd34e77d887cd9d31d` |
