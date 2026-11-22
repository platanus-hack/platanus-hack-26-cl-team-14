"""Health establishments of the Región Metropolitana that patients visit, from the official DEIS registry.

Source: DEIS, Establecimientos de Salud Vigentes, CSV of 17 November 2026
(https://datos.gob.cl/dataset/establecimientos-de-salud-vigentes, establecimientos_20261117.csv,
SHA-256 febff55e3612fb60076df9279f1090240cd02a39e9c7b79b822119dfdef84c8d).
Kept: RegionCodigo 13, EstadoFuncionamiento "Vigente…", with coordinates, and not one of the types without a
patient visit (remote care, SAMU regulation, mobile unit, "Otro"). kind = DEIS TipoEstablecimientoGlosa.
Output: data/mvp/health_points.csv, same columns as activity_points.csv (cut = DEIS ComunaCodigo).
Run: python3 -I tools/pack/deis_health_points.py establecimientos_20261117.csv
"""
import csv
import sys

SKIP = {"Atención Remota", "Centro de Regulación Médica de las Urgencias (SAMU)", "Unidad de Procedimientos Móvil", "Otro"}
rows = csv.DictReader(open(sys.argv[1], encoding="utf-8"), delimiter=";")
with open("data/mvp/health_points.csv", "w", newline="") as f:
    w = csv.writer(f)
    w.writerow(["kind", "name", "longitude", "latitude", "cut", "mapped_area_m2", "building_levels", "osm_id"])
    for r in rows:
        if r["RegionCodigo"] == "13" and r["EstadoFuncionamiento"].startswith("Vigente") and r["Latitud"] and r["TipoEstablecimientoGlosa"] not in SKIP:
            w.writerow([r["TipoEstablecimientoGlosa"], r["EstablecimientoGlosa"], r["Longitud"], r["Latitud"], r["ComunaCodigo"], "", "", "deis" + r["EstablecimientoCodigo"]])
