#!/usr/bin/env python3
"""Layer A: the fixed score of the simulation against real data (calm-condor).

Target: April 2026 average-weekday (LABORAL) Metro boardings per station half hour,
from data/raw/dtpm/subidas_2026_04.xlsx. The held-out split is frozen by a
deterministic rule, not by choice: normalize each Metro station name (upper, no
accents, single spaces), drop TOBALABA, LOS LEONES and EL GOLF (already model
inputs/demo stations), sort, take every 4th name starting at index 0.

Score (lower is better), on held-out stations only:
  WAPE_bins   = sum |pred - obs| / sum obs   over station x observed LABORAL bins
  WAPE_daily  = the same over station daily totals
A depth layer improves the simulation when WAPE_bins falls. A prediction is
eligible only if no held-out station's boarding values (any day type) entered
the model as input, fitting data or profile shape.

Run: python3 -I tools/pack/check_score.py [predictions.json]
predictions.json: {"<normalized station name>": [48 numbers or null]}.
Without it, prints the frozen holdout manifest and score_ready=false.
"""
import json
import re
import sys
import unicodedata
import zipfile
from collections import defaultdict

XLSX = 'data/raw/dtpm/subidas_2026_04.xlsx'
USED = {'TOBALABA', 'LOS LEONES', 'EL GOLF'}  # inputs/demo; never held out


def normalize(name):
    s = unicodedata.normalize('NFD', name.upper())
    s = ''.join(c for c in s if unicodedata.category(c) != 'Mn')
    return re.sub(r'\s+', ' ', s).strip()


z = zipfile.ZipFile(XLSX)
shared = re.findall(r'<si>(?:<t[^>]*>)?(.*?)(?:</t>)?</si>',
                    z.read('xl/sharedStrings.xml').decode('utf-8'), re.S)
shared = [re.sub(r'<.*?>', '', s) for s in shared]
best_rows, best_data = 0, ''
for name in ('sheet1', 'sheet2', 'sheet3', 'sheet4'):
    try:
        data = z.read(f'xl/worksheets/{name}.xml').decode('utf-8')
    except KeyError:
        continue
    if data.count('<row ') > best_rows:
        best_rows, best_data = data.count('<row '), data

rows = []
for rm in re.finditer(r'<row [^>]*>(.*?)</row>', best_data, re.S):
    cells = {}
    for cm in re.finditer(
            r'<c r="([A-Z]+)\d+"(?: s="\d+")?(?: t="(\w+)")?[^>]*>(?:<v>(.*?)</v>)?',
            rm.group(1)):
        cref, typ, val = cm.groups()
        if val is not None:
            cells[cref] = shared[int(val)] if typ == 's' else val
    rows.append(cells)

cols = {v: k for k, v in rows[0].items()}
col = lambda frag: next(c for n, c in cols.items() if frag.lower() in n.lower())
c_mode, c_day, c_stop = col('modo'), col('tipo'), col('parader')
c_half, c_board = col('media'), col('subida')

obs = defaultdict(lambda: [None] * 48)  # station -> 48 LABORAL bins
for r in rows[1:]:
    if r.get(c_mode, '').upper() != 'METRO' or r.get(c_day, '') != 'LABORAL':
        continue
    st = normalize(r[c_stop])
    i = round(float(r[c_half]) * 48)  # Media_hora is an Excel day fraction
    obs[st][i] = (obs[st][i] or 0.0) + float(r[c_board])

stations = sorted(obs)
holdout = sorted(set(s for k, s in enumerate(sorted(set(stations) - USED)) if k % 4 == 0))
print(f'Metro stations with LABORAL observations: {len(stations)}')
print(f'Frozen holdout ({len(holdout)}): {", ".join(holdout)}')
daily = {s: sum(v for v in obs[s] if v) for s in holdout}
print(f'Holdout daily total: {sum(daily.values()):,.1f} boardings')

if len(sys.argv) < 2:
    print('score_ready=false: pass predictions.json to compute WAPE.')
    sys.exit(0)

pred = json.load(open(sys.argv[1]))
num = den = dnum = dden = 0.0
for s in holdout:
    p = pred.get(s)
    if p is None:
        print(f'MISSING prediction: {s}')
        sys.exit(1)
    pd = od = 0.0
    for i in range(48):
        if obs[s][i] is None:
            continue
        pv = p[i] or 0.0
        num += abs(pv - obs[s][i]); den += obs[s][i]
        pd += pv; od += obs[s][i]
    dnum += abs(pd - od); dden += od
print(f'WAPE_bins  = {num / den:.4f}')
print(f'WAPE_daily = {dnum / dden:.4f}')
