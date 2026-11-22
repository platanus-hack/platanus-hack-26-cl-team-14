import { readFileSync } from 'node:fs';

// A street section by name: «Grecia entre Macul y Tobalaba» → the mapped road IDs that the engine edits.
const words = (name) => (name ?? '').normalize('NFD').replace(/\p{M}/gu, '')
  .toLowerCase().replace(/[^a-z0-9]+/g, ' ').trim().replace(/^(avenida|av|calle) /, '');

export const roadSectionTarget = {
  type: 'object', required: ['name', 'from', 'to'], additionalProperties: false,
  description: 'A street section the user names: the street and its two cross streets, for example {"name":"Grecia","from":"Macul","to":"Tobalaba"}. The tool finds the mapped roads and applies the change to each one.',
  properties: { name: { type: 'string' }, from: { type: 'string' }, to: { type: 'string' } },
};

let graph;
export function roadSection({ name, from, to }) {
  graph ??= JSON.parse(readFileSync(new URL('../../data/mvp/road_graph.json', import.meta.url), 'utf8'));
  const named = (query) => {
    const ways = Object.entries(graph.ways).filter(([, way]) => way.name);
    const exact = ways.filter(([, way]) => words(way.name) === words(query));
    const found = exact.length ? exact : ways.filter(([, way]) => ` ${words(way.name)} `.includes(` ${words(query)} `));
    if (!found.length) throw new Error(`The road map has no street named "${query}". No change was applied. Ask the user for another name of that street.`);
    return found;
  };
  const points = (ways) => [...new Set(ways.flatMap(([, way]) => way.parts.flat()))].map(node => metres(graph.nodes[node]));
  // Local metres.
  const metres = ([lon, lat]) => [lon * 92_880, lat * 110_540];
  const street = named(name);
  const streetPoints = points(street);
  const gaps = {};
  // The point of the street nearest to the cross street. Mapped names can end before the crossing: «Macul» is José Pedro Alessandri at Grecia.
  const crossing = (query) => {
    let best = [Infinity];
    for (const q of points(named(query))) for (const p of streetPoints) {
      const gap = Math.hypot(p[0] - q[0], p[1] - q[1]);
      if (gap < best[0]) best = [gap, p];
    }
    if (best[0] > 3000) throw new Error(`"${name}" and "${query}" are ${Math.round(best[0])} m apart in the road map. No change was applied. Ask the user for another cross street.`);
    gaps[query] = Math.round(best[0]);
    return best[1];
  };
  const a = crossing(from), b = crossing(to);
  const along = [b[0] - a[0], b[1] - a[1]];
  const length = Math.hypot(...along);
  // shortcut: keeps the roads whose middle node lies between the two crossings, near the straight line; upgrade to a path search for a street with sharp bends.
  const ids = street.filter(([, way]) => {
    const part = way.parts.flat();
    const [x, y] = metres(graph.nodes[part[part.length >> 1]]);
    const t = ((x - a[0]) * along[0] + (y - a[1]) * along[1]) / length ** 2;
    return t >= 0 && t <= 1 && Math.abs((x - a[0]) * along[1] - (y - a[1]) * along[0]) / length < length / 4;
  }).map(([id]) => id);
  if (!ids.length) throw new Error(`The road map has no part of "${name}" between "${from}" and "${to}". No change was applied. Ask the user for other cross streets.`);
  return { ids, crossing_gap_m: gaps };
}
