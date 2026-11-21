# work_at_home.csv

Workers who work at home, by comuna of residence. INE Census 2024 person microdata, `p44_lug_trab`.

`cut,at_home,travel,at_home_share`. 52 comunas of region 13.

- at_home: `p44_lug_trab` = 1 (works in this dwelling). travel: 2 or 3 (this comuna or another), including persons with an unknown or out-of-region work comuna.
- Region: **517,018** at home, 2,751,965 travel, share **0.1582**. Highest: San Pedro 0.326, San José de Maipo 0.270, Las Condes 0.241. Lowest: Independencia 0.108, Lo Prado 0.111, Conchalí 0.120.
- The engine's workers (`work_od.csv`, 2.69 M) are the travel group with a known RM work comuna, so this table is the 2024 base for a `stay_home` change: the engine's base already has these persons at home.
- Census "usual workplace", not a given day. EOD 2012 gives the daily view: 21.7 % of weekday workers make no trip to work (`context/research/generalist-cases.md`).

Rebuild: `python3 -I tools/pack/work_at_home.py` after downloading `personas_censo2024.zip` (`tools/scrape/censo2024-od.py`).
