"""Meta Movement Distribution Maps, latest 2026 file, Chile rows."""
import os, urllib.request
URL = "https://data.humdata.org/dataset/32167ba5-ef67-4254-8eaf-04cdb8b90c1d/resource/4974376a-1984-4731-9af1-f21ad8ae39ea/download/meta-movement-distribution-maps-october_16_2026-november_1_2026.csv"
out = "data/raw/meta-movement-distribution"; os.makedirs(out, exist_ok=True)
with urllib.request.urlopen(URL) as r, open(f"{out}/chile-2026-10-16_11-01.csv", "w") as f:
    f.write(r.readline().decode())
    f.writelines(l.decode() for l in r if b"CHL" in l)
