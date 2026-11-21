import json,sys
g=json.load(open(sys.argv[1])); r=json.load(open(sys.argv[2]))
names={z['id']:z['name'] for z in r['zones']} if r['zones'] else {}
import csv
cm={row['cut']:row for row in csv.DictReader(open(sys.argv[3]))}
# bbox of urban Santiago
x0,x1,y0,y1=-70.86,-70.45,-33.66,-33.30
def keep(c): return x0<c[0]<x1 and y0<c[1]<y1
polys=[]
for f in g['features']:
    geom=f['geometry']; rings=geom['coordinates'] if geom['type']=='MultiPolygon' else [geom['coordinates']]
    for poly in rings:
        outer=poly[0]
        if not any(keep(c) for c in outer): continue
        step=max(1,len(outer)//120)
        pts=[[round(c[0],4),round(c[1],4)] for c in outer[::step]]
        polys.append({'cut':f['properties']['cut'],'p':pts})
flows=[{'o':f['origin'],'d':round(f['base']-f['scenario'])} for f in r['flows'] if f['destination']=='13114' and f['base']!=f['scenario']]
flows=[dict(f,lon=float(cm[f['o']]['longitude']),lat=float(cm[f['o']]['latitude'])) for f in flows if f['d']>20]
ev=r['events'][0]
ret=[{'cut':k,'n':round(v),'lon':float(cm[k]['longitude']),'lat':float(cm[k]['latitude'])} for k,v in ev['attendees_by_return_comuna'].items() if v>30]
st=[{'name':s['name'],'lon':s['longitude'],'lat':s['latitude'],'d':s['delta_daily']} for s in r['stations']]
m=ev['attendees_by_census_mode']
modes={'car':m['1'],'pt':m['2'],'walk':m['3'],'other':m['4']+m['5']+m['6']+m['7']+m['-99']}
out={'polys':polys,'flows':flows,'ret':ret,'stations':st,'modes':{k:round(v) for k,v in modes.items()},
 'moved':r['summary']['workers_moved'],'attendance':ev['attendance'],'metro':round(ev['potential_metro_users']),'revision':r['revision']}
print('const D='+json.dumps(out,separators=(',',':'))+';')
