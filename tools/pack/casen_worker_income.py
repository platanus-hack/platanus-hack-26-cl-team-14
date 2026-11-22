"""CASEN 2024 household income band weights of RM workers, by home comuna.

Source: CASEN 2024 core file (`casen_2024.RData`, SHA-256 638c6aef…) and its commune file
(`casen_2024_provincia_comuna.dta`), joined on `folio` and `id_persona`.
Workers: `region` 13, `edad` >= 15, `activ` 1, `nucleo` > 0 (as context/research/v1-income-burden.md).
Band: monthly household monetary income `ymonecorh`, November 2024 CLP, cut at 745,000 / 1,312,767 / 2,323,474.
Zero income is in band 1. Weight: `expr`. The ministry warns that direct commune estimates can be
unreliable or biased (commune is not a survey domain); the engine uses them only to draw a band.

Needs pandas and pyreadr. Output: data/mvp/casen_worker_income.csv  cut,band_1,band_2,band_3,band_4,sample_rows
"""
import sys
import pandas as pd, pyreadr

main = pyreadr.read_r(sys.argv[1])["casen_2024"]
comuna = pd.read_stata(sys.argv[2], convert_categoricals=False)[["folio", "id_persona", "comuna"]]
w = main[(main.region == 13) & (main.edad >= 15) & (main.activ == 1) & (main.nucleo > 0)]
w = w.merge(comuna, on=["folio", "id_persona"])
w["band"] = pd.cut(w.ymonecorh, [-1, 745_000, 1_312_767, 2_323_474, float("inf")], labels=[1, 2, 3, 4]).astype(int)
t = w.pivot_table(index="comuna", columns="band", values="expr", aggfunc="sum", fill_value=0).astype(int)
t.columns = [f"band_{b}" for b in t.columns]
t["sample_rows"] = w.groupby("comuna").size()
t.index = t.index.astype(int).rename("cut")
t.to_csv("data/mvp/casen_worker_income.csv")
