const whole = { type: 'integer', minimum: 0 };
const text = { type: 'string', minLength: 1 };
const object = (properties, required = Object.keys(properties)) => ({
  type: 'object', properties, required, additionalProperties: false,
});
const roundTrip = object({
  in_vehicle_seconds: whole, wait_seconds: whole, walk_seconds: whole, cycle_seconds: whole, fare_clp: whole, source: text,
});
const option = object({
  id: text, place: text, interest_clp: { type: 'integer' }, duration_minutes: whole,
  admission_clp: whole, open: { type: 'boolean' }, places_available: whole, access_basis: text,
  route: { oneOf: [roundTrip, { type: 'null' }] },
});

export const optionalAttendanceChange = {
  ...object({
    kind: { const: 'optional_attendance' },
    request: object({
      person: object({
        group: { enum: ['worker_or_student', 'other_resident'] }, index: whole, home_block: text,
      }),
      available_minutes: whole, budget_clp: whole, preference_basis: text,
      time_values: object({
        in_vehicle_clp_per_hour: whole, wait_clp_per_hour: whole, walk_clp_per_hour: whole, cycle_clp_per_hour: whole, source: text,
      }),
      options: { type: 'array', minItems: 1, items: option },
    }),
  }),
  description: 'Compare supplied optional activities for one synthetic person. Use per-venue access and available places, full round-trip seconds and cash. Stay home has zero utility. This gives a conditional choice, not an attendance forecast. Read applied[].understood.',
};
