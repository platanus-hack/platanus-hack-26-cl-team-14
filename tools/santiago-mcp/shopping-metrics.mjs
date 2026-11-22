export function shoppingMetrics(raw) {
  const destinations = raw.shopping?.destinations_changed;
  if (!Array.isArray(destinations)) {
    throw new Error('This result does not contain shopping trips by changed destination comuna.');
  }

  return {
    rows: destinations.map(({ id, trips_base, trips_scenario }) => ({
      destination_cut: id,
      base: trips_base,
      scenario: trips_scenario,
    })),
    dimensions: ['destination_cut'],
    unit: 'modeled weekday outbound shopping trips',
    rowUnit: 'destination comunas',
    population: 'Shopping trips of residents, estimated from EOD 2012 rates and Census 2024 population.',
    scope: 'The engine includes destination comunas with an absolute change of at least one modeled trip before rounding. The reported counts are rounded to whole trips. Unlisted comunas are outside this result.',
  };
}
