"""Meta Movement Range Maps 2020-03 to 2022-05, Chile rows (GADM2 = comuna-like polygons)."""
import os, urllib.request, zipfile, io
out = "data/raw/meta-movement-range"; os.makedirs(out, exist_ok=True)
for url in ["https://data.humdata.org/dataset/c3429f0e-651b-4788-bb2f-4adbf222c90e/resource/3d77ce5c-ab6d-4864-b8a2-c8bafffac4f3/download/movement-range-data-2020-03-01-2020-12-31.zip",
            "https://data.humdata.org/dataset/c3429f0e-651b-4788-bb2f-4adbf222c90e/resource/55a51014-0d27-49ae-bf92-c82a570c2c6c/download/movement-range-data-2022-05-22.zip"]:
    z = zipfile.ZipFile(io.BytesIO(urllib.request.urlopen(url).read()))
    for n in z.namelist():
        if n.endswith(".txt") and "movement" in n:
            with z.open(n) as r, open(f"{out}/chile-{n}", "w") as f:
                f.write(r.readline().decode())
                f.writelines(l.decode() for l in r if b"\tCHL\t" in l)
