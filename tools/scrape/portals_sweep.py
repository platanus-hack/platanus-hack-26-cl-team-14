#!/usr/bin/env python3
"""List Santiago datasets from the IDE Chile and Las Condes catalogs."""

import csv
import html
import http.cookiejar
import json
import re
import ssl
import time
import unicodedata
import urllib.parse
import urllib.request
from pathlib import Path


OUTPUT_DIR = Path("data/raw/portals-sweep")
USER_AGENT = "team14-urban-data-catalog/1.0"
RM_TERMS = (
    "region metropolitana", "santiago", "alhue", "buin", "calera de tango",
    "cerrillos", "cerro navia", "colina", "conchali", "curacavi", "el bosque",
    "el monte", "estacion central", "huechuraba", "independencia", "isla de maipo",
    "la cisterna", "la florida", "la granja", "la pintana", "la reina", "lampa",
    "las condes", "lo barnechea", "lo espejo", "lo prado", "macul", "maipu",
    "maria pinto", "melipilla", "nunoa", "padre hurtado", "paine", "pedro aguirre cerda",
    "penaflor", "penalolen", "pirque", "providencia", "pudahuel", "puente alto",
    "quilicura", "quinta normal", "recoleta", "renca", "san bernardo", "san joaquin",
    "san jose de maipo", "san miguel", "san pedro", "san ramon", "talagante", "tiltil",
    "vitacura", "chile", "nacional",
)
ARCGIS_DATA_TYPES = {
    "CSV", "Feature Service", "GeoJson", "Image Service", "KML",
    "Shapefile", "Vector Tile Service",
}


def normalized(value):
    value = unicodedata.normalize("NFD", value or "")
    return "".join(c for c in value if unicodedata.category(c) != "Mn").lower()


def year_from(text):
    years = re.findall(r"\b(?:19|20)\d{2}\b", text or "")
    return years[-1] if years else ""


class Client:
    def __init__(self):
        jar = http.cookiejar.CookieJar()
        tls = ssl.create_default_context(cafile="/etc/ssl/cert.pem")
        self.opener = urllib.request.build_opener(
            urllib.request.HTTPCookieProcessor(jar),
            urllib.request.HTTPSHandler(context=tls),
        )
        self.last_request = {}

    def request(self, url, *, data=None, headers=None, method=None):
        host = urllib.parse.urlsplit(url).netloc
        wait = 1.0 - (time.monotonic() - self.last_request.get(host, 0))
        if wait > 0:
            time.sleep(wait)
        request = urllib.request.Request(
            url,
            data=data,
            headers={"User-Agent": USER_AGENT, **(headers or {})},
            method=method,
        )
        try:
            return self.opener.open(request, timeout=60)
        finally:
            self.last_request[host] = time.monotonic()

    def text(self, url, **kwargs):
        with self.request(url, **kwargs) as response:
            return response.read().decode("utf-8")

    def json(self, url, **kwargs):
        return json.loads(self.text(url, **kwargs))

    def alive(self, url):
        try:
            with self.request(url, method="HEAD") as response:
                return response.status < 400
        except Exception:
            return False


def las_condes(client):
    page = client.text("https://datos.lascondes.cl/datos")
    token = re.search(r'name="csrf-token" content="([^"]+)', page).group(1)
    headers = {"X-Requested-With": "XMLHttpRequest", "X-CSRF-TOKEN": token}
    inventory = client.json(
        "https://datos.lascondes.cl/obtener/inventario", data=b"", headers=headers
    )["inventario"]
    layers = client.json(
        "https://datos.lascondes.cl/obtener/capas", data=b"", headers=headers
    )["capas"]
    rows = []
    dead = 0
    for layer in layers:
        urls = layer.get("urls_opendata") or {}
        format_name = next((name for name in ("geojson", "csv", "shp") if urls.get(name)), "")
        url = "https://datos.lascondes.cl/proxy/s3/" + urls[format_name]
        alive = client.alive(url)
        dead += not alive
        rows.append({
            "portal": "Las Condes Open Data",
            "title": layer["nombre"],
            "publisher": layer.get("unidad_municipal") or "Municipalidad de Las Condes",
            "format": format_name.upper(),
            "year": year_from(layer.get("generated_at", "")),
            "url": url,
            "link_status": "alive" if alive else "dead",
        })
    return rows, {"total": len(inventory), "published": len(layers), "rm": len(rows), "dead": dead}


def ide_records(page):
    found = {}
    link_pattern = re.compile(
        r'<a href="(https://geoportal\.cl/geoportal/catalog/(\d+)/[^"]+)">\s*([^<]+?)\s*</a>'
    )
    for match in link_pattern.finditer(page):
        url, record_id, title = match.groups()
        if record_id in found:
            continue
        tail = page[match.end():match.end() + 2600]
        spans = [html.unescape(re.sub("<[^>]+>", "", value)).strip()
                 for value in re.findall(r"<span[^>]*>(.*?)</span>", tail, re.S)]
        description_match = re.search(r'<div class="my-2[^>]*>(.*?)</div>', tail, re.S)
        description = html.unescape(re.sub("<[^>]+>", "", description_match.group(1))).strip() if description_match else ""
        badges = [value for value in spans if value in {"Servicio", "Capa", "Documento", "Aplicación"}]
        dates = [value for value in spans if re.search(r"\b\d{2}-\d{2}-\d{4}\b", value)]
        publisher = next((value for value in spans if value not in badges and value not in dates and not value.startswith("Actualizado")), "")
        found[record_id] = {
            "portal": "IDE Chile Geoportal",
            "title": html.unescape(title).strip(),
            "publisher": publisher,
            "format": "/".join(badges) or "catalog record",
            "year": year_from(title + " " + description),
            "url": url,
            "link_status": "alive",
            "_search": " ".join((title, publisher, description)),
        }
    return list(found.values())


def ide_chile(client):
    base = "https://geoportal.cl/geoportal/catalog?action=search"
    first = client.text(base)
    total = int(re.search(r'<span class="font-medium">de </span>\s*(\d+)', first).group(1))
    pages = (total + 4) // 5
    all_rows = ide_records(first)
    for page_number in range(2, pages + 1):
        all_rows.extend(ide_records(client.text(base + f"&page={page_number}")))
    rows = []
    for row in all_rows:
        search = normalized(row.pop("_search"))
        if any(term in search for term in RM_TERMS):
            rows.append(row)
    return rows, {"total": total, "published": total, "rm": len(rows), "dead": 0}


def arcgis_portal(client, name, search_query, *, filter_rm=False):
    items = []
    start = 1
    total = 0
    while start != -1:
        params = urllib.parse.urlencode({
            "f": "json", "num": 100, "start": start, "q": search_query,
        })
        result = client.json("https://www.arcgis.com/sharing/rest/search?" + params)
        total = result["total"]
        items.extend(result["results"])
        start = result["nextStart"]
    datasets = [item for item in items if item["type"] in ARCGIS_DATA_TYPES]
    if filter_rm:
        datasets = [item for item in datasets if any(
            term in normalized(" ".join((item["title"], item.get("snippet", ""), " ".join(item.get("tags", [])))))
            for term in RM_TERMS
        )]
    rows = []
    for item in datasets:
        url = f"https://www.arcgis.com/home/item.html?id={item['id']}"
        rows.append({
            "portal": name,
            "title": item["title"],
            "publisher": item["owner"],
            "format": item["type"],
            "year": year_from(item["title"]),
            "url": url,
            "link_status": "alive",
        })
    return rows, {"total": total, "published": len(datasets), "rm": len(rows), "dead": 0}


def main():
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    client = Client()
    las_condes_rows, las_condes_stats = las_condes(client)
    gore_rows, gore_stats = arcgis_portal(
        client, "Gobierno de Santiago ArcGIS", "owner:UgitGore_01"
    )
    santiago_rows, santiago_stats = arcgis_portal(
        client, "Municipalidad de Santiago ArcGIS", "owner:IMS_Observatorio"
    )
    ocuc_rows, ocuc_stats = arcgis_portal(
        client,
        "Observatorio de Ciudades UC ArcGIS",
        "group:46da166267944c039ec26987be335330",
        filter_rm=True,
    )
    ide_rows, ide_stats = ide_chile(client)
    rows = sorted(
        las_condes_rows + gore_rows + santiago_rows + ocuc_rows + ide_rows,
        key=lambda row: (row["portal"], row["title"]),
    )
    with (OUTPUT_DIR / "datasets.csv").open("w", newline="", encoding="utf-8") as output:
        writer = csv.DictWriter(output, fieldnames=(
            "portal", "title", "publisher", "format", "year", "url", "link_status"
        ))
        writer.writeheader()
        writer.writerows(rows)
    stats = {
        "IDE Chile Geoportal": ide_stats,
        "Las Condes Open Data": las_condes_stats,
        "Gobierno de Santiago ArcGIS": gore_stats,
        "Municipalidad de Santiago ArcGIS": santiago_stats,
        "Observatorio de Ciudades UC ArcGIS": ocuc_stats,
        "notes": {
            "datos.gob.cl": "Not crawled: robots.txt disallows /api/ and requires a 10-second crawl delay.",
            "IDE Chile CSW": "The documented public CSW redirects to /login. The script uses the public HTML catalog.",
            "Providencia": "datos.providencia.cl fails DNS.",
            "Nunoa": "datos.nunoa.cl fails DNS.",
            "Puente Alto": "The Junar catalog API requires an auth key.",
            "Maipu": "The Google Sites catalog has no public catalog API.",
            "INE": "The official page lists old ArcGIS Hub links, but the Hub now returns 403 and its RM item is inaccessible.",
            "dead_definition": "Las Condes checks one direct download per layer. ArcGIS and IDE count live catalog links; linked services were not checked globally.",
        },
    }
    (OUTPUT_DIR / "stats.json").write_text(json.dumps(stats, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(stats, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
