#!/usr/bin/python3
"""Download the public files used by the visitor catalog."""

import shutil
import sys
import urllib.request
from pathlib import Path


SOURCES = {
    "domestic-total": "https://www.sernatur.cl/wp-content/uploads/2020/11/Nueva-Serie-Oficial-CDR-Viajes-totales.csv",
    "domestic-occasional": "https://www.sernatur.cl/wp-content/uploads/2020/11/Nueva-Serie-Oficial-CDR-Viajes-ocasionales.csv",
    "foreign-arrivals": "https://www.sernatur.cl/wp-content/uploads/2021/01/202608-LLEGADAS-DE-TURISTAS-EXTRANJEROS-V20261009-PUBLICACION.xlsx",
    "mop-south": "https://concesiones.mop.gob.cl/uploads/sites/4/2026/03/Diptico_SATA_2025_12.pdf",
    "mop-north": "https://concesiones.mop.gob.cl/uploads/sites/4/2026/03/Diptico_SCLA_2025_12.pdf",
}

name = sys.argv[1]
url = SOURCES[name]
output = Path("data/raw/tourism_visitors") / f"{name}.{url.rsplit('.', 1)[-1]}"
output.parent.mkdir(parents=True, exist_ok=True)
with urllib.request.urlopen(url) as response, output.open("wb") as file:
    shutil.copyfileobj(response, file)
print(output)
