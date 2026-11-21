"""Download the U. de Chile copy of Santiago EOD 2012 and count its CSV records."""

import csv
import io
import tarfile
from pathlib import Path
from shutil import copyfileobj
from urllib.request import urlopen

URL = "https://users.dcc.uchile.cl/~egraells/gds-data/eod2012.tgz"
OUT = Path(__file__).resolve().parents[2] / "data/raw/eod2012"


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    archive = OUT / "eod2012.tgz"
    with urlopen(URL) as response, archive.with_suffix(".part").open("wb") as output:
        copyfileobj(response, output)
    archive.with_suffix(".part").replace(archive)
    with tarfile.open(archive, "r|gz") as bundle, (OUT / "tables.tsv").open(
        "w", encoding="utf-8", newline=""
    ) as summary:
        writer = csv.writer(summary, delimiter="\t")
        writer.writerow(["table", "rows", "bytes", "columns"])
        for member in bundle:
            if not member.name.lower().endswith(".csv"):
                continue
            data = bundle.extractfile(member).read()
            try:
                text = data.decode("utf-8-sig")
            except UnicodeDecodeError:
                text = data.decode("latin-1")
            header = text.split("\n", 1)[0]
            delimiter = ";" if header.count(";") > header.count(",") else ","
            records = csv.reader(io.StringIO(text, newline=""), delimiter=delimiter)
            columns = next(records)
            count = sum(1 for record in records if any(field.strip() for field in record))
            table = member.name.removeprefix("eod2012/EOD_STGO/")
            writer.writerow([table, count, member.size, " | ".join(columns)])
            print(f"{table}\t{count}")


if __name__ == "__main__":
    main()
