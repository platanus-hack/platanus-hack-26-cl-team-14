"""Build the service options for each published public ride leg on a regular Monday."""
import csv
import gzip
import io
import json
import math
import sys
from collections import defaultdict, Counter
from pathlib import Path
from zipfile import ZipFile

feed_path, data_dir, out_path = map(Path, sys.argv[1:])
def time_seconds(s):
    h,m,s=map(int,s.split(':'))
    return h*3600+m*60+s
def coord(lon,lat): return round(float(lon),5), round(float(lat),5)
with ZipFile(feed_path) as z:
    def table(n):return csv.DictReader(io.TextIOWrapper(z.open(n+'.txt'),encoding='utf-8-sig'))
    service={r['service_id'] for r in table('calendar') if r['monday']=='1'}
    routes={r['route_id']:r for r in table('routes')}
    trips={r['trip_id']:r for r in table('trips') if r['service_id'] in service}
    stops={r['stop_id']:r for r in table('stops')}
    windows=defaultdict(list)
    for r in table('frequencies'):
        if r['trip_id'] in trips:
            windows[r['trip_id']].append((
                time_seconds(r['start_time']), time_seconds(r['end_time']),
                int(r['headway_secs']), int(r.get('exact_times') or 0),
            ))
    calls=defaultdict(list)
    for r in table('stop_times'):
        tid=r['trip_id']
        if tid in trips:calls[tid].append((int(r['stop_sequence']),r['stop_id'],time_seconds(r['arrival_time']),time_seconds(r['departure_time'])))
    for c in calls.values():c.sort()

coords={s:coord(v['stop_lon'],v['stop_lat']) for s,v in stops.items() if v['stop_lon']}
# Match the exact route and the ordered boarding/alighting stops; reject routes that skip either stop.
groups=defaultdict(list)
for tid,t in trips.items():
    groups[(t['route_id'],t['direction_id'],t['shape_id'],tuple(s for _,s,a,d in calls[tid]))].append(tid)
records=list(json.loads((data_dir/'public_routes.json').read_bytes()).values())
records.extend(json.loads(gzip.open(data_dir/'local_public_routes.json.gz').read())['paths'])
needed={}
for r in records:
 for l in r['segments']:
    if l['mode']=='walk':continue
    pts=r['path'][l['start']:l['end']+1]
    needed[(l['service'],tuple(pts[0]),tuple(pts[-1]))]=l['mode']
by_route=defaultdict(list)
for key,tids in groups.items():
    seen=defaultdict(list)
    for i,s in enumerate(key[3]):seen[coords[s]].append(i)
    by_route[key[0]].append((key,seen))
opts={}
for leg,mode in needed.items():
    route,first,last=leg
    options=[]
    for key,seen in by_route[route]:
        for i in seen.get(first,[]):
            for j in seen.get(last,[]):
                if j>i:options.append((key,i,j))
    opts[leg]=options
print('needed/covered',Counter(needed.values()),Counter(needed[k] for k,v in opts.items() if v),flush=True)
print('unresolved examples',[(k,needed[k]) for k,o in opts.items() if not o][:12],flush=True)
transit=[x for x in json.loads((data_dir/'transit_paths.json').read_bytes()) if x['mode']=='metro']
lineids={(x['route'],str(x['direction'])):i for i,x in enumerate(transit)}
surface_keys=sorted({k for leg,os in opts.items() if needed[leg]!='metro' for k,i,j in os})
shape_ids=sorted({k[2] for k in surface_keys})
shapes=defaultdict(list)
with ZipFile(feed_path) as z:
    for r in table('shapes'):
        if r['shape_id'] in shape_ids:
            shapes[r['shape_id']].append((int(r['shape_pt_sequence']),coord(r['shape_pt_lon'],r['shape_pt_lat'])))
shapes={k:[p for _,p in sorted(v)] for k,v in shapes.items()}
def xy(p):return (p[0]+70.65)*92915,-(p[1]+33.45)*110905
def project_stops(path,seq):
    # Match the entire ordered platform sequence. Loops can revisit the same street.
    from array import array
    line=[xy(p) for p in path];segments=[(a,b) for a,b in zip(line,line[1:])]
    grid=defaultdict(list)
    for n,((ax,ay),(bx,by)) in enumerate(segments):
        for i in range(math.floor(min(ax,bx)/150),math.floor(max(ax,bx)/150)+1):
            for j in range(math.floor(min(ay,by)/150),math.floor(max(ay,by)/150)+1):
                grid[i,j].append(n)
    prev=[(-1.,0,None)]; history=[]
    for sid in seq:
        x,y=xy(coords[sid]);i,j=math.floor(x/150),math.floor(y/150)
        ids={idx for a in range(i-1,i+2) for b in range(j-1,j+2) for idx in grid[a,b]}
        def candidates(ids):
            out=[]
            for n in sorted(ids):
                (ax,ay),(bx,by)=segments[n];dx,dy=bx-ax,by-ay
                if not (dx or dy):continue
                t=max(0,min(1,((x-ax)*dx+(y-ay)*dy)/(dx*dx+dy*dy)))
                out.append((n+t,(x-ax-t*dx)**2+(y-ay-t*dy)**2))
            return out
        def step(cs):
            out=[];q=0;best=math.inf;back=0
            for pos,cost in cs:
                while q<len(prev) and prev[q][0]<=pos:
                    if prev[q][1]<best:best=prev[q][1];back=q
                    q+=1
                if best<math.inf:out.append((pos,best+cost,back))
            return out
        cur=step(candidates(ids)) or step(candidates(range(len(segments))))
        history.append(cur);prev=cur
    q=min(range(len(prev)),key=lambda q:prev[q][1]);indices=[]
    for row in history[::-1]:
        v,_,q=row[q];indices.append(round(v,6))
    return indices[::-1]
shape_idx={k:i for i,k in enumerate(shape_ids)}
ptn_index={};templates=[];patterns=[]
for k in surface_keys:
    route,direction,shape,sids=k
    axis=project_stops(shapes[shape],sids)
    pi=len(patterns); patterns.append([shape_idx[shape],axis])
    timings = defaultdict(lambda: (set(), set()))
    for tid in groups[k]:
        cs=calls[tid];first=cs[0][3]
        off=tuple((a-first,d-first) for _,s,a,d in cs)
        fixed, headways = timings[off]
        if tid not in windows:
            fixed.add(first)
        for start, end, interval, exact in windows[tid]:
            if exact:
                fixed.update(range(start, end, interval))
            else:
                headways.add((start, end, interval))
    indices=[]
    for off, (fixed, headways) in sorted(timings.items()):
        # Fixed times and frequency windows have distinct meanings in GTFS.
        indices.append(len(templates))
        templates.append([pi, off, sorted(fixed), routes[route]['route_type'], sorted(headways)])
    ptn_index[k]=indices
legs=[]
for (service,first,last),os in sorted(opts.items()):
    metro=set();surface=set()
    for k,i,j in os:
        if routes[service]['route_type']=='1':
            track=lineids[k[0],k[1]];station=transit[track]['parent_station_ids']
            metro.add((track,station.index(stops[k[3][i]]['parent_station']),station.index(stops[k[3][j]]['parent_station'])))
        else:
            for ti in ptn_index[k]:surface.add((ti,i,j))
    legs.append([service,first,last,sorted(metro),sorted(surface)])
import hashlib
bundle={'source_url':'https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip',
        'source_sha256':hashlib.sha256(feed_path.read_bytes()).hexdigest(),
        'shapes':[shapes[k] for k in shape_ids],'patterns':patterns,'templates':templates,'legs':legs}
out_path.write_bytes(gzip.compress(json.dumps(bundle,separators=(',',':')).encode(),compresslevel=9,mtime=0))
print('surface shapes/patterns/templates',len(shapes),len(patterns),len(templates),'size',out_path.stat().st_size,'sha256',hashlib.sha256(out_path.read_bytes()).hexdigest(),flush=True)
