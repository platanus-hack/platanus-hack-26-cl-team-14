"""Join the simulator supermarket subset to OSM tags and separate price references."""
import csv
import unicodedata
from collections import Counter, defaultdict
from decimal import Decimal, ROUND_HALF_UP
from pathlib import Path
from statistics import median

data = Path(__file__).resolve().parents[2] / "data/mvp"
output = data / "supermarket_prices"
inputs = output / "sources"
source = "https://download.geofabrik.de/south-america/chile/chile-261120.osm.pbf"
price_source = "https://lukisi.cl/"
odepa_source = "https://datos.odepa.gob.cl/dataset/precios-consumidor/resource/9f885df4-afeb-4b75-8bab-9334f79db00f"
cba_source = "https://observatorio.ministeriodesarrollosocial.gob.cl/storage/docs/cba/nueva_serie/2026/Informe_CBA_2026_10.pdf"


def normalize(text):
    simple = unicodedata.normalize("NFKD", text.casefold())
    return " ".join("".join(c for c in simple if not unicodedata.combining(c)).split())


aliases = {
    "lider": ("lider", "lider"),
    "lider express": ("lider", "lider_express"),
    "express de lider": ("lider", "lider_express"),
    "tottus": ("tottus", "tottus"),
    "tottus express": ("tottus", "tottus_express"),
    "jumbo": ("jumbo", "jumbo"),
    "santa isabel": ("santa_isabel", "santa_isabel"),
    "unimarc": ("unimarc", "unimarc"),
    "acuenta": ("acuenta", "acuenta"),
    "superbodega acuenta": ("acuenta", "acuenta"),
    "super bodega acuenta": ("acuenta", "acuenta"),
    "supermercado acuenta": ("acuenta", "acuenta"),
    "super 10": ("super10", "super10"),
    "super10": ("super10", "super10"),
    "alvi": ("alvi", "alvi"),
    "supermercado alvi": ("alvi", "alvi"),
    "alvi mayorista": ("alvi", "alvi"),
    "central mayorista": ("central_mayorista", "central_mayorista"),
    "mayorista 10": ("mayorista10_legacy", "mayorista10_legacy"),
    "supermercado mayorista 10": ("mayorista10_legacy", "mayorista10_legacy"),
    "ekono": ("ekono", "ekono"),
    "montserrat": ("montserrat", "montserrat"),
    "erbi": ("erbi", "erbi"),
    "supermercados erbi": ("erbi", "erbi"),
    "supermercado erbi": ("erbi", "erbi"),
    "supermercados erbi nucleo ochagavia": ("erbi", "erbi"),
    "mass": ("mass", "mass"),
    "ok market": ("ok_market", "ok_market"),
    "oxxo": ("oxxo", "oxxo"),
    "spid": ("spid", "spid"),
    "ahorra food depot": ("ahorra_food_depot", "ahorra_food_depot"),
    "la oferta": ("la_oferta", "la_oferta"),
}


def banner(tags):
    candidates = {}
    for tag in ("brand", "name", "operator"):
        value = normalize(tags.get(tag, ""))
        if tag == "name" and value.startswith("unimarc "):
            value = "unimarc"
        if value in aliases:
            candidates[tag] = aliases[value]
    if not candidates:
        status = "unknown_named" if any(tags.get(k) for k in ("name", "brand", "operator")) else "unknown_untagged"
        return "unknown", "unknown", status
    chains = {chain for chain, _ in candidates.values()}
    if chains == {"super10", "mayorista10_legacy"}:
        return "super10", "super10", "brand_with_legacy_name"
    if len(chains) > 1:
        return "unknown", "unknown", "conflicting_banner_tags"
    for tag in ("name", "brand", "operator"):
        pair = candidates.get(tag)
        if pair and pair[1] in {"lider_express", "tottus_express"}:
            return pair[0], pair[1], tag
    tag = next(iter(candidates))
    chain, fmt = candidates[tag]
    return chain, fmt, tag


with (inputs / "lukisi_food17_2026-11-19_to_21.csv").open(encoding="utf-8") as f:
    prices = defaultdict(dict)
    for r in csv.DictReader(f):
        key = aliases[normalize(r["cadena"])][1]
        product_url = r["url_producto"]
        assert product_url not in prices[key]
        assert r["cantidad_envases"] == "1"
        prices[key][product_url] = int(r["precio_clp"])
assert len(prices) == 6 and {len(goods) for goods in prices.values()} == {17}
assert len({frozenset(goods) for goods in prices.values()}) == 1
totals = {chain: sum(goods.values()) for chain, goods in prices.items()}
fallback = median(totals.values())

sector_prices = defaultdict(dict)
with (inputs / "odepa_cba28_2026w47_observations.csv").open(encoding="utf-8") as f:
    for r in csv.DictReader(f):
        key = r["Producto"], r["Unidad"]
        assert r["Sector"] not in sector_prices[key]
        sector_prices[key][r["Sector"]] = Decimal(r["Precio promedio"].replace(",", "."))

odepa_total = Decimal(0)
with (inputs / "odepa_cba28_2026w47.csv").open(encoding="utf-8") as f:
    for r in csv.DictReader(f):
        key = r["producto_observado_odepa"], r["unidad_precio_odepa"]
        prices_by_sector = sector_prices[key]
        assert len(prices_by_sector) == 5
        mean_per_unit = sum(prices_by_sector.values()) / len(prices_by_sector)
        monthly = mean_per_unit * Decimal(r["cantidad_persona_mes"]) / Decimal(r["g_o_ml_por_unidad_odepa"])
        subtotal = monthly.quantize(Decimal("0.01"), rounding=ROUND_HALF_UP)
        assert subtotal == Decimal(r["clp_modelado_persona_mes"])
        odepa_total += subtotal

with (data / "activity_points.csv").open(encoding="utf-8") as f:
    packed = [r for r in csv.DictReader(f) if r["kind"] == "supermarket"]
with (inputs / "osm_supermarket_tags.csv").open(encoding="utf-8") as f:
    objects = {r["osm_id"]: r for r in csv.DictReader(f)}
assert len(packed) == len(objects) == len({r["osm_id"] for r in packed})
assert objects.keys() == {r["osm_id"] for r in packed}

rows = []
for point in packed:
    osm_id = point["osm_id"]
    obj = objects[osm_id]
    assert obj["osm_shop"] == point["kind"] and obj["name"] == point["name"]
    chain, fmt, evidence = banner({
        "name": obj["name"], "brand": obj["osm_brand"], "operator": obj["osm_operator"],
    })
    exact = fmt in totals
    rows.append({
        **point,
        "osm_type": obj["osm_type"], "osm_version": obj["osm_version"],
        "osm_url": f"https://www.openstreetmap.org/{obj['osm_type']}/{osm_id[1:]}",
        "osm_snapshot_utc": "2026-11-20T21:21:21Z", "osm_snapshot_url": source,
        "osm_shop": obj["osm_shop"], "osm_brand": obj["osm_brand"],
        "osm_operator": obj["osm_operator"], "osm_brand_wikidata": obj["osm_brand_wikidata"],
        "chain_key": chain, "banner_key": fmt, "banner_evidence": evidence,
        "common_basket_id": "lukisi_food17_2026-11-19_to_21_one_pack_each",
        "common_basket_clp_per_basket": int(totals[fmt] if exact else fallback),
        "common_basket_basis": "chain_public_price_reference_exact_banner" if exact else "estimated_same_basket_chain_median",
        "common_basket_price_chain": fmt if exact else "",
        "common_basket_source": price_source,
        "odepa_rm_supermarket_cba28_clp_per_person_month": int(round(odepa_total)),
        "odepa_reference_period": "2026-W47",
        "odepa_reference_basis": "regional modeled subtotal, 28 categories, five sector means",
        "odepa_reference_source": odepa_source,
        "national_cba_clp_per_person_month": 93678,
        "national_cba_period": "2026-10",
        "national_cba_source": cba_source,
    })

with (output / "supermarket_points_prices.csv").open("w", encoding="utf-8", newline="") as f:
    writer = csv.DictWriter(f, fieldnames=list(rows[0]), lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)

sernac = defaultdict(dict)
with (inputs / "sernac_pantry9_2026-11-21.csv").open(encoding="utf-8") as f:
    for quote in csv.DictReader(f):
        cut = {"Santiago": "13101", "Maipú": "13119", "Puente Alto": "13201"}[quote["comuna"]]
        fmt = aliases[normalize(quote["chain"])][1]
        key = cut, quote["comuna"], fmt
        assert quote["spec"] not in sernac[key] and quote["packs"] == "1"
        sernac[key][quote["spec"]] = int(quote["unit_clp"])
assert {len(specs) for specs in sernac.values()} == {9}
assert len({frozenset(specs) for specs in sernac.values()}) == 1

point_counts = Counter((row["cut"], row["banner_key"]) for row in rows)
with (output / "sernac_pantry9_by_comuna.csv").open("w", encoding="utf-8", newline="") as f:
    writer = csv.writer(f, lineterminator="\n")
    writer.writerow(["cut", "comuna", "banner_key", "specs", "packs_per_spec",
                     "sernac_pantry9_clp_per_basket", "matched_osm_objects",
                     "eligibility_basis"])
    for (cut, comuna, fmt), specs in sorted(sernac.items()):
        eligibility = "merchant_and_clp_80000_online_minimum" if fmt == "alvi" else "catalog_checkout_unverified"
        writer.writerow([cut, comuna, fmt, len(specs), 1, sum(specs.values()),
                         point_counts[cut, fmt], eligibility])
