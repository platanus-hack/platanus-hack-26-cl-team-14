"""Print the two create_replay calls for the fixed-camera ferry shot at MAP07."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2] / "context/research/mapocho"
legs = json.loads((ROOT / "water-route.geojson").read_text())["features"]
lines = json.loads((ROOT / "boat_line.json").read_text())
points = legs[0]["geometry"]["coordinates"].copy()
for leg in legs[1:]:
    points.extend(leg["geometry"]["coordinates"][1:])
minutes = [leg["properties"]["ride_minutes_at_17_kmh"] for leg in legs]
for line in lines:
    east = line["target"]["id"] == "MAPOCHO-E"
    line["geometry"] = {"type": "LineString", "coordinates": points if east else points[::-1]}
    line["properties"]["segment_minutes"] = minutes if east else minutes[::-1]

focus = {"longitude": -70.63556, "latitude": -33.43586}
for i, (case, changes) in enumerate([("base", []), ("ferry", lines)], 1):
    captions = [
        (0, "Un ferry se acerca a Pío Nono."),
        (9, "Unos pasajeros bajan y otros suben."),
        (17, "{{line_riders.0.riders_to_work}} viajes al trabajo hacia el este."),
    ] if changes else [(0, "El Mapocho sin ferry.")]
    args = {
        "id": "mapocho-muelle-" + case,
        "title": "Muelle MAP07 · " + ("con ferry" if changes else "sin ferry"),
        "question": "¿Cómo llegarían las personas al trabajo si hubiera un ferry en el Mapocho?",
        "scene": {"place": {"type": "point", **focus}, "half_hour": 14},
        "duration_half_hours": 6,
        "analyze": "Viajes al trabajo por sentido y movimiento ilustrativo en el muelle.",
        "changes": changes,
        "cues": [
            {"after_seconds": seconds, **focus, "distance": 155, "caption": caption}
            for seconds, caption in captions
        ],
    }
    if changes:
        args["chart"] = {
            "title": "Viajes al trabajo en barco", "unit": "viajes modelados por día hábil",
            "show_after_seconds": 17,
            "bars": [{"label": label, "field": f"line_riders.{index}.riders_to_work"}
                     for index, label in enumerate(["Hacia el este", "Hacia el oeste"])],
        }
    print(json.dumps({"jsonrpc": "2.0", "id": i, "method": "tools/call",
                      "params": {"name": "create_replay", "arguments": args}}, ensure_ascii=False))
