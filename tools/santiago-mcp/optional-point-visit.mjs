const unsigned = { type: 'integer', minimum: 0 };
const limits = {
  type: 'object',
  required: ['available_minutes', 'budget_clp'],
  properties: { available_minutes: unsigned, budget_clp: unsigned },
};
const quote = {
  type: 'object',
  required: ['total_minutes', 'cost_clp', 'available', 'route_source', 'cost_source', 'access_source'],
  properties: {
    total_minutes: { ...unsigned, description: 'Travel plus activity time. For a stop on a planned trip, use the extra time.' },
    cost_clp: { ...unsigned, description: 'The full cost of this option, including any extra travel cost.' },
    available: { type: 'boolean', description: 'Whether access, opening hours and the route allow the visit.' },
    route_source: { type: 'string' },
    cost_source: { type: 'string' },
    access_source: { type: 'string' },
  },
};

export const optionalPointVisitChange = {
  type: 'object',
  description: 'Compare supplied visit options for the same synthetic worker with a fixed home and job. Use packed activity-point OSM IDs. Supply both time and cash limits. State the source or assumption behind each quote. A null visit means the worker skips.',
  required: ['kind', 'person_index', 'home_block_id', 'job_block_id', 'baseline', 'scenario', 'skip_preference', 'options'],
  properties: {
    kind: { const: 'optional_point_visit' },
    person_index: unsigned,
    home_block_id: { type: 'string' },
    job_block_id: { type: 'string' },
    baseline: limits,
    scenario: limits,
    skip_preference: { type: 'integer', description: 'An option must score above this value.' },
    options: {
      type: 'array',
      items: {
        type: 'object',
        required: ['place_id', 'mode', 'preference', 'baseline'],
        properties: {
          place_id: { type: 'string' },
          mode: { type: 'string' },
          preference: { type: 'integer', description: 'Higher values represent a stronger preference. The score stays the same in both cases.' },
          baseline: quote,
          scenario: { ...quote, description: 'Omit this to use the baseline quote in both cases.' },
        },
      },
    },
  },
};
