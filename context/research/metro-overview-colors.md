# Metro line colors in the city overview

Emerson asked to see the original Metro line colors when the camera pulls back. At `1185b69d20e3c35d8c96ec88a7a2f04b3e33d334`, the surface ribbon appears only in replays with `scene.layer = "metro_service"`. It uses one blue for every line and fades during a full closure. The subway view hides the ribbon so it can show the tracks and trains. Ordinary city overviews have no surface network overlay. [Emerson's request](https://discord.com/channels/1573514576400752640/1573812338983706654/1573858441301332119) · [Native overlay](../../viz/src/city/metro_service.rs)

The checked-in route pack has one direction-0 shape per active line. Its packer reads `route_color` from the March DTPM GTFS. Use these fields for the overview. [Route pack](../../web/data/transit_paths.json) · [Packer](../../tools/pack/transit_paths.py) · [DTPM GTFS guide](https://www.dtpm.cl/descargas/gtfs/Descripcion%20GTFS%20estatico%20DTPM_20260210.pdf)

| Line | sRGB |
| --- | --- |
| L1 | `#C8102E` |
| L2 | `#F2A900` |
| L3 | `#703F2A` |
| L4 | `#151F6D` |
| L4A | `#0072CE` |
| L5 | `#00965E` |
| L6 | `#8E3A80` |

The existing ribbon is 16 m wide. With a 35° camera and 1080 px height, it spans about 1.5 output px at 18 km and 0.5 px at 55 km near the center of the view. This estimate uses the viewport scale at the camera target and excludes ground angle, occlusion and pixel quantization. [Camera and viewport formula](../../viz/src/main.rs) · [Pixel output](../../viz/src/pixels.rs)

For a native check, compare the same camera in ordinary surface view at 18 km and 55 km, then switch to the Tobalaba subway view. Use the route colors and a visible screen width in the overview. Check that the overlay reads as a map line across buildings and clears in subway view. In the tagged 08:00 Metro comparison, keep the same camera and confirm that the closed network fades. The visual thickness and overlay priority need a native frame and taste review. [Timed Metro check](metro-train-visibility.md)
