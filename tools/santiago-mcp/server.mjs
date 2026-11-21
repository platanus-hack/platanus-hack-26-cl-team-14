import { readFileSync } from 'node:fs';
import { createInterface } from 'node:readline';

const data = (file) => readFileSync(new URL(`../../data/mvp/${file}`, import.meta.url), 'utf8');
// Named job places (data/mvp/named_places.csv): id -> block, status and other candidate blocks.
const [, ...placeRows] = data('named_places.csv').trim().split('\n').map((line) => line.split(','));
const places = Object.fromEntries(placeRows.map((r) => [r[0], { block: r[5], status: r[12], longitude: +r[3], latitude: +r[4], nearby_candidate_blocks: r[9] ? r[9].split(' ') : [] }]));
// Comuna of each Metro station point (the point, not every entrance).
const inRing = ([x, y], ring) => ring.reduce((c, [x1, y1], i) => {
  const [x2, y2] = ring[(i + 1) % ring.length];
  return (y1 > y) !== (y2 > y) && x < x1 + (y - y1) * (x2 - x1) / (y2 - y1) ? !c : c;
}, false);
const communes = JSON.parse(data('communes.geojson')).features;
const comunaName = Object.fromEntries(communes.map((f) => [f.properties.cut, f.properties.name]));
// Census 2024 p45_medio_transporte codes (data/mvp/README.md).
const censusMode = { 1: 'car', 2: 'public transport (bus, Metro, train, taxi, colectivo)', 3: 'walking', 4: 'bicycle or scooter', 5: 'motorcycle', 6: 'horse or boat', 7: 'other', '-99': 'no answer' };
const named = (counts, names) => Object.fromEntries(Object.entries(counts).map(([k, v]) => [names[k] ?? k, v]));
const comunaAt = (p) => communes.find((f) => f.geometry.coordinates.some(([outer, ...holes]) => inRing(p, outer) && !holes.some((h) => inRing(p, h))))?.properties;
const currentStations = new Set(JSON.parse(data('metro_training.json')).map((s) => s.id));

const api = process.env.SANTIAGO_API_URL ?? 'http://127.0.0.1:8787';
const comunas = '13101 Santiago, 13102 Cerrillos, 13103 Cerro Navia, 13104 Conchalí, 13105 El Bosque, 13106 Estación Central, 13107 Huechuraba, 13108 Independencia, 13109 La Cisterna, 13110 La Florida, 13111 La Granja, 13112 La Pintana, 13113 La Reina, 13114 Las Condes, 13115 Lo Barnechea, 13116 Lo Espejo, 13117 Lo Prado, 13118 Macul, 13119 Maipú, 13120 Ñuñoa, 13121 Pedro Aguirre Cerda, 13122 Peñalolén, 13123 Providencia, 13124 Pudahuel, 13125 Quilicura, 13126 Quinta Normal, 13127 Recoleta, 13128 Renca, 13129 San Joaquín, 13130 San Miguel, 13131 San Ramón, 13132 Vitacura, 13201 Puente Alto, 13202 Pirque, 13203 San José de Maipo, 13301 Colina, 13302 Lampa, 13303 Tiltil, 13401 San Bernardo, 13402 Buin, 13403 Calera de Tango, 13404 Paine, 13501 Melipilla, 13502 Alhué, 13503 Curacaví, 13504 María Pinto, 13505 San Pedro, 13601 Talagante, 13602 El Monte, 13603 Isla de Maipo, 13604 Padre Hurtado, 13605 Peñaflor';
const tool = {
  name: 'simulate_city',
  description: `Run one usual work day of Santiago (2.7 million synthetic workers, Census 2024) after a list of changes, and update the open city view. The workers choose a job place and a mode again. Each call starts from the baseline and applies the complete list together; an empty list restores the baseline. Translate the user's question into changes: a new building, mall or office is jobs and/or residents at a point (use its longitude and latitude); the jobs of a named existing place, or an event there, use kind "jobs" or "event" with {type:"place",id}, ids with their jobs status: ${placeRows.map((r) => `${r[0]} (${r[12]})`).join(', ')}; never send a point for a listed place; an event at any listed place uses its point, whatever its jobs status; for a jobs change, an unresolved place returns an error, so ask the user for a block id, and a multi_block place changes only its main census block: say that, and do not say how large the effect on the whole place would be; a named place takes no residents; to remove, reduce or scale an existing place that is not listed, use op "set" 0 or "multiply" at its point: the engine takes the nearest existing point within 500 m, which can be a neighbour block; applied[].understood gives the jobs_before that it took; a policy for a whole comuna uses the comuna id; a faster or slower trip is minutes; closing or reopening an existing Metro station is kind "station" with target {type:"station",id:<station name, for example "TOBALABA">}, op "set" and value 0 (closed) or 1 (open). An event (a concert, a match) is kind "event" with a point target, op "set", value = attendance and assumed_end_half_hour (0 to 47, 40 = 20:00); the result has events[] with the attendees by mode, by return comuna and the Metro users. The event estimate is demand for the whole return period, separate from the commuter station totals: the station deltas and the trip deltas do not include it. The end half hour does not select the service of that hour: the estimate is the same for an end at 20:00 and at 23:00. Opening a planned Metro line is kind "station" with target {type:"line",id:"L6"|"L7"|"L8"|"L9"}, op "set", value 1; a proposed new station is kind "station" with target {type:"point",id:<short id>,name,longitude,latitude}, op "set", value 1. A new station has base_daily 0 and its scenario_daily is modeled weekday Metro entries; the result lists them in new_stations. The engine models only which station people enter: it has no capacity, travel-time saving, transfer or mode change for a new station, so do not state any of these. For a question the kinds cannot express, the engine returns it in not_modeled, and you must tell the user that the engine cannot simulate that part yet. All results are model values, not measurements; say that, and read the assumptions. State only what a field of the result says. Do not infer distances, durations, causes, sectors, or what a real-world effect would be; a station's place is its comuna field. summary.workers_moved counts workers who change job place (lugar de trabajo), which can be in the same comuna. Give the unit of each number (daily Metro boardings on a work day, workers, jobs). Answer in 6 lines or fewer unless the user asks for more. Comuna ids: ${comunas}.`,
  inputSchema: {
    type: 'object',
    required: ['changes'],
    properties: {
      stations: { type: 'array', items: { type: 'string' }, description: 'Optional. Names of the Metro stations the user asked about, for example ["UNIVERSIDAD DE CHILE"]. Their rows always come back in asked_stations.' },
      changes: {
        type: 'array',
        items: {
          type: 'object',
          required: ['kind', 'target', 'op', 'value'],
          properties: {
            kind: { enum: ['jobs', 'residents', 'minutes', 'station', 'event'], description: 'jobs: work places. residents: persons who live there (workers follow at the comuna rate). minutes: door-to-door travel minutes between comunas. station: close or reopen an existing Metro station, open a planned line, or open a new station at a point.' },
            target: {
              type: 'object',
              required: ['type'],
              description: 'jobs and residents: {type:"point",longitude,latitude} for a place (new or existing), {type:"comuna",id} for a whole comuna, {type:"block",id} for a census block id, {type:"place",id} for a named place. minutes: {type:"comuna",id} for all trips to and from a comuna, or {type:"pair",origin,destination} for one direction between two comunas.',
              properties: {
                type: { enum: ['point', 'comuna', 'block', 'place', 'pair', 'station', 'line'] },
                name: { type: 'string' },
                id: { type: 'string' },
                longitude: { type: 'number' },
                latitude: { type: 'number' },
                origin: { type: 'string' },
                destination: { type: 'string' },
              },
            },
            op: { enum: ['add', 'multiply', 'set'], description: 'add: value is a count or minutes, negative to remove. multiply: value is a factor. set: value is the new amount.' },
            value: { type: 'number' },
            assumed_end_half_hour: { type: 'integer', description: 'event only: the half hour of the day when the event ends, 0 to 47.' },
          },
        },
      },
    },
  },
};

async function dispatch({ method, params }) {
  if (method === 'initialize') return {
    protocolVersion: '2024-11-05',
    capabilities: { tools: {} },
    serverInfo: { name: 'santiago', version: '2.0.0' },
  };
  if (method === 'ping') return {};
  if (method === 'tools/list') return { tools: [tool] };
  if (method !== 'tools/call' || params?.name !== tool.name) return null;
  try {
    const placesUsed = [];
    for (const change of params.arguments.changes ?? []) {
      if (change.target?.type !== 'place') continue;
      const place = places[change.target.id];
      // A named place is a job block; a residents change there would ask the engine for a home block it does not have.
      if (!place) throw new Error(`Unknown place "${change.target.id}". Known places: ${Object.keys(places).join(', ')}.`);
      // An event needs only the place point.
      if (change.kind === 'event') { change.target = { type: 'point', longitude: place.longitude, latitude: place.latitude }; continue; }
      if (change.kind !== 'jobs') throw new Error(`A named place takes kind "jobs" or "event", not "${change.kind}". Use a point or a comuna for residents.`);
      if (!place.block) throw new Error(`Place "${change.target.id}" is unresolved for jobs: no mapped building contains its point (data/mvp/named_places.csv). Ask the user for a census block id.`);
      placesUsed.push({ place: change.target.id, block: place.block, status: place.status, nearby_candidate_blocks: place.nearby_candidate_blocks });
      change.target = { type: 'block', id: place.block };
    }
    const { stations: asked = [], ...request } = params.arguments;
    const response = await fetch(new URL('/api/scenario', api), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request),
    });
    if (!response.ok) throw new Error(await response.text());
    // The map reads the full result from the engine. The agent gets a short copy: the 5 largest station gains and losses and the 5 largest zone job changes.
    const { trips_base, trips_scenario, trips_delta, stations, zones, flows, blocks_changed, ...result } = await response.json();
    const byDelta = stations.map(({ id, name, longitude, latitude, base_daily, scenario_daily, delta_daily }) => {
      const comuna = comunaAt([longitude, latitude]);
      return { id, name, comuna: comuna?.name, comuna_cut: comuna?.cut, base_daily, scenario_daily, delta_daily };
    })
      .sort((a, b) => b.delta_daily - a.delta_daily);
    result.events = result.events.map(({ attendees_by_census_mode, attendees_by_return_comuna, ...event }) => ({
      ...event, attendees_by_mode: named(attendees_by_census_mode, censusMode), attendees_by_return_comuna: named(attendees_by_return_comuna, comunaName) }));
    delete result.summary.top_station_gains;
    delete result.summary.top_station_losses;
    const plain = (name) => name.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toUpperCase().trim();
    result.asked_stations = asked.map((name) => byDelta.find((s) => plain(s.name) === plain(name)) ?? { name, not_found: true });
    const opened = byDelta.filter((s) => !currentStations.has(s.id));
    if (opened.length) result.new_stations = { count: opened.length, entries_daily: opened.reduce((t, s) => t + s.scenario_daily, 0), largest: opened.slice(0, 5) };
    result.station_gains = byDelta.slice(0, 5).filter((s) => s.delta_daily > 0);
    result.station_losses = byDelta.slice(-5).reverse().filter((s) => s.delta_daily < 0);
    result.zones_largest_job_change = zones.map(({ id, name, jobs_base, jobs_scenario, residents_working_base, residents_working_scenario }) => ({ id, name, jobs_base, jobs_scenario, residents_working_base, residents_working_scenario }))
      .filter((z) => z.jobs_scenario !== z.jobs_base || z.residents_working_scenario !== z.residents_working_base)
      .sort((a, b) => Math.abs(b.jobs_scenario - b.jobs_base) - Math.abs(a.jobs_scenario - a.jobs_base)).slice(0, 5);
    if (placesUsed.length) result.places_used = placesUsed;
    result.blocks_changed = blocks_changed.slice(0, 5);
    result.omitted = 'Other stations, zones, comuna flows and trip groups are in the city view, not here. Do not describe them.';
    return { content: [{ type: 'text', text: JSON.stringify(result) }] };
  } catch (error) {
    return { isError: true, content: [{ type: 'text', text: error.message }] };
  }
}

for await (const line of createInterface({ input: process.stdin })) {
  const request = JSON.parse(line);
  if (!Object.hasOwn(request, 'id')) continue;
  const result = await dispatch(request);
  const reply = result === null ? { error: { code: -32601, message: 'Unknown method or tool' } } : { result };
  process.stdout.write(JSON.stringify({ jsonrpc: '2.0', id: request.id, ...reply }) + '\n');
}
