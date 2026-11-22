const unit = 'modeled weekday outbound shopping trips';
const population = 'Shopping trips of residents, estimated from EOD 2012 rates and Census 2024 population.';

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
    unit,
    rowUnit: 'destination comunas',
    population,
    scope: 'The engine includes destination comunas with an absolute change of at least one modeled trip before rounding. The reported counts are rounded to whole trips. Unlisted comunas are outside this result.',
  };
}

export const shoppingSiteDatasets = [
  'shopping_site_total', 'shopping_site_origins', 'shopping_site_modes', 'shopping_site_departures',
];

function siteView(data, shoppers) {
  if (data === 'shopping_site_total' && Number.isFinite(shoppers?.trips)) {
    return { rows: [[undefined, shoppers.trips]], rowUnit: 'reported site totals',
      scope: 'The total is the direct reported value, rounded to whole trips.' };
  }
  if (data === 'shopping_site_origins' && Array.isArray(shoppers?.by_home_top5)) {
    const origins = shoppers.by_home_top5;
    return { rows: origins.map(({ id, trips }) => [id, trips]), dimension: 'origin_cut', rowUnit: 'reported origin comunas',
      scope: 'Up to five origin comunas are reported. Missing origins are not zero. Rounded origin counts need not sum to the site total.',
      may_be_truncated: origins.length >= 5 };
  }
  if (data === 'shopping_site_modes' && shoppers?.by_mode) {
    return { rows: Object.entries(shoppers.by_mode), dimension: 'mode', rowUnit: 'reported travel modes',
      scope: 'Mode is a separate breakdown. Missing modes are outside this result. Rounded mode counts need not sum to the site total.' };
  }
  if (data === 'shopping_site_departures' && Array.isArray(shoppers?.by_departure_half_hour)) {
    return { rows: shoppers.by_departure_half_hour.map((trips, bin) => [bin, trips]), dimension: 'half_hour', rowUnit: 'reported departure half-hours',
      scope: 'Departure half-hours are a separate breakdown. Bin 0 starts at 00:00 and bin 47 at 23:30. Rounded bin counts need not sum to the site total.' };
  }
  throw new Error('The selected site does not contain this shopping trip breakdown.');
}

export function shoppingSiteMetrics(raw, { data, shopping_site, state }) {
  if (!Array.isArray(raw.shopping?.sites)) {
    throw new Error('This result does not contain shopping sites.');
  }
  const sites = raw.shopping.sites.flatMap(({ longitude, latitude, shops, shoppers }, index) =>
    shops === 0 ? [] : [{ longitude, latitude, index, variant: shops < 0 ? 'base' : 'scenario', shoppers }]);
  const matches = shopping_site === undefined ? sites : sites.filter(site =>
    site.longitude === shopping_site.longitude && site.latitude === shopping_site.latitude &&
    (shopping_site.variant === undefined || site.variant === shopping_site.variant) &&
    (shopping_site.index === undefined || site.index === shopping_site.index));
  if (matches.length !== 1) {
    const options = sites.map(({ longitude, latitude, variant, index }) => ({ longitude, latitude, variant, index }));
    throw new Error(`Select one shopping site by its reported coordinates. Include the variant or zero-based index from this result if needed. Available sites: ${JSON.stringify(options)}.`);
  }

  const { longitude, latitude, variant, index, shoppers } = matches[0];
  if (state !== variant) {
    throw new Error(`The selected shopping site reports only the ${variant} side. Use state "${variant}".`);
  }
  const view = siteView(data, shoppers);
  return {
    rows: view.rows.map(([group, trips]) => ({
      ...(view.dimension && { [view.dimension]: group }),
      base: variant === 'base' ? trips : null,
      scenario: variant === 'scenario' ? trips : null,
    })),
    dimensions: view.dimension ? [view.dimension] : [],
    unit,
    rowUnit: view.rowUnit,
    population,
    scope: `Site at ${longitude}, ${latitude}, row ${index} in this result. The engine reports only the ${variant} side for this site. The other side is unavailable. ${view.scope}`,
    may_be_truncated: view.may_be_truncated,
  };
}
