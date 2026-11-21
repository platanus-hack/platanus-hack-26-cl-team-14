"""
Pull casos policiales for a comuna of the Región Metropolitana from CEAD.

Endpoint: https://cead.minsegpublica.gob.cl/wp-content/themes/gobcl-wp-master/data/get_estadisticas_delictuales.php
Method: POST. Returns one HTML table with <td> rows for País, Región, Provincia, Comuna.

Usage: python3 cead_comuna_casos.py 13101 2024 2025 > data/raw/crime-safety/cead_comuna_13101.html

seleccion=1  -> Unidad Territorial.
medida=1     -> casos policiales (frecuencia).
tipoVal=1    -> valor absoluto.
familia 1..7 -> todas las familias delictuales (7 familias, 22 grupos, 45 subgrupos).
"""
import sys
import urllib.parse
import urllib.request

URL = "https://cead.minsegpublica.gob.cl/wp-content/themes/gobcl-wp-master/data/get_estadisticas_delictuales.php"

def main():
    comuna = sys.argv[1]
    anios = sys.argv[2:]
    region = comuna[:2]
    provincia = comuna[:3]
    fields = [("medida", "1"), ("tipoVal", "1"), ("seleccion", "1"), ("descarga", "false")]
    for a in anios:
        fields.append(("anio[]", a))
    fields += [("region[]", region), ("provincia[]", provincia), ("comuna[]", comuna)]
    for f in range(1, 8):
        fields.append(("familia[]", str(f)))
    data = urllib.parse.urlencode(fields).encode()
    req = urllib.request.Request(URL, data=data, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req, timeout=30) as r:
        sys.stdout.buffer.write(r.read())

if __name__ == "__main__":
    main()
