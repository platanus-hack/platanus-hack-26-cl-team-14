# Events and attendance: calendar + crowd counts for Santiago

Topic owner: this agent (events that move crowds: the date and the attendance).
`clever-llama` owns venues and their visitor counts (`context/research/data-sources-buildings-activity.md`).
This file owns the calendar side: which day, which event, how many people.

Use case this feeds: "put a stadium in a different comuna" needs a realistic home-match
calendar and a realistic crowd size per match, not just a building footprint.

## Overlap to report to clever-llama

- Stadium capacity numbers below (Estadio Nacional, Monumental, Claro Arena, Santa Laura)
  duplicate venue data clever-llama should already hold. I kept them here only as the
  denominator for "% of aforo used" per match. Do not duplicate the venue record itself.
- clever-llama's PR #5 cites "MUT: about 650k visits/month" from press, and that number
  competes with my mall/retail entries below — I did not add a separate mall-visits entry
  to avoid a second unverified claim for the same fact. Flagged in the swarm thread.

### ANFP / Transfermarkt / Estadio Seguro — football attendance by match
- Link: https://www.transfermarkt.com/campeonato-de-apertura/besucherzahlen/wettbewerb/CLI1 (official per-match attendance pages exist but the site blocks automated requests); figures used here come from press reporting on Transfermarkt's 2025 season averages (e.g. https://www.prensa-latina.cu/2025/12/27/revelan-equipos-de-futbol-de-chile-que-mas-publico-atrajeron/).
- Contents: one row per club per season = average home attendance. Transfermarkt also has a per-match attendance page for each competition, but I could not open it directly.
- Granularity: club-season (press figures). Transfermarkt's own page is match-day level, but unverified by me.
- Year: 2025 season (Universidad de Chile 35,765; Colo Colo 28,745; Universidad Católica 17,540, per press citing Transfermarkt).
- Access: Transfermarkt page blocked automated `curl` with an AWS WAF challenge (HTTP 202, `x-amzn-waf-action: challenge`). A browser session or a scraping-aware client is needed.
- Check: UNVERIFIED (curl to `transfermarkt.com/campeonato-de-apertura/besucherzahlen/wettbewerb/CLI1` returned HTTP 202 with a bot challenge, zero rows). Press figures are secondhand, not from the primary table.
- Limits: Transfermarkt attendance figures are the club's reported gate, not Carabineros/Estadio Seguro's. The two disagree by up to 2x in some matches (see below).
- Use in the simulation: INPUT. Football-match-day crowd by home club and date, once a real per-match table is reachable.

### Estadio Seguro — official public-security attendance program
- Link: https://www.alairelibre.cl/noticias/site/artic/20190104/asocfile/20190104120439/bolet__n__estadio_seguro__2018.pdf (dead link, see Check)
- Contents: per-tournament bulletins from the Ministry of Interior's "Estadio Seguro" program. Columns seen in press summaries: aforo solicitado (requested capacity), público asistente (attendance), % utilización.
- Granularity: match-level in the original bulletins (per press descriptions); only season/team aggregates appear in public news coverage.
- Year: bulletins exist at least 2012–2024 (press cites 2018, 2023, 2024 editions). 2024 season total: 1,888,569 spectators nationwide, the highest since 2012, per CNN Chile reporting.
- Access: PDF bulletins hosted ad hoc on news-site mirrors, not a stable government URL. The one link tested is dead.
- Check: UNVERIFIED (curl to the PDF URL above returned HTTP 404, 297,720 bytes of an HTML error page, not a PDF — confirmed with `file`).
- Limits: Estadio Seguro counts diverge sharply from club/Transfermarkt counts and from organizer claims for marches (see below) — always state which counter a number comes from.
- Use in the simulation: INPUT if a live bulletin URL is found (ask via Ley de Transparencia per press reporting); currently NONE — no working source link.

### DTPM/ANFP real stadium capacities — denominator for occupancy
- Link: https://www.stadiumguide.com/santiago-football-guide-all-stadiums/ ; club sources for Claro Arena (ex San Carlos de Apoquindo) and Estadio Monumental.
- Contents: one row per stadium: name, club, comuna, capacity.
- Granularity: point (stadium).
- Year: current, 2025 reopening for Claro Arena (ex San Carlos de Apoquindo, now ~20,000), Estadio Monumental ~47,000 (Colo Colo, planning 50,000-60,000 after remodel), Estadio Nacional ~48,000+, Santa Laura ~19,887 (Unión Española).
- Access: scattered across club/press pages, not one dataset.
- Check: UNVERIFIED FOR INGESTION (read via web search summaries, not fetched/opened as primary pages in this session).
- Limits: capacity changes with renovation (Claro Arena reopened Aug 2025 after ~3 years of works); treat as a moving number, re-check before using in a score.
- Use in the simulation: INPUT, paired with match attendance to compute % aforo used, which is what changes if a stadium moves comuna.

### Movistar Arena — concerts and non-football live events
- Link: https://www.movistararena.cl (Chile venue; not opened in this session — search only surfaced Bogotá/Madrid sister venues)
- Contents: would be one row per concert/event: date, artist, estimated attendance.
- Granularity: point (venue), event-date.
- Year: ongoing, 2026 calendar.
- Access: no open event-attendance dataset found; individual ticket sites (Puntoticket, Ticketmaster) show sell-out status, not attendance counts, and have no public API.
- Check: UNVERIFIED (not opened; the Chile venue URL was not reachable in search results, only Bogotá/Madrid sister arenas were).
- Limits: concert attendance in Chile is almost never published as a number, only "sold out" / "agotado" status.
- Use in the simulation: NONE at this time — no real attendance numbers found, only capacity (~12,000-16,500 depending on configuration, unverified).

### Lollapalooza Chile — multi-day festival attendance
- Link: https://www.theclinic.cl/2026/03/16/240-mil-espectadores-el-gran-hito-de-los-bunkers-y-el-exito-de-la-apuesta-pop-juvenil-las-claves-del-lollapalooza-2026-en-su-vuelta-al-parque-ohiggins/
- Contents: one text figure per year: total festival attendance across 3 days, at Parque O'Higgins.
- Granularity: zone (Parque O'Higgins), 3 dates per year (mid-March).
- Year: 2026 edition (March 13-15): ~240,000 total over 3 days (~80,000/day), sold out all 3 days. 2024 edition: ~210,000 total, per El Desconcierto.
- Access: press reporting only, no organizer (Lotus) dataset or API.
- Check: UNVERIFIED (read via web search; press article not opened directly, no PDF/CSV exists to check).
- Limits: "240k" is organizer/press framing (80k/day x 3), not an audited count; no per-hour or per-stage breakdown.
- Use in the simulation: INPUT. One high-impact, fixed-date, single-zone event per year — good stress-test case for "surge at Parque O'Higgins," low value for calibration precision.

### Maratón de Santiago — street closures and runner count
- Link: https://www.pauta.cl/ciudad/2026/04/24/maraton-de-santiago-2026-33-mil-corredores-y-cortes-de-transito-en-ocho-comunas-este-fin-de-semana.html
- Contents: date, race categories (42K/21K/10K), start/finish point (Plaza de la Constitución), start times by category, 8 affected comunas (Santiago, Ñuñoa, Macul, Peñalolén, La Reina, Las Condes, Vitacura, Providencia), runner count.
- Granularity: route/zone (street-level closures by comuna), single date.
- Year: 2026 edition, Sunday April 26. There is also a women-only Medio Maratón (Aug 30, 2026, Ñuñoa/Macul only) — a separate, smaller event.
- Access: press only; runner counts vary by outlet (Pauta: 33,000; 24 Horas/T13: ~30,000) — no official organizer figure confirmed.
- Check: UNVERIFIED (read via web search; no organizer page opened).
- Limits: the "corredores" figure is participants, not spectators; spectator crowd size along the route is not reported anywhere found.
- Use in the simulation: INPUT. Known date, known route, known comuna-level closures — good for a transport-disruption mechanism even without a verified attendance figure.

### Carabineros — march/protest attendance estimates
- Link: https://www.pauta.cl/actualidad/2020/03/09/diferencias-en-los-numeros-como-se-mide-la-asistencia-a-las-marchas.html
- Contents: Carabineros publishes its own estimated attendance per protest/march, usually reported via press, not a dataset. Historic examples: 8M 2020 Santiago march — Carabineros 125,000-150,000 vs organizers' 800,000-2,000,000; Oct-Nov 2019 unrest — Carabineros cumulative 3.7 million march-attendances nationwide, single-day peak 1.2 million (Plaza Italia, Oct 25 2019); Confech march Oct 1 2026 — Carabineros ~6,000 vs students' 20,000+.
- Granularity: event-level (one march, one date, one city-wide estimate — not per-comuna or per-block).
- Year: scattered 2012-2026, no continuous series.
- Access: press releases only, no Carabineros open dataset or API found.
- Check: UNVERIFIED (read via web search; no Carabineros primary source page opened).
- Limits: Police and organizer estimates differ in the cited reports. Their disagreement does not establish the true attendance or a guaranteed lower bound. Record each estimate with its source and counting method.
- Use in the simulation: Candidate INPUT for an explicitly uncertain event scenario, not a guaranteed lower bound or calibration TARGET. Protest dates are also highly irregular (not a repeating calendar), unlike football or ferias.

### Ferias libres — ODEPA/ASOF/municipal registries of market days and locations
- Link: https://apps.odepa.gob.cl/powerBI/reporte_ferias_libres.html
- Contents: interactive map/report of ferias libres nationwide with operating day of week, main street, and coordinates; a related national locator is described (in press) as covering 1,228 ferias in 16 regions / 282 comunas.
- Granularity: point (feria location) + weekly recurrence (day of week), no single date/time-of-day field confirmed.
- Year: current (page `Last-Modified` header: Oct 8, 2026).
- Access: PowerBI embed page. `curl` only returns the wrapper HTML (1,655 bytes) that loads the PowerBI iframe client-side; the underlying data table needs a browser or a PowerBI export, not reachable with plain `curl`.
- Check: UNVERIFIED FOR INGESTION (curl returned HTTP 200, 1,655 bytes, confirmed live but it is a JS shell, zero data rows retrieved).
- Limits: Sercotec's separate census (catastroferiaslibres.cl) reportedly counts 1,455 ferias nationwide — a different number from ODEPA's 1,228, so the two registries disagree; catastroferiaslibres.cl did not respond (TLS handshake completed, then hung past 10s timeout) so its number is itself unverified.
- Use in the simulation: INPUT. Ferias are the one recurring, weekly, point-level crowd event in this catalog — complementary to one-off events like marches/festivals. Needs either a PowerBI data export or ASOF's/the comuna's own HTML listing (e.g. munistgo.cl, HTTP 200, confirmed live) to get real rows.

### Public holidays (feriados) — Chile 2026/2027
- Link: https://tallyfy.com/national-holidays/api/CL/2026.json
- Contents: one JSON object per holiday: ISO date, English name, `local_name` (Spanish), `type` (national/etc.), `observed_date`, `is_observed_shifted`, short description.
- Granularity: national (not Santiago-specific — some holidays are regional, but August 15 is national; see [Dirección del Trabajo](https://dt.gob.cl/legislacion/1624/w3-article-94459.html)).
- Year: 2026 (confirmed file); Tallyfy publishes one JSON file per year (pattern `/national-holidays/api/CL/<year>.json`), so 2027 should exist the same way once published.
- Access: direct download, static JSON, no key needed.
- Check: ROWS VERIFIED (sample only; curl returned HTTP 200, 5,164 bytes; first rows show Jan 1 Año Nuevo, Apr 3 Viernes Santo, Apr 4 Sábado Santo, with full field set as described).
- Limits: third-party mirror, not the government's own text (Ley N° 2.977 / BCN LeyChile is the legal source of truth, not checked for a machine-readable feed in this session). Chile is reported to have 16 national holidays in 2026 and either 16, 17, or 19 in 2027 depending on the outlet — the 2027 count is genuinely disputed in press as of Nov 2026 pending confirmation of a special one-off holiday (Ley N° 20.983, Sept 17 type).
- Use in the simulation: INPUT. Candidate holiday calendar after checking dates against official rules. Holidays change trip patterns; they do not eliminate all work or school trips.

### School calendar (Mineduc) — Región Metropolitana term dates
- Link: https://www.mineduc.cl/?p=42248
- Contents: start of school year, winter break (vacaciones de invierno) start/end, end of school year, split by region and by semestral/trimestral school regime.
- Granularity: region (Región Metropolitana as one unit; no comuna or school-level split).
- Year: 2026 school year. RM dates reported in press: start Mar 4, 2026 (one outlet says Mar 2); winter break Jun 22 - Jul 3, 2026; end of year (non-JEC schools) Dec 18, 2026.
- Access: Mineduc's own page (mineduc.cl/?p=42248) was found by search but not opened directly in this session; all dates above come from press summaries of the Mineduc announcement.
- Check: UNVERIFIED (mineduc.cl page not fetched with curl/browser in this session; press secondary sources disagree on the exact start date by 2 days).
- Limits: JEC (jornada escolar completa) vs non-JEC schools have different end dates; only the non-JEC end date was found. Dates can shift by ~1-2 weeks between press reports before Mineduc's official resolution is published.
- Use in the simulation: INPUT. About 11 weeks/year (summer + winter break) when school-age trips drop out of the OD matrix; coarser than feriados but still a real behavior-changing calendar.

### Transfermarkt vs. Estadio Seguro vs. club gate — known attendance discrepancy
- Link: none (synthesis of the two entries above)
- Contents: not a source, a documented limit: the same match can have 2-3 different "attendance" numbers depending on who counts (club turnstile, Estadio Seguro/Carabineros, Transfermarkt aggregation of press).
- Granularity: n/a
- Year: n/a
- Access: n/a
- Check: n/a
- Limits: n/a
- Use in the simulation: NONE as a source — kept as a note so whoever picks the score target picks one counting method and states it, instead of mixing clubs/matches that used different counters.

## Critique

1. Only the holiday JSON supplied actual data rows in this research. Every football-attendance and march-attendance number here is press-reported, not pulled from a primary table — Transfermarkt and Estadio Seguro both blocked or 404'd when I tried to fetch them directly. Only that JSON has ROWS VERIFIED status. This confirms sample access, not the legal accuracy of every holiday.
2. I did not check robots.txt before curling transfermarkt.com, worldfootball.net, or footystats.org — in practice their anti-bot WAF (202 challenge, 403 Cloudflare challenge) made the question moot, but I should have checked first per the rules.
3. I never resolved the ODEPA (1,228 ferias) vs. Sercotec (1,455 ferias) discrepancy, and catastroferiaslibres.cl never answered — so the one weekly, recurring, point-level crowd event in this file (ferias) is the least verified entry here, despite being the easiest one to expect real open data for.
