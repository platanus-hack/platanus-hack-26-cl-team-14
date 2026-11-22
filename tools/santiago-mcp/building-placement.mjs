import { randomUUID } from 'node:crypto';
import { readFileSync, readdirSync } from 'node:fs';

const webData = new URL('../../web/data/', import.meta.url);
const features = (url) => JSON.parse(readFileSync(url, 'utf8')).features;
const words = (name) => (name ?? '').normalize('NFD').replace(/\p{M}/gu, '')
  .toLowerCase().replace(/[^a-z0-9]+/g, ' ').trim();
const polygon = (part) => part.geometry.type === 'Polygon';
const asPoint = (place) => [place.longitude, place.latitude];

const point = {
  type: 'array', items: { type: 'number' }, minItems: 2, maxItems: 2,
  description: 'WGS84 [longitude, latitude].',
};
const selector = {
  oneOf: [
    { type: 'object', required: ['name'], additionalProperties: false, properties: { name: { type: 'string' } } },
    { type: 'object', required: ['points'], additionalProperties: false, properties: {
      points: { type: 'array', minItems: 1, items: point, description: 'One point inside each mapped building. Use multiple points for a group.' },
    } },
  ],
};

export const buildingPlacementChange = {
  type: 'object',
  description: 'Place the real mapped shape of a building or group at another map location. Copy keeps the original. Relocate moves it. Replace removes the selected destination building and either copies or relocates the source, as agreed. First ask which experiment the user wants and what to measure. Clarify any uncertain source, destination, or source retention. Ask once about optional total added jobs. Describe the geometry and weekday scope, then wait for explicit confirmation. Report estimated Metro entries by station for a full weekday. The camera hour does not schedule this change. Do not estimate total visitors or hourly effects. Leave unknown building uses and stocks unknown.',
  required: ['kind', 'op', 'source'],
  additionalProperties: false,
  properties: {
    kind: { const: 'building_placement' },
    op: { enum: ['copy', 'relocate', 'replace'] },
    source: selector,
    destination: {
      oneOf: [
        { type: 'object', required: ['point'], additionalProperties: false, properties: { point } },
        { type: 'object', required: ['name'], additionalProperties: false, properties: { name: { type: 'string' } } },
        { type: 'object', required: ['current_point'], additionalProperties: false, properties: { current_point: { const: true } } },
      ],
    },
    replacing: selector,
    keep_source: { type: 'boolean', description: 'Replace only. True keeps the source, false relocates it.' },
    jobs: { type: 'integer', minimum: 0, description: 'Agreed total additional jobs at the destination. Omit for geometry only.' },
  },
  allOf: [{
    if: { properties: { op: { const: 'replace' } } },
    then: { required: ['replacing', 'keep_source'] },
    else: { required: ['destination'] },
  }],
};

function center(parts) {
  const coordinates = parts.flatMap(part => part.geometry.coordinates[0]);
  return [0, 1].map(i => {
    const values = coordinates.map(p => p[i]);
    return (Math.min(...values) + Math.max(...values)) / 2;
  });
}

let geometry;
function mappedSites() {
  return geometry ??= {
    landmarks: features(new URL('santiago_landmarks.geojson', webData)).filter(polygon),
    buildings: features(new URL('../../data/mvp/buildings.geojson', import.meta.url)).filter(polygon),
    groups: readdirSync(webData).filter(file => file.endsWith('_parts.geojson')).map(file => {
      const parts = features(new URL(file, webData));
      const name = file.replace('_parts.geojson', '');
      const labels = [name, ...parts.filter(p => ['podium', 'grounds'].includes(p.properties.kind)).map(p => p.properties.name)];
      return { name, labels, parts: parts.filter(part => polygon(part) && part.properties.height_m > 0) };
    }),
  };
}

function selectName(rows, label, names) {
  const query = words(label);
  if (!query) throw new Error('Give the name of a mapped building or a point inside it.');
  const matching = (value) => rows.filter(row => names(row).some(name => ` ${words(name)} `.includes(` ${value} `)));
  const exact = rows.filter(row => names(row).some(name => words(name) === query));
  let matches = exact.length ? exact : matching(query);
  if (!matches.length && /^(torre|edificio) /.test(query)) {
    matches = matching(query.replace(/^(torre|edificio) /, ''));
  }
  if (matches.length > 1) throw new Error(`Several mapped buildings match "${label}". Ask for the full name or a point inside it.`);
  return matches[0];
}

function resolveName(name, places) {
  const { groups, landmarks, buildings } = mappedSites();
  const place = selectName(places, name, p => [p.id, p.name]);
  const labels = p => [p.id, p.properties.name];
  const landmark = selectName(landmarks, place?.id ?? name, labels) ?? selectName(landmarks, name, labels);
  const group = selectName(groups, place?.id ?? name, g => g.labels) ?? selectName(groups, name, g => g.labels);
  const building = !landmark && !group && selectName(buildings, name, labels);
  return { place, landmark, group, building };
}

async function resolveSource(selector, places, lookup, needsNative, clearDestination = false) {
  if (selector.points) {
    const parts = await Promise.all(selector.points.map(point => lookup({ point })));
    return { parts, anchor: center(parts) };
  }
  const { place, landmark, group, building } = resolveName(selector.name, places);
  const outlines = (group && !(landmark && clearDestination)) ? group.parts : [landmark || building].filter(Boolean);
  const parts = outlines.length
    ? await Promise.all(outlines.map(part => needsNative ? lookup({ outline: part.geometry }) : part))
    : place ? [await lookup({ point: asPoint(place) })] : [];
  if (!parts.length) throw new Error(`Could not match "${selector.name}" to a mapped building. Ask for a point inside the building.`);
  return { parts, anchor: place ? asPoint(place) : center(parts) };
}

export async function expandBuildingPlacements(changes, places, engineJSON) {
  const expanded = [];
  const lookup = locator => engineJSON('/api/building-footprint', locator);
  for (const change of changes) {
    if (change.kind !== 'building_placement') {
      expanded.push(change);
      continue;
    }
    const { op, source, destination, replacing, keep_source, jobs = 0 } = change;
    const replacingSite = op === 'replace';
    const move = op === 'relocate' || (replacingSite && keep_source === false);
    const origin = await resolveSource(source, places, lookup, move);
    const removed = replacingSite ? await resolveSource(replacing, places, lookup, true, true) : undefined;
    let at = destination?.point ?? removed?.anchor;
    if (destination?.name) {
      const site = resolveName(destination.name, places);
      at = site.place ? asPoint(site.place) :
        center(site.group?.parts ?? [site.landmark || site.building].filter(Boolean));
    }
    if (destination?.current_point) {
      const snapshot = await engineJSON('/api/scenario-latest');
      if (snapshot?.scene?.place?.type !== 'point') throw new Error('Select a destination point on the map first.');
      at = asPoint(snapshot.scene.place);
    }
    if (!at?.every(Number.isFinite)) throw new Error('Give a mapped destination name or point.');
    if (!Number.isSafeInteger(jobs) || jobs < 0) throw new Error('Use one nonnegative whole number for total jobs.');
    const nativeIds = new Set(origin.parts.map(part => part.id));
    if (removed?.parts.some(part => nativeIds.has(part.id))) throw new Error('Select a different building to replace.');
    const id = move ? undefined : `copy-${randomUUID()}`;
    for (const part of removed?.parts ?? []) {
      expanded.push({ kind: 'building', op: 'remove', target: { id: part.id } });
    }
    for (const [index, part] of origin.parts.entries()) {
      const geometry = {
        type: 'Polygon',
        coordinates: part.geometry.coordinates.map(ring => ring.map(p => p.map((v, i) => v + (at[i] - origin.anchor[i])))),
      };
      const { height_m, use } = part.properties;
      const properties = { ...(height_m > 0 && { height_m }), ...(use && { use }), jobs: 0, residents: 0 };
      expanded.push({
        kind: 'building', op: move ? 'move' : 'add',
        target: { id: move ? part.id : `${id}-${index + 1}` }, geometry,
        ...(!move && { properties }),
      });
    }
    if (jobs > 0) {
      expanded.push({ kind: 'jobs', op: 'add', target: { type: 'point', longitude: at[0], latitude: at[1] }, value: jobs });
    }
  }
  return expanded;
}
