#!/usr/bin/env python3
"""Stage the landing and browser simulator for Workers Static Assets."""
import gzip
import json
import mimetypes
from pathlib import Path
import shutil
import subprocess
import tempfile


LANDING = Path(__file__).resolve().parent
ROOT = LANDING.parent
PUBLIC = Path("landing/public")
VIEWER = Path("web/wasm/viewer_bg.wasm")
ASSET_LIMIT = 25 * 1024**2
PART_SIZE = 16 * 1024**2
GENERATED = (
    VIEWER,
    *(Path("web/wasm", name) for name in (
        "baseline.json",
        "building-polygons-startup.bin.gz",
        "car-topology.bin.gz",
        "engine.wasm",
        "folders.json",
        "road-surface-deferred.bin",
        "road-surface.bin.gz",
        "viewer.js",
    )),
)
PUBLIC_PATHS = (
    PUBLIC,
    Path("web/data"),
    Path("web/wasm"),
    Path("data/mvp"),
    Path("data/replays"),
    Path("data/validation/event_shape_2026_04/manifest.json"),
    Path("tools/santiago-mcp"),
    Path("viz/assets/barlow"),
)


def tracked_files():
    files = subprocess.check_output(
        ["git", "ls-files", "-z", "--", *PUBLIC_PATHS], cwd=ROOT
    )
    return [
        Path(name.decode()) for name in files.split(b"\0")
        if name and not Path(name.decode()).name.startswith(".")
    ]


def pack_file(source, dest, route):
    size = source.stat().st_size
    if size <= ASSET_LIMIT:
        target = dest / route
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
        return None

    directory = Path("_simulator-assets") / route
    (dest / directory).mkdir(parents=True)
    mime = "application/gzip" if route.suffix == ".gz" else mimetypes.guess_type(route)[0]
    entry = {"type": mime or "application/octet-stream", "bytes": size, "parts": []}
    with source.open("rb") as data:
        for index, chunk in enumerate(iter(lambda: data.read(PART_SIZE), b"")):
            part = directory / str(index)
            (dest / part).write_bytes(chunk)
            entry["parts"].append(f"/{part}")
    if route == VIEWER:
        compressed = directory / "compressed"
        with (dest / compressed).open("wb") as file, source.open("rb") as data:
            with gzip.GzipFile(filename="", mode="wb", fileobj=file, compresslevel=6, mtime=0) as encoded:
                shutil.copyfileobj(data, encoded)
        if (dest / compressed).stat().st_size <= ASSET_LIMIT:
            entry["gzip"] = f"/{compressed}"
        else:
            (dest / compressed).unlink()
    return entry


def main():
    files = {*tracked_files(), *GENERATED}
    missing = [path for path in sorted(files) if not (ROOT / path).is_file()]
    if missing:
        raise SystemExit("Missing deploy files. Run tools/wasm/build.sh first.\n" + "\n".join(map(str, missing)))

    with tempfile.TemporaryDirectory(prefix=".stage-", dir=LANDING) as temp:
        stage = Path(temp)
        dest = stage / "dist"
        dest.mkdir()
        manifest = {}
        for path in sorted(files):
            route = path.relative_to(PUBLIC) if path.is_relative_to(PUBLIC) else path
            entry = pack_file(ROOT / path, dest, route)
            if entry:
                manifest[f"/{route}"] = entry

        viewer_files = json.loads((ROOT / "web/wasm/viewer-files.json").read_text())
        for name in viewer_files:
            if not (dest / name).is_file() and f"/{name}" not in manifest:
                raise SystemExit(f"Viewer asset missing from deployment: {name}")

        (stage / "large-assets.json").write_text(json.dumps(manifest, indent=2) + "\n")
        previous = LANDING / "dist"
        if previous.exists():
            previous.replace(stage / "old-dist")
        dest.replace(previous)
        (stage / "large-assets.json").replace(LANDING / "large-assets.json")

    print(f"Staged {len(files)} files. Split {len(manifest)} files above 25 MiB.")


if __name__ == "__main__":
    main()
