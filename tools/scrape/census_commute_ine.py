#!/usr/bin/env python3
"""Download INE's 2002/2024 commuting dashboard data for the Santiago metro."""

import json
import pathlib
import subprocess


URL = "https://conmutacion-laboral.ine.gob.cl/"
OUT = pathlib.Path("data/raw/census-commute-ine")

page = subprocess.run(
    ["curl", "-fsSL", URL], check=True, capture_output=True, text=True
).stdout
start = page.index("const DATA=") + len("const DATA=")
end = page.index("\n\n(function(){", start)
data = json.loads(page[start:end].strip().removesuffix(";"))

OUT.mkdir(parents=True, exist_ok=True)
for name in ("desajuste", "destino"):
    rows = data[name]["Santiago"]["2024"]
    (OUT / f"{name}-santiago-2024.json").write_text(
        json.dumps(rows, ensure_ascii=False, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )
