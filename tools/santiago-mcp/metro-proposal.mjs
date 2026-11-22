import { randomUUID } from 'node:crypto';

const object = (properties, required = []) => ({ type: 'object', additionalProperties: false, properties, required });
const coordinates = { type: 'array', items: { type: 'number' }, minItems: 2, maxItems: 2 };
const point = {
  ...object({ name: { type: 'string' }, coordinates }),
  anyOf: [{ required: ['name'] }, { required: ['coordinates'] }],
};
const stationCount = { type: 'integer', minimum: 0, maximum: 100 };
const headway = { type: 'number', exclusiveMinimum: 0 };
const draftID = { type: 'string' };
const defaultHeadway = 5;
const defaultSpacing = 1120;
const assumedSpeed = 40;
const newDwell = 0.5;

export const proposeMetroTool = {
  name: 'propose_metro',
  description: 'Create or revise a Metro draft without running the simulation or moving the viewport. Use explicit [longitude, latitude] points or exact catalog names. For extend, give an existing line and its terminal in from. To revise, use the latest draft_id, optional intermediate_count or headway_minutes, and moves with stable station IDs. Summarize the ordered stations, schematic alignment and service assumptions. Ask for a new explicit confirmation before applying the latest draft.',
  inputSchema: {
    oneOf: [
      object({
        operation: { enum: ['create', 'extend'] }, title: { type: 'string' },
        line: { type: 'string' }, from: point, to: point,
        intermediate_count: stationCount, headway_minutes: headway,
      }, ['operation', 'from', 'to']),
      object({
        draft_id: draftID, intermediate_count: stationCount, headway_minutes: headway,
        moves: { type: 'array', items: object({ station_id: { type: 'string' }, point }, ['station_id', 'point']) },
      }, ['draft_id']),
    ],
  },
};

export const applyMetroTool = {
  name: 'apply_metro',
  description: 'Apply the exact latest Metro draft after explicit confirmation in a new user message. Send only its draft_id. The tool applies the whole proposal in one scenario request. It reports modeled work trips and unavailable trip purposes.',
  inputSchema: object({ draft_id: draftID }, ['draft_id']),
};

const normalized = value => value?.normalize('NFD').replace(/[\u0300-\u036f]/g, '').trim().toUpperCase();
const lineID = value => normalized(value)?.replace(/^LINEA\s*/, 'L').replace(/\s+/g, '');
const radians = n => n * Math.PI / 180;

function meters([lon, lat], [nextLon, nextLat]) {
  const a = Math.sin(radians(nextLat - lat) / 2) ** 2
    + Math.cos(radians(lat)) * Math.cos(radians(nextLat)) * Math.sin(radians(nextLon - lon) / 2) ** 2;
  return 12742000 * Math.asin(Math.sqrt(a));
}

function interpolate(a, b, fraction) {
  return a.map((value, axis) => value + (b[axis] - value) * fraction);
}

function pickPoint(input, catalog, knownPlaces, inside, label) {
  if (Array.isArray(input?.coordinates)) {
    const coords = input.coordinates;
    if (coords.length !== 2 || !coords.every(Number.isFinite) || !inside(coords)) {
      throw new Error('Elige un punto dentro de la ciudad modelada, en orden longitud, latitud.');
    }
    return { name: input.name?.trim() || label, coordinates: coords };
  }

  const name = normalized(input?.name);
  const metroStops = new Set(Object.values(catalog.lines).filter(line => line.mode === 'metro').flatMap(line => line.stop_ids));
  const matches = [...[...metroStops].map(id => ({ id, ...catalog.stops[id] })), ...knownPlaces]
    .filter(place => name && [place.id, place.name, ...(place.aliases ?? [])].some(token => normalized(token) === name));
  if (!matches.length) throw new Error(`No encontré «${input?.name ?? label}». Indica un lugar del catálogo o sus coordenadas.`);
  if (matches.some(place => meters(place.coordinates, matches[0].coordinates) > 75)) {
    throw new Error(`Hay varios puntos para «${input.name}». Indica cuál por coordenadas.`);
  }
  const selected = matches[0];
  return { name: selected.name, coordinates: selected.coordinates };
}

function terminalPatterns(input, line, catalog) {
  const patterns = Object.entries(catalog.lines)
    .filter(([, pattern]) => pattern.mode === 'metro' && lineID(pattern.route_id) === lineID(line));
  const key = normalized(input?.name);
  const terminals = [...new Set(patterns.flatMap(([, p]) => [p.stop_ids[0], p.stop_ids.at(-1)]))];
  const terminalsMatched = terminals.filter(id => {
    const stop = catalog.stops[id];
    if (input?.coordinates) return meters(stop.coordinates, input.coordinates) <= 75;
    return key === normalized(id) || key === normalized(stop.name);
  });
  if (terminalsMatched.length !== 1) {
    throw new Error(`Indica un terminal de ${line ?? 'la línea'} sin ambigüedad. Opciones: ${terminals.map(id => catalog.stops[id].name).join(', ')}.`);
  }
  const id = terminalsMatched[0];
  const matching = patterns.flatMap(([patternId, pattern]) => {
    if (pattern.stop_ids.at(-1) === id) return [{ patternId, pattern, prepend: false }];
    if (pattern.stop_ids[0] === id) return [{ patternId, pattern, prepend: true }];
    return [];
  });
  if (!matching.some(entry => entry.prepend) || !matching.some(entry => !entry.prepend)) {
    throw new Error('El terminal indicado debe tener servicios de ida y vuelta en el catálogo.');
  }
  return { start: { id, ...catalog.stops[id], existing: true }, matching, route: matching[0].pattern.route_id };
}

function station(draft, position, name) {
  const ordinal = draft.nextStation++;
  return { id: `${draft.projectId}-s${ordinal}`, name, coordinates: position, existing: false };
}

function checkedCount(count) {
  if (!Number.isInteger(count) || count < 0 || count > 100) {
    throw new Error('Indica una cantidad entera de estaciones intermedias entre 0 y 100.');
  }
  return count;
}

function redistribute(draft, count, labelAt) {
  const stops = draft.stations;
  while (stops.length < count + 2) {
    let widest = 0;
    for (let i = 1; i < stops.length - 1; i++) {
      if (meters(stops[i].coordinates, stops[i + 1].coordinates) >
          meters(stops[widest].coordinates, stops[widest + 1].coordinates)) widest = i;
    }
    const midpoint = interpolate(stops[widest].coordinates, stops[widest + 1].coordinates, 0.5);
    stops.splice(widest + 1, 0, station(draft, midpoint, labelAt(midpoint)));
  }
  while (stops.length > count + 2) {
    const index = stops.findLastIndex((stop, i) => i > 0 && i < stops.length - 1 && !stop.moved);
    if (index === -1) throw new Error('La nueva cantidad quitaría una estación que ya moviste. Conserva al menos esas estaciones.');
    stops.splice(index, 1);
  }
}

function validateDraft(draft) {
  if (!Number.isFinite(draft.headway) || draft.headway <= 0) throw new Error('Indica una frecuencia positiva en minutos.');
  const stops = draft.stations;
  if (stops.some((stop, i) => stops.slice(0, i).some(other => meters(stop.coordinates, other.coordinates) < 1))) {
    throw new Error('Las estaciones deben estar en puntos distintos.');
  }
}

function segmentMinutes(stops) {
  return stops.slice(1).map((stop, i) => Math.max(0.1, Number((meters(stops[i].coordinates, stop.coordinates) * 0.06 / assumedSpeed).toFixed(2))));
}

function servedGeometry(pattern, catalog) {
  const stops = pattern.stop_ids.map(id => catalog.stops[id].coordinates);
  const shape = catalog.shapes[pattern.shape_id];
  if (!shape) return stops;
  let cursor = 0;
  const calls = stops.map(point => {
    let best = cursor;
    for (let i = cursor + 1; i < shape.length; i++) {
      if (meters(point, shape[i]) < meters(point, shape[best])) best = i;
    }
    cursor = best;
    return best;
  });
  return calls.at(-1) > calls[0] ? shape.slice(calls[0], calls.at(-1) + 1) : stops;
}

function lineChanges(draft, catalog) {
  const { stations, projectId, headway: interval } = draft;
  const newStops = Object.fromEntries(stations.filter(stop => !stop.existing).map(stop => [stop.id, stop.coordinates]));
  const legs = segmentMinutes(stations);
  const ids = stations.map(stop => stop.id);
  const geometry = coordinates => ({ type: 'LineString', coordinates });
  const properties = { metro_project_id: projectId, new_stops: newStops };
  if (draft.operation === 'create') {
    return [false, true].map(reverse => {
      const ordered = reverse ? stations.toReversed() : stations;
      return {
        kind: 'line', op: 'add', target: { id: `${projectId}-${reverse ? 'in' : 'out'}` },
        geometry: geometry(ordered.map(stop => stop.coordinates)),
        properties: {
          ...properties, mode: 'metro', stop_ids: ordered.map(stop => stop.id),
          segment_minutes: reverse ? legs.toReversed() : legs,
          dwell_minutes: ordered.map(() => newDwell),
          headway_minutes: interval, demand_headway_minutes: interval,
        },
      };
    });
  }

  const sorted = draft.matching.toSorted((a, b) => b.pattern.stop_ids.length - a.pattern.stop_ids.length);
  const outbound = sorted.find(entry => !entry.prepend);
  const inbound = sorted.find(entry => entry.prepend && entry.pattern.direction !== outbound.pattern.direction);
  if (!inbound) throw new Error('No encontré dos patrones en sentidos opuestos para estimar la extensión.');
  const representatives = new Set([outbound.patternId, inbound.patternId]);
  return draft.matching.map(({ patternId, pattern, prepend }) => {
    const tail = stations.slice(1);
    const extra = prepend ? tail.toReversed() : tail;
    const path = servedGeometry(pattern, catalog);
    const terminal = stations[0].coordinates;
    // Keep the catalog geometry and join the added tail at the existing terminal.
    const seam = prepend ? path[0] : path.at(-1);
    const join = meters(seam, terminal) < 0.01 ? [] : [terminal];
    const added = extra.map(stop => stop.coordinates);
    const extend = (before, after) => prepend ? [...after, ...before] : [...before, ...after];
    const representative = representatives.has(patternId);
    return {
      kind: 'line', op: 'set', target: { id: patternId },
      geometry: geometry(extend(path, prepend ? [...added, ...join] : [...join, ...added])),
      properties: {
        ...properties,
        ...(representative && { demand_headway_minutes: interval }),
        stop_ids: extend(pattern.stop_ids, prepend ? ids.slice(1).reverse() : ids.slice(1)),
        segment_minutes: extend(pattern.segment_minutes, prepend ? legs.toReversed() : legs),
        dwell_minutes: extend(pattern.dwell_minutes, extra.map(() => newDwell)),
        pickup_type: extend(pattern.pickup_type, extra.map(() => 0)),
        drop_off_type: extend(pattern.drop_off_type, extra.map(() => 0)),
      },
    };
  });
}

const missingActivities = ['school', 'shopping', 'health', 'events'].map(purpose => ({
  purpose, status: 'unavailable', reason: 'Sin estimación de uso de la línea para este propósito.',
}));

export function metroDrafts({ getCatalog, knownPlaces, areaAt }) {
  let latest;
  let applying = false;
  const stationPoint = (input, label) => pickPoint(input, getCatalog(), knownPlaces, areaAt, label);
  const labelAt = (point, i) => {
    const district = areaAt(point);
    return ['Estación intermedia' + (i === undefined ? '' : ` ${i}`), district].filter(Boolean).join(' · ');
  };

  function findDraft(id) {
    if (!latest || id !== latest.id) throw new Error('Esa propuesta no es la vigente. Revisa y confirma la versión más reciente.');
    if (applying) throw new Error('La propuesta vigente ya se está aplicando.');
    return latest;
  }

  function propose(args) {
    if (applying) throw new Error('Espera el resultado de la propuesta que se está aplicando.');
    let draft;
    if (args.draft_id) {
      const previous = findDraft(args.draft_id);
      draft = { ...previous, stations: structuredClone(previous.stations) };
      for (const move of args.moves ?? []) {
        const stop = draft.stations.find(stop => stop.id === move.station_id);
        if (!stop || stop.existing) throw new Error('Elige una estación propuesta que aparezca en la última versión.');
        Object.assign(stop, stationPoint(move.point, stop.name), { moved: true });
      }
      if (args.intermediate_count !== undefined) redistribute(draft, checkedCount(args.intermediate_count), labelAt);
      draft.headway = args.headway_minutes ?? draft.headway;
    } else {
      if (!['create', 'extend'].includes(args.operation)) throw new Error('Indica si quieres crear o extender una línea.');
      const extension = args.operation === 'extend' ? terminalPatterns(args.from, args.line, getCatalog()) : null;
      const origin = extension?.start ?? stationPoint(args.from, 'Extremo inicial');
      const end = stationPoint(args.to, 'Extremo final');
      const count = checkedCount(args.intermediate_count ?? Math.max(0, Math.round(meters(origin.coordinates, end.coordinates) / defaultSpacing) - 1));
      draft = {
        operation: args.operation, title: args.title?.trim() || (extension ? `Extensión de ${extension.route}` : 'Nueva línea de Metro'),
        projectId: `metro-${randomUUID()}`, nextStation: 1, headway: args.headway_minutes ?? defaultHeadway,
        matching: extension?.matching, stations: [],
      };
      const start = extension ? origin : station(draft, origin.coordinates, origin.name);
      const finish = station(draft, end.coordinates, end.name);
      draft.stations = [start];
      for (let i = 1; i <= count; i++) {
        const position = interpolate(origin.coordinates, end.coordinates, i / (count + 1));
        draft.stations.push(station(draft, position, labelAt(position, i)));
      }
      draft.stations.push(finish);
    }
    validateDraft(draft);
    draft.id = randomUUID();
    latest = draft;
    return {
      draft_id: draft.id, operation: draft.operation, title: draft.title,
      stations: draft.stations.map(({ id, name, coordinates, existing }) => ({ id, name, coordinates, existing })),
      intermediate_count: draft.stations.length - 2,
      patterns: draft.matching?.length ?? 2,
      assumptions: {
        alignment: 'Trazado esquemático entre estaciones. Los accesos propuestos son ilustrativos.',
        spacing: 'Sugerencia inicial basada en la mediana de los pares contiguos del catálogo Metro: 1.120 m en línea recta.',
        speed_kmh: assumedSpeed, new_stop_dwell_minutes: newDwell,
        assumed_direction_headway_minutes: draft.headway,
        preserves_catalog_service: draft.operation === 'extend',
        demand_scope: 'Viajes de ida al trabajo en un día hábil. Procedencia por modo: transporte público, auto y otros.',
        unavailable_activities: missingActivities,
      },
      status: 'pending_confirmation',
    };
  }

  async function apply(args, run) {
    const draft = findDraft(args.draft_id);
    applying = true;
    try {
      const changes = lineChanges(draft, getCatalog());
      const result = await run(changes);
      latest = undefined;
      return {
        ...result, projectId: draft.projectId, projectTitle: draft.title,
        stations: draft.stations.map(({ id, name, coordinates, existing }) => ({ id, name, coordinates, existing })),
        patterns: changes.map(change => change.target.id),
        demand: result.full.metro_project_demand?.find(item => item.project_id === draft.projectId) ?? null,
        unavailable_activities: missingActivities,
      };
    } finally {
      applying = false;
    }
  }

  return { propose, apply };
}
