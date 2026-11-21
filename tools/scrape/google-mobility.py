"""Google COVID-19 Community Mobility Reports, Chile rows only (2020-02 to 2022-10)."""
import os, urllib.request
URL = "https://www.gstatic.com/covid19/mobility/Global_Mobility_Report.csv"
out = "data/raw/google-mobility"; os.makedirs(out, exist_ok=True)
with urllib.request.urlopen(URL) as r, open(f"{out}/chile.csv", "w") as f:
    f.write(r.readline().decode())
    for line in r:
        if line.startswith(b"CL,"):
            f.write(line.decode())
