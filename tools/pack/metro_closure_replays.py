"""Print the two create_replay calls for the timed Metro closure."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
changes = json.loads((ROOT / "context/research/metro_closure_window.json").read_text())
stations = json.loads((ROOT / "data/mvp/metro_training.json").read_text())
tobalaba = next(station for station in stations if station["id"] == "TB-TOB")
focus = {axis: tobalaba[axis] for axis in ("longitude", "latitude")}
scene = {
    "place": {"type": "station", "id": tobalaba["id"]},
    "half_hour": 16,
    "layer": "metro_service",
}
question = "¿Qué pasaría si cerráramos todo el Metro de Santiago entre las 7 y las 10 de la mañana de un día hábil?"
mode_path = "closure_window.workers_by_mode_"

for request_id, (case, title, edits) in enumerate(
    [("baseline", "Metro operativo", []), ("closure", "Metro cerrado", changes)], 1
):
    service = "cerrado" if edits else "abierto"
    args = {
        "id": f"metro-window-{case}",
        "title": title,
        "question": question,
        "scene": scene,
        "stations": [tobalaba["id"]],
        "changes": edits,
        "duration_half_hours": 4,
        "analyze": "Salidas al trabajo por modo de 07:00 a 10:00. El visor empieza a las 08:00.",
        "cues": [
            {
                "after_seconds": 0,
                "longitude": -70.6505,
                "latitude": -33.443,
                "distance": 18000,
                "caption": f"A las 08:00 el Metro está {service}.",
            },
            {"after_seconds": 5, **focus, "distance": 360},
        ],
    }
    if edits:
        args["cues"][1]["caption"] = (
            "Auto sube de {{closure_window.workers_by_mode_base.car}} "
            "a {{closure_window.workers_by_mode_scenario.car}}."
        )
        args["chart"] = {
            "title": "Cómo salen al trabajo",
            "unit": "salidas modeladas, 07:00–10:00",
            "show_after_seconds": 12,
            "keys": {
                "car": "Auto", "public": "Transporte público", "walk": "A pie",
                "bicycle": "Bicicleta", "other": "Otro",
            },
            "bars": [
                {"label": "Antes", "field": mode_path + "base"},
                {"label": "Cierre", "field": mode_path + "scenario"},
            ],
        }
        args["comparison"] = {
            "fields": [mode_path + "scenario.public"],
            "unit": "salidas modeladas en transporte público · 07:00–10:00",
        }
    print(json.dumps({
        "jsonrpc": "2.0", "id": request_id, "method": "tools/call",
        "params": {"name": "create_replay", "arguments": args},
    }, ensure_ascii=False))
