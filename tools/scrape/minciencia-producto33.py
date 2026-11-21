"""MinCiencia producto33: UDD/Telefonica mobility index by comuna, 2020-2021."""
import os, urllib.request
BASE = "https://raw.githubusercontent.com/MinCiencia/Datos-COVID19/master/output/producto33/"
out = "data/raw/minciencia-producto33"; os.makedirs(out, exist_ok=True)
for name in ["IndiceDeMovilidad.csv", "IndiceDeMovilidad-IM.csv", "IndiceDeMovilidad-IM_interno.csv", "IndiceDeMovilidad-IM_externo.csv"]:
    urllib.request.urlretrieve(BASE + name, f"{out}/{name}")
