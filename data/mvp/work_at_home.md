# work_at_home.csv

Workers who work at home, by comuna of residence. INE Census 2024 person microdata, `p44_lug_trab`.

`cut,at_home,travel,at_home_share`. 52 comunas of region 13.

- at_home: `p44_lug_trab` = 1 (works in this dwelling). travel: 2 or 3 (this comuna or another), including persons with an unknown or out-of-region work comuna.
- Region: **517,018** at home, 2,751,965 travel, share **0.1582**. Highest: San Pedro 0.326, San José de Maipo 0.270, Las Condes 0.241. Lowest: Independencia 0.108, Lo Prado 0.111, Conchalí 0.120.
- The engine's 2,688,814 workers are `work_od.csv` (2,740,454, known work comuna) filtered to a work comuna in region 13. Of the 2,751,965 travel persons, 11,511 have no known work comuna and 51,640 work outside region 13. Same Census file, different universe.
- These 517,018 are already outside the engine's base. A `stay_home` scenario applies only the **additional** share of eligible travellers who change behaviour; subtracting 15.82 % again would count the base exclusion twice. The Census does not give telework days per week.
- Census "usual workplace", not a given day. EOD 2012 gives the daily view: 21.7 % of weekday workers make no trip to work (`context/research/generalist-cases.md`).

Rebuild: `python3 -I tools/pack/work_at_home.py` after downloading `personas_censo2024.zip` (`tools/scrape/censo2024-od.py`).
