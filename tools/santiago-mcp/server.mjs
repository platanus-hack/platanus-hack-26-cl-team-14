import { createInterface } from 'node:readline';

const api = process.env.SANTIAGO_API_URL ?? 'http://127.0.0.1:8787';
const tool = {
  name: 'simulate_city',
  description: 'Apply a complete set of housing and job multipliers to Santiago. Returns modeled zone and commute changes and updates the open city view. Use an empty changes list to restore baseline. Multipliers apply relative to the Census baseline.',
  inputSchema: {
    type: 'object',
    required: ['changes'],
    properties: {
      changes: {
        type: 'array',
        items: {
          type: 'object',
          required: ['zone_id', 'jobs_multiplier', 'housing_multiplier'],
          properties: {
            zone_id: { type: 'string', description: 'Five-digit comuna CUT. Providencia is 13123 and Las Condes is 13114.' },
            jobs_multiplier: { type: 'number', exclusiveMinimum: 0 },
            housing_multiplier: { type: 'number', exclusiveMinimum: 0 },
          },
        },
      },
    },
  },
};

async function dispatch({ method, params }) {
  if (method === 'initialize') return {
    protocolVersion: '2024-11-05',
    capabilities: { tools: {} },
    serverInfo: { name: 'santiago', version: '1.0.0' },
  };
  if (method === 'ping') return {};
  if (method === 'tools/list') return { tools: [tool] };
  if (method !== 'tools/call' || params?.name !== tool.name) return null;
  try {
    const response = await fetch(new URL('/api/simulate', api), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(params.arguments),
    });
    if (!response.ok) throw new Error(await response.text());
    const { revision, summary, zone_deltas, elapsed_ms } = await response.json();
    return { content: [{ type: 'text', text: JSON.stringify({ revision, summary, zone_deltas, elapsed_ms }) }] };
  } catch (error) {
    return { isError: true, content: [{ type: 'text', text: error.message }] };
  }
}

for await (const line of createInterface({ input: process.stdin })) {
  const request = JSON.parse(line);
  if (!Object.hasOwn(request, 'id')) continue;
  const result = await dispatch(request);
  const reply = result === null ? { error: { code: -32601, message: 'Unknown method or tool' } } : { result };
  process.stdout.write(JSON.stringify({ jsonrpc: '2.0', id: request.id, ...reply }) + '\n');
}
