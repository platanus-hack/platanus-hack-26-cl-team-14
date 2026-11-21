# Local demo tile viewer

Use Rust1.97.1 or later. Build the separate binary:

```sh
CARGO_TARGET_DIR=/tmp/costanera-view-target cargo build --locked --manifest-path viz/Cargo.toml --bin view_demo_tiles -j1
```

The operator opens the captured bundle:

```sh
RUST_MIN_STACK=16777216 /tmp/costanera-view-target/debug/view_demo_tiles /absolute/path/to/demo-bundle
```

Use the 16 MiB thread-stack setting shown above for the full bundle. The tested
MUT launch with the default stack aborted with an IO Task Pool stack overflow.
The same binary and bundle opened with `RUST_MIN_STACK=16777216`. This is a
measured workaround; the exact failing function and root cause are unconfirmed.

Left drag orbits. Right drag, middle drag, or Shift-left drag pans. Scroll zooms.
The default view looks from the southeast. The camera target is
120 metres above the capture origin. No city simulation systems run.
For the small first-tile gate, add `--distance 100 --target-y 20`.
These options change only the camera. The full-scene defaults remain 850 and 120 metres.
The camera uses no tone-mapping lookup texture.

## Bundle contract

`manifest.json` contains `tiles`, `attribution`, and `google_logo`.
Each tile has a relative `path` to a local GLB and a column-major `transform`
with 16 double-precision values. Capture composes inherited tile transforms,
the glTF Y-up axis conversion, and Earth-to-local conversion before writing it.
The resulting coordinates are metres: X east, Y up, Z south. The anchor uses
the selected site center and the existing terrain elevation.

The viewer casts only the local placement matrix to rendering precision.
It preserves each GLB node hierarchy and disables additional axis rotation.
GLB node translations must also remain local. The capture implementation bakes
the composed placement into scene-root matrices in double precision and writes
an identity outer manifest matrix. It keeps the hierarchy below those roots.
This avoids cancellation of Earth-scale translations after Bevy loads the nodes.
Capture must remove Draco
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
and visible attribution. Then capture the full selected area. The operator must check
the chosen demo site, level overlap, orbit/zoom stability, and offline reopening
without an API key. No visual acceptance is implied by a successful build.
The implementation worker does not open the native window.

## Observed MUT result

At source head `85b9def9fbad92acf9ffe00762b2b415258b6950`, the capture produced
995 MUT tiles: 51,610,382 downloaded bytes and 47,509,360 prepared GLB bytes.
Capture took 378.46 seconds. These numbers do not measure viewer load time or
frame rate. The selected first tile had 5,963 vertices and 4,963 triangles.

The operator observed a native frame with textured geometry from the full
995-tile bundle and visible credits after the 16 MiB stack launch. This frame
does not establish venue recognition, correct orientation, level overlap,
working controls, offline reopening, or frame rate. Those checks remain open.
Monumental capture and native checks remain open. No automated tests ran.

## Selection correction and saved-data replay

The original horizontal-only selection also admitted far-side Earth geometry.
A far-side tile can project into the same east/south square while its local
height is about −12.7 million metres. The earlier native MUT frame therefore
does not establish valid placement or complete local coverage.

Capture now rejects a box only when its entire upper Y extent is below the
Earth-centre plane in the anchor frame. Coarse parents that cross that plane
remain eligible. Sphere support uses the affine matrix row lengths, so scale
and shear cannot make the new hemisphere bound too small. GLB axes, root
placement, and ADD/REPLACE selection are unchanged.

All saved Google volumes are boxes: 7,465 nodes for Monumental and 7,931 for
MUT. For curved region volumes, capture skips the new hemisphere cull because
corner samples can miss an interior extremum. The existing region X/Z corner
approximation remains an unexercised generic-format limit. This work does not
claim general region correctness.

A read-only manual replay of the saved traversal, without downloads, retained
268 of 650 Monumental tiles and 595 of 995 MUT tiles. It rejected the exact
far-side example, retained a near-side example, and retained the global parent
that crosses the Earth-centre plane. All retained GLB bytes and hashes stayed
unchanged. Actual retained vertex bounds, in metres, are:

| Site | X | Y | Z |
| --- | --- | --- | --- |
| Monumental | −262.64 to 280.67 | 17.67 to 73.30 | −261.79 to 272.30 |
| MUT | −265.48 to 278.38 | −32.79 to 107.31 | −262.59 to 271.50 |

Whole edge tiles can extend beyond the requested square. Retained geometry
has no Earth-scale coordinates in this audit. The replay evaluates the new
predicate against saved data; it is not execution of the corrected Rust binary.
Sphere arithmetic was checked manually for far-side, crossing, near-side and
sheared examples. No real sphere volume was present. The correction has not
yet been built or captured. New native and denied-network observations remain
pending. The original bundles and capture statistics were preserved.
