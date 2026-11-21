# Ink and light in the city view

Ink Studio adds a Bayer bias before it selects the closest palette color. Its Rust port uses a four-by-four grid and weights the RGB channels by 0.30, 0.55, and 0.15. The palette belongs to the scene. [Upstream renderer](https://github.com/siliconjungle/shapeshift-labs-ink-studio/blob/f4718f0f0908a378ed559e148a514db76a2a2da8/cli/render.js#L2842) · [Rust port](../../viz/src/ink/raster.rs)

Bevy draws opaque meshes before transparent meshes. The city samples opaque color and depth between those passes. A local depth difference finds silhouettes and folds. A brightness difference finds edges between surfaces at the same depth. This lets the renderer print the streets and buildings in blue and draw engine edits in coral with the same depth buffer. [Bevy pass order](https://github.com/bevyengine/bevy/blob/v0.20.1/crates/bevy_core_pipeline/src/core_3d/mod.rs#L162) · [City shader](../../viz/src/pixels/vote.wgsl)

The replay clock drives both the displayed hour and the lighting. Window lights and stippled tree shapes are illustrations. Tree positions come from the packed OpenStreetMap nodes. [Visual direction](../decisions/20261121-1836-city-ink.md)
