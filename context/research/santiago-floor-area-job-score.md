# April job weights keep the lowest frozen error

The revised floor-area weights increased absolute error by 787.81 weekday boardings against the April file. Adoption required a reduction of at least 1,000. The comparison used one L9 population executable, all stations open, and a separate jobs.csv input for each run.

| Job weights | Absolute error, boardings | Frozen half-hour WAPE | Tobalaba predicted, observed bins | Pearson r, training work arrivals |
| --- | ---: | ---: | ---: | ---: |
| Original | 238,349.70 | 0.41788655 | 39,823.55 | 0.903796 |
| April work arrivals | 234,698.53 | 0.41148514 | 53,717.01 | 0.996324 |
| Revised Santiago floor area | 235,486.35 | 0.41286638 | 40,193.73 | 0.909615 |

The frozen scorer used 31 stations and 1,123 observed half-hours with 570,369.4 total boardings. Tobalaba belongs to training and had 81,612.8 observed boardings. The correlation pairs the job mass nearest each training station within 500 m with its April 15 work arrivals. It includes all 95 training stations. The April weights used these arrivals during fitting. No held-out observations entered the weights.

The builder preserves the 10,782 modeled jobs on the 31 receiving blocks with zero office, retail and industrial weight. It distributes the remaining 307,577 covered jobs by positive weight and keeps 42,289 jobs outside the Santiago crosswalk. It includes all 532 new receiving blocks, of which 438 have positive weight. All 52 comuna totals match the original stock of 2,688,814 jobs. The formula excludes education and health floor area. The source does not document occupancy or gross versus net area. [Floor-area rule and sources](land-use-blocks.md) · [Revised rule registered before the run](https://discord.com/channels/1573514576400752640/1573688249287708682/1573706701457203281)

Rebuild with `python3 -I tools/pack/jobs_landuse.py`. To reproduce a score, use the same pack in three separate directories. Place each candidate in its directory as `jobs.csv`. Run `population PACK predictions.json l9` and then `python3 tools/pack/check_score.py predictions.json`. The run used population executable SHA256 `9166af3735a8d6cf6a40a71a0a5bd08a30c5e4bcf27bfb4f53416bdb72162b1f`. The revised input SHA256 was `1f25e09ced1484945561bba3973723302c8ece5313bbb5b152b01af3e12b35a6`. Its prediction SHA256 was `51bfba090778d93dfad2770fbe64a472f9e6d0eca7bbcea33228a48bb69a2621`.
