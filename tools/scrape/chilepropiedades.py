# Sale and rent listings from chilepropiedades.cl (robots.txt allows /propiedades and /ver-publicacion, Crawl-delay 2).
# Coordinates: the site gives only the street address, so we geocode it with Nominatim (1 request per second).
# Usage: python3 tools/scrape/chilepropiedades.py  -> data/raw/chilepropiedades/listings.csv
import csv, json, os, re, time, urllib.parse, urllib.request

UA = "team14-hackathon-research/0.1 (urban data catalog)"
OUT = "data/raw/chilepropiedades"
COMUNAS = ["santiago", "providencia", "nunoa", "las-condes", "la-florida", "maipu", "puente-alto", "estacion-central"]
OPS = ["venta", "arriendo-mensual"]
PAGES = 2  # 10 listings per page


def get(url):
    time.sleep(2)
    return urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent": UA}), timeout=30).read().decode("utf-8", "ignore")


def geocode(address, comuna):
    time.sleep(1.1)
    q = urllib.parse.urlencode({"q": f"{address}, {comuna}, Chile", "format": "json", "limit": 1, "countrycodes": "cl",
                                "viewbox": "-70.95,-33.25,-70.40,-33.75", "bounded": 1})  # Gran Santiago box
    r = json.load(urllib.request.urlopen(urllib.request.Request("https://nominatim.openstreetmap.org/search?" + q, headers={"User-Agent": UA}), timeout=30))
    return (r[0]["lat"], r[0]["lon"]) if r else ("", "")


os.makedirs(OUT, exist_ok=True)
urls = []
for op in OPS:
    for c in COMUNAS:
        for p in range(PAGES):
            html = get(f"https://chilepropiedades.cl/propiedades/{op}/departamento/{c}/{p}")
            urls += sorted(set("https://chilepropiedades.cl" + u for u in re.findall(r'href="(/ver-publicacion/[^"]+)"', html)))
print(len(urls), "detail urls")

cols = ["id", "operation", "type", "comuna", "address", "price", "currency", "area_m2", "bedrooms", "bathrooms", "year_built", "date_posted", "lat", "lon", "url"]
with open(f"{OUT}/listings.csv", "w", newline="") as f:
    w = csv.writer(f)
    w.writerow(cols)
    for u in dict.fromkeys(urls):
        ld = json.loads(re.search(r'<script type="application/ld\+json">(.*?)</script>', get(u), re.S).group(1))
        g = next(x for x in ld["@graph"] if x["@type"] == "RealEstateListing")
        a, extra = g["about"], {p["name"]: p["value"] for p in g["about"].get("additionalProperty", [])}
        addr = a["address"]
        lat, lon = geocode(addr["streetAddress"], addr["addressLocality"])
        price, cur = g["offers"]["price"], g["offers"]["priceCurrency"]
        cur = "CLF" if float(price) < 100000 else cur  # site mislabels some UF prices as CLP
        w.writerow([u.rsplit("/", 1)[1], u.split("/")[4], extra.get("Tipo de propiedad"), addr["addressLocality"], addr["streetAddress"],
                    price, cur, a.get("floorSize", {}).get("value"),
                    extra.get("Dormitorios"), extra.get("Baños"), extra.get("Año construcción"), g.get("datePosted"), lat, lon, u])
        f.flush()
