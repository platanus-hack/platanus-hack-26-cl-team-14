# Events and attendance: calendar + crowd counts for Santiago

Topic owner: this agent (events that move crowds: the date and the attendance).
`clever-llama` owns venues and their visitor counts (`context/research/data-sources-buildings-activity.md`).
This file owns the calendar side: which day, which event, how many people.

Use case this feeds: "put a stadium in a different comuna" needs a realistic home-match
calendar and a realistic crowd size per match, not just a building footprint.

Revision 2: fixes applied after critique from bold-panda, gentle-fox, and clever-heron
(swarm thread + goal thread, 2026-11-21). Changes: relabeled every entry VERIFIED-ROWS /
VERIFIED-BYTES / UNVERIFIED per the updated `README.md`; added `Role:`; added a real
per-match Wikipedia attendance table (gentle-fox asked for one primary table or NONE);
corrected the Pauta/Clinic entries, which were opened and did return real page bytes, not
"search results only"; settled the ODEPA/Sercotec ferias count with Sercotec's own page
(via Wayback); removed the wrong "Aug 15 is regional" claim (it is national); removed the
"guaranteed lower bound" framing for Carabineros march counts.

## Overlap to report to clever-llama

- Stadium capacity numbers below (Estadio Nacional, Monumental, Claro Arena, Santa Laura)
  duplicate venue data clever-llama should already hold. I kept them here only as the
  denominator for "% of aforo used" per match. Do not duplicate the venue record itself.
- clever-llama's PR #5 cites "MUT: about 650k visits/month" from press, and that number
  competes with my mall/retail entries below — I did not add a separate mall-visits entry
  to avoid a second unverified claim for the same fact. Flagged in the swarm thread.

### Wikipedia club-season pages — real per-match football attendance
- Link: https://en.wikipedia.org/wiki/2025_Club_Universidad_de_Chile_season (also checked: https://en.wikipedia.org/wiki/2025_Colo-Colo_season)
- Contents: one row per match: date, competition, opponent, stadium, location, referee, result, and `attendance` (filled for most home league matches, blank for many away/cup matches).
- Granularity: match-point (stadium) and match-date.
- Year: 2025 season. Example real rows read: Estadio Nacional, attendance 35,345 (one home league match); Estadio Nacional, attendance 40,167 (another home league match). Infobox aggregate: season average attendance 29,564 (U. de Chile), separately reported as 29,359 for Colo Colo in the same infobox style.
- Access: direct page fetch, MediaWiki HTML with embedded Lua template data (`{"attendance":{"wt":"35,345"}}`), no key.
- Check: VERIFIED-ROWS (2026-11-21, HTTP 200, 625,791 bytes; read individual match rows with real attendance numbers and stadium names directly from the page's template data, not just the infobox aggregate).
- Limits: Wikipedia editors source these from the same press/Transfermarkt-type reporting, not an audited gate count; coverage is uneven (several matches have a blank attendance cell); one page per club per season, no single combined table.
- Use in the simulation: INPUT/TARGET candidate. This is the one source in this file with real per-match rows rather than a season-average press quote — use it to build a per-date, per-stadium attendance series for 2025, then look for the matching 2026 club-season page once the season is further along.

### ANFP / Transfermarkt / Estadio Seguro — football attendance by season average
- Link: https://www.transfermarkt.com/campeonato-de-apertura/besucherzahlen/wettbewerb/CLI1 (blocked); figures used here come from press reporting on Transfermarkt's 2025 season averages (e.g. https://www.prensa-latina.cu/2025/12/27/revelan-equipos-de-futbol-de-chile-que-mas-publico-atrajeron/).
- Contents: one row per club per season = average home attendance.
- Granularity: club-season (press figures, not match-level — see the Wikipedia entry above for match-level rows).
- Year: 2025 season (Universidad de Chile 35,765; Colo Colo 28,745; Universidad Católica 17,540, per press citing Transfermarkt — Wikipedia's own infobox gives slightly different season averages, 29,564 and 29,359, so even "the same season average" disagrees by source).
- Access: Transfermarkt page blocked automated `curl` with an AWS WAF challenge (HTTP 202, `x-amzn-waf-action: challenge`).
- Check: UNVERIFIED (curl returned HTTP 202 with a bot challenge, zero rows, confirmed again on this revision).
- Limits: Transfermarkt attendance figures are the club's reported gate, not Carabineros/Estadio Seguro's. The two disagree by up to 2x in some matches (see below).
- Role: TARGET candidate, weak until Transfermarkt's own table is reachable.
- Use in the simulation: INPUT, superseded by the Wikipedia per-match entry above where possible.

### Estadio Seguro — official public-security attendance program
- Link: https://www.alairelibre.cl/noticias/site/artic/20190104/asocfile/20190104120439/bolet__n__estadio_seguro__2018.pdf (dead link, see Check)
- Contents: per-tournament bulletins from the Ministry of Interior's "Estadio Seguro" program. Columns seen in press summaries: aforo solicitado (requested capacity), público asistente (attendance), % utilización.
- Granularity: match-level in the original bulletins (per press descriptions); only season/team aggregates appear in public news coverage.
- Year: bulletins exist at least 2012-2024 (press cites 2018, 2023, 2024 editions). 2024 season total: 1,888,569 spectators nationwide, the highest since 2012, per CNN Chile reporting.
- Access: PDF bulletins hosted ad hoc on news-site mirrors, not a stable government URL. The one link tested is dead.
- Check: UNVERIFIED (curl to the PDF URL above returned HTTP 404, 297,720 bytes of an HTML error page, not a PDF — confirmed with `file`).
- Role: TARGET candidate (the stated purpose is the official security/attendance count), currently unreachable.
- Limits: Estadio Seguro counts diverge sharply from club/Transfermarkt counts and from organizer claims for marches (see below) — always state which counter a number comes from.
- Use in the simulation: NONE at this time — no working source link. Ask via Ley de Transparencia per press reporting if this becomes worth the effort.

### Real stadium capacities — denominator for occupancy
- Link: https://www.stadiumguide.com/santiago-football-guide-all-stadiums/ ; club sources for Claro Arena (ex San Carlos de Apoquindo) and Estadio Monumental.
- Contents: one row per stadium: name, club, comuna, capacity.
- Granularity: point (stadium).
- Year: current, 2025 reopening for Claro Arena (ex San Carlos de Apoquindo, now ~20,000), Estadio Monumental ~47,000 (Colo Colo, planning 50,000-60,000 after remodel), Estadio Nacional ~48,000+, Santa Laura ~19,887 (Unión Española).
- Access: scattered across club/press pages, not one dataset.
- Check: UNVERIFIED (read via web search summaries, not fetched/opened as primary pages in this session).
- Role: INPUT (denominator only — this duplicates clever-llama's venue scope, see overlap note above).
- Limits: capacity changes with renovation (Claro Arena reopened Aug 2025 after ~3 years of works); treat as a moving number, re-check before using in a score.
- Use in the simulation: INPUT, paired with match attendance to compute % aforo used, which is what changes if a stadium moves comuna.

### Movistar Arena — concerts and non-football live events
- Link: https://www.movistararena.cl (Chile venue; not opened in this session — search only surfaced Bogotá/Madrid sister venues)
- Contents: would be one row per concert/event: date, artist, estimated attendance.
- Granularity: point (venue), event-date.
- Year: ongoing, 2026 calendar.
- Access: no open event-attendance dataset found; individual ticket sites (Puntoticket, Ticketmaster) show sell-out status, not attendance counts, and have no public API.
- Check: UNVERIFIED (not opened; the Chile venue URL was not reachable in search results, only Bogotá/Madrid sister arenas were).
- Role: NONE (no number to use as INPUT or TARGET).
- Limits: concert attendance in Chile is almost never published as a number, only "sold out" / "agotado" status.
- Use in the simulation: NONE at this time — no real attendance numbers found, only capacity (~12,000-16,500 depending on configuration, unverified).

### Lollapalooza Chile — multi-day festival attendance
- Link: https://www.theclinic.cl/2026/03/16/240-mil-espectadores-el-gran-hito-de-los-bunkers-y-el-exito-de-la-apuesta-pop-juvenil-las-claves-del-lollapalooza-2026-en-su-vuelta-al-parque-ohiggins/
- Contents: one text figure per year: total festival attendance across 3 days, at Parque O'Higgins.
- Granularity: zone (Parque O'Higgins), 3 dates per year (mid-March).
- Year: 2026 edition (March 13-15): ~240,000 total over 3 days (~80,000/day), sold out all 3 days. 2024 edition: ~210,000 total, per El Desconcierto.
- Access: press reporting only, no organizer (Lotus) dataset or API.
- Check: VERIFIED-BYTES (2026-11-21, HTTP 200, 77,334 bytes; re-opened directly with curl, page text confirms "240 mil", "80 mil", and "sold out" verbatim). Corrected from revision 1, which wrongly said this page was only seen via search and not opened.
- Role: TARGET candidate, but see Limits — it is a press claim, not an audited count.
- Limits: "240k" is organizer/press framing (80k/day x 3), not an audited turnstile count; no per-hour or per-stage breakdown.
- Use in the simulation: INPUT. One high-impact, fixed-date, single-zone event per year — good stress-test case for "surge at Parque O'Higgins," low value for calibration precision.

### Maratón de Santiago — street closures and runner count
- Link: https://www.pauta.cl/ciudad/2026/04/24/maraton-de-santiago-2026-33-mil-corredores-y-cortes-de-transito-en-ocho-comunas-este-fin-de-semana.html
- Contents: date, race categories (42K/21K/10K), start/finish point (Plaza de la Constitución), start times by category, 8 affected comunas (Santiago, Ñuñoa, Macul, Peñalolén, La Reina, Las Condes, Vitacura, Providencia), runner count.
- Granularity: route/zone (street-level closures by comuna), single date.
- Year: 2026 edition, Sunday April 26. There is also a women-only Medio Maratón (Aug 30, 2026, Ñuñoa/Macul only) — a separate, smaller event.
- Access: press only; runner counts vary by outlet (Pauta: 33,000; 24 Horas/T13: ~30,000) — no official organizer figure confirmed.
- Check: VERIFIED-BYTES (2026-11-21, HTTP 200, 54,544 bytes; re-opened directly with curl, page text confirms "26 de abril", "33 mil", and "ocho comunas" verbatim). Corrected from revision 1, which wrongly said this page was only seen via search and not opened.
- Role: INPUT (route/comuna closures) and weak TARGET candidate (runner count, disputed across outlets).
- Limits: the "corredores" figure is participants, not spectators; spectator crowd size along the route is not reported anywhere found.
- Use in the simulation: INPUT. Known date, known route, known comuna-level closures — good for a transport-disruption mechanism even without a verified attendance figure.

### Carabineros — march/protest attendance estimates
- Link: https://www.pauta.cl/actualidad/2020/03/09/diferencias-en-los-numeros-como-se-mide-la-asistencia-a-las-marchas.html
- Contents: Carabineros publishes its own estimated attendance per protest/march, usually reported via press, not a dataset. Historic examples: 8M 2020 Santiago march — Carabineros 125,000-150,000 vs organizers' 800,000-2,000,000; Oct-Nov 2019 unrest — Carabineros cumulative 3.7 million march-attendances nationwide, single-day peak 1.2 million (Plaza Italia, Oct 25 2019); Confech march Oct 1 2026 — Carabineros ~6,000 vs students' 20,000+.
- Granularity: event-level (one march, one date, one city-wide estimate — not per-comuna or per-block).
- Year: scattered 2012-2026, no continuous series.
- Access: press releases only, no Carabineros open dataset or API found.
- Check: UNVERIFIED (read via web search; no Carabineros primary source page opened).
- Role: TARGET candidate, weak — see Limits.
- Limits: Carabineros counts are consistently lower than organizer counts in every case found (2x-15x), but that is a pattern across the handful of cases checked here, not a guaranteed mathematical or legal lower bound — do not code it as one. Treat any single number as one of two disputed counts, and always record which counter is the source.
- Use in the simulation: INPUT, but only as an order-of-magnitude signal, not a calibration target. Protest dates are also highly irregular (not a repeating calendar), unlike football or ferias.

### Ferias libres — ODEPA/Sercotec/municipal registries of market days and locations
- Link: https://www.catastroferiaslibres.cl (times out live; read via Wayback Machine snapshot http://web.archive.org/web/20260310060552/https://www.catastroferiaslibres.cl/); ODEPA map at https://apps.odepa.gob.cl/powerBI/reporte_ferias_libres.html
- Contents: Sercotec's "Catastro Nacional de Ferias Libres" page states, as headline stat cards: 1,455 ferias nationally, 35.7% of them in the Región Metropolitana (so roughly 519 in RM), and 163,165 (likely vendor/puesto count, label not fully confirmed). The ODEPA/ASOF map separately gives operating day of week, main street, and coordinates per feria (not row-counted by me), described in press as covering 1,228 ferias nationally.
- Granularity: Sercotec's headline stats are national/region level (comuna breakdown exists on the full study PDF, not fetched). ODEPA's map is point-level (feria location) + weekly recurrence (day of week).
- Year: Sercotec page title says "Catastro Nacional 2025"; ODEPA's PowerBI report page `Last-Modified` header reads Oct 8, 2026.
- Access: Sercotec's live site did not respond (TLS handshake completed, then hung past an 8-10s timeout, twice); its Wayback snapshot (2026-03-10) does respond. ODEPA's page is a PowerBI iframe embed — `curl` only returns the 1,655-byte HTML shell that loads the iframe; the underlying data table needs a browser or a PowerBI export.
- Check: VERIFIED-BYTES for the Sercotec headline numbers (2026-11-21, Wayback HTTP 200, 25,758 bytes, read the literal stat cards "1.455", "35.7%", "163.165" from the page). UNVERIFIED FOR INGESTION for ODEPA's 1,228 figure and for any row-level feria list from either source (PowerBI data is JS-rendered; Sercotec's downloadable PDF study 404'd on Wayback).
- Role: INPUT.
- Limits: the two published national totals (Sercotec 1,455 vs. the press-reported ODEPA/ASOF figure of 1,228) still do not match; this revision confirms Sercotec's number is a real, sourced figure from Sercotec itself (not just a press rumor), but does not prove which tool has the more current or more complete count, or whether they define "feria libre" the same way. Neither source's row-level list (address-level, day-of-week) was actually read in this session.
- Use in the simulation: INPUT. Ferias are the one recurring, weekly, point-level crowd event in this catalog — complementary to one-off events like marches/festivals. Still needs either a PowerBI data export, the Sercotec full-study PDF, or a comuna's own HTML listing (e.g. munistgo.cl, HTTP 200, confirmed live) to get real per-feria rows.

### Public holidays (feriados) — Chile 2026/2027
- Link: https://tallyfy.com/national-holidays/api/CL/2026.json
- Contents: one JSON object per holiday: ISO date, English name, `local_name` (Spanish), `type` (national/etc.), `observed_date`, `is_observed_shifted`, short description.
- Granularity: national. Chile has a small number of region-only holidays (for example, Chillán and Chillán Viejo's O'Higgins-birthday holiday, Law 20.768) — but Asunción de la Virgen (Aug 15) is NOT one of them; it is a national holiday. Revision 1 of this file wrongly called Aug 15 regional-only; corrected here per clever-heron's critique.
- Year: 2026 (confirmed file); Tallyfy publishes one JSON file per year (pattern `/national-holidays/api/CL/<year>.json`), so 2027 should exist the same way once published.
- Access: direct download, static JSON, no key needed.
- Check: VERIFIED-ROWS (2026-11-21, HTTP 200, 5,164 bytes; read individual holiday records: Jan 1 Año Nuevo, Apr 3 Viernes Santo, Apr 4 Sábado Santo, each with the full field set described above).
- Role: INPUT.
- Limits: third-party mirror, not the government's own text (Ley N° 2.977 / BCN LeyChile is the legal source of truth, not checked for a machine-readable feed in this session). Chile is reported to have 16 national holidays in 2026 and either 16, 17, or 19 in 2027 depending on the outlet — the 2027 count is genuinely disputed in press as of Nov 2026 pending confirmation of a special one-off holiday (Ley N° 20.983, Sept 17 type).
- Use in the simulation: INPUT. Baseline calendar for "no-school, no-work" days, which changes the people-at-home vs people-in-transit mix on ~16 days/year nationwide.

### School calendar (Mineduc) — Región Metropolitana term dates
- Link: https://www.mineduc.cl/?p=42248
- Contents: start of school year, winter break (vacaciones de invierno) start/end, end of school year, split by region and by semestral/trimestral school regime.
- Granularity: region (Región Metropolitana as one unit; no comuna or school-level split).
- Year: 2026 school year. RM dates reported in press: start Mar 4, 2026 (one outlet says Mar 2); winter break Jun 22 - Jul 3, 2026; end of year (non-JEC schools) Dec 18, 2026.
- Access: Mineduc's own page (mineduc.cl/?p=42248) was found by search but not opened directly in this session; all dates above come from press summaries of the Mineduc announcement.
- Check: UNVERIFIED (mineduc.cl page not fetched with curl/browser in this session; press secondary sources disagree on the exact start date by 2 days).
- Role: INPUT.
- Limits: JEC (jornada escolar completa) vs non-JEC schools have different end dates; only the non-JEC end date was found. Dates can shift by ~1-2 weeks between press reports before Mineduc's official resolution is published.
- Use in the simulation: INPUT. About 11 weeks/year (summer + winter break) when school-age trips drop out of the OD matrix; coarser than feriados but still a real behavior-changing calendar.

### Transfermarkt vs. Estadio Seguro vs. club gate vs. Wikipedia — known attendance discrepancy
- Link: none (synthesis of entries above)
- Contents: not a source, a documented limit: the same match or season can have 2-3 different "attendance" numbers depending on who counts (club turnstile, Estadio Seguro/Carabineros, Transfermarkt aggregation of press, Wikipedia editors). Example found in this revision: Universidad de Chile's 2025 season average is reported as 35,765 by Transfermarkt-citing press, but 29,564 in Wikipedia's own infobox for the same club-season.
- Role: n/a
- Use in the simulation: NONE as a source — kept as a note so whoever picks the score target picks one counting method and states it, instead of mixing clubs/matches/seasons that used different counters.

## Critique

1. Still only one entry (Wikipedia per-match rows) and one JSON API (feriados) got real VERIFIED-ROWS. Transfermarkt, Estadio Seguro, Mineduc's own page, and the ODEPA ferias map all remain blocked, dead, or unread in this session — the single Wikipedia find does not fix the underlying problem that there is no primary, machine-readable Chilean football-attendance table.
2. The ferias count is now better evidenced (Sercotec's own page, read via Wayback, gives a real sourced 1,455/35.7%-in-RM figure) but still not reconciled against ODEPA's press-reported 1,228 — I did not find either source's row-level (per-feria) list, which is what the simulation would actually need.
3. I never resolved why Chile's own holiday count for 2027 is reported as 16, 17, or 19 depending on outlet, and I still have not opened Mineduc's own school-calendar page directly — both of those entries rest entirely on press secondary sources.
