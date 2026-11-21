"""Apple Mobility Trends (2020-01 to 2022-04), Chile and Santiago rows, from the ActiveConclusion mirror."""
import os, urllib.request
URL = "https://raw.githubusercontent.com/ActiveConclusion/COVID19_mobility/master/apple_reports/applemobilitytrends.csv"
out = "data/raw/apple-mobility"; os.makedirs(out, exist_ok=True)
with urllib.request.urlopen(URL) as r, open(f"{out}/chile.csv", "w") as f:
    f.write(r.readline().decode())
    for line in r:
        if b"Chile" in line:
            f.write(line.decode())
