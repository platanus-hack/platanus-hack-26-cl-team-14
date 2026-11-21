// One marker represents one synthetic-worker group, not one vehicle.
// Paths come from the route owner; comuna car paths represent flows, not observed routes.
const COLORS = {
  car: [255, 205, 112], public: [102, 213, 231],
  walk: [245, 245, 235], bicycle: [137, 230, 150], other: [170, 170, 170]
};
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
  return [path[i - 1][0] + (path[i][0] - path[i - 1][0]) * part,
    path[i - 1][1] + (path[i][1] - path[i - 1][1]) * part];
}

/**
 * result.sample_trips: [{origin_cut, destination_cut, mode, half_hour, count, path?}].
 * data.carRoutes: {"origin_cut-destination_cut": [[longitude, latitude], ...]}.
 * data.roads/cycleways are [{id, kind, path}]
 * or GeoJSON LineString FeatureCollections. Transit paths stay with their owner.
 * timeSeconds is city time, seconds after midnight. The caller owns playback.
 * shortcut: each routed group crosses its path within its departure half-hour;
 * this is a visual replay window, not a modeled travel time or service schedule.
 */
export function moveLayers(result, data, timeSeconds) {
  const moving = (result?.sample_trips ?? [])
    .filter(trip => timeSeconds >= trip.half_hour * 1800 && timeSeconds < (trip.half_hour + 1) * 1800)
    .map(trip => ({...trip, path: trip.path ?? (trip.mode === 'car'
      ? data.carRoutes?.[`${trip.origin_cut}-${trip.destination_cut}`] : undefined)}))
    .filter(trip => trip.path);
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
    new deck.ScatterplotLayer({
      id: 'movement-workers', data: moving, pickable: true,
      getPosition: trip => along(trip.path, (timeSeconds - trip.half_hour * 1800) / 1800),
      getFillColor: trip => COLORS[trip.mode],
      getRadius: trip => Math.sqrt(trip.count), radiusUnits: 'pixels',
      radiusMinPixels: 3, radiusMaxPixels: 9,
      stroked: true, getLineColor: [20, 30, 35], lineWidthUnits: 'pixels', getLineWidth: 1,
      updateTriggers: {getPosition: timeSeconds}
    })
  ];
}
