# V1 engine scale on an M2

A release build evaluated the current worker and student model on Francisco's MacBook Air (Apple M2, 16 GiB). The population inputs represent **3,966,150 synthetic people**: 2,688,814 workers and 1,277,336 students with a home and destination inside the Metropolitan Region. The closure edits model worker mode changes.

| Request | Wall time | Peak resident memory |
| --- | ---: | ---: |
| Zero edits | 0.91 s | 207 MiB |
| Remove all Metro service and close 126 stations | 13.44 s | 271 MiB |

One run per request, measured with macOS `/usr/bin/time -l`. Times include engine startup, scenario computation, and writing JSON. Build commit `7887c11ec1a2bd2a9365f58d40b765638e80d6ec` from [the closure PR](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/368). Run the release `scenario` binary with `data/mvp` and either a file containing `[]` or `context/research/metro_full_closure.json` from that checkout.

The viewer mesh has 739,230 footprint records. Of these, 279,385 are mapped and 459,845 are generated infill. The separate [activity pack](../../data/mvp/activity_points.csv) has 16,697 mapped points, including 15,605 classified as shop, supermarket or mall. The footprint count measures drawn geometry. The activity count measures records that can supply candidate destinations.

A full optional-visit run still needs a benchmark. Select candidates by service and location before evaluating individual preferences. A scan of all 16,697 activity points for each of the 3.97 million current people would require about 66.2 billion comparisons before route checks. Measure the candidate lookup, travel costs, and choices together on the real pack.
