"""Pack proposed Metro stations, their design paths and published interchanges."""
import json
from collections import defaultdict
from pathlib import Path

import metro_planned_extensions
import metro_planned_l7
import metro_planned_lines

DEST = Path(__file__).resolve().parents[2] / "data/mvp/metro_planned.geojson"
# Existing station IDs are the GTFS parent IDs in web/data/transit_paths.json.
INTERCHANGES = {
    "L7-10": "CA-PCA", "L7-11": "BA-BQ", "L7-12": "PV",
    "L8-06": "MAC", "L8-11": "ESP", "L8-14": "LE-LEN",
    "L9-01": "CA-PCA", "L9-02": "SL", "L9-03": "MAT",
    "L9-05": "BIO", "L9-11": "SRO", "L9-19": "PPA",
}
EXTENSION_ANCHORS = {"L6-west-01": "CER", "L6-east-01": "LE-LEN"}


def main():
    features = (metro_planned_l7.features() + metro_planned_lines.features()
                + metro_planned_extensions.features())
    corridors = defaultdict(list)
    for feature in features:
        props = feature["properties"]
        if props["id"].startswith(("l8", "l9")):
            props["id"] = props["id"].upper()
        feature["id"] = props["id"]
        corridor = props.setdefault("corridor", props["line"])
        if props["kind"] == "station":
            corridors[corridor].append(props)
    for rows in corridors.values():
        rows.sort(key=lambda p: p["order"])
        ids = [p["id"] for p in rows]
        for i, props in enumerate(rows):
            props["neighbor_ids"] = ids[max(0, i - 1):i] + ids[i + 1:i + 2]
            station_id = props["id"]
            if station_id in EXTENSION_ANCHORS:
                props["existing_neighbor_id"] = EXTENSION_ANCHORS[station_id]
            if station_id in INTERCHANGES:
                props["gtfs_parent_station_id"] = INTERCHANGES[station_id]
    by_id = {f["id"]: f["properties"] for f in features}
    by_id["L6-east-01"]["planned_interchange_id"] = "L7-13"
    by_id["L7-13"]["planned_interchange_id"] = "L6-east-01"
    data = {"type": "FeatureCollection", "features": features}
    DEST.write_text(json.dumps(data, ensure_ascii=False, separators=(",", ":")) + "\n")


if __name__ == "__main__":
    main()
