// One marker represents one synthetic-worker group, not one vehicle.
// Paths come from the route owner; comuna car paths represent flows, not observed routes.
const COLORS = {
  car: [255, 205, 112], public: [102, 213, 231], metro: [240, 245, 246],
  walk: [245, 245, 235], bicycle: [137, 230, 150], other: [170, 170, 170]
};
// Small top-down silhouettes, pointing right. Public transport art is representative.
const DRAWINGS = {
  car: [64, 32, '<rect x="4" y="3" width="12" height="26" rx="4" fill="#202d34"/><rect x="45" y="3" width="12" height="26" rx="4" fill="#202d34"/><rect x="2" y="7" width="60" height="18" rx="7"/><path d="M18 9h22l5 14H17z" fill="#405764"/><path d="M47 10h6v12h-6" fill="#eff9fa"/>'],
  public: [72, 28, '<rect x="3" y="3" width="66" height="22" rx="5"/><path d="M12 7h9v14h-9zm13 0h9v14h-9zm13 0h9v14h-9zm17 0h7v14h-7z" fill="#405764"/><path d="M64 8h3v12h-3" fill="#eff9fa"/>'],
  metro: [96, 28, '<rect x="0" y="7" width="96" height="14" fill="#405764"/><rect x="1" y="3" width="28" height="22" rx="4"/><rect x="34" y="3" width="28" height="22" rx="4"/><rect x="67" y="3" width="28" height="22" rx="6"/><path d="M7 8h16v12H7zm33 0h16v12H40zm33 0h14v12H73z" fill="#405764"/><path d="M91 9h3v10h-3" fill="#eff9fa"/>'],
  bicycle: [48, 32, '<g fill="none" stroke="#202d34" stroke-width="4"><circle cx="10" cy="21" r="8"/><circle cx="38" cy="21" r="8"/></g><path d="M10 21l12-13 7 13H10l9-13h14l5 13M20 5h8m4-2h6" fill="none" stroke="currentColor" stroke-width="4" stroke-linejoin="round"/>'],
  walk: [28, 32, '<ellipse cx="13" cy="18" rx="8" ry="11" fill="#202d34"/><path d="M6 23l8-6 8 8M8 8l8 4 8-6" fill="none" stroke="currentColor" stroke-width="4" stroke-linecap="round"/><circle cx="16" cy="12" r="7"/><circle cx="18" cy="11" r="3" fill="#ffe3c4"/>']
};
const ICONS = Object.fromEntries(Object.entries(DRAWINGS).map(([mode, [width, height, drawing]]) => {
  const color = `rgb(${COLORS[mode] ?? COLORS.public})`;
  return [mode, {width, height, url: 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" fill="${color}" color="${color}">${drawing}</svg>`)}];
}));
ICONS.other = ICONS.walk;
ICONS.bus = ICONS.public;
const geometry = new WeakMap();

function along(path, fraction) {
  let lengths = geometry.get(path);
  if (!lengths) {
    lengths = [0];
    for (let i = 1; i < path.length; i++) {
      const a = path[i - 1], b = path[i];
      lengths.push(lengths[i - 1] + Math.hypot((b[0] - a[0]) * Math.cos(a[1] * Math.PI / 180), b[1] - a[1]));
    }
    geometry.set(path, lengths);
  }
  const distance = fraction * lengths[lengths.length - 1];
  let i = 1;
  while (i < lengths.length - 1 && lengths[i] <= distance) i++;
  const part = (distance - lengths[i - 1]) / (lengths[i] - lengths[i - 1]);
  const a = path[i - 1], b = path[i];
  return {position: [a[0] + (b[0] - a[0]) * part, a[1] + (b[1] - a[1]) * part],
    angle: Math.atan2(b[1] - a[1], (b[0] - a[0]) * Math.cos(a[1] * Math.PI / 180)) * 180 / Math.PI};
}

/**
 * result.trips_scenario: [{origin_cut, destination_cut, mode, half_hour, count, path?, glyph?}].
 * An optional glyph selects route art (for example metro), never a measured mode split.
 * data.carRoutes: {"origin_cut-destination_cut": [[longitude, latitude], ...]}.
 * data.roads/cycleways are [{id, kind, path}]
 * or GeoJSON LineString FeatureCollections. Transit paths stay with their owner.
 * timeSeconds is city time, seconds after midnight. The caller owns playback.
 * shortcut: each routed group crosses its path within its departure half-hour;
 * this is a visual replay window, not a modeled travel time or service schedule.
 */
export function moveLayers(result, data, timeSeconds) {
  const moving = (result?.trips_scenario ?? result?.sample_trips ?? [])
    .filter(trip => timeSeconds >= trip.half_hour * 1800 && timeSeconds < (trip.half_hour + 1) * 1800)
    .map(trip => ({...trip, path: trip.path ?? (trip.mode === 'car'
      ? data.carRoutes?.[`${trip.origin_cut}-${trip.destination_cut}`] : undefined)}))
    .filter(trip => trip.path)
    .map(trip => ({...trip, ...along(trip.path, (timeSeconds - trip.half_hour * 1800) / 1800)}));
  return [
    new deck.PathLayer({
      id: 'movement-roads', data: data.roads?.features ?? data.roads ?? [],
      getPath: row => row.path ?? row.geometry.coordinates,
      getColor: [124, 144, 151, 75], getWidth: 1, widthUnits: 'pixels'
    }),
    new deck.PathLayer({
      id: 'movement-cycleways', data: data.cycleways?.features ?? data.cycleways ?? [],
      getPath: row => row.path ?? row.geometry.coordinates,
      getColor: COLORS.bicycle, getWidth: 2, widthUnits: 'pixels'
    }),
    new deck.IconLayer({
      id: 'movement-workers', data: moving, pickable: true,
      getPosition: trip => trip.position, getAngle: trip => trip.angle, billboard: false,
      getIcon: trip => ICONS[trip.glyph ?? trip.mode],
      getSize: trip => 10 + Math.sqrt(trip.count) / 12, sizeUnits: 'pixels',
      sizeMinPixels: 10, sizeMaxPixels: 18
    })
  ];
}
