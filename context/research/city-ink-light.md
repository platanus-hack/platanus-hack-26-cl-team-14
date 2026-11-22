# Ink and light in the city view

Ink Studio adds a Bayer bias before it selects the closest palette color. Its Rust port uses a four-by-four grid and weights the RGB channels by 0.30, 0.55, and 0.15. The palette belongs to the scene. [Upstream renderer](https://github.com/siliconjungle/shapeshift-labs-ink-studio/blob/f4718f0f0908a378ed559e148a514db76a2a2da8/cli/render.js#L2842) · [Rust port](../../viz/src/ink/raster.rs)

Bevy draws opaque meshes before transparent meshes. The city samples opaque color and depth between those passes. A local depth difference finds silhouettes and folds. A brightness difference finds edges between surfaces at the same depth. This lets the renderer print the streets and buildings in blue and draw engine edits in coral with the same depth buffer. [Bevy pass order](https://github.com/bevyengine/bevy/blob/v0.20.1/crates/bevy_core_pipeline/src/core_3d/mod.rs#L162) · [City shader](../../viz/src/pixels/vote.wgsl)

The replay clock drives both the displayed hour and the lighting. Window lights and stippled tree shapes are illustrations. Tree positions come from the packed OpenStreetMap nodes. [Visual direction](../decisions/20261121-1836-city-ink.md)

## Reading the scenario at panel size

Hue and shape distinguish categories on a map. A light-to-dark sequence gives an order within a category. Use blue-violet for buildings, gray for streets, green for vegetation and blue for water. Keep one material identity across the day and night views. A tree crown also has an irregular outline and stippled interior. [Roth on Bertin's visual variables](https://geography.wisc.edu/cartography/projects/publications/Roth_2015_EG.pdf) · [ColorBrewer scheme types](https://colorbrewer2.org/learnmore/schemes_full.html)

An oblique view shows building height, but a tower can hide part of a street. Keep its true footprint and draw the same closure symbol at each mapped end. Check that the symbols remain distinct at 640 pixels across, including in grayscale. [Penn State on 3D maps](https://courses.ems.psu.edu/geog486/node/918) · [Takahashi et al. on urban route occlusion](https://web-ext.u-aizu.ac.jp/~shigeo/research/urban/index-e.html) · [W3C on use of color](https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html)

An architectural demolition drawing uses colored hatching to identify work within an existing structure. In Santiago, reserve coral for the engine change. For a removal, compare the intact building with its marked footprint at the same camera and hour. For a closure, keep the road visible around the change. Match night and day separately. A desktop panel check does not establish projector readability. [Demolition drawing](https://docs.planning.org.uk/20210521/115/QTCHY1RPGOB00/rdvs60gjp2ad1moh.pdf) · [Ordnance Survey on visual hierarchy](https://github.com/OrdnanceSurvey/GeoDataViz-Toolkit/blob/master/Cartographic%20Design%20Principles/README.md)
