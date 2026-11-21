"""Get the MATSim Santiago open scenario v2b (10% sample, 665,201 agents). Output: data/raw/matsim-santiago/."""
import gzip, io, os, urllib.request, zipfile

URL = "https://svn.vsp.tu-berlin.de/repos/public-svn/matsim/scenarios/countries/cl/santiago/v2b.zip"
OUT = "data/raw/matsim-santiago"
os.makedirs(OUT, exist_ok=True)
path = f"{OUT}/v2b.zip"
if not os.path.exists(path):
    urllib.request.urlretrieve(URL, path)  # 126 MB

z = zipfile.ZipFile(path)
for name in z.namelist():
    if "/santiago/input/" in name and not name.endswith("/"):  # skip the 170 Java jars
        z.extract(name, OUT)
plans = gzip.open(io.BytesIO(z.read("v2b/santiago/input/randomized_expanded_plans.xml.gz")), "rt")
print(sum(line.count("<person ") for line in plans), "agents")
