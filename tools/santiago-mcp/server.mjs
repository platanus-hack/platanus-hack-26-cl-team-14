import { createInterface } from 'node:readline';

const api = process.env.SANTIAGO_API_URL ?? 'http://127.0.0.1:8787';
const comunas = '13101 Santiago, 13102 Cerrillos, 13103 Cerro Navia, 13104 Conchalí, 13105 El Bosque, 13106 Estación Central, 13107 Huechuraba, 13108 Independencia, 13109 La Cisterna, 13110 La Florida, 13111 La Granja, 13112 La Pintana, 13113 La Reina, 13114 Las Condes, 13115 Lo Barnechea, 13116 Lo Espejo, 13117 Lo Prado, 13118 Macul, 13119 Maipú, 13120 Ñuñoa, 13121 Pedro Aguirre Cerda, 13122 Peñalolén, 13123 Providencia, 13124 Pudahuel, 13125 Quilicura, 13126 Quinta Normal, 13127 Recoleta, 13128 Renca, 13129 San Joaquín, 13130 San Miguel, 13131 San Ramón, 13132 Vitacura, 13201 Puente Alto, 13202 Pirque, 13203 San José de Maipo, 13301 Colina, 13302 Lampa, 13303 Tiltil, 13401 San Bernardo, 13402 Buin, 13403 Calera de Tango, 13404 Paine, 13501 Melipilla, 13502 Alhué, 13503 Curacaví, 13504 María Pinto, 13505 San Pedro, 13601 Talagante, 13602 El Monte, 13603 Isla de Maipo, 13604 Padre Hurtado, 13605 Peñaflor';
const tool = {
  name: 'simulate_city',
  description: `Run one usual work day of Santiago (2.7 million synthetic workers, Census 2024) after a list of changes, and update the open city view. The workers choose a job place and a mode again. Each call starts from the baseline and applies the complete list together; an empty list restores the baseline. Translate the user's question into changes: a new building, mall or office is jobs and/or residents at a point (use its longitude and latitude); a policy for a whole comuna uses the comuna id; a faster or slower trip is minutes; closing or reopening an existing Metro station is kind "station" with target {type:"station",id:<station name, for example "TOBALABA">}, op "set" and value 0 (closed) or 1 (open). For a question the kinds below cannot express (a new Metro station or line, an event), send kind "station" with a point target, or kind "event": the engine returns it in not_modeled, and you must tell the user that the engine cannot simulate that part yet. All results are model values, not measurements; say that, and read the assumptions. Comuna ids: ${comunas}.`,
  inputSchema: {
    type: 'object',
    required: ['changes'],
    properties: {
      changes: {
        type: 'array',
        items: {
          type: 'object',
          required: ['kind', 'target', 'op', 'value'],
          properties: {
            kind: { enum: ['jobs', 'residents', 'minutes', 'station', 'event'], description: 'jobs: work places. residents: persons who live there (workers follow at the comuna rate). minutes: door-to-door travel minutes between comunas. station: close or reopen an existing Metro station.' },
            target: {
              type: 'object',
              required: ['type'],
              description: 'jobs and residents: {type:"point",longitude,latitude} for a place (new or existing), {type:"comuna",id} for a whole comuna, {type:"block",id} for a census block id. minutes: {type:"comuna",id} for all trips to and from a comuna, or {type:"pair",origin,destination} for one direction between two comunas.',
              properties: {
                type: { enum: ['point', 'comuna', 'block', 'pair', 'station'] },
                id: { type: 'string' },
                longitude: { type: 'number' },
                latitude: { type: 'number' },
                origin: { type: 'string' },
                destination: { type: 'string' },
              },
            },
            op: { enum: ['add', 'multiply', 'set'], description: 'add: value is a count or minutes, negative to remove. multiply: value is a factor. set: value is the new amount.' },
            value: { type: 'number' },
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
    serverInfo: { name: 'santiago', version: '2.0.0' },
  };
  if (method === 'ping') return {};
  if (method === 'tools/list') return { tools: [tool] };
  if (method !== 'tools/call' || params?.name !== tool.name) return null;
  try {
    const response = await fetch(new URL('/api/scenario', api), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(params.arguments),
    });
    if (!response.ok) throw new Error(await response.text());
    // The trip groups are for the map. The agent does not need them.
    const { trips_base, trips_scenario, ...result } = await response.json();
    return { content: [{ type: 'text', text: JSON.stringify({ ...result, trips_omitted: true }) }] };
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
