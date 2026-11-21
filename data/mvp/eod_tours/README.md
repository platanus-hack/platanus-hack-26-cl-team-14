# Santiago weekday tour profiles

These aggregates describe the EOD 2012 survey area. Households gave their diaries between June 2012 and November 2013. The builder reads the local Universidad de Chile survey mirror. Its archive SHA-256 is `6dce8fb939fafc7b236654ecd511caec9069773fede6692abb579a9e7360d726`. Equivalence with the official SECTRA archive remains unverified. [Survey source and inventory](../../../context/research/sources/verify-transport.md)

| File | Use |
|---|---|
| `rates.csv` | Multiply the eligible people in a home sector by the retained tours per eligible person. The errand denominator includes every resident in that sector. |
| `od.csv` | Within purpose and home sector, divide each retained destination weight by the row total across destinations. This is the errand destination distribution. School OD comes from the school pack. |
| `time_mode.csv` | Sample a regional joint time and mode profile for the purpose. Each row carries both starts, both ends, both aggregate mode codes and its tour weight. |
| `higher_ed_seed.csv` | Seed the higher-education sector allocation with expanded outbound trips to study. This uses eligible home origins even if the diary lacks a direct return. |
| `metro_coverage.csv` | Multiply the April training arrival target for the broad purpose by `coverage_share`. The file contains survey counts only. |

Use `home_sector` and `destination_sector` with the [point-to-sector tables](../eod_sectors/README.md). Survey sectors run from 1 through 7. Mode codes come from the survey's `Tablas_parametros/ModoAgregado.csv`. Half-hour bin 0 starts at 00:00, bin 1 at 00:30, and bin 48 at 00:00 the following day. All times use the local clock. Sample the timing tuple together. The time/mode profile pools all home and destination sectors within each purpose.

## Eligibility and expansion

37,326 people have a positive `Factor_LaboralNormal`. Their weights sum to 6,651,734.841 residents. They include 6,050 people who reported zero trips. For preschool, school and higher education use `DondeEstudia` 1–3, 4 and 6–8 respectively. For shopping, health and public errands use all residents. Do not add denominators across purposes.

The outbound purpose is 3 for study, 11 for shopping, 5 for health and 12 with `ActividadDestino` 7 for a public errand. **All 2,862 normal-weekday purpose-12 trips have a blank destination activity.** The public-errand rows contribute zero to this candidate because the survey cannot identify the required trips. The regional trip rate for actual public errands is unknown.

The first trip of the diary can use the reported household zone as a proxy for home. For any later departure, the previous trip must be consecutive, have return-home purpose 7, end in the household zone, and arrive before or at the new departure. The higher-education seed uses the same home check. The outbound trip starts in the household zone. The next trip in the same diary must have purpose 7, start in the activity zone and end in the household zone. The two sector codes must also reverse. Both trips need positive normal-weekday factors. The builder uses diary order to unwrap the clock across midnight. It drops missing times, survey time-error code 5, overlapping trip intervals and tours lasting 24 hours or longer. A zone match can cover many addresses.

The person factor alone weights residents. Each tour uses the person factor times the **outbound** trip factor once. Of 12,859 complete pairs, 5,510 have different outbound and return trip factors. The paired tour carries one return for each outbound movement. Separate return-leg expansion would produce a different total.

## Coverage and suppressed groups

Every published OD and joint time/mode cell must contain at least two distinct diaries. The builder drops tours in singleton cells and repeats the check on both margins until all retained cells qualify. A removed tour contributes to neither rates nor OD, timing or Metro coverage. For the independent higher-education destination seed, singleton sector pairs account for 647.547 expanded outbound trips. The registered seed smoothing takes place downstream.

| Purpose | Eligible outbound trips | Complete tours | Retained tours |
|---|---:|---:|---:|
| Preschool | 246,042 | 216,439 | 127,665 |
| School | 952,243 | 854,683 | 653,562 |
| Higher education | 341,325 | 247,031 | 43,119 |
| Shopping | 1,636,479 | 1,248,082 | 1,021,708 |
| Health | 356,120 | 263,848 | 94,527 |
| Public errands | 0 | 0 | 0 |

Values are expanded normal-weekday counts, rounded here to whole tours. Of 3,532,209 eligible outward trips, 469,450 lacked the required home-origin or sector/zone match. A further 228,601 lacked a matching consecutive return. Time checks removed 4,075. The original 12,859 complete diary pairs represent 2,830,083 weighted tours. The 8,622 retained pairs represent 1,940,581 weighted tours. Requiring two diaries removes **31.4%** of the complete weighted mass and **82.5%** for higher education. Rates and time distributions apply only to these retained tours.

## Metro coverage

ESTUDIO uses EOD purposes 3 and 4. OTROS uses 5, 6 and 8–14. For each broad purpose, sort the stages in trip order, omit walking (stage mode 8) and cycling (9), and check whether the last remaining stage uses Metro (4). A missing or invalid stage sequence or unknown final mode cannot contribute to this denominator. One normal-weekday diary in these broad purposes lacks a complete stage sequence. Its individual weight is withheld.

The coverage numerator counts retained outward trips with aggregate mode 4 and a verified final motorized Metro stage. Both counts use the expanded outbound weight. The intermediate columns show how much final-Metro demand survives the Metro-only mode filter, the covered purpose filter and the complete-tour filter before singleton suppression.

| Broad purpose | Final motorized stage Metro | Retained outward Metro-only | Share retained | Share outside candidate |
|---|---:|---:|---:|---:|
| ESTUDIO | 226,300.794 | 12,577.084 | 5.5577% | 94.4423% |
| OTROS | 260,467.935 | 3,581.116 | 1.3749% | 98.6251% |

These shares precede station reachability and any training fit. EOD records a 2012 travel pattern. Home-based direct returns and the two-diary rule particularly reduce coverage for mixed trips and infrequent schedules. For the remaining steps use the [registered rule](../../../context/research/student-other-trip-rule.md).

## Rebuild

From the repository root, point the builder at the four local survey tables in `EOD_STGO`:

```sh
python3 tools/pack/eod_tours.py --survey data/raw/eod2012/x/eod2012/EOD_STGO
```
