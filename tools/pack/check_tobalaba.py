#!/usr/bin/env python3
"""Critic check: recompute Tobalaba Metro boardings from the raw DTPM workbook.

Run: python3 -I tools/pack/check_tobalaba.py
Compares against claims in the MVP thread and data/mvp/tobalaba_observations.json.
"""
import zipfile, re, sys, json, subprocess
from collections import defaultdict

XLSX = 'data/raw/dtpm/subidas_2026_04.xlsx'

z = zipfile.ZipFile(XLSX)
shared = re.findall(r'<si>(?:<t[^>]*>)?(.*?)(?:</t>)?</si>',
                    z.read('xl/sharedStrings.xml').decode('utf-8'), re.S)
shared = [re.sub(r'<.*?>', '', s) for s in shared]

# find the data sheet: the one with the most rows
best, best_rows = None, 0
for name in ('sheet1', 'sheet2', 'sheet3', 'sheet4'):
    data = z.read(f'xl/worksheets/{name}.xml').decode('utf-8')
    n = data.count('<row ')
    if n > best_rows:
        best, best_rows, best_data = name, n, data
print(f'data sheet: {best}, rows: {best_rows}')

rows = []
for rm in re.finditer(r'<row [^>]*>(.*?)</row>', best_data, re.S):
    cells = {}
    for cm in re.finditer(r'<c r="([A-Z]+)\d+"(?: s="\d+")?(?: t="(\w+)")?[^>]*>(?:<v>(.*?)</v>)?', rm.group(1)):
        cref, typ, val = cm.groups()
        if val is None:
            continue
        cells[cref] = shared[int(val)] if typ == 's' else val
    rows.append(cells)

header = rows[0]
print('header:', header)
cols = {v: k for k, v in header.items()}
sys.stdout.flush()

# locate columns by name fragments
def col(frag):
    for name, c in cols.items():
        if frag.lower() in name.lower():
            return c
    raise KeyError(frag)

c_mode, c_day, c_stop, c_half, c_board = (col('modo'), col('tipo'),
                                          col('parader'), col('media'), col('subida'))
tot = defaultdict(float)
half = defaultdict(float)
nrec = 0
for r in rows[1:]:
    if r.get(c_mode, '').upper() != 'METRO':
        continue
    if 'TOBALABA' not in r.get(c_stop, '').upper():
        continue
    day = r.get(c_day, '')
    b = float(r.get(c_board, 0))
    tot[day] += b
    nrec += 1
    if day.upper().startswith('LABORAL'):
        half[r.get(c_half)] += b

print('Tobalaba Metro records:', nrec)
for d, v in sorted(tot.items()):
    print(f'  {d}: {v:,.1f}')
top = sorted(half.items(), key=lambda x: -x[1])[:3]
print('top weekday half hours:', [(h, round(v, 1)) for h, v in top])

# compare with the merged observation layer
obs = json.loads(subprocess.run(['git', 'show', 'origin/main:data/mvp/tobalaba_observations.json'],
                                capture_output=True, text=True).stdout)
print('json records:', len(obs['observations']))
jt = defaultdict(float)
for o in obs['observations']:
    jt[o['day_type']] += o['boardings']
for d, v in sorted(jt.items()):
    print(f'  json {d}: {v:,.1f}')
print('station:', obs['station'])
