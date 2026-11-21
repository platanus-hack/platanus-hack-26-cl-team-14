"""Weekday EOD 2012 trips along the Mapocho: both ends within 800 m of the river, 3 km or more apart east-west after projection onto it.

Usage: python3 -I tools/pack/mapocho_corridor.py <EOD_STGO dir> <nominatim json of "Río Mapocho">
EOD: https://users.dcc.uchile.cl/~egraells/gds-data/eod2012.tgz
River: https://nominatim.openstreetmap.org/search?q=R%C3%ADo+Mapocho&format=json&polygon_geojson=1
Trip weight = person Factor_LaboralNormal x trip FactorLaboralNormal.
"""
import math
def utm19s_to_lonlat(x, y):
    a, f, k0 = 6378137.0, 1 / 298.257223563, 0.9996
    e2 = f * (2 - f)
    ep2 = e2 / (1 - e2)
    m = (y - 10000000.0) / k0
    mu = m / (a * (1 - e2 / 4 - 3 * e2**2 / 64 - 5 * e2**3 / 256))
    e1 = (1 - math.sqrt(1 - e2)) / (1 + math.sqrt(1 - e2))
    p1 = (mu + (3 * e1 / 2 - 27 * e1**3 / 32) * math.sin(2 * mu)
          + (21 * e1**2 / 16 - 55 * e1**4 / 32) * math.sin(4 * mu)
          + (151 * e1**3 / 96) * math.sin(6 * mu) + (1097 * e1**4 / 512) * math.sin(8 * mu))
    s, c, t = math.sin(p1), math.cos(p1), math.tan(p1)
    n = a / math.sqrt(1 - e2 * s * s)
    r = a * (1 - e2) / (1 - e2 * s * s) ** 1.5
    t2, c1, d = t * t, ep2 * c * c, (x - 500000.0) / (n * k0)
    lat = p1 - (n * t / r) * (d**2 / 2 - (5 + 3 * t2 + 10 * c1 - 4 * c1**2 - 9 * ep2) * d**4 / 24
                              + (61 + 90 * t2 + 298 * c1 + 45 * t2**2 - 252 * ep2 - 3 * c1**2) * d**6 / 720)
    lon = (d - (1 + 2 * t2 + c1) * d**3 / 6
           + (5 - 2 * c1 + 28 * t2 - 3 * c1**2 + 8 * ep2 + 24 * t2**2) * d**5 / 120) / c
    return math.degrees(lon) - 69.0, math.degrees(lat)

import csv, json, sys, collections
eod_dir, nom = sys.argv[1], sys.argv[2]
with open(nom) as source:
    mapocho = next(
        result for result in json.load(source)
        if result['osm_type'] == 'relation' and str(result['osm_id']) == '7548664'
    )
rel = mapocho['geojson']['coordinates']
# Urban stretch: Lo Barnechea (La Dehesa) to Pudahuel (Ruta 68 area).
LON0, LON1 = -70.80, -70.52
K = 111320 * math.cos(math.radians(-33.43))
xy = lambda lon, lat: ((lon + 70.65) * K, (lat + 33.43) * 110574)
segs = []
for line in rel:
    pts = [xy(*p) for p in line if LON0 <= p[0] <= LON1]
    segs += [(a, b) for a, b in zip(pts, pts[1:]) if math.dist(a, b) < 500]
# Distance between trip ends is east-west after projection onto the river, not true river length.
def near(p):
    best = (1e18, 0)
    for a, b in segs:
        dx, dy = b[0]-a[0], b[1]-a[1]
        t = max(0, min(1, ((p[0]-a[0])*dx + (p[1]-a[1])*dy) / (dx*dx + dy*dy or 1)))
        q = (a[0]+t*dx, a[1]+t*dy)
        d = math.dist(p, q)
        if d < best[0]: best = (d, q[0])
    return best
num = lambda s: float(s.replace(',', '.'))
MODES = {1:'auto',2:'bus',3:'bus',4:'metro',5:'colectivo',6:'taxi',7:'bus',8:'auto+metro',9:'bus+metro',10:'bus+metro',11:'colectivo+metro',12:'taxi+metro',13:'otros+metro',14:'otros+bus',15:'otros+bus+metro',16:'otros',17:'caminata',18:'bicicleta'}
pf = {}
for r in csv.DictReader(open(eod_dir + '/personas.csv', encoding='latin-1'), delimiter=';'):
    if r['Factor_LaboralNormal']: pf[r['Persona']] = num(r['Factor_LaboralNormal'])
names = {r['Id']: r['Comuna'] for r in csv.DictReader(open(eod_dir + '/Tablas_parametros/Comunas.csv', encoding='latin-1'))}
pairs = collections.Counter()
tot = 0; along = collections.Counter(); by_mode = collections.Counter(); by_hour = collections.Counter(); minutes = []
for R in (800,):
    for r in csv.DictReader(open(eod_dir + '/viajes.csv', encoding='latin-1'), delimiter=';'):
        f = r['FactorLaboralNormal']
        if not f or not r['OrigenCoordX'] or not r['DestinoCoordX']: continue
        f = num(f) * pf.get(r['Persona'], 0); tot += f
        o = xy(*utm19s_to_lonlat(num(r['OrigenCoordX']), num(r['OrigenCoordY'])))
        d = xy(*utm19s_to_lonlat(num(r['DestinoCoordX']), num(r['DestinoCoordY'])))
        if abs(o[1]) > 8000 or abs(d[1]) > 8000: continue
        do, xo = near(o)
        if do > R: continue
        dd, xd = near(d)
        if dd > R or abs(xo - xd) < 3000: continue
        along['trips'] += f
        by_mode[MODES.get(int(r['ModoAgregado'] or 16), 'otros')] += f
        by_hour[r['HoraIni'][:2]] += f
        pairs[tuple(sorted((names[r['ComunaOrigen']], names[r['ComunaDestino']])))] += f
        if r['TiempoViaje']: minutes.append((int(r['TiempoViaje']), f))
print('weekday trips with origin and destination coordinates (expanded):', round(tot))
print('both ends <=800 m from river, >=3 km apart east-west after projection:', round(along['trips']))
print('share of weekday trips with origin and destination coordinates: %.2f%%' % (100*along['trips']/tot))
for m, v in by_mode.most_common(): print(' mode', m, round(v), '%.1f%%' % (100*v/along['trips']))
minutes.sort(); acc=0; half=sum(f for _,f in minutes)/2
for t,f in minutes:
    acc+=f
    if acc>=half: print('median reported trip minutes:', t); break
print('top start hours:', [(h, round(v)) for h, v in by_hour.most_common(6)])
print('top comuna pairs:', [(a+' - '+b, round(v)) for (a,b), v in pairs.most_common(8)])
