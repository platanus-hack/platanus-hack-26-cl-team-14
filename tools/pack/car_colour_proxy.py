"""Pack the regional colour proxy from the frozen Chileautos listing sample."""
import csv
import sys
from collections import Counter
from pathlib import Path

from car_colours import colour_family

SOURCE = "https://github.com/luminati-io/Chileautos-Cars-dataset-samples/tree/0ddf2e01af5ce5b793494bf44491e43dae38d100"
CAR_CATEGORIES = {"autos-vehículo", "autos, camionetas y 4x4", "sedán", "sedan", "suv"}

counts = Counter()
with Path(sys.argv[1]).open(encoding="utf-8-sig", newline="") as file:
    for row in csv.DictReader(file):
        if (row["location"] == "Metropolitana de Santiago" and row["Condicion"] == "Usado"
                and row["Category"].casefold() in CAR_CATEGORIES):
            label = row["Color"].strip()
            family = "red" if label.casefold() in {"bordo", "borgoña"} else colour_family(label)
            counts[family] += 1

with Path("data/mvp/car_colour_proxy.csv").open("w", newline="") as file:
    writer = csv.writer(file, lineterminator="\n")
    writer.writerow(["cut", "colour", "share", "source"])
    for family, count in sorted(counts.items()):
        writer.writerow(["13", family, f"{count / counts.total():.9f}", SOURCE])
print(counts.total(), "listings", dict(sorted(counts.items())))
