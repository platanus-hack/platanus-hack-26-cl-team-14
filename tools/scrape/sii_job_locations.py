#!/usr/bin/env python3
"""Download SII company worker bands and registered addresses."""

import pathlib
import subprocess
import time


FILES = {
    "workers-2020-2024.zip": "https://www.sii.cl/estadisticas/nominas/PUB_EMPRESAS_PJ_2020_A_2024.zip",
    "addresses.zip": "https://www.sii.cl/estadisticas/nominas/PUB_NOM_DIRECCIONES.zip",
    "workers-by-commune.xlsb": "https://www.sii.cl/sobre_el_sii/empresas/PUB_COMU_TRTRAB.xlsb",
}
OUT = pathlib.Path("data/raw/sii-job-locations")
OUT.mkdir(parents=True, exist_ok=True)

for index, (filename, url) in enumerate(FILES.items()):
    if index:
        time.sleep(1)
    subprocess.run(
        ["curl", "-fL", url, "-o", str(OUT / filename)], check=True
    )
