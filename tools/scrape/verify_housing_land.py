"""Save bounded public source samples and request evidence. Run from the repo root."""
import json
import time
import urllib.error
import urllib.parse
import urllib.request
import urllib.robotparser
from pathlib import Path

OUT = Path('data/raw/verify-housing-land')
OUT.mkdir(parents=True, exist_ok=True)
AGENT = 'housing-land-research/1.0'
last = {}
robots = {}
records = []


def request(url):
    host = urllib.parse.urlsplit(url).netloc
    time.sleep(max(0, 1.1 - (time.monotonic() - last.get(host, 0))))
    last[host] = time.monotonic()
    try:
        with urllib.request.urlopen(urllib.request.Request(url, headers={'User-Agent': AGENT}), timeout=25) as response:
            return response.status, response.read(), response.headers.get('Content-Type'), response.url
    except urllib.error.HTTPError as error:
        return error.code, error.read(), error.headers.get('Content-Type'), error.url


def fetch(name, url):
    origin = '{0.scheme}://{0.netloc}'.format(urllib.parse.urlsplit(url))
    if origin not in robots:
        status, body, _, _ = request(origin + '/robots.txt')
        (OUT / (urllib.parse.urlsplit(url).netloc + '-robots.txt')).write_bytes(body)
        parser = urllib.robotparser.RobotFileParser()
        parser.parse(body.decode('utf-8', errors='replace').splitlines() if status == 200 else [])
        robots[origin] = parser
    if not robots[origin].can_fetch(AGENT, url):
        records.append({'name': name, 'url': url, 'status': 'ROBOTS_DISALLOW'})
        return None
    try:
        status, body, content_type, final_url = request(url)
        (OUT / name).write_bytes(body)
        records.append(dict(name=name, url=url, final_url=final_url, status=status, bytes=len(body), content_type=content_type))
        print(name, status, len(body), flush=True)
        (OUT / 'manifest.json').write_text(json.dumps(records, indent=2))
        return body
    except (urllib.error.URLError, TimeoutError) as error:
        records.append(dict(name=name, url=url, status='NETWORK_ERROR', error=str(error)))
        (OUT / 'manifest.json').write_text(json.dumps(records, indent=2))
        print(name, str(error), flush=True)


sources = {
    'sii.html': 'https://www.sii.cl/preguntas_frecuentes/aval_contrib_bbrr/001_165_8108.htm',
    'ipv.html': 'https://si3.bcentral.cl/siete/ES/Siete/Cuadro/CAP_ESTADIST_EXPERIM/MN_EXPERIM01/IS_PRECIOS_GEO_2008?idSerie=F034.IPVZ41.FLU.BCCH.2008.0.T',
    'vmv.html': 'https://www.bcentral.cl/areas/estadisticas/estadisticas-experimentales/vmv',
    'toctoc.html': 'https://www.toctoc.com/',
    'ine-permits.html': 'https://www.ine.gob.cl/estadisticas-por-tema/industria-energia-y-construccion/permisos-de-edificacion',
    'zoning.html': 'https://geoportal.cl/geoportal/catalog/36394/zonificacion-planes-reguladores-comunales-region-metropolitana',
    'zoning-service.json': 'https://geoide.minvu.cl/server/rest/services/IPT/PRC_Metropolitana/MapServer?f=pjson',
    'green-service.json': 'https://geoide.minvu.cl/server/rest/services/Catastros/%C3%81reas_Verdes_MINVU_INE_2023/FeatureServer?f=pjson',
    'green-layer.json': 'https://geoide.minvu.cl/server/rest/services/Catastros/%C3%81reas_Verdes_MINVU_INE_2023/FeatureServer/0?f=pjson',
    'settlements-service.json': 'https://geoide.minvu.cl/server/rest/services/Catastros/Catastros_campamentos/FeatureServer?f=pjson',
}

if __name__ == '__main__':
    for name, url in sources.items():
        fetch(name, url)
