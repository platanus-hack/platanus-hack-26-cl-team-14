# Browser startup

Measured on the MacBook Air M3 with a new Chrome profile for each run. The browser window was 2804 × 1754. The city canvas was 1823 × 1754. Time starts at navigation and ends when the city reports ready and the page shows the form. Bytes include response bodies from the page and the worker up to that point.

| Metric | Baseline | Final | Target |
| --- | ---: | ---: | ---: |
| Startup transfer with gzip | 96.10 MB | 34.32 MB | ≤40 MB |
| Startup transfer without HTTP compression | 192.12 MB | 92.57 MB | |
| Local time with gzip | 17.12 s | 4.72–5.73 s | ≤8 s |
| Local time without HTTP compression | 17.82 s | 6.02 s | ≤8 s |
| Time at 20 Mbit/s with gzip | 47.19 s | 21.06 s | ≤30 s |
| Local frame rate after startup | 59.7 FPS | 59.7 FPS | Within 10% |

Frame rate varied with the laptop load. In a separate 12-second check, frame rate was 36.0 FPS before the road split and 34.2 FPS after it, a 4.8% drop. Local load time in that check was 13.25 s before the split and 11.20 s after it. The baseline and final screenshots show the same opening view, with ink and depth of field.

## Attempts

Each row uses the previous row as its reference. The table shows gzip transfer and local time.

| Attempt | Change | Transfer | Saved | Time |
| --- | --- | ---: | ---: | ---: |
| 0 | Baseline | 96.10 MB | | 17.12 s |
| 1 | Pack road coordinates and keep the original geometry | 84.18 MB | 11.92 MB | 17.23 s |
| 2 | Fetch road and ferry catalogs when an edit needs them | 70.35 MB | 13.83 MB | 16.10 s |
| 3 | Build the usual day once and start the engine on the first request | 64.39 MB | 5.96 MB | 11.38 s |
| 4 | Fetch nearby detailed building polygons first | 52.42 MB | 11.98 MB | 12.16 s |
| 5 | Pack the nodes and edges used for traffic at startup | 44.63 MB | 7.79 MB | 9.98 s |
| 6 | Fetch distant roads on demand and draw one tile per frame | 34.32 MB | 10.31 MB | 4.72–5.73 s |

All six changes reduced startup transfer. No optimization was rejected. Attempt 6 reached all three startup targets.

The road packs use integer coordinate deltas and variable-length integers. A correction keeps the original binary value when a coordinate differs from the grid value. The build keeps the original packs for the paths that need their full fields. [Geobuf encoder](https://github.com/mapbox/geobuf/blob/master/encode.js), [Protocol Buffers encoding](https://protobuf.dev/programming-guides/encoding/)

The measurement server served gzip response bodies for uncompressed static files. The production host needs the same HTTP compression for Wasm and text files. Serve the repository root and include the generated files in `web/wasm/`. Keep the original data files available for edits. [NGINX static gzip](https://nginx.org/en/docs/http/ngx_http_gzip_static_module.html), [Chrome throttling](https://developer.chrome.com/docs/devtools/settings/throttling)

## Results

The native engine built. Eight saved experiments had zero differences in `compare.py`, with float tolerance of 1e-9. The startup snapshot also matched the native empty day.

| Browser form case | Result |
| --- | --- |
| Copy the tower to Puente Alto and add 5,863 jobs | 5,779 assigned workers |
| Close TB-TOB | Daily entries drop from 39,823.6 to 0 |
| Add 5,000 jobs at Gran Torre Costanera | 6,489 reassigned workers |
| Submit an empty change list | 2,457,348.4 boardings in base and scenario |
| Add a boat line with two piers | Accepted, with 316.4 riders to work |

Chrome listed all seven MCP tools. The form kept its blue and white square controls. The screenshots and logs are in `/tmp/wasm-perf`.

The first tool call took 5.75 s on the local server. It starts the engine. A fast pan can show terrain before distant roads arrive. Next, check compression and the first edit on a phone with the chosen static host.
