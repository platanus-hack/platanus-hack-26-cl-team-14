#!/usr/bin/env python3
"""Layer A: the fixed score of the simulation against real data (calm-condor).

Target: April 2026 average-weekday (LABORAL) Metro boardings per station half
hour, read from the merged data/mvp/metro_observations.json (PR #35). The
held-out membership below is FROZEN: it was derived once by rule (normalize
the 126 source names, drop the 3 model-input stations TOBALABA / LOS LEONES /
EL GOLF, sort, every 4th from index 0) and is now pinned by parent ID and
source hash, so later name or source edits cannot move it.

Score (lower is better), on the frozen 1,123 observed held-out cells:
  WAPE_bins  = sum |pred - obs| / sum obs   over held-out station x observed bins
  WAPE_daily = the same over per-station sums of those same observed cells
A depth layer improves the simulation when WAPE_bins falls. Eligibility: no
held-out station's boarding values (any day type) entered the model as input,
fitting data or profile shape; and every observed held-out cell has a finite,
nonnegative prediction. A missing, null, negative or nonfinite prediction on
an observed cell makes the run ineligible: coverage is reported and no score
is printed. The denominator never shrinks. Repeated runs against this holdout
are development validation, not an untouched final test.

Run: python3 -I tools/pack/check_score.py predictions.json
predictions.json: {"<normalized name or parent id>": [48 numbers or null]}.
Without an argument, prints the frozen manifest and score_ready=false.
"""
import json
import math
import sys

ARTIFACT = 'data/mvp/metro_observations.json'
MANIFEST = 'data/mvp/metro_observations_manifest.json'
# Frozen source provenance (PR #35): GTFS_20260321_v3.zip and the April export.
GTFS_SHA256 = 'f10945e80beebc39ea2ac70de997d3d69714897b575de6fbf4e3ca643013b77b'
BOARDINGS_SHA256 = '24da85944374d1b07ac2b8c5bf696279370e2ecd9d6dd1d76daca84b5dd26ec4'
# Frozen holdout: normalized source name -> GTFS parent ID. Membership is by ID.
HOLDOUT = {
    'ALCANTARA': 'AL', 'BELLAVISTA DE LA FLORIDA': 'LF', 'CAMINO AGRICOLA': 'AG',
    'CERRILLOS': 'CER', 'CONCHALI': 'CCH', 'DEL SOL': 'DS', 'EINSTEIN': 'EI',
    'ELISA CORREA': 'ECO', 'FERNANDO CASTILLO VELASCO': 'FCV', 'GRECIA': 'RGR',
    'HOSPITAL SOTERO DEL RIO': 'HSR', 'LA CISTERNA': 'LC-LCI', 'LAS MERCEDES': 'LME',
    'LO CRUZAT': 'LCR', 'LO VIAL': 'LV', 'LOS ORIENTALES': 'LOR', 'MANQUEHUE': 'MQ',
    'MONSENOR EYZAGUIRRE': 'MEY', 'NUNOA': 'NUN-NUO', 'PARQUE BUSTAMANTE': 'PB',
    'PEDRERO': 'PE', 'PLAZA DE PUENTE ALTO': 'PPA', 'PRINCIPE DE GALES': 'PDG',
    'QUINTA NORMAL': 'QN', 'RONDIZONNI': 'RO', 'SAN JOSE DE LA ESTRELLA': 'SJE',
    'SANTA ANA': 'AN-NA', 'SANTA ROSA': 'SRO', 'TRINIDAD': 'TRI',
    'UNIVERSIDAD DE SANTIAGO': 'US', 'VILLA FREI': 'VFR',
}
# Frozen observed-cell set and denominator; drift in the artifact aborts the score.
EXPECTED_CELLS = 1123
EXPECTED_TOTAL = 570369.4

manifest = json.load(open(MANIFEST))
if (manifest['sources']['gtfs']['sha256'] != GTFS_SHA256
        or manifest['sources']['exported_boardings']['sha256'] != BOARDINGS_SHA256):
    sys.exit('ABORT: source hashes changed; the frozen holdout no longer applies.')

obs = {}  # parent id -> 48 LABORAL values (None where unobserved)
names = {}
for s in json.load(open(ARTIFACT))['stations']:
    pid = s['station']['id']
    if pid in HOLDOUT.values():
        obs[pid] = s['half_hour_bins']['LABORAL']
        names[pid] = s['source_name']
cells = sum(1 for b in obs.values() for v in b if v is not None)
total = sum(v for b in obs.values() for v in b if v is not None)
if sorted(obs) != sorted(HOLDOUT.values()) or cells != EXPECTED_CELLS \
        or round(total, 1) != EXPECTED_TOTAL:
    sys.exit(f'ABORT: artifact drift: {len(obs)} stations, {cells} cells, '
             f'{total:,.1f} (frozen: 31 / {EXPECTED_CELLS} / {EXPECTED_TOTAL:,})')
print(f'Frozen holdout verified: 31 stations, {cells} observed cells, '
      f'{total:,.1f} boardings/day.')

if len(sys.argv) < 2:
    print('score_ready=false: pass predictions.json to compute WAPE.')
    sys.exit(0)

raw = json.load(open(sys.argv[1]))
pred = {}
for key, bins in raw.items():
    pid = HOLDOUT.get(key, key)
    pred[pid] = bins
bad = []
for pid in HOLDOUT.values():
    p = pred.get(pid)
    if p is None or len(p) != 48:
        bad.append(f'{pid} ({names[pid]}): missing or not 48 bins')
        continue
    for i in range(48):
        if obs[pid][i] is None:
            continue
        v = p[i]
        if not isinstance(v, (int, float)) or not math.isfinite(v) or v < 0:
            bad.append(f'{pid} bin {i}: prediction {v!r}')
if bad:
    covered = EXPECTED_CELLS - sum(1 for b in bad if 'bin' in b)
    print(f'INELIGIBLE: incomplete prediction; coverage {covered}/{EXPECTED_CELLS} '
          f'observed cells. No score. First problems:')
    print('\n'.join(bad[:10]))
    sys.exit(1)

num = den = dnum = dden = 0.0
for pid in HOLDOUT.values():
    pd = od = 0.0
    for i in range(48):
        o = obs[pid][i]
        if o is None:
            continue
        num += abs(pred[pid][i] - o)
        den += o
        pd += pred[pid][i]
        od += o
    dnum += abs(pd - od)
    dden += od
print(f'WAPE_bins  = {num / den:.4f}')
print(f'WAPE_daily = {dnum / dden:.4f}')
print('Reminder: eligibility also requires that no held-out boardings entered '
      'the model; that is a process claim the modeler must state.')
