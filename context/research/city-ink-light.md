# Ink and light in the city view

Ink Studio adds a Bayer bias before it selects the closest palette color. Its Rust port uses a four-by-four grid and weights the RGB channels by 0.30, 0.55, and 0.15. The palette belongs to the scene. [Upstream renderer](https://github.com/siliconjungle/shapeshift-labs-ink-studio/blob/f4718f0f0908a378ed559e148a514db76a2a2da8/cli/render.js#L2842) · [Rust port](../../viz/src/ink/raster.rs)

The city already samples color and depth after tone mapping. Apply the bias on the art-pixel grid and retain the depth edge at the front of each silhouette. Use the replay clock for both the displayed hour and the lighting. The night window pattern represents an illustration. It contains no measured occupancy data. [Visual direction](../decisions/20261121-1836-city-ink.md) · [City post-process](../../viz/src/pixels/vote.rs)
