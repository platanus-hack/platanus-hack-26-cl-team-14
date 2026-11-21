# Local Costanera viewer

Use Rust1.97.1 or later. Build the separate binary:

```sh
CARGO_TARGET_DIR=/tmp/costanera-view-target cargo build --locked --manifest-path viz/Cargo.toml --bin view_costanera -j1
```

The operator opens the captured bundle:

```sh
/tmp/costanera-view-target/debug/view_costanera /absolute/path/to/costanera
```

Left drag orbits. Right drag, middle drag, or Shift-left drag pans. Scroll zooms.
The first view looks toward the tower from the southeast. The camera target is
120 metres above the footprint origin. No city simulation systems run.

## Bundle contract

`manifest.json` contains `tiles`, `attribution`, and `google_logo`.
Each tile has a relative `path` to a local GLB and a column-major `transform`
with 16 double-precision values. Capture composes inherited tile transforms,
the glTF Y-up axis conversion, and Earth-to-local conversion before writing it.
The resulting coordinates are metres: X east, Y up, Z south. The anchor uses
the tower footprint center and the existing terrain elevation.

The viewer casts only the local placement matrix to rendering precision.
It preserves each GLB node hierarchy and disables additional axis rotation.
GLB node translations must also remain local. Capture must remove Draco
compression and keep geometry, materials, textures, and external resources local.
The logo path is relative to the bundle. Attribution strings appear together
on a persistent white bar. The viewer makes no network requests and reads no API key.

## Research and manual gate

The crate pins Bevy0.20.1. Context7 resolved that exact version before this work.
The [official GLB example](https://github.com/bevyengine/bevy/blob/v0.20.1/examples/gltf/load_gltf.rs)
uses `WorldAssetRoot` with `GltfAssetLabel::Scene(0)`. The
[coordinate conversion source](https://github.com/bevyengine/bevy/blob/v0.20.1/crates/bevy_gltf/src/convert_coordinates.rs)
documents optional scene and mesh rotation. Both remain disabled because capture
has already composed the required axes. The
[loader extension table](https://github.com/bevyengine/bevy/blob/v0.20.1/crates/bevy_gltf/src/lib.rs#L91)
does not support Draco compression, so capture decodes it before display.

First open one real decoded tile. Confirm orientation, texture attachment,
and visible attribution. Then capture the full area. The operator must check
the tower and mall, level overlap, orbit/zoom stability, and offline reopening
without an API key. No visual acceptance is implied by a successful build.
The implementation worker does not open the native window.
