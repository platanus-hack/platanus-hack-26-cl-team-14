"""Export regular Monday Metro timetables on the existing directional transit_paths geometry."""
import csv
import gzip
import hashlib
import io
import json
import math
import sys
from collections import defaultdict
from pathlib import Path
from zipfile import ZipFile

source, geometry, output = map(Path, sys.argv[1:] or (
    'data/raw/dtpm-gtfs-march/GTFS_20260321_v3.zip',
    'web/data/transit_paths.json',
    'web/data/metro_schedule.json.gz',
))
with ZipFile(source) as feed:
    def table(name):
        return csv.DictReader(io.TextIOWrapper(feed.open(name + '.txt'), encoding='utf-8-sig'))

    weekdays = {r['service_id'] for r in table('calendar') if r['monday'] == '1'}
    routes = {r['route_id']: r['route_short_name'] for r in table('routes') if r['route_type'] == '1'}
    runs = {r['trip_id']: (routes[r['route_id']], int(r['direction_id']))
            for r in table('trips') if r['service_id'] in weekdays and r['route_id'] in routes}
    stops = {r['stop_id']: r for r in table('stops')}
    calls = defaultdict(list)
    def secs(clock):
        h, m, s = map(int, clock.split(':'))
        return 3600*h + 60*m + s
    for r in table('stop_times'):
        if r['trip_id'] in runs:
            calls[r['trip_id']].append((int(r['stop_sequence']), r['stop_id'],
                                       secs(r['arrival_time']), secs(r['departure_time'])))
    feed_info = next(table('feed_info'))

def xy(lon, lat):
    return (float(lon) + 70.65)*92915, -(float(lat) + 33.45)*110905

lines = {(row['route'], row['direction']): row for row in json.loads(geometry.read_bytes())
         if row['mode'] == 'metro'}
paths = {key: [xy(*p) for p in row['path']] for key, row in lines.items()}
station_indices = {key: {sid: i for i,sid in enumerate(row['parent_station_ids'])}
                   for key, row in lines.items()}
positions = {}
def on_track(key, platform):
    if (key,platform) not in positions:
        x,y=xy(stops[platform]['stop_lon'],stops[platform]['stop_lat'])
        best=(math.inf, 0)
        points=paths[key]
        for i,((ax,ay),(bx,by)) in enumerate(zip(points,points[1:])):
            dx,dy=bx-ax,by-ay
            if not (dx or dy): continue
            t=max(0,min(1,((x-ax)*dx+(y-ay)*dy)/(dx*dx+dy*dy)))
            best=min(best,(math.hypot(x-ax-dx*t, y-ay-dy*t),i+t))
        positions[key,platform]=best
    return positions[key,platform][1]

result = {key: [] for key in paths}
bad = []
for tid, key in runs.items():
    times=sorted(calls[tid])
    timeline=[[a,d,round(on_track(key,s),6),station_indices[key][stops[s]['parent_station']]] for _,s,a,d in times]
    if any(b[2]<=a[2] or b[0]<a[1] for a,b in zip(timeline,timeline[1:])):
        bad.append((tid,[(a,b) for a,b in zip(timeline,timeline[1:]) if b[2]<=a[2] or b[0]<a[1]]))
    result[key].append([tid,timeline])
for rows in result.values():
    rows.sort(key=lambda r:r[1][0][0])
bundle={'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
        'feed_version':feed_info['feed_version'],
        'publisher':feed_info['feed_publisher_name'],
        'source_url':'https://www.dtpm.cl/descargas/gtfs/GTFS_20260321_v3.zip',
        'lines': [{'route':route,'direction':direction,'trips':trips}
                  for (route,direction),trips in sorted(result.items())]}
encoded=json.dumps(bundle,separators=(',',':')).encode()
output.write_bytes(gzip.compress(encoded,compresslevel=9,mtime=0))
print('size',output.stat().st_size,'sha256',hashlib.sha256(output.read_bytes()).hexdigest(),
      'source_sha256',bundle['source_sha256'],'calls',sum(map(len,calls.values())),
      'runs',len(calls),'max offset to shape in m',max(positions.values())[0])
print('backtracking calls or negative travel time',len(bad),bad[:4])
print('first last',min(t[1][0][0] for rows in result.values() for t in rows),max(t[1][-1][1] for rows in result.values() for t in rows))
