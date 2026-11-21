import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { createInterface } from 'node:readline';

const data = (file) => readFileSync(new URL(`../../data/mvp/${file}`, import.meta.url), 'utf8');
// Named job places (data/mvp/named_places.csv): id -> block, status and other candidate blocks.
const [, ...placeRows] = data('named_places.csv').trim().split('\n').map((line) => line.split(','));
const places = Object.fromEntries(placeRows.map((r) => [r[0], { block: r[5], status: r[12], longitude: +r[3], latitude: +r[4], nearby_candidate_blocks: r[9] ? r[9].split(' ') : [], nearest_block: r[10], nearest_m: r[11] }]));
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
const stationCatalog = JSON.parse(data('metro_training.json'));
const currentStations = new Set(stationCatalog.map(s => s.id));

const api = process.env.SANTIAGO_API_URL ?? 'http://127.0.0.1:8787';
const comunas = '13101 Santiago, 13102 Cerrillos, 13103 Cerro Navia, 13104 Conchalí, 13105 El Bosque, 13106 Estación Central, 13107 Huechuraba, 13108 Independencia, 13109 La Cisterna, 13110 La Florida, 13111 La Granja, 13112 La Pintana, 13113 La Reina, 13114 Las Condes, 13115 Lo Barnechea, 13116 Lo Espejo, 13117 Lo Prado, 13118 Macul, 13119 Maipú, 13120 Ñuñoa, 13121 Pedro Aguirre Cerda, 13122 Peñalolén, 13123 Providencia, 13124 Pudahuel, 13125 Quilicura, 13126 Quinta Normal, 13127 Recoleta, 13128 Renca, 13129 San Joaquín, 13130 San Miguel, 13131 San Ramón, 13132 Vitacura, 13201 Puente Alto, 13202 Pirque, 13203 San José de Maipo, 13301 Colina, 13302 Lampa, 13303 Tiltil, 13401 San Bernardo, 13402 Buin, 13403 Calera de Tango, 13404 Paine, 13501 Melipilla, 13502 Alhué, 13503 Curacaví, 13504 María Pinto, 13505 San Pedro, 13601 Talagante, 13602 El Monte, 13603 Isla de Maipo, 13604 Padre Hurtado, 13605 Peñaflor';
const plain = (name) => name.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toUpperCase().trim();
const scalarChange = {
  type: 'object',
  required: ['kind', 'target', 'op', 'value'],
  properties: {
    kind: { enum: ['jobs', 'residents', 'minutes', 'station', 'event', 'cars'], description: 'cars: household access to a car. Use target {type:"comuna",id:<home CUT>}. Use set with an absolute share (0 to 1) or multiply with a factor. Report car_mode_changes workers_base and workers_scenario by mode. Label the result exploratory and state the EOD 2012 household-access assumption and the source level returned by the engine. jobs: work places. residents: persons who live there (workers follow at the comuna rate). minutes: door-to-door travel minutes between comunas. station: close or reopen an existing Metro station, open a planned line, or open a new station at a point.' },
    target: {
      type: 'object',
      required: ['type'],
      description: 'jobs and residents: {type:"point",longitude,latitude} for a place (new or existing), {type:"comuna",id} for a whole comuna, {type:"block",id} for a census block id, {type:"place",id} for a named place. minutes: {type:"comuna",id} for all trips to and from a comuna, or {type:"pair",origin,destination} for one direction between two comunas.',
      properties: {
        type: { enum: ['point', 'comuna', 'block', 'place', 'pair', 'station', 'line', 'corridor', 'permits'] },
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
    audience: { enum: ['colo_colo'], description: 'event only: colo_colo uses the fixed Colo-Colo fan base by home comuna, the same fans at any venue. Only for a Colo-Colo audience.' },
  },
};

const idTarget = { type: 'object', required: ['id'], properties: { id: { type: 'string' } } };
const point = { type: 'array', items: { type: 'number' }, minItems: 2, maxItems: 2 };
const path = { type: 'array', items: point, minItems: 2 };
const numbers = { type: 'array', items: { type: 'number' } };
const stopRules = { type: 'array', items: { type: 'integer', minimum: 0, maximum: 3 } };
const lineProperties = {
  mode: { enum: ['bus', 'metro', 'boat'] }, stop_ids: { type: 'array', items: { type: 'string' } },
  new_stops: { type: 'object', additionalProperties: point, description: 'Stops that are not in the catalog (for example river piers): ID → [longitude, latitude]. List them in stop_ids too.' },
  vehicle_capacity: { type: 'number', exclusiveMinimum: 0, description: 'Passengers per vehicle. Caps line_riders per half hour.' },
  segment_minutes: numbers, dwell_minutes: numbers, headway_minutes: { type: ['number', 'null'], exclusiveMinimum: 0, description: 'Use a positive headway for new service. Use null to restore a null catalog headway.' },
  pickup_type: stopRules, drop_off_type: stopRules,
};
const featureChanges = [
  {
    kind: 'building',
    description: 'Edit a footprint. Add needs a new ID, a closed WGS84 polygon and properties. Select an existing footprint by catalog ID or a point inside it. Use set for properties and move for geometry.',
    target: { oneOf: [idTarget, { type: 'object', required: ['point'], properties: { point } }] },
    geometries: ['Polygon'],
    properties: { use: { enum: ['office', 'retail', 'industrial', 'residential', 'unknown'] }, height_m: { type: 'number', exclusiveMinimum: 0 }, jobs: { type: 'number', minimum: 0 }, residents: { type: 'number', minimum: 0 } },
  },
  {
    kind: 'road',
    description: 'Edit one mapped road by ID. Add and move need geometry. Use a MultiLineString only to move an existing multipart road. Set changes class, speed or direction. Use name only on add.',
    target: idTarget, geometries: ['LineString', 'MultiLineString'],
    properties: { class: { type: 'string' }, speed_kmh: { type: 'number' }, oneway: { enum: [true, false, 'yes', 'no', '-1'] }, name: { type: 'string' } },
  },
  {
    kind: 'line',
    description: 'Edit one directed bus, Metro or boat service pattern by catalog ID. Add with a new ID, mode, ordered stop_ids, one segment_minutes value per leg, and headway_minutes. A two-way service is two adds. Stops outside the catalog go in new_stops. Each added line returns line_riders: workers near both ends, trips to work by stop and half hour, and the mode they leave. Supply segment_minutes when changing stop order. A geometry-only move changes the drawing.',
    target: idTarget, geometries: ['LineString'],
    properties: lineProperties,
  },
].map(({ kind, description, target, geometries, properties }) => ({
  type: 'object', description, required: ['kind', 'target', 'op'], additionalProperties: false,
  if: { properties: { op: { const: 'set' } } },
  then: { required: ['properties'], properties: { properties: { minProperties: 1 } } },
  properties: {
    kind: { enum: [kind] }, target, op: { enum: ['add', 'set', 'move', 'remove'] },
    properties: { type: 'object', properties, additionalProperties: false },
    geometry: { oneOf: geometries.map(type => ({
      type: 'object', required: ['type', 'coordinates'],
      properties: { type: { enum: [type] }, coordinates: type === 'LineString' ? path : { type: 'array', items: path } },
    })) },
  },
}));

const tool = {
  name: 'simulate_city',
  description: `santiaguito.cl simulator. Run one usual work day of Santiago (2.7 million synthetic workers, Census 2024) after a list of changes, and update the open city view. The workers choose a job place and a mode again. Each call starts from the baseline and applies the complete list together; an empty list restores the baseline. Translate the user's question into changes: edit a physical building, road or directed transit service with kind building, road or line. Use a catalog ID for an existing feature. Ask for the ID or missing details if the target is ambiguous. For a change only to jobs or residents at a point, use jobs or residents; the jobs of a named existing place, or an event there, use kind "jobs" or "event" with {type:"place",id}, ids with their jobs status: ${placeRows.map((r) => r[12] === 'unresolved' ? `${r[0]} (unresolved; the nearest modeled jobs point is ${r[11]} m from the place point, in census block ${r[10]})` : `${r[0]} (${r[12]})`).join(', ')}; never send a point for a listed place; an event at any listed place uses its point, whatever its jobs status; for a jobs change, an unresolved place returns an error: offer the user the census block of its nearest modeled jobs point (listed above), ask whether to use that block for this edit, and after a yes use {type:"block",id} and say that the jobs are the modeled jobs of that block; and a multi_block place changes only its main census block: say that, and do not say how large the effect on the whole place would be; a named place takes no residents; to remove, reduce or scale an existing place that is not listed, use op "set" 0 or "multiply" at its point: the engine takes the nearest existing point within 500 m, which can be a neighbour block; applied[].understood gives the jobs_before that it took; a policy for a whole comuna uses the comuna id; a faster or slower trip is minutes; before closing or reopening an existing Metro station call set_scene for its base_peak or the requested time, then keep that scene for the comparison. Closing or reopening that station is kind "station" with target {type:"station",id:<station name, for example "TOBALABA">}, op "set" and value 0 (closed) or 1 (open). An event (a concert, a match) is kind "event" with a point target, op "set", value = attendance and assumed_end_half_hour (0 to 47, 40 = 20:00); the result has events[] with the attendees by mode, by return comuna and the Metro users. The event estimate is demand for the whole return period, separate from the commuter station totals: the station deltas and the trip deltas do not include it. The end half hour does not select the service of that hour: the estimate is the same for an end at 20:00 and at 23:00. For a Colo-Colo match add audience "colo_colo": the event then also has fan_base, the same Colo-Colo fans by home comuna at any venue, with by_mode (public, car, other) for the whole audience and by_home_top5, the 5 largest of 52 home comunas; use fan_base for a venue comparison and say it is a fixed Colo-Colo fan profile, not the workers of the venue comuna. For any other audience (a concert, another club) do not use colo_colo: say that the only fixed audience profile is Colo-Colo fans and use it only if the user agrees to it as a proxy. Homes from building permits already authorized (MINVU, 2023 to 2025) are kind "residents" with target {type:"permits",id:"2023-2025" or one year such as "2024"}, op "multiply", value = the share of those authorized dwellings that the user assumes are finished and additional to the Census 2024 baseline (0 to 1); applied[].understood gives dwellings_authorized, residents_added, comunas and preliminary_years. Call these residents hypothetical additional residents; a permit is not a finished home. Opening a planned Metro line is kind "station" with target {type:"line",id:"L7"|"L8"|"L9"} or {type:"corridor",id:"L6-east"|"L6-west"} (type line "L6" opens both L6 extensions), op "set", value 1; a proposed new station is kind "station" with target {type:"point",id:<short id>,name,longitude,latitude}, op "set", value 1. A new station has base_daily 0 and its scenario_daily is modeled weekday Metro entries; the result lists them in new_stations. The engine models only which station people enter: it has no capacity, travel-time saving, transfer or mode change for a new station, so do not state any of these. Call a result for a new station or line an exploratory model scenario: future demand has no validation. The one historical check is the Line 2 extension to four stations, replayed with the remaining 2026 network and commuter inputs: the model gave 59,798 entries against 50,625 observed in April 2024 (+18.1% in total); the sum of absolute station errors was 20,345 entries, 40.2% of the observed total; it is not a confidence range. For a question the kinds cannot express, the engine returns it in not_modeled, and you must tell the user that the engine cannot simulate that part yet. All results are model values, not measurements; say that, and read the assumptions. For city edits label the answer exploratory. Report each edit in order with its assumptions. Missing building occupancy is unknown. Road minutes are free-flow car times. Line minutes are public transit times. A null route time means unreachable. State only what a field of the result says. Do not infer distances, durations, causes, sectors, Metro lines, or what a real-world effect would be; report each station's own change: the result has no link between a loss at one station and a gain at another, so never say that riders moved from one to the other; a station's place is its comuna field. summary.workers_moved counts workers who change job place (lugar de trabajo), which can be in the same comuna. Give the unit of each number (daily Metro boardings on a work day, workers, jobs). Answer in 6 lines or fewer unless the user asks for more. Comuna ids: ${comunas}.`,
  inputSchema: {
    type: 'object',
    required: ['changes'],
    properties: {
      stations: { type: 'array', items: { type: 'string' }, description: 'Optional. Names or ids of the Metro stations the user asked about, for example ["UNIVERSIDAD DE CHILE"] or ["L7-05"]. Their rows always come back in asked_stations; a name shared by two stations returns both rows.' },
      changes: {
        type: 'array',
        items: { oneOf: [scalarChange, ...featureChanges] },
      },
    },
  },
};

const sceneTool = {
  name: 'set_scene',
  description: 'Focus the city view on a place and a half hour. For a station change, first call set_scene for that station with half_hour "base_peak". Wait for the scene receipt, then call simulate_city. Keep the same scene for the comparison. The engine retains it when applying the change. For a time the user specifies, use its bin (0 = midnight, 1 = 00:30, 47 = 23:30).',
  inputSchema: {
    type: 'object', required: ['place', 'half_hour'],
    properties: {
      place: { oneOf: [
        { type: 'object', required: ['type', 'id'], properties: {
          type: { enum: ['station', 'place', 'comuna', 'line'] },
          id: { type: 'string', description: 'Station name or ID, named place name or ID, comuna name or CUT, or transit route or pattern ID.' },
        } },
        { type: 'object', required: ['type', 'longitude', 'latitude'], properties: {
          type: { enum: ['point'] }, longitude: { type: 'number' }, latitude: { type: 'number' },
        } },
      ] },
      half_hour: { oneOf: [{ type: 'integer', minimum: 0, maximum: 47 }, { enum: ['base_peak'] }],
        description: 'base_peak selects the earliest maximum of the station’s 48 modeled baseline bins.' },
    },
  },
};

// Cached replays for the demo: create before the pitch, list and run during it. The engine computes nothing on run.
const replays = new URL('../../data/replays/', import.meta.url);
// An id is a file name inside data/replays, never a path.
const replayFile = (id) => {
  if (!/^[a-z0-9-]+$/.test(id)) throw new Error('A replay id uses only a-z, 0-9 and -.');
  return new URL(`${id}.json`, replays);
};
const readReplay = (id) => JSON.parse(readFileSync(replayFile(id), 'utf8'));
const listReplaysTool = {
  name: 'list_replays',
  description: 'List the cached demo replays: id, title, the presenter question, the changes, the scene place and start half hour, the duration and what to analyze. Use it to match the presenter question to a replay.',
  inputSchema: { type: 'object', properties: {} },
};
const runReplayTool = {
  name: 'run_replay',
  description: 'Play one cached replay in the open city view. Before you call it, show the presenter the replay assumptions from list_replays: the entities that change, the start time, the duration and what to analyze. The result is the saved engine result; report it with the same rules as simulate_city.',
  inputSchema: { type: 'object', required: ['id'], properties: { id: { type: 'string' } } },
};
const createReplayTool = {
  name: 'create_replay',
  description: 'Before the demo only. Run the general engine once with a scene and a list of changes (the same changes as simulate_city) and save the full result as a cached replay in data/replays/<id>.json. An existing id is replaced.',
  inputSchema: {
    type: 'object', required: ['id', 'title', 'question', 'scene', 'changes', 'duration_half_hours', 'analyze'],
    properties: {
      id: { type: 'string', pattern: '^[a-z0-9-]+$' },
      title: { type: 'string' },
      question: { type: 'string', description: 'The question the presenter types.' },
      scene: sceneTool.inputSchema,
      changes: tool.inputSchema.properties.changes,
      stations: tool.inputSchema.properties.stations,
      duration_half_hours: { type: 'integer', minimum: 1, maximum: 48, description: 'How long the replay runs from the scene half hour.' },
      analyze: { type: 'string', description: 'What the replay analyzes, for example congestion at the stadium stations.' },
      cues: {
        type: 'array', description: 'Camera moves during the replay, after the first scene. Example: whole city, then zoom to a station.',
        items: { type: 'object', required: ['after_seconds', 'longitude', 'latitude', 'distance'], properties: {
          after_seconds: { type: 'number', minimum: 0, description: 'Seconds after the replay starts.' },
          longitude: { type: 'number' }, latitude: { type: 'number' },
          distance: { type: 'number', exclusiveMinimum: 0, description: 'Camera distance in meters: 360 street, 4000 comuna, 18000 whole city.' },
        } },
      },
    },
  },
};

function catalogEntry(rows, query) {
  const matches = rows.filter(row => plain(row.name) === plain(query));
  const exact = rows.find(row => plain(row.id) === plain(query));
  if (exact) return exact;
  if (matches.length === 1) return matches[0];
  if (matches.length) throw new Error(`Select a catalog ID for ${query}: ${matches.map(row => row.id).join(', ')}.`);
  throw new Error(`Unknown catalog entry: ${query}.`);
}

function lineCenter(id) {
  const planned = JSON.parse(data('metro_planned.geojson')).features.filter(f =>
    f.geometry.type === 'Point' && f.properties.corridor === id);
  let points = planned.map(f => f.geometry.coordinates);
  if (!points.length) {
    const { lines, stops } = JSON.parse(data('editable_transit.json'));
    const patterns = lines[id] ? [lines[id]] : Object.values(lines).filter(line => line.route_id === id);
    const stopIds = new Set(patterns.flatMap(line => line.stop_ids));
    points = [...stopIds].map(stop => stops[stop].coordinates);
  }
  if (!points.length) throw new Error(`Unknown transit route or pattern ID: ${id}.`);
  const middle = axis => {
    const values = points.map(p => p[axis]);
    return (Math.min(...values) + Math.max(...values)) / 2;
  };
  return { id, longitude: middle(0), latitude: middle(1) };
}

async function engineJSON(path, body) {
  const options = body === undefined ? {} : {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body),
  };
  const response = await fetch(new URL(path, api), options);
  if (!response.ok) throw new Error(await response.text());
  return response.json();
}

async function setScene({ place, half_hour }) {
  const { type, id } = place;
  if (half_hour === 'base_peak' && type !== 'station') throw new Error('Select a station for base_peak.');
  let entry;
  let latest;
  switch (type) {
    case 'station':
      latest = await engineJSON('/api/scenario-latest');
      entry = catalogEntry(latest?.stations ?? stationCatalog, id);
      break;
    case 'place':
      entry = catalogEntry(placeRows.map(r => ({ id: r[0], name: r[1], ...places[r[0]] })), id);
      break;
    case 'comuna': {
      const rows = data('communes.csv').trim().split('\n').slice(1).map(row => row.split(','));
      entry = catalogEntry(rows.map(([id, name, , , , x, y]) => ({ id, name, longitude: +x, latitude: +y })), id);
      break;
    }
    case 'line': entry = lineCenter(id); break;
    case 'point': entry = place; break;
    default: throw new Error('Select a station, named place, comuna, line or point.');
  }
  const center = { type, id: entry.id, longitude: entry.longitude, latitude: entry.latitude };
  if (half_hour === 'base_peak') {
    // run.sh seeds the baseline. An empty standalone engine can seed it through the scene route without clearing edits.
    if (!latest) {
      await engineJSON('/api/scene', { place: center, half_hour: 0 });
      latest = await engineJSON('/api/scenario-latest');
      entry = catalogEntry(latest.stations, entry.id);
    }
    const peak = Math.max(...entry.base_half_hour);
    if (!(peak > 0)) throw new Error('This station has no positive baseline boardings. Select an explicit half hour.');
    half_hour = entry.base_half_hour.indexOf(peak);
  }
  return engineJSON('/api/scene', { place: center, half_hour });
}

// The viewer keeps full geometry and routing data. Keep the chat receipt small.
function featureSummary(feature) {
  if (!feature) return null;
  const { properties, geometry } = feature;
  const fields = Object.fromEntries(Object.entries(properties).filter(([, value]) =>
    value === null || typeof value !== 'object'));
  return { ...fields, geometry_type: geometry.type, stop_count: properties.stop_ids?.length };
}

function editSummary({ before, after, effects, ...edit }) {
  const pairs = effects.pairs;
  if (pairs) {
    effects = {
      changed_pair_count: pairs.length,
      newly_reachable: pairs.filter(p => p.before_minutes === null).length,
      newly_unreachable: pairs.filter(p => p.after_minutes === null).length,
      sample_pairs: pairs.slice(0, 5),
    };
  }
  if (edit.kind === 'line' && before && after) {
    const fields = ['stop_ids', 'segment_minutes', 'dwell_minutes', 'pickup_type', 'drop_off_type'];
    const changed = [];
    const count = Math.max(before.properties.stop_ids.length, after.properties.stop_ids.length);
    for (let index = 0; index < count; index++) {
      const changes = {};
      for (const field of fields) {
        const previous = before.properties[field][index];
        const next = after.properties[field][index];
        if (previous !== next) changes[field] = { before: previous ?? null, after: next ?? null };
      }
      if (Object.keys(changes).length) changed.push({ index, stop_before: before.properties.stop_ids[index], stop_after: after.properties.stop_ids[index], changes });
    }
    if (changed.length) effects.service_calls = { changed_count: changed.length, sample: changed.slice(0, 5) };
  }
  return { ...edit, before: featureSummary(before), after: featureSummary(after), effects };
}

async function dispatch({ method, params }) {
  if (method === 'initialize') return {
    protocolVersion: '2024-11-05',
    capabilities: { tools: {} },
    serverInfo: { name: 'santiago', version: '2.0.0' },
  };
  if (method === 'ping') return {};
  const tools = [tool, sceneTool, listReplaysTool, runReplayTool, createReplayTool];
  if (method === 'tools/list') return { tools };
  if (method !== 'tools/call' || !tools.some(t => t.name === params?.name)) return null;
  try {
    const args = params.arguments ?? {};
    let text;
    if (params.name === sceneTool.name) text = await setScene(args);
    else if (params.name === listReplaysTool.name) text = readdirSync(replays).filter(f => f.endsWith('.json')).map(f => readReplay(f.slice(0, -5)).meta);
    else if (params.name === runReplayTool.name) {
      const { meta, result } = readReplay(args.id);
      if (meta.cues) result.scene.cues = meta.cues;
      await engineJSON('/api/replay', result);
      text = { replay: meta, result: shortResult(structuredClone(result), meta.stations ?? [], meta.places_used ?? []) };
    } else if (params.name === createReplayTool.name) {
      const { scene, changes, stations = [], ...meta } = args;
      replayFile(meta.id);
      await setScene(scene);
      const { full, short, placesUsed } = await runScenario({ changes: structuredClone(changes), stations });
      const replay = { meta: { ...meta, scene: full.scene, changes, stations, places_used: placesUsed, created_at: new Date().toISOString() }, result: full };
      writeFileSync(replayFile(meta.id), JSON.stringify(replay));
      text = { saved: `data/replays/${meta.id}.json`, replay: replay.meta, result: short };
    } else text = (await runScenario(args)).short;
    return { content: [{ type: 'text', text: JSON.stringify(text) }] };
  } catch (error) {
    return { isError: true, content: [{ type: 'text', text: error.message }] };
  }
}

// Changes from the chat: named places become census blocks, then the engine runs once. The map gets the full result; the agent a short copy.
async function runScenario(args) {
    const placesUsed = [];
    for (const change of args.changes ?? []) {
      if (change.kind === 'line' && change.op === 'set') {
        const fields = Object.keys(change.properties ?? {});
        if ('value' in change || !fields.length || fields.some(key => !Object.hasOwn(lineProperties, key))) {
          throw new Error('Set a line property in properties, such as {"headway_minutes":5}.');
        }
      }
      if (change.target?.type !== 'place') continue;
      const place = places[change.target.id];
      // A named place is a job block; a residents change there would ask the engine for a home block it does not have.
      if (!place) throw new Error(`Unknown place "${change.target.id}". Known places: ${Object.keys(places).join(', ')}.`);
      // An event needs only the place point.
      if (change.kind === 'event') { change.target = { type: 'point', longitude: place.longitude, latitude: place.latitude }; continue; }
      if (change.kind !== 'jobs') throw new Error(`A named place takes kind "jobs" or "event", not "${change.kind}". Use a point or a comuna for residents.`);
      if (!place.block) throw new Error(`Place "${change.target.id}" is unresolved for jobs: no mapped building contains its point (data/mvp/named_places.csv). The nearest modeled jobs point is ${place.nearest_m} m from the place point, in census block ${place.nearest_block}. Ask the user whether to use that block for this edit; after a yes, use {type:"block",id:"${place.nearest_block}"}.`);
      placesUsed.push({ place: change.target.id, block: place.block, status: place.status, nearby_candidate_blocks: place.nearby_candidate_blocks });
      change.target = { type: 'block', id: place.block };
    }
    const { stations: asked = [], ...request } = args;
    const full = await engineJSON('/api/scenario', request);
    return { full, short: shortResult(structuredClone(full), asked, placesUsed), placesUsed };
}

// The agent's copy: the 5 largest station gains and losses and the 5 largest zone job changes.
function shortResult(full, asked, placesUsed) {
    const { trips_base, trips_scenario, trips_delta, stations, zones, flows, blocks_changed, ...result } = full;
    if (result.city_edits) result.city_edits = result.city_edits.map(editSummary);
    for (const note of [...result.applied, ...result.not_modeled]) {
      if (['building', 'road', 'line'].includes(note.kind)) {
        delete note.geometry;
        delete note.properties;
      }
    }
    const byDelta = stations.map(({ id, name, longitude, latitude, base_daily, scenario_daily, delta_daily }) => {
      const comuna = comunaAt([longitude, latitude]);
      return { id, name, comuna: comuna?.name, comuna_cut: comuna?.cut, base_daily, scenario_daily, delta_daily };
    })
      .sort((a, b) => b.delta_daily - a.delta_daily);
    result.events = result.events.map(({ attendees_by_census_mode, attendees_by_return_comuna, ...event }) => ({
      ...event, attendees_by_mode: named(attendees_by_census_mode, censusMode), attendees_by_return_comuna: named(attendees_by_return_comuna, comunaName),
      ...(event.fan_base && { fan_base: (({ by_home, ...f }) => ({ ...f, by_home_top5: by_home.sort((x, y) => y.attendees - x.attendees).slice(0, 5) }))(event.fan_base) }) }));
    delete result.summary.top_station_gains;
    delete result.summary.top_station_losses;
    result.asked_stations = asked.flatMap((name) => {
      const byId = byDelta.filter((s) => s.id === name);
      const matches = byId.length ? byId : byDelta.filter((s) => plain(s.name) === plain(name));
      return matches.length ? matches.map((s) => ({ asked: name, ...s, new_station: !currentStations.has(s.id) })) : [{ asked: name, not_found: true }];
    });
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
    return result;
}

for await (const line of createInterface({ input: process.stdin })) {
  const request = JSON.parse(line);
  if (!Object.hasOwn(request, 'id')) continue;
  const result = await dispatch(request);
  const reply = result === null ? { error: { code: -32601, message: 'Unknown method or tool' } } : { result };
  process.stdout.write(JSON.stringify({ jsonrpc: '2.0', id: request.id, ...reply }) + '\n');
}
