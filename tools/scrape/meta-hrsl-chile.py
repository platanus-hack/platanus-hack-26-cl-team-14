"""Meta/CIESIN High Resolution Population Density, Chile 2020, 30 m grid, CSV (lat, lon, population)."""
import os, urllib.request, zipfile
URL = "https://data.humdata.org/dataset/dd47e052-02cc-4a3f-972a-421d600b3d85/resource/bb560451-9c50-4d57-8ff3-872fa260c102/download/chl_general_2020_csv.zip"
out = "data/raw/meta-hrsl-chile"; os.makedirs(out, exist_ok=True)
urllib.request.urlretrieve(URL, f"{out}/chl_general_2020_csv.zip")
zipfile.ZipFile(f"{out}/chl_general_2020_csv.zip").extractall(out)
