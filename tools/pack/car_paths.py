"""Route cars between comuna reference points on major roads, at assumed free-flow speeds."""
import csv
import heapq
from collections import defaultdict

from roads import PACK, distance, read_roads, rounded_path, write_json

# Published OSRM profile speeds, in km/h. Mapped speed limits cap each direction.
SPEEDS = {"motorway": 90, "motorway_link": 45, "trunk": 85, "trunk_link": 40,
          "primary": 65, "primary_link": 30, "secondary": 55, "secondary_link": 25}


def car_graph(points, ways):
    graph = defaultdict(list)
    anchors = set()
    for _, tags, nodes in ways:
        access = next((tags[k] for k in ("motorcar", "motor_vehicle", "vehicle", "access") if k in tags), "yes")
        if access in {"no", "private", "permit", "destination"}:
            continue
        kind = tags["highway"]
        implied = "yes" if kind == "motorway" or tags.get("junction") == "roundabout" else "no"
        oneway = tags.get("oneway", implied)
        if oneway in {"reversible", "alternating"}:
            continue
        if kind in {"primary", "secondary"}:
            anchors.update(nodes)
        for direction in ("forward", "backward"):
            if direction == "forward" and oneway == "-1":
                continue
            if direction == "backward" and oneway in {"yes", "1", "true"}:
                continue
            limit = tags.get("maxspeed:" + direction, tags.get("maxspeed", ""))
            speed = min(SPEEDS[kind], int(limit)) if limit.isdecimal() else SPEEDS[kind]
            ordered = nodes if direction == "forward" else nodes[::-1]
            for a, b in zip(ordered, ordered[1:]):
                metres = distance(points[a], points[b])
                graph[a].append((b, metres * 3.6 / speed, metres))
    return graph, anchors


def route(graph, origin, targets):
    previous, elapsed, metres = {}, {origin: 0}, {origin: 0}
    queue = [(0, origin)]
    remaining = set(targets)
    while queue and remaining:
        time, node = heapq.heappop(queue)
        if time != elapsed[node]:
            continue
        remaining.discard(node)
        for dest, seconds, length in graph.get(node, ()):
            new_time = time + seconds
            if new_time < elapsed.get(dest, float("inf")):
                previous[dest] = node
                elapsed[dest] = new_time
                metres[dest] = metres[node] + length
                heapq.heappush(queue, (new_time, dest))
    return previous, elapsed, metres


def main():
    points, ways = read_roads()
    graph, anchors = car_graph(points, ways)
    anchors = sorted(anchors)
    snapped, gaps = {}, {}
    with (PACK / "communes.csv").open() as f:
        for row in csv.DictReader(f):
            pos = (float(row["longitude"]), float(row["latitude"]))
            anchor = min(anchors, key=lambda node: distance(pos, points[node]))
            cut = row["cut"]
            snapped[cut], gaps[cut] = anchor, round(distance(pos, points[anchor]))
    paths = []
    with (PACK / "car_minutes.csv").open("w", newline="") as f:
        out = csv.writer(f)
        out.writerow(["origin_cut", "destination_cut", "car_minutes", "distance_km",
                      "origin_snap_m", "destination_snap_m"])
        for origin, start in snapped.items():
            previous, times, lengths = route(graph, start, snapped.values())
            for dest, end in snapped.items():
                if origin == dest or end not in times:
                    continue
                nodes = [end]
                while nodes[-1] != start:
                    nodes.append(previous[nodes[-1]])
                path = rounded_path([points[n] for n in reversed(nodes)], 20)
                paths.append({"origin_cut": origin, "destination_cut": dest, "path": path})
                out.writerow([origin, dest, round(times[end] / 60, 2), round(lengths[end] / 1000, 3),
                              gaps[origin], gaps[dest]])
    output = PACK / "car_paths.json"
    write_json(output, paths)
    print(f"{len(paths)} directed pairs, {output.stat().st_size} path bytes")


if __name__ == "__main__":
    main()
