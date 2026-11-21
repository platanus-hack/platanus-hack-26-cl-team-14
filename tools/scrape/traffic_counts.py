"""Download the two public traffic-source files used by the catalog."""

from pathlib import Path
from urllib.request import urlopen


OUT = Path("data/raw")
SOURCES = {
    "ine-permisos/2024.xlsx": "https://www.ine.gob.cl/docs/default-source/parque-de-vehiculos/cuadros-estadisticos/resultados/2024.xlsx?sfvrsn=5801b9b3_4",
    "uoct-intersections/2023.pdf": "https://www.transporteinforma.cl/content/uploads/2023/10/Intersecciones-de-Semaforos-conectados-a-Sistema-UOCT-RM.pdf",
}

for relative_path, url in SOURCES.items():
    destination = OUT / relative_path
    destination.parent.mkdir(parents=True, exist_ok=True)
    with urlopen(url) as response:
        destination.write_bytes(response.read())
    print(destination, destination.stat().st_size)
