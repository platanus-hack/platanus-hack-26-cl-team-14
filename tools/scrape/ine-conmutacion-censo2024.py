"""INE Conmutacion laboral CPV2024: comuna-to-comuna work flows, 2002 and 2024.

The visualizer embeds all its tables in one `DATA={...}` literal.
Output: data/raw/ine-conmutacion/od_<area>_<year>.csv and saldo_<area>_<year>.csv
"""
import csv, json, pathlib, urllib.request

URL = "https://conmutacion-laboral.ine.gob.cl/"
OUT = pathlib.Path("data/raw/ine-conmutacion")


def load_data():
    html = urllib.request.urlopen(URL, timeout=180).read().decode("utf-8", "replace")
    i = html.index("DATA=") + 5
    depth = 0
    for j in range(i, len(html)):
        if html[j] == "{":
            depth += 1
        elif html[j] == "}":
            depth -= 1
            if depth == 0:
                break
    return json.loads(html[i:j + 1])


def write(name, rows, cols):
    path = OUT / name
    with path.open("w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=cols)
        w.writeheader()
        w.writerows(rows)
    print(path, len(rows), "rows")


OUT.mkdir(parents=True, exist_ok=True)
data = load_data()
for area, years in data["destino"].items():
    slug = area.lower().replace(" ", "-")
    for year, rows in years.items():
        write(f"od_{slug}_{year}.csv", rows, ["o", "on", "d", "dn", "frecuencia", "orden"])
for area, years in data["relativas"].items():
    slug = area.lower().replace(" ", "-")
    for year, rows in years.items():
        write(f"saldo_{slug}_{year}.csv", rows,
              ["cut", "nombre", "entrada", "salida", "saldo", "categoria", "ocupados"])
