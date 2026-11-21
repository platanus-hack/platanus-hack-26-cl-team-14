"""Aggregate colours and body classes for passenger vehicles with Pudahuel permits."""
import csv
import re
import sys
import unicodedata
from collections import Counter, defaultdict
from pathlib import Path

SOURCE = "https://datos.gob.cl/dataset/permisos-de-circulacion-2020-2026/resource/3eb7c5f3-dbd2-40f2-9fdf-49fc42e68fd2"
PASSENGER_TYPES = {"AUTOMOVIL", "STATION WAGON", "SUV", "HATCHBACK", "SEDÁN",
                   "TODO TERRENO", "JEEP", "CABRIOLET"}
WORDS = {
    "white": "BLANCO BLANCA WHITE",
    "grey": "GRIS GRAFITO ACERO GRAY GREY",
    "silver": "PLATA PLATEADO PLATEADA PLAT PLATEDO SILVER",
    "black": "NEGRO NEGRA BLACK",
    "red": "ROJO ROJA BURDEO BURDEOS RED",
    "blue": "AZUL BLUE",
    "light_blue": "CELESTE",
    "green": "VERDE GREEN",
    "beige": "BEIGE CHAMPANA",
    "brown": "CAFE MARRON BRONCE BROWN",
    "orange": "NARANJO NARANJA ORANGE",
    "gold": "DORADO DORADA GOLD",
    "yellow": "AMARILLO AMARILLA YELLOW",
    "purple": "VIOLETA MORADO MORADA PURPURA",
    "pink": "ROSADO ROSADA ROSA",
}
FAMILY = {word: colour for colour, words in WORDS.items() for word in words.split()}


def colour_family(label):
    text = unicodedata.normalize("NFKD", label.strip().upper()).encode("ascii", "ignore").decode()
    first = re.split("[^A-Z]+", text, maxsplit=1)[0]
    return FAMILY.get(first, "unknown")


def main(path):
    vehicles = defaultdict(set)
    body_types = defaultdict(set)
    payments = 0
    with path.open(encoding="latin-1") as f:
        for row in csv.DictReader(f, delimiter=";"):
            plate = row["PLACA"].strip().upper()
            if row["ANIO_DEL_PERMISO"] != "2023" or not plate:
                continue
            body = row["TIPO_VEHICULO"]
            body_types[plate].add(body)
            if body in PASSENGER_TYPES:
                payments += 1
                vehicles[plate].add(colour_family(row["COLOR"]))

    counts = Counter()
    conflicts = 0
    for colours in vehicles.values():
        known = colours - {"unknown"}
        if len(known) > 1:
            conflicts += 1
        counts[next(iter(known)) if len(known) == 1 else "unknown"] += 1
    with Path("data/mvp/car_colours.csv").open("w", newline="") as out:
        writer = csv.writer(out, lineterminator="\n")
        writer.writerow(["cut", "colour", "share", "source"])
        for colour, count in sorted(counts.items()):
            writer.writerow(["13124", colour, f"{count / len(vehicles):.9f}", SOURCE])
    bodies = Counter()
    for plate in vehicles:
        recorded = body_types[plate]
        body = next(iter(recorded)) if len(recorded) == 1 else "unknown"
        bodies[body] += 1
    with Path("data/mvp/car_bodies.csv").open("w", newline="") as out:
        writer = csv.writer(out, lineterminator="\n")
        writer.writerow(["cut", "year", "source", "class", "share"])
        for body, count in sorted(bodies.items()):
            label = unicodedata.normalize("NFKD", body).encode("ascii", "ignore").decode()
            writer.writerow(["13124", 2023, SOURCE, label.lower().replace(" ", "_"),
                             f"{count / len(vehicles):.12f}"])
    print(f"{payments} eligible payments; {len(vehicles)} distinct plates; "
          f"{counts['unknown']} unknown including {conflicts} conflicting families")
    print(dict(counts))


if __name__ == "__main__":
    main(Path(sys.argv[1]))
