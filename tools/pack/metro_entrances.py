"""Join Santiago Metro entrances to the engine's GTFS parent station IDs."""
import csv
import gzip
import hashlib
import io
import json
import math
import re
import struct
import subprocess
import sys
import tempfile
import unicodedata
import xml.etree.ElementTree as ET
from collections import Counter, defaultdict
from pathlib import Path
from zipfile import ZipFile

feed, pbf, footprints, out = map(Path, sys.argv[1:] or (
    'data/raw/dtpm-gtfs-march/GTFS_20260321_v3.zip',
    'data/raw/osm-buildings-city/urban.osm.pbf',
    'web/data/city_building_polygons.bin.gz',
    'web/data/metro_entrances.json.gz',
))
with ZipFile(feed) as z:
    stops=list(csv.DictReader(io.TextIOWrapper(z.open('stops.txt'),encoding='utf-8-sig')))
stations = {s['stop_id']:s for s in stops if s['location_type']=='1'}
def coord(s):
    return [float(s['stop_lon']),float(s['stop_lat'])]
def metres(p,q):
    return math.hypot((p[0]-q[0])*92915,(p[1]-q[1])*110905)
def norm(s):
    s=''.join(c for c in unicodedata.normalize('NFD',s.lower()) if not unicodedata.combining(c))
    return ' '.join(re.findall(r'[a-z0-9]+',s))
def station_name(s):
    s=norm(s)
    return re.sub(r'\s+(?:l?[1-6]a?|metro)$','',re.sub(r'^(?:entradas?\s+)?(?:estacion\s+|metro\s+)?','',s))
def tags(e):
    return {t.get('k'):t.get('v') for t in e.findall('tag')}
by_station=defaultdict(list)
official=[]
for s in stops:
    if s['location_type']!='2':continue
    sid=s['parent_station'];p=coord(s);ref=s['stop_id'].split(':')[-1]
    e={'gtfs':s['stop_id'],'position':p,'name':s['stop_name']}
    by_station[sid].append(e)
    official.append((sid,e,ref,p))
with tempfile.TemporaryDirectory() as tmp:
    selected=Path(tmp)/'entrances.osm'
    subprocess.run(['osmium','tags-filter','--omit-referenced',str(pbf),'n/railway=subway_entrance',
                    'r/public_transport=stop_area','-o',str(selected)],check=True)
    xml=ET.parse(selected).getroot()
nodes={n.get('id'):n for n in xml.findall('node')}
relations=defaultdict(list)
for r in xml.findall('relation'):
    name=tags(r).get('name','')
    for member in r.findall('member'):
        if member.get('type')=='node' and member.get('ref') in nodes:
            relations[member.get('ref')].append((r.get('id'),name))
names={station_name(s['stop_name']):sid for sid,s in stations.items()}
candidates=[];unmatched=[];methods=Counter()
for nid,n in nodes.items():
    p=[float(n.get('lon')),float(n.get('lat'))];nt=tags(n)
    near=sorted((metres(p,c),sid) for sid,_,_,c in official)
    parent=near[0][1]
    # A named station relation must agree spatially with the closest official entrance.
    # Interchange lines share the same GTFS parent. An entrance is emitted once.
    direct=[rid for rid,label in relations[nid] if names.get(station_name(label))==parent]
    next_station=min(d for d,sid in near if sid!=parent)
    if direct and near[0][0]<=250 and next_station-near[0][0]>=50:
        method='relation'
    elif near[0][0]<=25 and next_station-near[0][0]>=100:
        method='gtfs proximity'
    else:
        unmatched.append((nid,near[0],relations[nid]));continue
    entry={'osm':int(nid),'position':p,'name':nt.get('name',''),'ref':nt.get('ref','')}
    if direct: entry['relation']=int(direct[0])
    candidates.append((parent,entry))
    methods[method]+=1
# Merge sources one-to-one. GTFS coordinates remain exact; OSM-only exits keep their OSM location.
# Multiple OSM doors at one access remain separate mapped positions.
pairings=[];matched=set()
for i,(parent,osm) in enumerate(candidates):
    for sid,e,ref,p in official:
        if parent!=sid:continue
        d=metres(p,osm['position']);same=bool(osm['ref']) and norm(osm['ref'])==norm(ref)
        if d<=8 or (same and d<=35):
            pairings.append((not same,d,i,e['gtfs'],e))
gtfs_used=set();matched_dist=[]
for _,d,i,gid,e in sorted(pairings,key=lambda x:x[:4]):
    if i in matched or gid in gtfs_used:continue
    osm=candidates[i][1];e['osm']=osm['osm']
    if 'relation' in osm:e['relation']=osm['relation']
    matched.add(i);gtfs_used.add(gid);matched_dist.append(d)
supplemental=0
for i,(sid,e) in enumerate(candidates):
    if i in matched or not e.get('relation') or not e['ref']:
        continue
    # A repeated exit letter with conflicting source coordinates is still one exit.
    # Keep the official point when the two sources disagree.
    known={norm(ref) for parent,entry,ref,position in official if parent==sid}
    known.update(norm(item['ref']) for item in by_station[sid] if item.get('ref'))
    if norm(e['ref']) in known:
        continue
    by_station[sid].append(e)
    supplemental+=1
# Decode each needed tile in memory. The tile assignment uses the building centre, so look
# in adjacent tiles as well. The stored vertices use 10^-5 degree offsets from west/south.
def add_porches(by_station, pack):
    with gzip.open(pack, 'rb') as f:
        data = f.read()
    magic,cols,rows,west,south,east,north,step = struct.unpack_from('<4sHHiiiiI',data)
    if magic != b'SGB1':
        raise ValueError('Expected a city polygon pack')
    offsets = struct.unpack_from(f'<{cols*rows+1}I',data,28)
    longitude_m,latitude_m = .92915,1.10905
    entrances = [e for values in by_station.values() for e in values]
    pending = {}
    hits=Counter()

    def contains(r,p):
        x,y=p; inside=False
        for i,a in enumerate(r):
            b=r[(i+1)%len(r)]
            if (a[1]>y)!=(b[1]>y) and x<(b[0]-a[0])*(y-a[1])/(b[1]-a[1])+a[0]:
                inside=not inside
        return inside

    def buildings_at(points, visit):
        tiles=defaultdict(lambda:defaultdict(list))
        for index,(x,y) in points.items():
            col,row=math.floor(x/step),math.floor(y/step)
            for r in range(max(0,row-1),min(rows,row+2)):
                for c in range(max(0,col-1),min(cols,col+2)):
                    tiles[r*cols+c][(int(x//200),int(y//200))].append((index,(x,y)))
        for tile,buckets in tiles.items():
            base,end=offsets[tile:tile+2]
            if base==end:continue
            count,nrings,nvertices=struct.unpack_from('<III',data,base)
            starts=struct.unpack_from(f'<{count+1}I',data,base+12)
            rings_at=base+12+4*(count+1)
            rings=struct.unpack_from(f'<{nrings+1}I',data,rings_at)
            verts_at=rings_at+4*(nrings+1)
            def ring(i):
                a,b=rings[i:i+2]
                return list(struct.iter_unpack('<HH',memoryview(data)[verts_at+4*a:verts_at+4*b]))
            for i in range(count):
                a,b=starts[i:i+2]
                outer=ring(a)
                xs,ys=zip(*outer)
                left,right,bottom,top=min(xs),max(xs),min(ys),max(ys)
                for cx in range(left//200,right//200+1):
                    for cy in range(bottom//200,top//200+1):
                        for k,pt in buckets.get((cx,cy),[]):
                            x,y=pt
                            if left<=x<=right and bottom<=y<=top and contains(outer,pt):
                                if not any(contains(ring(h),pt) for h in range(a+1,b)):
                                    visit(k,pt,outer)

    def facade(index, p, outer):
        hits[index]+=1
        x,y=p; nearest=None
        for i,(ax,ay) in enumerate(outer):
            bx,by=outer[(i+1)%len(outer)]
            dx,dy=(bx-ax)*longitude_m,(by-ay)*latitude_m
            if not (dx or dy):continue
            rx,ry=(x-ax)*longitude_m,(y-ay)*latitude_m
            t=max(0,min(1,(dx*rx+dy*ry)/(dx*dx+dy*dy)))
            ex,ey=dx*t-rx,dy*t-ry
            distance=math.hypot(ex,ey)
            if nearest is None or distance<nearest[0]:
                nearest=distance,ex,ey
        if nearest is None or not 0<nearest[0]<=4: return
        distance,ex,ey=nearest
        nx,ny=ex/distance,ey/distance
        px=x+nx*(distance+1.8)/longitude_m
        py=y+ny*(distance+1.8)/latitude_m
        pending[index]=((px,py),(nx,ny),distance)

    points={i:(e['position'][0]*100000-west,e['position'][1]*100000-south) for i,e in enumerate(entrances)}
    buildings_at(points,facade)
    # Keep the porch outside all buildings, including across shared walls.
    inside=set()
    buildings_at({i:proposal[0] for i,proposal in pending.items()},
                 lambda i,point,wall:inside.add(i))
    for i,((x,y),(nx,ny),distance) in pending.items():
        if i in inside or hits[i]>1:continue
        entrances[i]['porch']={'position':[round((west+x)/100000,8),round((south+y)/100000,8)],
                               'outward':[round(nx,6),round(ny,6)],'facade_m':round(distance,2)}
    print('physical entrances inside footprints',len(hits),'within 4m',len(pending),
          'porches',sum('porch' in e for e in entrances),
          'overlap excluded',sum(i in inside or hits[i]>1 for i in pending))
add_porches(by_station,footprints)
bundle={'gtfs_source':'https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip',
        'gtfs_sha256':hashlib.sha256(feed.read_bytes()).hexdigest(),
        'osm_source':'https://download.geofabrik.de/south-america/chile-261120.osm.pbf',
        'osm_credit':'© OpenStreetMap contributors','osm_license':'ODbL-1.0',
        'porch_note':'Cartographic canopy position 1.8 m beyond the exterior facade. Outward is a unit vector [east,north]. The entrance position is the source coordinate.',
        'footprints_sha256':hashlib.sha256(footprints.read_bytes()).hexdigest(),
        'stations':dict(sorted(by_station.items()))}
out.write_bytes(gzip.compress(json.dumps(bundle,ensure_ascii=False,separators=(',',':')).encode(),mtime=0))
print('stations',len(by_station),'gtfs',len(official),'OSM',len(nodes),'joined',len(candidates),
      methods,'merged',len(matched),'explicit_extra',supplemental,'unresolved',len(unmatched))
print('records',sum(map(len,by_station.values())),'size',out.stat().st_size,'sha256',hashlib.sha256(out.read_bytes()).hexdigest())
print('GTFS to OSM merged distance min/median/max',min(matched_dist),sorted(matched_dist)[len(matched_dist)//2],max(matched_dist))
print('BL',by_station['BL'])
print('unresolved',unmatched)
