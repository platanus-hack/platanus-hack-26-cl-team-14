import { shoppingMetrics } from './shopping-metrics.mjs';

const operations = ['sum', 'count', 'mean', 'share', 'percentile', 'histogram'];
const states = ['base', 'scenario', 'delta', 'compare'];
const dimensions = ['station_id', 'station', 'half_hour', 'origin_cut', 'destination_cut'];
const comparisons = ['eq', 'lt', 'lte', 'gt', 'gte'];
const weights = ['equal', 'value', 'census_public_workers'];
const datasets = ['metro_entries', 'commute_flows', 'route_minutes', 'shopping_trips'];

const field = (values) => ({ enum: values });
const object = (required, properties) => ({ type: 'object', additionalProperties: false, required, properties });
const comparison = { op: field(comparisons), value: { type: 'number' } };
const filterSchema = object(['field', 'op', 'value'], {
  field: field(dimensions), ...comparison,
  value: { type: ['number', 'string'] },
});
const filterArray = { type: 'array', items: filterSchema };

export const queryResultTool = {
  name: 'query_result',
  description: 'Answer a measurement question about an already published scenario or a saved replay without running it. Use raw results only. metro_entries counts modeled Metro entries by station_id, station or half_hour (0 is midnight, 47 is 23:30). Use grain day for exact daily station totals, or half_hour for 30-minute bins. commute_flows counts modeled workers by origin_cut and destination_cut for up to 200 changed comuna pairs. shopping_trips counts modeled weekday outbound shopping trips in the reported changed destination comunas. route_minutes uses one city_edit_index and measures modeled minutes by directed comuna pair. For route minutes, base and scenario mean immediately before and after the selected edit. Read state_labels. Compare uses the same valid rows for both states. Use sum for totals of entries, workers or trips. Count measures records or their declared weight. Route weights are equal by default. census_public_workers is allowed for transit routes and uses fixed Census 2024 counts, not scenario passengers. Use group_by and filters to match the user’s scope. For shares, select a weight explicitly: value counts entries or workers, equal counts table rows. Read unit, population, and coverage before answering. Results are text or tables. This tool does not open or chart a replay.',
  inputSchema: object(['source', 'data', 'state', 'aggregate'], {
    source: object(['type'], { type: field(['current', 'replay']), id: { type: 'string' } }),
    data: field(datasets),
    state: field(states),
    aggregate: field(operations),
    grain: field(['day', 'half_hour']),
    city_edit_index: { type: 'integer', minimum: 0, description: 'Zero-based raw city edit index. For route minutes only. May omit if only one edit has route pairs.' },
    filters: filterArray,
    group_by: { type: 'array', items: field(dimensions), uniqueItems: true, maxItems: 2 },
    weight: field(weights),
    percentile: { type: 'number', minimum: 0, maximum: 100 },
    breaks: { type: 'array', minItems: 1, maxItems: 12, items: { type: 'number' }, description: 'Increasing histogram cutoffs. Each cutoff starts the next bin.' },
    share_filters: filterArray,
    share_value: object(['op', 'value'], comparison),
    order: field(['value_asc', 'value_desc']),
    limit: { type: 'integer', minimum: 1, maximum: 100 },
  }),
};

const finite = Number.isFinite;
const sum = (values) => values.reduce((total, value) => total + value, 0);
const pairKey = ({ origin_cut, destination_cut }) => `${origin_cut}:${destination_cut}`;

function stationRows(raw, grain) {
  if (!Array.isArray(raw.stations)) throw new Error('This result has no Metro station entries.');
  return raw.stations.flatMap(s => {
    const row = { station_id: s.id, station: s.name };
    return grain === 'day'
      ? [{ ...row, base: s.base_daily, scenario: s.scenario_daily }]
      : Array.from({ length: 48 }, (_, half_hour) => ({
          ...row, half_hour, base: s.base_half_hour?.[half_hour], scenario: s.scenario_half_hour?.[half_hour],
        }));
  });
}

function dataset(raw, q) {
  if (q.data === 'metro_entries') {
    const day = q.grain === 'day';
    return {
      rows: stationRows(raw, q.grain), dimensions: day ? dimensions.slice(0, 2) : dimensions.slice(0, 3),
      unit: 'modeled Metro entries', rowUnit: day ? 'station-day rows' : 'station-half-hour rows',
      population: 'Modeled Metro station entries on one weekday. Entries can include repeat trips.',
      scope: day ? 'Daily station totals.' : 'Rounded half-hour entry counts. Their sum can differ from the daily totals.',
    };
  }
  if (q.data === 'commute_flows') {
    if (!Array.isArray(raw.flows)) throw new Error('This result has no reported commuter flows.');
    return {
      rows: raw.flows.map(row => ({ origin_cut: row.origin, destination_cut: row.destination, base: row.base, scenario: row.scenario })),
      dimensions: dimensions.slice(3), unit: 'modeled workers', rowUnit: 'reported changed directed comuna pairs',
      population: 'Modeled weekday workers in the reported changed home-to-work comuna pairs, across all modes.',
      scope: 'Up to 200 largest changed pairs. Changes below one worker are excluded. Missing pairs cannot be counted as zero.',
      may_be_truncated: raw.flows.length >= 200,
    };
  }
  if (q.data === 'shopping_trips') return shoppingMetrics(raw);
  const edits = (raw.city_edits ?? []).flatMap((edit, index) =>
    ['road', 'line'].includes(edit.kind) && edit.effects?.pairs?.length ? [{ edit, index }] : []);
  const selected = q.city_edit_index === undefined && edits.length === 1 ? edits[0] :
    edits.find(({ index }) => index === q.city_edit_index);
  if (!selected) {
    const options = edits.map(({ edit, index }) => ({ index, kind: edit.kind, id: edit.id }));
    throw new Error(`Select one raw route edit. Available edits: ${JSON.stringify(options)}.`);
  }
  const { edit, index } = selected;
  const transit = edit.kind === 'line';
  const rows = edit.effects.pairs.map(p => ({
    origin_cut: p.origin_cut, destination_cut: p.destination_cut, base: p.before_minutes, scenario: p.after_minutes,
  }));
  if (new Set(rows.map(pairKey)).size !== rows.length) throw new Error('The selected edit repeats a directed route pair.');
  return {
    rows, dimensions: dimensions.slice(3), transit, city_edit_index: index, unit: 'modeled minutes',
    rowUnit: 'reported directed comuna route pairs',
    population: transit ? 'Public transport routes between fixed comuna points departing at 08:00 in the routing model.'
      : 'Free-flow car routes between fixed comuna points.',
    stateLabels: {
      base: 'Immediately before the selected route edit',
      scenario: 'Immediately after the selected route edit',
      delta: 'Change from the selected route edit',
    },
    scope: 'The selected edit may follow earlier edits. Both sides use the same reported pairs with finite times. The distribution represents modeled routes, not individual journey times.',
  };
}

function matches(value, { op, value: target }) {
  switch (op) {
    case 'eq': return typeof value === 'string' ? value.toUpperCase() === target.toUpperCase() : value === target;
    case 'lt': return value < target;
    case 'lte': return value <= target;
    case 'gt': return value > target;
    case 'gte': return value >= target;
  }
}

function validateFilters(filters, fields) {
  if (!Array.isArray(filters)) throw new Error('Use an array of filters.');
  for (const f of filters) {
    const numeric = f?.field === 'half_hour';
    if (!fields.includes(f?.field) || Object.keys(f).length !== 3 || !comparisons.includes(f.op) ||
        (numeric ? !Number.isInteger(f.value) : f.op !== 'eq' || typeof f.value !== 'string')) {
      throw new Error(`Unsupported filter. Available dimensions: ${fields.join(', ')}. Only half_hour accepts numeric comparisons.`);
    }
  }
}

function validate(q) {
  const schema = queryResultTool.inputSchema.properties;
  if (Object.keys(q).some(key => !Object.hasOwn(schema, key)) ||
      !datasets.includes(q.data) || !states.includes(q.state) || !operations.includes(q.aggregate)) {
    throw new Error('Use a supported dataset, state, and aggregate from query_result.');
  }
  if (!weights.includes(q.weight) || !Number.isInteger(q.limit) || q.limit < 1 || q.limit > 100 ||
      (q.grain !== undefined && (q.data !== 'metro_entries' || !['day', 'half_hour'].includes(q.grain))) ||
      (q.city_edit_index !== undefined && (q.data !== 'route_minutes' || !Number.isInteger(q.city_edit_index) || q.city_edit_index < 0))) {
    throw new Error('Invalid weight, limit, station grain, or route edit index.');
  }
  if ((q.aggregate === 'percentile' ? !finite(q.percentile) || q.percentile < 0 || q.percentile > 100 : q.percentile !== undefined) ||
      (q.aggregate === 'histogram' ? !Array.isArray(q.breaks) || q.breaks.length < 1 || q.breaks.length > 12 ||
        q.breaks.some((b, i) => !finite(b) || (i > 0 && b <= q.breaks[i - 1])) : q.breaks !== undefined)) {
    throw new Error('Use a percentile from 0 to 100, or one to twelve increasing histogram cutoffs.');
  }
  if (q.aggregate === 'share') {
    if ((!q.share_filters?.length && !q.share_value) ||
        (q.share_value && (Object.keys(q.share_value).length !== 2 || !comparisons.includes(q.share_value.op) || !finite(q.share_value.value)))) {
      throw new Error('A share needs a numerator filter or a numeric threshold.');
    }
  } else if (q.share_value !== undefined || q.share_filters !== undefined) {
    throw new Error('Use numerator filters only for shares.');
  }
  if (q.weight === 'value' && (q.aggregate !== 'share' || q.state === 'delta' || q.data === 'route_minutes')) {
    throw new Error('Value weights are for shares of nonnegative entries or workers.');
  }
  if (q.order !== undefined && (!['value_asc', 'value_desc'].includes(q.order) || q.aggregate === 'histogram')) {
    throw new Error('Value ordering is available for scalar results and shares.');
  }
  if (q.data === 'route_minutes' && q.aggregate === 'sum') throw new Error('Use a mean, percentile, histogram, share or count for route minutes.');
}

function statistic(values, q) {
  const weight = sum(values.map(r => r.weight));
  if (q.aggregate === 'count') return weight;
  if (!weight) return null;
  if (q.aggregate === 'sum') return sum(values.map(r => r.value));
  if (q.aggregate === 'mean') return sum(values.map(r => r.value * r.weight)) / weight;
  if (q.aggregate === 'share') {
    const matching = values.filter(r => (!q.share_value || matches(r.value, q.share_value)) &&
      (q.share_filters ?? []).every(f => matches(r.row[f.field], f)));
    const numerator = sum(matching.map(r => r.weight));
    return { percent: 100 * numerator / weight, numerator, denominator: weight };
  }
  if (q.aggregate === 'histogram') {
    const bins = [null, ...q.breaks].map((min, i) => ({ min_inclusive: min, max_exclusive: q.breaks[i] ?? null, count: 0 }));
    for (const r of values) bins[q.breaks.filter(b => r.value >= b).length].count += r.weight;
    return bins;
  }
  const sorted = values.toSorted((a, b) => a.value - b.value);
  const target = weight * q.percentile / 100;
  let current = 0;
  return (sorted.find(r => (current += r.weight) >= target) ?? sorted.at(-1)).value;
}

export function queryResult(raw, args, loadPublicWorkers) {
  if (args.aggregate === 'share' && args.weight === undefined) {
    throw new Error('Choose the denominator for the share: equal for rows, value for entries or workers, or Census public workers for transit routes.');
  }
  const q = { filters: [], group_by: [], weight: 'equal', limit: 100, ...args };
  validate(q);
  const data = dataset(raw, q);
  validateFilters(q.filters, data.dimensions);
  validateFilters(q.share_filters ?? [], data.dimensions);
  if (!Array.isArray(q.group_by) || q.group_by.length > 2 || new Set(q.group_by).size !== q.group_by.length ||
      q.group_by.some(field => !data.dimensions.includes(field))) {
    throw new Error(`Unsupported grouping. Available dimensions: ${data.dimensions.join(', ')}.`);
  }
  const included = row => q.filters.every(f => matches(row[f.field], f));
  const selected = data.rows.filter(included);
  const paired = q.state === 'compare' || q.state === 'delta' || q.data === 'route_minutes';
  const value = (row, state) => state === 'delta' ? row.scenario - row.base : row[state];
  const valid = row => paired ? finite(row.base) && finite(row.scenario) : finite(value(row, q.state));
  let census;
  let censusRows;
  if (q.weight === 'census_public_workers') {
    if (!data.transit || q.aggregate === 'sum') throw new Error('Census public worker weights are valid only for modeled public transport route minutes.');
    censusRows = loadPublicWorkers();
    census = new Map(censusRows.map(r => [pairKey(r), r.workers]));
  }
  const comparable = selected.filter(valid);
  const rows = census ? comparable.filter(row => census.has(pairKey(row))) : comparable;
  const weightOf = (row, state) => census ? census.get(pairKey(row)) : q.weight === 'value' ? value(row, state) : 1;
  const eligible = new Set(rows);
  const byGroup = new Map();
  for (const row of selected) {
    const group = q.group_by.map(field => row[field]);
    const key = JSON.stringify(group);
    if (!byGroup.has(key)) byGroup.set(key, []);
    byGroup.get(key).push(row);
  }
  if (!q.group_by.length && !byGroup.size) byGroup.set('[]', []);
  const groups = [];
  for (const [key, members] of byGroup) {
    const items = members.filter(row => eligible.has(row));
    const observations = state => items.map(row => ({ row, value: value(row, state), weight: weightOf(row, state) }));
    const totalWeight = state => sum(observations(state).map(r => r.weight));
    const stat = state => statistic(observations(state), q);
    groups.push({
      group: Object.fromEntries(JSON.parse(key).map((v, i) => [q.group_by[i], v])),
      selected_rows: members.length, valid_rows: items.length,
      valid_weight: q.state === 'compare' ? { base: totalWeight('base'), scenario: totalWeight('scenario') } : totalWeight(q.state),
      value: q.state === 'compare' ? { base: stat('base'), scenario: stat('scenario') } : stat(q.state),
    });
  }
  if (q.order) {
    const score = group => {
      const value = q.state === 'compare' ? group.value.scenario : group.value;
      return q.aggregate === 'share' ? value?.percent : value;
    };
    const sign = q.order === 'value_asc' ? 1 : -1;
    groups.sort((a, b) => {
      const left = score(a);
      const right = score(b);
      return finite(left) && finite(right) ? sign * (left - right) : Number(finite(right)) - Number(finite(left));
    });
  }
  const results = groups.slice(0, q.limit);
  const weightUnit = census ? 'observed Census 2024 public work commuters' : q.weight === 'value' ? data.unit : data.rowUnit;
  const coverage = {
    raw_rows: data.rows.length, selected_rows: selected.length, valid_rows: rows.length,
    missing_or_unreachable_rows: selected.length - comparable.length,
    may_be_truncated: data.may_be_truncated ?? false,
    groups: byGroup.size, groups_returned: results.length,
  };
  if (census) {
    coverage.pairs_without_positive_public_weight = comparable.length - rows.length;
    coverage.commuters_with_paired_times = sum(rows.map(row => weightOf(row)));
    coverage.observed_public_commuters_in_scope = sum(censusRows.filter(included).map(r => r.workers));
  }
  return {
    data: q.data, state: q.state, aggregate: q.aggregate, revision: raw.revision ?? null, city_edit_index: data.city_edit_index,
    unit: q.aggregate === 'share' ? 'percent' : ['count', 'histogram'].includes(q.aggregate) ? weightUnit : data.unit,
    measured_unit: data.unit, weight_unit: weightUnit, state_labels: data.stateLabels,
    population: data.population, scope: data.scope,
    weighting: census ? 'Fixed observed Census 2024 public work commuters by directed comuna pair, across the workday. These are not scenario passenger counts.'
      : q.weight === 'value' ? 'Weight each selected row by its modeled count.' : 'Each selected row contributes equally.',
    method: {
      percentile: 'First value whose cumulative weight reaches the requested percentile.',
      share: 'The denominator is the eligible weight within each result group. The numerator applies the share filters to that same group.',
      histogram: 'A cutoff belongs to the bin that starts at that value. Null bounds are open ends.',
    }[q.aggregate],
    coverage, results,
  };
}
