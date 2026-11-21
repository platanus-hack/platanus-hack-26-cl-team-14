# People in Santiago who live elsewhere

The best open source for domestic overnight visitors is Sernatur's origin-destination table. EMAT measures paid lodging nights. The 2024 Census counts regular workers who cross a regional border. These sources measure different people and must not be added together. No source below identifies every person present in Santiago on a given day.

### Sernatur domestic overnight trips, total series
- Link: https://www.sernatur.cl/wp-content/uploads/2020/11/Nueva-Serie-Oficial-CDR-Viajes-totales.csv
- Contents: One origin comuna, destination comuna, and year per row; monthly estimated overnight trips. The columns include CUT codes, regions, tourist destination, and January to December.
- Granularity: comuna origin to comuna destination; month. It stops at the comuna.
- Year: 2019 onward; the sampled first row is 2019.
- Access: direct download; semicolon CSV, 157,153,162 bytes. Run `/usr/bin/python3 tools/scrape/tourism_visitors.py domestic-total`.
- Check: ROWS VERIFIED (bounded sample) 2026-11-21, HTTP 206, 2,048 bytes; header and two Iquique origin rows read.
- Limits: Overnight domestic travel only. Phone signal estimates are not persons. Sernatur changed to CDR in 2025 and adjusted 2019–2024; the 2026 method is still under review. Filter destination to Greater Santiago comunas and origin outside the area.
- Use in the simulation: INPUT for monthly visitor origin and destination weights; TARGET for overnight trip counts.

### Sernatur domestic overnight trips, occasional series
- Link: https://www.sernatur.cl/wp-content/uploads/2020/11/Nueva-Serie-Oficial-CDR-Viajes-ocasionales.csv
- Contents: Same origin-destination and monthly fields as the total series, restricted to occasional overnight trips.
- Granularity: comuna origin to comuna destination; month. It stops at the comuna.
- Year: 2019 onward; the sampled first row is 2019.
- Access: direct download; semicolon CSV, 152,483,628 bytes. Run `/usr/bin/python3 tools/scrape/tourism_visitors.py domestic-occasional`.
- Check: ROWS VERIFIED (bounded sample) 2026-11-21, HTTP 206, 2,048 bytes; header and first Iquique origin row read.
- Limits: A subset of the total series, so do not add both. The change to CDR and the historical adjustment also apply here. It excludes same-day visitors and regular commuters.
- Use in the simulation: INPUT for occasional overnight visits; TARGET for the occasional trip share.

### Sernatur foreign tourist arrivals by border crossing
- Link: https://www.sernatur.cl/wp-content/uploads/2021/01/202608-LLEGADAS-DE-TURISTAS-EXTRANJEROS-V20261009-PUBLICACION.xlsx
- Contents: Excel sheets give monthly arrivals by nationality, border crossing, and region. A Santiago airport arrival is an entry to Chile, not proof of a stay in Santiago.
- Granularity: border point and nationality; month. No destination comuna.
- Year: January 2008 to August 2026 in the downloaded workbook.
- Access: direct download; XLSX, 4.0 MB. Run `/usr/bin/python3 tools/scrape/tourism_visitors.py foreign-arrivals`.
- Check: BYTES VERIFIED; DATA ROWS UNVERIFIED. 2026-11-21, HTTP 200, XLSX opened; `sheet2.xml` starts with nationality/month tables and `sheet3.xml` with border crossing/month tables. No numeric record is documented in this check.
- Limits: Includes estimated Chileans resident abroad. August 2026 figures are preliminary. Transit and onward travelers can inflate Santiago presence. This cannot locate lodging.
- Use in the simulation: INPUT for foreign arrival seasonality; TARGET for airport entry totals only.

### INE EMAT via Sernatur SIET
- Link: https://datosturismo.sernatur.cl/siet/reporteDinamicoEMAT
- Contents: One region or tourist destination per row, with lodging arrivals, nights, mean stay, occupancy, and room prices. The visible Metropolitan Region row has 2,312,364 arrivals and 5,106,449 nights for the selected period.
- Granularity: region or tourist destination; month through filters. The checked row stops at the region.
- Year: January to September 2026 in the opened table.
- Access: web report and Excel export; export URL and size not checked.
- Check: ROWS VERIFIED (bounded sample) 2026-11-21, HTTP 200, Metropolitan Region data row read in the open report.
- Limits: Counts stays in surveyed lodging establishments, not unique visitors. It misses homes, informal lodging, and same-day travel. The displayed domestic/foreign breakdown is N/A at the checked aggregation.
- Use in the simulation: TARGET for paid lodging nights and mean stay; INPUT for monthly overnight load.

### JAC air passenger flows via Sernatur SIET
- Link: https://datosturismo.sernatur.cl/siet/reporteTempJAC
- Contents: Monthly passengers by city pair, domestic or international, with total, arrivals, and departures. The opened default report shows January 2024 with 2,716,049 total passengers across the selected national scope.
- Granularity: airport city pair; month. No destination within Santiago.
- Year: 2023–2024 in the opened default report; other years need filter checks.
- Access: web report and Excel export; export URL and size not checked.
- Check: ROWS VERIFIED (bounded sample) 2026-11-21, HTTP 200, January and February 2024 rows read.
- Limits: The visible default total is all routes, not Santiago arrivals. City pairs describe flight legs, so connecting passengers and local residents cannot be separated. A Santiago filter is needed before use.
- Use in the simulation: TARGET for airport passenger flows after filtering Santiago; INPUT for arrival timing.

### Inside Airbnb Santiago summary listings
- Link: https://data.insideairbnb.com/chile/rm/santiago/2026-09-23/visualisations/listings.csv
- Contents: One advertised listing per row, with coordinates, neighbourhood, room type, price, minimum nights, review counts, and availability.
- Granularity: point; one snapshot. Review and availability fields summarize longer periods.
- Year: 2026-09-23 snapshot.
- Access: direct download; CSV. The sample response returned a 4,096-byte range. Avoid host identifiers and names during ingestion.
- Check: ROWS VERIFIED (bounded sample) 2026-11-21, HTTP 206, CSV header and first listing row read.
- Limits: Listings are advertised supply, not occupied rooms or visitor counts. Platform coverage and coordinates are imperfect. The file also contains host names and IDs; do not ingest those fields.
- Use in the simulation: INPUT for the spatial distribution of short-term rental capacity only.

### Census 2024 interregional work commuters
- Link: https://censo2024.ine.gob.cl/wp-content/uploads/2025/12/sintesis_resultados_censo2024.pdf
- Contents: Figure 33 and Table 15 count employed people who reside in one region and work in another. The Metropolitan Region received 72,798 such workers and sent 51,885 in 2024.
- Granularity: region; census year. It stops at the region in this checked report.
- Year: 2024.
- Access: direct download; PDF. The report's underlying commune origin-destination matrix was not checked.
- Check: ROWS VERIFIED (bounded sample) 2026-11-21, HTTP 200, Figure 33 and Table 15 values read from the PDF.
- Limits: The report says regular commuting need not happen daily. The 72,798 figure is Metropolitan Region, not Santiago city, and does not show where workers stay or which days they travel.
- Use in the simulation: TARGET for the regional inbound worker stock; INPUT for a boundary inflow constraint.

### MOP Ruta 5 South and South Access toll transactions
- Link: https://concesiones.mop.gob.cl/uploads/sites/4/2026/03/Diptico_SATA_2025_12.pdf
- Contents: Monthly transactions by toll plaza and vehicle category. The December 2025 report lists 8,955,180 vehicle transactions across the concession.
- Granularity: toll plaza; month. No person origin or destination.
- Year: 2021–2025 monthly history in the opened report; detailed plaza split for December 2025.
- Access: direct download; PDF, 681,709 bytes. Run `/usr/bin/python3 tools/scrape/tourism_visitors.py mop-south`.
- Check: ROWS VERIFIED (bounded sample) 2026-11-21, HTTP 200, PDF signature and December 2025 count read.
- Limits: One vehicle can make many transactions. The concession includes trips inside the region and traffic beyond the city. It cannot identify nonresidents or occupancy.
- Use in the simulation: TARGET for vehicle flows at relevant entry plazas after selecting them; not a visitor count.

### MOP Santiago–Colina–Los Andes toll transactions
- Link: https://concesiones.mop.gob.cl/uploads/sites/4/2026/03/Diptico_SCLA_2025_12.pdf
- Contents: Monthly toll transactions by plaza and vehicle category. The December 2025 report lists 2,817,883 transactions for the whole concession.
- Granularity: toll plaza; month. No person origin or destination.
- Year: 2025 report with earlier comparison months.
- Access: direct download; PDF. Run `/usr/bin/python3 tools/scrape/tourism_visitors.py mop-north`.
- Check: ROWS VERIFIED (bounded sample) 2026-11-21, HTTP 200, PDF signature and December 2025 count read.
- Limits: The whole-concession total is not an inbound count. Local traffic and repeat transactions dominate some plazas. Select boundary plazas before calibration.
- Use in the simulation: TARGET for northern road boundary flow after plaza selection.

### Sernatur protected-area visits via SIET
- Link: https://datosturismo.sernatur.cl/siet/reporteDinamicoSNASPE
- Contents: Visits to protected areas, with region, nationality or residence, age, sex, and accessibility categories. The opened Metropolitan Region row has 33,314 visits for 2024.
- Granularity: protected area or region; month through filters. The checked row stops at the region.
- Year: January to December 2024 in the opened report.
- Access: web report and Excel export; export URL and size not checked.
- Check: ROWS VERIFIED (bounded sample) 2026-11-21, HTTP 200, Metropolitan Region row read.
- Limits: Protected areas are only one narrow activity type. Visits do not measure unique visitors to Santiago and can occur outside the urban area.
- Use in the simulation: NONE for the current urban visitor model.

## Critique

1. The national air and border series do not show whether a person stays in Santiago. I kept them as boundary constraints, not city visitor counts.
2. The best spatial lodging source is platform supply, not occupancy. EMAT gives measured nights but its checked row stops at the region.
3. No open daily origin-destination series for interregional bus, train, or car visitors was confirmed. The census commuter figure measures regular workers, not daily arrivals. I did not check the SIET export URLs, the full Sernatur CSVs, or a Greater Santiago comuna filter.
