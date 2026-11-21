"""Download the checked survey archives or bounded CSV samples. Standard library only."""
import argparse
import json
from pathlib import Path
import ssl
import time
import urllib.request

SOURCES = {'enut2023': ('https://www.ine.gob.cl/docs/default-source/uso-del-tiempo-tiempo-libre/bbdd/ii-enut/250403-ii-enut-bdd-csv-v2.zip?sfvrsn=ed041caf_7', None, '.zip'), 'enut2015': ('https://www.ine.gob.cl/docs/default-source/uso-del-tiempo-tiempo-libre/bbdd/documentos/base_datos_enut_csv.zip?sfvrsn=b399edf0_5', None, '.zip'), 'epf-personas': ('https://www.ine.gob.cl/docs/default-source/encuesta-de-presupuestos-familiares/bbdd/ix-epf-(octubre-2021---septiembre-2022)/base-personas-ix-epf-(formato-csv).csv?sfvrsn=8138b0c8_8&download=true', 65536, '.csv'), 'epf-gastos': ('https://www.ine.gob.cl/docs/default-source/encuesta-de-presupuestos-familiares/bbdd/ix-epf-(octubre-2021---septiembre-2022)/base-gastos-ix-epf-(formato-csv).csv?sfvrsn=2b58ad51_4&download=true', 65536, '.csv'), 'ene2026': ('https://www.ine.gob.cl/docs/default-source/ocupacion-y-desocupacion/bbdd/2026/csv/ene-2026-08-jas.csv?sfvrsn=11d31bf4_4&download=true', 65536, '.csv')}
SOURCES["eod2012"] = ("https://drive.google.com/uc?export=download&id=193675NwL6QNDs6VJFHU0mXiX5-6SdBPJ", None, ".zip")
SOURCES["santiago-rancagua2026"] = ("https://zenodo.org/records/21270302/files/Transporte-FCFM/Santiago-Rancagua-SP-dataset-v1.0.0.zip?download=1", None, ".zip")


parser = argparse.ArgumentParser()
parser.add_argument("source", choices=SOURCES)
args = parser.parse_args()
url, size, suffix = SOURCES[args.source]
# macOS Python uses the system CA file for this public INE endpoint.
context = ssl.create_default_context(cafile="/etc/ssl/cert.pem")
output = Path("data/raw/time-use-behaviour")
output.mkdir(parents=True, exist_ok=True)
headers = {"Range": f"bytes=0-{size-1}"} if size else {}
request = urllib.request.Request(url, headers=headers)
with urllib.request.urlopen(request, context=context, timeout=60) as response:
    data = response.read(size) if size else response.read()
    evidence = {"url": url, "status": response.status, "bytes": len(data),
                "content_range": response.headers.get("Content-Range"),
                "checked_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}
name = args.source + ("-sample" if size else "")
(output / (name + suffix)).write_bytes(data)
(output / (name + "-download.json")).write_text(json.dumps(evidence, indent=2))
print(json.dumps(evidence))
