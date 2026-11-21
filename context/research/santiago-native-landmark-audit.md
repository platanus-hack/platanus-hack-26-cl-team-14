# Santiago landmark recognition in the native viewer

The pitch proposal opens on one traveler and pulls back to the city. V2 needs that view to read as Santiago. In the browser mock, Tagua verified the labeled frame on a second computer. With labels and pins hidden, the river and terrain still read. The tower and plaza had no distinct shape in that mock. [Pitch proposal](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/354) · [Browser review](https://discord.com/channels/1573514576400752640/1573809836699820163/1573812787182833824)

## Native source audit

| Landmark | What the viewer draws | What to check in the capture |
| --- | --- | --- |
| Mapocho | Detailed bank polygons in the center and a 30 m display ribbon along the outer line. | The river gives a continuous east-to-west cue. |
| Costanera Center | A dedicated Gran Torre mesh from a mapped footprint, a 300 m height, and 64 levels. | The tower stands out from nearby buildings at the pull-back distance. |
| Cerro San Cristóbal | The terrain grid supplies the hill. The named-building loader skips its landmark point. | The hill crest reads separately from the surrounding slopes. |
| Plaza Italia | The landmark file has no plaza record. The viewer draws the surrounding street and ground packs. I found no dedicated plaza mesh in the named loader. | The viewer can point to the junction. The existing road shape may need a closer view. |

[Landmark data](../../web/data/santiago_landmarks.geojson) · [Terrain source](../../web/data/terrain.json) · [Named meshes and river](../../viz/src/city/ground.rs) · [Scene setup](../../viz/src/city.rs) · [Plaza Italia coordinates](https://www.wikidata.org/wiki/Q2415377)

## Capture plan

Use the final pull-back camera once it exists. For an earlier source check, start the viewer with the baseline result loaded and the following position. This is a proposed starting view. Its framing remains unverified.

```sh
SANTIAGO_FOCUS=-70.630,-33.430 SANTIAGO_DISTANCE=8500 \
SANTIAGO_CLOCK_SECONDS=39600 SANTIAGO_CAPTURE=/tmp/santiago-landmarks.png \
cargo run --release --manifest-path viz/Cargo.toml
```

Set `SANTIAGO_ENGINE` to the reviewer’s own engine. Keep the same 1920 × 1080 view, camera, and 11:00 clock for each comparison. Open the still to check that it includes the river bend, tower, hill, and plaza. Show the unlabeled still to a person who has not seen the project. Ask which city they see and which landmarks they can point to before showing the labeled mock. Record the answers and the viewer commit. A browser capture does not establish recognition in the native view. [Viewer setup](../../viz/README.md) · [Camera and capture source](../../viz/src/main.rs)
