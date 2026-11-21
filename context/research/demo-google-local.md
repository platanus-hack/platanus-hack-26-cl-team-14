# Local Google scenes for the demo

Human scope: MUT and Estadio Monumental only. The human withdrew Costanera. Open the selected demo scenes only on Vicente's Mac. Other demo sites require confirmation from the transcript. Do not integrate simulation entities.

## Permission and sources

The human brief states permission for storage and local use. This is an explicit project assumption. Standard Map Tiles policy does not grant offline storage permission.

Sources read on 21 November 2026:

- [Google renderer guide](https://developers.google.com/maps/documentation/tile/create-renderer): retain child session parameters and append the API key. Aggregate tile copyrights by frequency.
- [Google Tiles overview](https://developers.google.com/maps/documentation/tile/3d-tiles): photorealistic geometry, textures and root tileset entry point.
- [Google policies and official logo assets](https://developers.google.com/maps/documentation/tile/policies): keep the logo and data credits visible. The included PNG comes unchanged from the official attribution ZIP.
- [draco-gltf Import API](https://docs.rs/draco-gltf/0.8.1/draco_gltf/struct.Import.html): read compressed primitives, write ordinary geometry, preserve the document and resources.
- [glTF Draco extension](https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_draco_mesh_compression/README.md): decode compressed attributes before use.
- [draco-gltf implementation](https://github.com/Filyus/draco-rust/blob/main/crates/draco-gltf/src/writer.rs): geometry writes retain materials and unrelated primitive fields. The writer removes the Draco declarations after the last compressed primitive.

Context7 resolved the Google Maps JavaScript and glTF libraries. It had no matching Map Tiles or draco-gltf entry. The browser component is outside this brief. Direct official documentation supplies the tile contract.

## Capture

Build with Rust 1.97.1:

```sh
cargo build --locked --manifest-path tools/demo-tiles-capture/Cargo.toml -j1
```

Pass a new output path. Supply the key through `GOOGLE_MAPS_API_KEY`. Do not put a key in the command arguments.

```sh
tools/demo-tiles-capture/target/debug/capture_demo_tiles /tmp/vicente-mut-one --site mut --one-tile
```

One-tile mode ranks bounded candidates by proximity. It skips fragments with fewer than 1000 vertices. This sample rule does not prove full coverage. First render the selected tile. Check decoded geometry, textures, node translations and orientation. Then capture the full square into a different new directory:

```sh
tools/demo-tiles-capture/target/debug/capture_demo_tiles /tmp/vicente-mut-full --site mut
```

Site presets use the existing mapped OSM-object centres from `tools/pack/named_places.py`:

- `--site mut`: relation 18760408, longitude −70.6011321, latitude −33.4172801.
- `--site monumental`: relation 14111047, longitude −70.6059663, latitude −33.5065368.

These points define capture anchors. They do not assert a resolved simulation block or complete footprint coverage. The Monumental row in `named_places.csv` has unresolved block mapping. The height uses the existing triangular DEM interpolation in `web/data/terrain.i16`. The manifest records the selected site and source object. No free-form coordinate or Costanera preset is provided.

The square extends 250 metres east, west, north and south. Projected tile bounds conservatively retain whole edge tiles. Selection descends above 2 metres geometric error, or stops at a leaf. ADD retains parent content. REPLACE removes parent content when children replace it. External tileset transforms inherit the parent transform. Downloads use at most four simultaneous requests.

Original JSON, GLBs and companion resources stay under `source/`. Prepared GLBs and companion files stay under `tiles/`. Draco geometry becomes ordinary accessors. Textures, materials and hierarchy stay in the document. External references become local paths.

The manifest contains no request URLs, keys or sessions. Original source tilesets contain their returned session references. Keep the whole captured bundle local and out of Git.

## Placement contract

The capture composes inherited tile placement, glTF Y-up to Z-up conversion, Earth-origin subtraction and the local basis in f64. It multiplies this placement into every glTF scene-root node before serialization. Each prepared root is therefore already local: X east, Y up, Z south. The manifest contains an identity `tiles[].transform` as 16 column-major f64 numbers. Do not apply a second axis conversion.

The existing DEM uses EGM2008 orthometric heights. This version uses that number as the Earth anchor height without geoid conversion. This preserves the requested local reference but does not establish an ellipsoidal height. Check the resulting vertical offset in the first frame.

The viewer must load all local files without a key or network. It must show the local Google logo and aggregated attribution. The first downloaded fragment exposed Earth-scale root translations. The capture now cancels them in f64 before the Bevy loader. Inspect the new prepared roots before the first native frame.

## Acceptance state

The capture and viewer compiled with locked dependencies and Rust 1.97.1 at source head `85b9def9fbad92acf9ffe00762b2b415258b6950`. No automated tests ran.

The selected MUT first tile contains 5,963 vertices and 4,963 triangles, with local root coordinates, UVs, three embedded JPEG textures and Google copyright. This fragment does not establish full venue coverage.

The full MUT capture produced 995 tiles: 51,610,382 downloaded bytes and 47,509,360 prepared GLB bytes in 378.46 seconds. Timing fields describe capture only. They do not measure viewer loading or frame rate.

The default-stack native launch aborted with an IO Task Pool stack overflow. A retry of the same binary and bundle with `RUST_MIN_STACK=16777216` opened a native frame with textured geometry and visible credits. Use this tested 16 MiB stack setting for the full bundle:

```sh
RUST_MIN_STACK=16777216 /path/to/view_demo_tiles /path/to/mut-bundle
```

The exact failing function and root cause remain unconfirmed. All 995 prepared GLBs have one node and scene depth one, with no node cycles or invalid scene-node references found in the local audit. This excludes a hierarchy-cycle explanation for this bundle; it does not identify the loader failure.

Venue recognition, orientation, level overlap, controls, offline reopening and frame rate remain unverified. Monumental capture and native checks remain pending. Draco decoding is implemented but has not been exercised by an actual downloaded compressed tile.
