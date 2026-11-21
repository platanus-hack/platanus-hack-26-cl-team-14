"""Save bounded samples. Run /usr/bin/python3 tools/scrape/verify_housing_land.py."""
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
    'ine-app.json': 'https://www.arcgis.com/sharing/rest/content/items/2c09e6be8f2f49af922fcb7184ea0ecf/data?f=json',
    'ine-webmap.json': 'https://www.arcgis.com/sharing/rest/content/items/34fdbaa793e643eb9f87c4c4751ff4a7/data?f=json',
    'ine-layer.json': 'https://services5.arcgis.com/hUyD8u3TeZLKPe4T/arcgis/rest/services/CRF_PE_v2/FeatureServer/1?f=pjson',
    'vmv-note.pdf': 'https://www.bcentral.cl/documents/33528/3662096/Nota-tecnica-VMV.pdf',
}


def query(name, layer, **params):
    fetch(name, layer + '/query?' + urllib.parse.urlencode({'f': 'json', **params}))


if __name__ == '__main__':
    for name, url in sources.items():
        fetch(name, url)
    query('green-sample.json', sources['green-layer.json'].split('?')[0],
          where="COMUNA = 'SANTIAGO'",
          outFields='CUT,COMUNA,NOMBRE_EP,SUP_TOTAL_,SUP_VEGETA,USO,TIPO_EP,ID_AV',
          resultRecordCount=5, returnGeometry='true', outSR=4326)
    service = sources['settlements-service.json'].split('?')[0]
    layers = json.loads((OUT / 'settlements-service.json').read_bytes())['layers']
    for year, fields in [
        (2024, 'FOLIO,NOMBRE,CUT_R,CUT,COMUNA,AN_CAT,N_HOG,HECTAREAS'),
        (2026, 'FOLIO,NOM,REG,COM,INGRESO,EST,HOGARESCAT,HOGARESCEN,SUPERFIC_1'),
    ]:
        layer_id = next(x['id'] for x in layers if x['name'] == 'Catastro ' + str(year))
        layer = service + '/' + str(layer_id)
        fetch('settlements-layer-' + str(layer_id) + '.json', layer + '?f=pjson')
        query('settlements-' + str(year) + '-urban-sample.json', layer,
              where='1=1', geometry='-70.85,-33.65,-70.45,-33.30',
              geometryType='esriGeometryEnvelope', inSR=4326,
              spatialRel='esriSpatialRelIntersects', outFields=fields,
              resultRecordCount=5, returnGeometry='true', outSR=4326)
    layer = sources['ine-layer.json'].split('?')[0]
    query('ine-permits-sample.json', layer, where='COD_REGION=13', outFields='*',
          resultRecordCount=5, orderByFields='AÑO DESC,OBJECTID ASC',
          returnGeometry='true', outSR=4326)
    stats = [{'statisticType': t, 'onStatisticField': 'AÑO', 'outStatisticFieldName': n}
             for t, n in [('min', 'first_year'), ('max', 'last_year')]]
    query('ine-permits-years.json', layer, where='COD_REGION=13', outStatistics=json.dumps(stats))
