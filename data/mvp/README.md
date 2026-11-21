# Data pack

## communes.csv and communes.geojson

The 52 comunas of the RM. Same builder as `zones.csv`: `python3 -I tools/pack/zones.py`.

| Column | Meaning |
| --- | --- |
| `cut` | 5-digit INE comuna code |
| `name` | INE comuna name |
| `population` | sum of `n_per` over the comuna's block-table rows (containers included) |
| `households` | sum of `n_hog` |
| `dwellings` | sum of `n_vp`, private dwellings |
| `longitude`, `latitude` | area-weighted centroid of the `Comunal_CPV24` polygon |

- Totals: population 7,400,741, households 2,599,621, dwellings 2,763,387. Providencia `13123`: 143,974 / 69,157 / 76,376.
- Check: all origins of `work_od.csv` are in this file. RM to RM workers in `work_od.csv`: 2,688,814.
- `communes.geojson`: the same 52 polygons (MultiPolygon, properties `cut`, `name`, `population`). The builder rounds coordinates to 4 decimals (about 10 m) and removes repeated points. 2.0 MB. For the browser only.

## work_od.csv and school_od.csv

The two origin-destination matrices at comuna scale. Builder: `python3 -I tools/pack/od.py`.

| Column | Meaning |
| --- | --- |
| `origin_cut` | 5-digit INE comuna code of the home. Always a comuna of the RM |
| `destination_cut` | 5-digit INE comuna code of the work place or of the school. Any comuna of Chile |
| `workers` / `students` | Persons on the pair |

Both files keep only a pair where each code is one of the 346 comunas of the MINEDUC directory. A country code, a region-only code such as `13000`, and `998`/`999` cannot pass. Codes become 5 digits: the microdata write regions 1 to 9 with 4 digits, so Valparaíso is `5101` in the raw file and `05101` here.

### work_od.csv

- Source: INE Census 2024 person microdata, through `tools/scrape/censo2024-od.py` to `data/raw/censo2024-personas/od_work_rm.csv`. Home is `comuna`, work place is `p44_lug_trab_esp`.
- Rows: 9,128. Workers: 2,740,454. Inside the RM: 2,688,814. To another region: 51,640.
- Check against the INE visualizer: Santiago (`13101`) to Las Condes (`13114`) is **28,583** workers.
- Only real commutes. `p44_lug_trab` must be 2 (works in this comuna) or 3 (works in another comuna). The excluded answers, over the 7,400,741 RM persons of the census, are:

  | Answer | Persons |
  | --- | --- |
  | Kept, codes 2 and 3 with a known comuna | 2,740,454 |
  | No work-place question (not employed) | 3,812,159 |
  | Code 1, works at home | 517,018 |
  | Code 5, no fixed place | 313,358 |
  | Region known, comuna not (`13000` and the other `NN000`) | 8,495 |
  | Code 4, works in another country | 3,793 |
  | Destination `-99` | 3,016 |
  | `p44_lug_trab` `-99` | 2,448 |

  The 8 rows sum to 7,400,741, the census RM total. `data/raw/censo2024-personas/excluded_destinations.csv` holds the same counts by code.
- Code 1 is not a trip, and its destination repeats the home comuna, so leaving it in would raise the diagonal (835,484 workers) by 62%. Code 4 puts a **country** code in the destination column. Both are out.
- Code 5 is 313,358 people, 8.7% of the 3,588,582 persons with a work-place answer, and it is not spread evenly (12.9% in Peñalolén and La Pintana against 3.2% in Melipilla). Those people travel and no matrix can say where. The model misses their trips.
- `p44` is the usual work place, not a trip on a given day. The destination stops at the comuna, so the matrix cannot put a job inside a comuna. Use `jobs.csv` for that.

