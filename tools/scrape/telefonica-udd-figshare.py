"""Figshare collection 5214272: Telefonica Chile XDR mobility indices by comuna (Pappalardo, Ferres et al. 2023), CC BY 4.0."""
import json, os, urllib.request
out = "data/raw/telefonica-udd-figshare"; os.makedirs(out, exist_ok=True)
arts = json.load(urllib.request.urlopen("https://api.figshare.com/v2/collections/5214272/articles"))
for a in arts:
    for f in json.load(urllib.request.urlopen(f"https://api.figshare.com/v2/articles/{a['id']}/files")):
        urllib.request.urlretrieve(f["download_url"], f"{out}/{f['name']}")
