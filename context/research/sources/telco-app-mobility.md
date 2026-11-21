# Phone and app mobility data for Santiago

Owner: quiet-panda (claude-fable-5-1). Checked 2026-11-21. Labels: VERIFIED-ROWS = real records read; VERIFIED-BYTES = file or page answered; UNVERIFIED = no data. Scripts: `tools/scrape/<source>.py`. Samples: `data/raw/<source>/`.

**Short answer.** No open source gives phone traces or trips below the comuna for Santiago today.
Open phone data stops at the comuna (2020 to 2021) or the province (2020 to 2026). Raw telco data (Telefónica XDR, Entel CDR)
exists only inside academic agreements or government contracts. App vendors (Waze, Strava) give data to governments, not to a hackathon team.

## Open today, rows read

### Telefónica Chile + UDD: mobility indices by comuna (Pappalardo, Ferres et al., Scientific Data 2023)
- Link: https://api.figshare.com/v2/collections/5214272/articles (paper: https://doi.org/10.1038/s41597-022-01893-3)
- Contents: one row = comuna × day. Columns: region, rid, comuna, cid, area, IM_interno (trips inside the comuna per device), IM_externo (trips to other comunas), IM, date. Built from 31 billion XDR of Telefónica Movistar. Also `quarantines.csv` (168 comuna quarantine periods) and the analysis notebook.
- Granularity: comuna. Daily.
- Year: 2020-02-26 to 2020-09-20.
- Access: direct download. CSV, 7.1 MB, 71,553 rows, 345 comunas (13,171 rows in region 13).
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 71,553 rows. `tools/scrape/telefonica-udd-figshare.py`.
- Limits: pandemic period only. Index (trips per device), not counts of trips and not OD pairs. Movistar share only (about 30 percent). CC BY 4.0. The MinCiencia copy (`Datos-COVID19/output/producto33`) is gone: HTTP 404. The UDD report pages `datascience.udd.cl/reports/` and `/movilidad/` redirect to the institute home page.
- Use in the simulation: TARGET for the observed mobility index by comuna during quarantine. Match the trips-per-device index, not an absolute trip count.
- Role: TARGET (mobility index, not trip count)

### Meta Movement Distribution Maps (HDX)
- Link: https://data.humdata.org/dataset/movement-distribution
- Contents: one row = polygon × day × distance band. Columns: gadm_id, gadm_name, country, polygon_level, home_to_ping_distance_category (0, (0,10), [10,100), 100+ km), distance_category_ping_fraction, ds. Facebook app users with location on.
- Granularity: GADM level 2 = province (Santiago, Cordillera, Maipo, Melipilla, Talagante, Chacabuco). Daily.
- Year: 2020 to 2026, new file each 2 weeks. Latest file 2026-10-16 to 2026-11-01.
- Access: direct download. CSV, 174 MB per 2-week file (world), Chile rows 3,672 per file.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 3,672 Chile rows. `tools/scrape/meta-movement-distribution.py`.
- Limits: province, not comuna. Fraction of pings, not trips. Facebook users only. CC BY.
- Use in the simulation: TARGET for the fraction of recorded pings in each home-distance band, by province and day. This is not a fraction of people or a measure of who stayed home all day.
- Role: TARGET

### Meta Movement Range Maps (HDX)
- Link: https://data.humdata.org/dataset/movement-range-maps
- Contents: one row = polygon × day. Columns: ds, country, polygon_source, polygon_id, polygon_name, all_day_bing_tiles_visited_relative_change, all_day_ratio_single_tile_users, baseline_name, baseline_type.
- Granularity: GADM level 2 = province (52 polygons in Chile, 6 in the Metropolitan Region). Daily.
- Year: 2020-03-01 to 2022-05-22.
- Access: direct download. Two ZIP files, 57 MB and 73 MB (world). Chile rows: 15,588 (2020) and 24,582 (2021 to 2022).
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 40,170 Chile rows. `tools/scrape/meta-movement-range.py`.
- Limits: ended 2022. Relative change against February 2020. Province only.
- Use in the simulation: TARGET for the stay-at-home share by province during the pandemic shock. Pairs with the Telefónica comuna index.
- Role: TARGET

### Google COVID-19 Community Mobility Reports
- Link: https://www.gstatic.com/covid19/mobility/Global_Mobility_Report.csv
- Contents: one row = place × day. Columns: percent change from baseline for retail_and_recreation, grocery_and_pharmacy, parks, transit_stations, workplaces, residential.
- Granularity: region and province (sub_region_2: Santiago Province, Cordillera, Maipo, Melipilla, Talagante, Chacabuco). Daily.
- Year: 2020-02-15 to 2022-10-15.
- Access: direct download. CSV, 216 MB (world). Chile rows: 68,716. 974 days × 7 rows for the Metropolitan Region.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 68,716 Chile rows. `tools/scrape/google-mobility.py`.
- Limits: ended 2022. Province, not comuna. Baseline is January 2020. Android users with location history.
- Use in the simulation: TARGET by activity type (work, retail, transit, parks) per province for a shock. Only open source that splits mobility by purpose.
- Role: TARGET

### Apple Mobility Trends Reports (community mirror)
- Link: https://raw.githubusercontent.com/ActiveConclusion/COVID19_mobility/master/apple_reports/applemobilitytrends.csv
- Contents: one row = place × transport type. One column per day: routing requests in Apple Maps relative to 2020-01-13 = 100.
- Granularity: city "Santiago" and "Santiago Metropolitan Region". Daily. Types: driving, walking (no transit for Chile).
- Year: 2020-01-13 to 2022-04-12.
- Access: direct download. CSV. 22 Chile rows.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 22 Chile rows. `tools/scrape/apple-mobility.py`. Apple's own site no longer serves the file.
- Limits: routing requests, not trips. One number per day for the whole city. Mirror, not the publisher.
- Use in the simulation: NONE beyond a weak city-wide driving vs walking check for 2020 to 2022.
- Role: NONE

### Meta High Resolution Population Density, Chile 2020 (HDX)
- Link: https://data.humdata.org/dataset/chile-high-resolution-population-density-maps-demographic-estimates
- Contents: one row = 30 m cell. Columns: longitude, latitude, chl_general_2020 (persons). Also men, women, youth, elderly, children files.
- Granularity: 30 m grid (point). No time step.
- Year: 2020 (census 2017 counts spread by satellite building detection).
- Access: direct download. CSV in ZIP, 26 MB, 5,750,979 rows. Santiago box (-70.85 to -70.45, -33.65 to -33.3): 529,342 cells, 7.17 M people.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 5,750,979 rows. `tools/scrape/meta-hrsl-chile.py`.
- Limits: not phone data. Model output, not a count. Census 2024 manzana counts (other file) are better where they exist.
- Use in the simulation: INPUT to place 7 M agents at home when a manzana has no count. Fallback only.
- Role: INPUT

### Meta Commuting Zones, March 2023 (HDX)
- Link: https://data.humdata.org/dataset/commuting-zones
- Contents: one row = zone. Columns: region, fbcz_id, name, win_population, win_roads_km, area, country, geography (WKT polygon). Built from Facebook home and work locations.
- Granularity: zone (31 zones in Chile; one is Greater Santiago). No time step.
- Year: 2023.
- Access: direct download. CSV, 16.5 MB, 31 Chile rows.
- Check: VERIFIED-ROWS 2026-11-21, HTTP 200, 31 Chile rows. `data/raw/meta-commuting-zones/all.csv` (curl, no script).
- Limits: one polygon for all of Santiago. No flows inside the zone.
- Use in the simulation: NONE. Only confirms the study area border.
- Role: NONE

## Closed: academic agreements (no download, by design)

### Telefónica R&D Chile XDR and CDR via UDD Data Science Institute (Graells-Garrido, Ferres, Opitz, Rowe)
- Link: https://arxiv.org/abs/2204.09482 (data note), https://www.cepal.org/sites/default/files/courses/files/m2-eduardograells-movilidad-antenas-telefonicas-2022.pdf (slides)
- Contents: antenna-level XDR trajectories of more than 1 million devices in Santiago (2,076 towers in the urban area), with DPI app usage. Papers: mode share update of EOD 2012 (2022), spatial subdivisions (2020), gender mobility gap (2020), mall attraction (2018), daily journeys with 0.89 correlation to EOD 2012 (2016).
- Granularity: antenna, aggregated to comuna in papers. Minutes.
- Year: 2015 to 2020 in papers.
- Access: form or request. "Anonymized datasets are available from Telefónica R&D Chile" under an agreement; the operator keeps ownership. Not for us this weekend.
- Check: UNVERIFIED (closed data). Paper and slides opened, HTTP 200.
- Limits: raw data never leaves Telefónica. Only published aggregates and fitted parameters.
- Use in the simulation: INPUT of published parameters: mode shares by comuna 2020 (arXiv 2204.09482), trip distance distributions, mall catchments. Copy numbers from the papers, not data.
- Role: INPUT

### Ferres and Gauvin 2026: CDR 2016 Santiago with socioeconomic bias correction
- Link: https://arxiv.org/abs/2604.16193, code https://github.com/leoferres/mobmrp
- Contents: Movistar voice CDR, May to July 2016, Santiago Metropolitan Region, with socioeconomic group and gender. Units: 329 distritos, 47 comunas, 730 census zones.
- Granularity: census zone (zona). Weekday.
- Year: 2016.
- Access: closed data. Code open (HTTP 200).
- Check: UNVERIFIED (closed data). Abstract and repo opened.
- Limits: 2016, before Metro lines 3 and 6. Voice calls only.
- Use in the simulation: INPUT method only: how to correct phone data toward census population by socioeconomic group. Useful if we ever get phone data.
- Role: INPUT

### UDD 2023 thesis: comuna trip matrices from XDR against the SECTRA 2019 model
- Link: https://repositorio.udd.cl/bitstreams/d06aa764-97bc-4cc9-9206-6e5e3feee130/download
- Contents: comuna × comuna trip matrix from Telefónica XDR, compared with SECTRA's 2019 validated model.
- Granularity: comuna. Day.
- Year: about 2021 to 2022.
- Access: PDF only. No data.
- Check: UNVERIFIED (not opened in full; link from search).
- Limits: thesis, no data release.
- Use in the simulation: NONE as data. Reference for an OD matrix at comuna level if we need to validate a gravity model.
- Role: NONE

## Closed: government contracts

### Entel Ocean + MTT Lab (Ministerio de Transportes) and Entel + ISCI
- Link: https://informacioncorporativa.entel.cl/press-releases/ministry-of-transport-and-telecommunications-and-entel-ocean-analyzed-experience-of-the-data-laboratory-on-mobility-patterns-in-pandemics
- Contents: five mobility indicators from Entel phone data (time of leaving home, mean displacement distance, and others) "from the region or commune to the census area". 2014 pilot with Subtrans and Ericsson. 2026 alliance with Sé Santiago (CORFO) for mobility planning.
- Granularity: census zone (claimed). Daily.
- Year: 2020 to 2021.
- Access: contract. Nothing published as data.
- Check: UNVERIFIED (press release, HTTP 200). No dataset exists in public.
- Limits: press claims only.
- Use in the simulation: NONE now. If the team wants real phone OD at zone level, this is the door: Subsecretaría de Transportes already pays for it.
- Role: NONE

### Waze for Cities
- Link: https://www.waze.com/partners/wazeforcities
- Contents: jam segments (speed below free flow) and user alerts, every 2 minutes. The Ministerio de Transportes joined in 2019 (Pérez Karich, Urban Studies 2025).
- Granularity: road segment (point). 2 minutes.
- Year: 2019 to today for partners.
- Access: "free data-sharing partnership between Waze and municipal governments or transit agencies". Not for individuals.
- Check: UNVERIFIED (page opened, HTTP 200; no data).
- Limits: only governments. Congestion, not people.
- Use in the simulation: NONE for us. TARGET for road congestion if a municipality shares its feed.
- Role: NONE

### Strava Metro
- Link: https://metro.strava.com/ and https://metro.strava.com/academics
- Contents: counts of bike and walk trips by street segment and hour.
- Granularity: street segment. Hour.
- Year: 2019 to today.
- Access: free with application for city governments and advocacy groups. Academic 2027 cohort closed on 2026-10-23. No Chilean city on the partner list.
- Check: UNVERIFIED (pages opened, HTTP 200; no data).
- Limits: Strava users (sport cyclists). No Chile partner known.
- Use in the simulation: NONE now.
- Role: NONE

## Closed: paid vendors

### Kido Dynamics
- Link: https://www.kidodynamics.com/
- Contents: telco-based OD matrices and tourism flows.
- Granularity: zone. Hour.
- Year: current.
- Access: paid, custom quote. Case studies only in Spain. No Chile, no Entel, no Movistar on the site.
- Check: UNVERIFIED (page opened, HTTP 200).
- Limits: the hint "Kido Dynamics in Chile" has no support on the web.
- Use in the simulation: NONE.
- Role: NONE

### Mapbox Movement
- Link: https://www.mapbox.com/movement-data
- Contents: activity index per 100 m tile per hour from Mapbox SDK pings.
- Granularity: 100 m tile. Hour.
- Year: 2020 to today.
- Access: paid, "licensed by geography on an annual basis". Coverage list names US, Canada, Western Europe, Australia, Japan, "select markets in Central America". Chile is not listed.
- Check: UNVERIFIED (page opened, HTTP 200).
- Use in the simulation: NONE.
- Role: NONE

### Veraset, Grandata, SafeGraph, Advan, Dewey
- Link: https://www.veraset.com/ (IDB blog on the Veraset Latin America map returns HTTP 403; the EL-BID GitHub repo for it is 404)
- Contents: GPS pings from ad SDKs (Veraset, Grandata; Grandata covered 12 Latin American countries in 2020). Advan and SafeGraph foot traffic is US only.
- Granularity: point. Second.
- Year: 2019 to today.
- Access: paid. Veraset gives academic grants on request.
- Check: UNVERIFIED (Veraset page HTTP 200, no coverage table for Chile).
- Limits: small and biased samples; price unknown.
- Use in the simulation: NONE this weekend.
- Role: NONE

### Uber Movement
- Link: https://movement.uber.com/ (redirects to https://www.uber.com/business/movement-decommissioning, HTTP 406)
- Contents: was zone-to-zone travel times. Santiago was never in the list.
- Check: VERIFIED-BYTES 2026-11-21 (redirect to the decommissioning page).
- Use in the simulation: NONE.
- Role: NONE

## Leads not verified

### Google Environmental Insights Explorer, Santiago
- Link: https://insights.sustainability.google/places/ChIJL68lBEHFYpYRHJ1Dr1dLYhc
- Contents: yearly trips and km by mode (car, bus, rail, walk, bike) for the city, from Google location data. Export needs a city account.
- Granularity: city. Year.
- Year: 2018 to 2024 for most cities.
- Check: UNVERIFIED. HTTP 200 but the page is a JavaScript shell; curl gets only the title. Needs a browser.
- Use in the simulation: TARGET for mode share of the whole city, if Santiago has data. One number per mode per year. Check with a browser before trust.
- Role: TARGET

### TomTom Traffic Index, Santiago
- Link: https://www.tomtom.com/traffic-index/santiago-traffic/
- Contents: one congestion level and time lost per year for the city.
- Check: UNVERIFIED (HTTP 200, JavaScript shell, numbers not in the HTML).
- Use in the simulation: NONE. One number for the whole city.
- Role: NONE

### Overture Places and Foursquare OS Places
- Link: https://overturemaps-us-west-2.s3.us-west-2.amazonaws.com/?prefix=release/2026-11-18.0/theme=places/type=place/ and https://docs.foursquare.com/data-products/docs/access-fsq-os-places
- Contents: POI points, not mobility. Overture release 2026-11-18.0 exists (HTTP 200, parquet parts of about 800 MB each). Foursquare moved from public S3 to a portal with sign-up (October 2025).
- Check: VERIFIED-BYTES (S3 listing only; rows need DuckDB). Covered by `context/research/data-sources-buildings-activity.md` (clever-llama).
- Use in the simulation: INPUT for destinations, owned by the activity-places file. No visit counts.
- Role: INPUT

### Subtel antenna portal
- Link: https://antenas.subtel.gob.cl/leydetorres/inicio.html
- Contents: map of mobile towers. No download found in the HTML. Subtel statistics are national xlsx series, not by comuna.
- Check: UNVERIFIED (HTTP 200, no file link).
- Use in the simulation: NONE unless we get CDR data.
- Role: NONE

## Critique
1. Everything open stops at the province or the comuna, and the comuna data is 2020 only. Nothing here gives an OD pair below the comuna. The simulation must get its flows from EOD 2012, DTPM Bip! trips, and census commute data (other files), not from phones.
2. I did not open the Scientific Data paper itself (login redirect) or the full UDD thesis. The figshare column meanings come from the CSV header and the notebook, not from the paper text.
3. Google EIE was not checked in a browser. It may be the only open, current mode-share number for Santiago, or it may have no Santiago data. Veraset and Grandata prices and Chile sample sizes were not asked.
