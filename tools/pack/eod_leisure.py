# EOD 2012 leisure, shopping and work trips by day type: share, mode, hour, distance.
# Data: https://users.dcc.uchile.cl/~egraells/gds-data/eod2012.tgz (viajes.csv, personas.csv, DistanciaViaje.csv).
# Run: python3 -I tools/pack/eod_leisure.py <extracted EOD_STGO dir>
import csv, sys, collections
D = sys.argv[1]
def rows(f):
    with open(D + "/" + f, encoding="latin1") as h:
        yield from csv.DictReader(h, delimiter=";")
num = lambda s: float(s.replace(",", ".") or 0)
DAYS = {"lab": ("FactorLaboralNormal", "Factor_LaboralNormal"), "sab": ("FactorSabadoNormal", "Factor_SabadoNormal"), "dom": ("FactorDomingoNormal", "Factor_DomingoNormal")}
pw = {d: {} for d in DAYS}
for r in rows("personas.csv"):
    for d, (_, pf) in DAYS.items(): pw[d][r["Persona"]] = num(r[pf])
dist = {r["Viaje"]: (num(r["DistEuclidiana"]), r["Imputada"]) for r in rows("DistanciaViaje.csv")}
def grp(m):
    m = int(m or 0)
    return "auto" if m == 1 else "publico" if m in (2,3,4,5,6,7,9,10,11,12) else "caminata" if m == 17 else "bici" if m == 18 else "otro"
def pg(p):
    return {"13": "recreacion", "11": "compras", "1": "trabajo", "3": "estudio", "7": "volver", "6": "visita", "9": "comer"}.get(p, "otros")
tot = {d: collections.Counter() for d in DAYS}
mode = {d: collections.defaultdict(collections.Counter) for d in DAYS}
hour = {d: collections.defaultdict(collections.Counter) for d in DAYS}
dl = {d: collections.defaultdict(list) for d in DAYS}
tt = {d: collections.defaultdict(list) for d in DAYS}
n = {d: collections.Counter() for d in DAYS}
for r in rows("viajes.csv"):
    for d, (tf, _) in DAYS.items():
        w = num(r[tf]) * pw[d].get(r["Persona"], 0)
        if w <= 0: continue
        g = pg(r["Proposito"])
        tot[d][g] += w; n[d][g] += 1
        mode[d][g][grp(r["ModoAgregado"])] += w
        if r["HoraIni"]: hour[d][g][int(r["HoraIni"].split(":")[0])] += w
        di, imp = dist.get(r["Viaje"], (-1, "1"))
        if di > 0 and imp == "0": dl[d][g].append((di, w))
        t = num(r["TiempoViaje"])
        if 0 < t <= 300: tt[d][g].append((t, w))
def wmed(v):
    v.sort(); s = sum(w for _, w in v); a = 0
    for x, w in v:
        a += w
        if a >= s/2: return x
for d in DAYS:
    T = sum(tot[d].values()); NR = T - tot[d]["volver"]
    print(f"\n### {d} total viajes {T:,.0f} (muestra {sum(n[d].values())}); sin volver {NR:,.0f}")
    for g in ["trabajo","estudio","compras","recreacion","visita","comer","otros","volver"]:
        m = mode[d][g]; s = sum(m.values())
        h = hour[d][g]; hs = sum(h.values()); pk = max(h, key=h.get) if h else None
        print(f"{g:11s} n={n[d][g]:5d} exp={tot[d][g]:10,.0f} %todos={100*tot[d][g]/T:5.1f} %sinvolver={100*tot[d][g]/NR if g!='volver' else 0:5.1f} | " +
              " ".join(f"{k}={100*m[k]/s:.0f}" for k in ["auto","publico","caminata","bici","otro"]) +
              f" | pico {pk}h {100*h[pk]/hs if pk is not None else 0:.0f}% 10-14h={100*sum(h[x] for x in range(10,14))/hs:.0f}% 15-20h={100*sum(h[x] for x in range(15,20))/hs:.0f}% | dist_med_km={wmed(dl[d][g])/1000 if dl[d][g] else 0:.1f} t_med={wmed(tt[d][g])}")
print()
for d in DAYS:
    pub = sum(mode[d][g]["publico"] for g in mode[d]); T = sum(tot[d].values())
    print(d, "publico", round(pub), "share", round(pub/T,3), "ocio+compras+visita sin volver", round(100*(tot[d]["recreacion"]+tot[d]["compras"]+tot[d]["visita"]+tot[d]["comer"])/(T-tot[d]["volver"]),1))
