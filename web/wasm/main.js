// The page joins four parts: the data files, the engine (Wasm in a worker), the viewer (Wasm + WebGPU)
// and the MCP server (tools/santiago-mcp/server.mjs, unchanged). Each control is one or two MCP tool calls.
import init, { santiagoCursor, santiagoReady, santiagoScenario, santiagoZoom } from './viewer.js';
import { preloaded, saved } from './node/fs.js';
import { send } from './node/readline.js';

const root = new URL('../../', import.meta.url);
const $ = (selector) => document.querySelector(selector);
// The bottom band: a line of text while the page works, hidden when the viewer shows its own subtitle.
const note = (text) => { $('#note').textContent = text ?? ''; $('#note').hidden = !text; };

// The MCP server writes the result cards to this file. The viewer reads it four times per second.
const resultCards = 'result-display.json';
// The page loads the startup files before the viewer starts.
const files = new Map();
const downloads = new Map();
function loadFile(path) {
  if (!downloads.has(path)) {
    const url = new URL(path, root);
    downloads.set(path, fetch(url).then(async (response) => {
      if (!response.ok) throw new Error(`Download of ${path} failed with status ${response.status}`);
      const bytes = new Uint8Array(await response.arrayBuffer());
      files.set(path, bytes);
      preloaded.set(url.href, bytes);
    }));
  }
  return downloads.get(path);
}
globalThis.santiagoFile = (path) => {
  if (path === resultCards) return saved(path) && new TextEncoder().encode(saved(path));
  const bytes = files.get(path.replace(/^\.\//, ''));
  if (!bytes) console.error(`The viewer asked for a file that the page did not download: ${path}`);
  return bytes;
};
const baselineText = fetch(new URL('baseline.json', import.meta.url)).then((response) => {
  if (!response.ok) throw new Error(`Download of baseline.json failed with status ${response.status}`);
  return response.text();
});
const list = await (await fetch(new URL('viewer-files.json', import.meta.url))).json();
let done = 0;
const [baseline] = await Promise.all([baselineText, ...list.map(async (path) => {
  await loadFile(path);
  note(`santiaguito.cl · ${Math.round(100 * ++done / list.length)} %`);
})]);

// The engine answers the same paths and JSON as its HTTP API.
const waiting = new Map();
let ready;
function startEngine() {
  const worker = new Worker(new URL('engine-worker.js', import.meta.url), { type: 'module' });
  return new Promise((resolve, reject) => {
    worker.onmessage = ({ data }) => data.ready ? resolve(worker) : waiting.get(data.id)(data);
    worker.onerror = (event) => {
      const message = `El motor se detuvo: ${event.message}`;
      note(message);
      reject(new Error(message));
    };
    worker.postMessage(baseline);
  });
}
let calls = 0;
const engine = async (path, body) => {
  const worker = await (ready ??= startEngine());
  return new Promise((resolve) => { waiting.set(++calls, resolve); worker.postMessage({ id: calls, path, body }); });
};

// The MCP server calls the engine with fetch. Here that call goes to the worker, and
// after each change the viewer gets the full latest result, as it does from the native event stream.
const engineAddress = 'http://engine.invalid';
const network = globalThis.fetch;
globalThis.fetch = async (input, options) => {
  const url = new URL(input);
  if (url.origin !== engineAddress) return network(input, options);
  const reply = await engine(url.pathname, options?.body && JSON.parse(options.body));
  if (reply.status === 200 && options?.method === 'POST' && url.pathname !== '/api/building-footprint') {
    const { body: latest } = await engine('/api/scenario-latest');
    const edits = latest.city_edits ?? [];
    await Promise.all([
      edits.some(edit => edit.kind === 'road' && edit.before) && loadFile('web/data/road_centerlines.geojson.gz'),
      edits.some(edit => edit.kind === 'line' && edit.after?.properties?.mode === 'boat') && loadFile('data/mvp/editable_transit.json'),
    ]);
    santiagoScenario(JSON.stringify(latest));
  }
  return new Response(typeof reply.body === 'string' ? reply.body : JSON.stringify(reply.body), { status: reply.status });
};

// The MCP server reads requests from stdin and writes answers to stdout. The page is both.
const answers = new Map();
globalThis.process = {
  env: { SANTIAGO_API_URL: engineAddress, SANTIAGO_RESULT_DISPLAY_FILE: resultCards },
  stdout: { write: (line) => { const answer = JSON.parse(line); answers.get(answer.id)(answer); } },
  stderr: { write: (text) => console.error(text) },
};
let requests = 0;
/** One MCP request, for example mcp('tools/list'). A relay can pass the requests of an agent here. */
const mcp = (method, params) => new Promise((resolve) => {
  answers.set(++requests, resolve);
  send(JSON.stringify({ jsonrpc: '2.0', id: requests, method, params }));
});
globalThis.santiagoMcp = mcp;
const tool = async (name, args = {}) => {
  const { result } = await mcp('tools/call', { name, arguments: args });
  if (result.isError) throw new Error(result.content[0].text);
  return JSON.parse(result.content[0].text);
};
// The server waits for requests and does not return, so the page does not wait for this import.
import(new URL('tools/santiago-mcp/server.mjs', root)).catch((error) => note(String(error)));

// A trackpad pinch reaches the page as a wheel event with ctrlKey. The viewer zooms; it does not pan.
$('#frame').addEventListener('wheel', (event) => {
  if (!event.ctrlKey) return;
  event.preventDefault();
  event.stopPropagation();
  santiagoZoom(-event.deltaY * 0.02);
}, { capture: true, passive: false });

// The viewer event loop can leave `init` with an exception. That is a normal start.
await init().catch((error) => { if (!String(error).includes('Using exceptions for control flow')) throw error; });

// Show the usual day while the engine stays idle until the first tool call.
santiagoScenario(baseline);

// The band stays until the viewer has the buildings on the screen. A hidden tab does not draw, so the wait has a limit.
note('santiaguito.cl · dibujando la ciudad');
for (const start = performance.now(); !santiagoReady() && performance.now() - start < 60000;) await new Promise((resolve) => setTimeout(resolve, 250));

// The form is the MCP tool interface: the page builds it from the input schema that the server gives.
const el = (tag, props = {}, ...children) => {
  const element = Object.assign(document.createElement(tag), props);
  element.append(...children);
  return element;
};
const small = (text, onclick) => el('button', { type: 'button', textContent: text, onclick });
const variantName = (schema, index) => {
  const tag = schema.properties?.kind ?? schema.properties?.type ?? schema;
  // A long list of kinds does not fit the control: the `kind` field below it has them all.
  const kinds = tag.enum?.length > 3 ? [...tag.enum.slice(0, 2), '…'] : tag.enum;
  return tag.const ?? kinds?.join(' · ') ?? schema.required?.join(', ') ?? schema.type ?? `${index + 1}`;
};

// The valid ids of each kind of place, from the same files the MCP server reads. A field also accepts the name.
const table = async (file) => (await (await fetch(new URL(`data/mvp/${file}`, root))).text()).trim().split('\n').slice(1).map((line) => line.split(','));
const catalog = (kind, rows) => document.body.append(el('datalist', { id: `catalog-${kind}` },
  ...rows.map(([value, label]) => el('option', { value, label }))));
catalog('comuna', await table('communes.csv'));
catalog('station', (await (await fetch(new URL('data/mvp/metro_training.json', root))).json()).map(({ id, name }) => [id, name]));
const places = await table('named_places.csv');
catalog('place', places);
// A `name` field takes the readable name; the card on the city shows it as its title.
catalog('placename', places.map(([id, name]) => [name, id]));
// Text fields whose name tells the kind of id. An `id` field takes its kind from the `type` beside it.
const kindOf = { origin: 'comuna', destination: 'comuna', stations: 'station', name: 'placename' };
// A target names its place by type: a point has coordinates, a pair has two comunas, the other types have an id.
const placeFields = ['id', 'longitude', 'latitude', 'origin', 'destination'];
const fieldsOf = { point: ['longitude', 'latitude'], pair: ['origin', 'destination'] };

// The user clicks the map once. The viewer gives the longitude and latitude under the cursor.
const pickOnMap = (fill) => {
  note('Haz clic en el mapa para elegir el punto.');
  // The viewer learns the cursor position on its next frame, so the page waits two frames after the click.
  $('#frame').addEventListener('click', () => requestAnimationFrame(() => requestAnimationFrame(() => {
    const point = santiagoCursor();
    if (point) fill([...point].map((value) => +value.toFixed(6)));
    note(point ? undefined : 'Ese punto está fuera del mapa. Elige otro.');
  })), { once: true });
};

/** Controls for one JSON Schema node. Returns [element, read]; read() gives the value, or undefined when empty. */
function control(schema, name) {
  if (schema.oneOf) {
    const box = el('div', { className: 'group' });
    const pick = el('select', {}, ...schema.oneOf.map((variant, i) => el('option', { value: i, textContent: variantName(variant, i) })));
    let read;
    const show = () => {
      const variant = schema.oneOf[pick.value];
      const [child, readChild] = control(variant, name);
      read = readChild;
      // "?" shows the text that the MCP server gives an agent for this kind of value.
      const help = el('div', { className: 'help', textContent: variant.description ?? '', hidden: true });
      const head = el('div', { className: 'row' }, pick);
      if (variant.description) head.append(small('?', () => { help.hidden = !help.hidden; }));
      box.replaceChildren(head, help, child);
    };
    pick.onchange = show;
    show();
    return [box, () => read()];
  }
  // A fixed value: the form shows it and sends it.
  if ('const' in schema) return [el('span', { textContent: schema.const }), () => schema.const];
  if (schema.enum) {
    const pick = el('select', {}, ...schema.enum.map((value, i) => el('option', { value: i, textContent: `${value}` })));
    return [pick, () => schema.enum[pick.value]];
  }
  const type = [schema.type].flat().find((name) => name !== 'null');
  if (schema.properties) return fields(schema);
  if (type === 'array' && schema.items.type !== 'array') return items(schema, name);
  if (type === 'boolean') {
    const box = el('input', { type: 'checkbox' });
    return [box, () => box.checked];
  }
  if (type === 'number' || type === 'integer') {
    const input = el('input', { type: 'number', step: type === 'integer' ? 1 : 'any' });
    const { minimum = -Infinity, maximum = Infinity, exclusiveMinimum = -Infinity } = schema;
    return [input, () => {
      if (input.value === '') return undefined;
      const value = Number(input.value);
      // The schema gives the limits. A value the engine cannot hold stops here with a clear message.
      if (value < minimum || value > maximum || value <= exclusiveMinimum || (type === 'integer' && !Number.isSafeInteger(value))) {
        const limits = [Number.isFinite(minimum) && `desde ${minimum}`, Number.isFinite(exclusiveMinimum) && `mayor que ${exclusiveMinimum}`, Number.isFinite(maximum) && `hasta ${maximum}`].filter(Boolean).join(' ');
        throw new Error(`${name}: usa un número ${type === 'integer' ? 'entero ' : ''}${limits || 'más chico'}.`);
      }
      return value;
    }];
  }
  if (type === 'string') {
    const input = el('input', { type: 'text' });
    if (kindOf[name]) input.setAttribute('list', `catalog-${kindOf[name]}`);
    return [input, () => input.value || undefined];
  }
  if (schema.additionalProperties?.type) return named(schema, name);
  // Coordinates of a line or a polygon: the user writes the JSON.
  const text = el('textarea', { rows: 3, placeholder: 'JSON' });
  return [text, () => {
    if (!text.value.trim()) return undefined;
    try { return JSON.parse(text.value); } catch { throw new Error(`${name}: el texto no es JSON válido.`); }
  }];
}

// An object: the required fields, then "+ opcional" to add an optional field by name.
function fields(schema) {
  const box = el('div', { className: 'group' });
  const rows = new Map(); // name -> [row, read]
  const required = schema.required ?? [];
  const more = el('select');
  // The schema of some objects does not list every field the server reads (a line target of type "mode").
  const other = '+ otro campo';
  const refresh = () => {
    const free = Object.keys(schema.properties).filter((name) => !rows.has(name));
    if (schema.additionalProperties !== false) free.push(other);
    more.replaceChildren(el('option', { value: '', textContent: '+ opcional' }), ...free.map((name) => el('option', { textContent: name })));
    more.hidden = !free.length;
  };
  const remove = (name) => { rows.get(name)?.[0].remove(); rows.delete(name); refresh(); };
  const add = (name) => {
    const [child, read] = control(schema.properties[name], name);
    const help = schema.properties[name].description;
    const row = el('div', { className: 'row' }, el('span', { textContent: name }), child);
    // "?" shows the text that the MCP server gives an agent for this field.
    if (help) row.append(small('?', () => { row.querySelector('.help')?.remove() ?? row.append(el('div', { className: 'help', textContent: help })); }));
    if (name === 'longitude' && schema.properties.latitude) row.append(small('mapa', () => pickOnMap(([longitude, latitude]) => {
      row.querySelector('input').value = longitude;
      rows.get('latitude')[0].querySelector('input').value = latitude;
    })));
    if (!required.includes(name)) row.append(small('×', () => remove(name)));
    rows.set(name, [row, read]);
    box.insertBefore(row, more);
    refresh();
  };
  const extra = []; // [key input, read]
  const addOther = () => {
    const key = el('input', { type: 'text', placeholder: 'nombre' });
    const value = el('textarea', { rows: 1, placeholder: 'valor JSON' });
    // A value that is not JSON goes as plain text.
    const pair = [key, () => { try { return JSON.parse(value.value); } catch { return value.value; } }];
    const row = el('div', { className: 'row' }, key, value, small('×', () => { row.remove(); extra.splice(extra.indexOf(pair), 1); }));
    extra.push(pair);
    box.insertBefore(row, more);
    refresh();
  };
  box.append(more);
  required.forEach(add);
  more.onchange = () => more.value === other ? addOther() : add(more.value);
  refresh();
  // The `type` of a place decides its other fields and which ids its `id` field offers.
  const kinds = schema.properties.type?.enum;
  if (kinds && rows.has('type')) {
    const pick = rows.get('type')[0].querySelector('select');
    const fit = () => {
      const kind = kinds[pick.value];
      if (!required.includes('id')) for (const name of placeFields.filter((each) => schema.properties[each])) {
        const wanted = (fieldsOf[kind] ?? ['id']).includes(name);
        if (wanted && !rows.has(name)) add(name);
        if (!wanted) remove(name);
      }
      rows.get('id')?.[0].querySelector('input').setAttribute('list', `catalog-${kind}`);
    };
    pick.addEventListener('change', fit);
    fit();
  }
  return [box, () => {
    const value = Object.fromEntries([...rows].map(([name, [, read]]) => [name, read()]).filter(([, each]) => each !== undefined));
    for (const name of required) if (value[name] === undefined) throw new Error(`Falta ${name}.`);
    for (const [key, read] of extra) if (key.value) value[key.value] = read();
    return value;
  }];
}

// A free map, for example pier id -> point: "+" adds a name and its value.
function named(schema, name) {
  const box = el('div', { className: 'group' });
  const pairs = [];
  const plus = small('+', () => {
    const key = el('input', { type: 'text', placeholder: 'id', className: 'key' });
    const pair = [key, control(schema.additionalProperties, name)];
    const row = el('div', { className: 'row' }, key, pair[1][0], small('×', () => { row.remove(); pairs.splice(pairs.indexOf(pair), 1); }));
    pairs.push(pair);
    box.insertBefore(row, plus);
  });
  box.append(plus);
  return [box, () => pairs.length ? Object.fromEntries(pairs.map(([key, [, read]]) => [key.value, read()])) : undefined];
}

// A list: "+" adds one item, "×" removes it.
function items(schema, name) {
  const box = el('div', { className: 'group' });
  const reads = [];
  const plus = small('+', () => {
    const [child, read] = control(schema.items, name);
    const row = el('div', { className: 'row' }, child, small('×', () => { row.remove(); reads.splice(reads.indexOf(read), 1); }));
    reads.push(read);
    box.insertBefore(row, plus);
  });
  box.append(plus);
  for (let i = 0; i < (schema.minItems ?? 0); i++) plus.click();
  // A point is a list of two numbers, longitude and latitude.
  if (schema.minItems === 2 && schema.maxItems === 2 && schema.items.type === 'number') {
    box.querySelectorAll('input').forEach((input, i) => { input.placeholder = ['longitud', 'latitud'][i]; });
    box.append(small('mapa', () => pickOnMap((point) => box.querySelectorAll('input').forEach((input, i) => { input.value = point[i]; }))));
  }
  return [box, () => reads.map((read) => read()).filter((value) => value !== undefined)];
}

// The two tools that change the city. Saved replays are not in this form.
const shown = ['set_scene', 'simulate_city'];
const tools = (await mcp('tools/list')).result.tools.filter(({ name }) => shown.includes(name));
// The tool reply, as an agent receives it. Totals and short lists come first and start open;
// long notes go last. A list item shows its name, not only its number.
const number = new Intl.NumberFormat('es-CL', { maximumFractionDigits: 2 });
const plain = (value) => typeof value === 'number' ? number.format(value) : `${value}`;
const single = (value) => value === null || typeof value !== 'object';
const long = (value) => typeof value === 'string' && value.length > 60;
const notes = (value) => long(value) || (Array.isArray(value) && value.some(long));
const parts = (value) => Array.isArray(value)
  ? value.map((each, i) => [[i + 1, !single(each) && (each.name ?? each.id ?? each.kind)].filter(Boolean).join(' · '), each])
  : Object.entries(value);
const short = (value) => !single(value) && !notes(value) && (Array.isArray(value) ? value.length > 0 && value.length <= 5 : Object.values(value).every(single));
// A row of numbers: "; " between them, because the decimal mark is a comma.
const series = (value) => Array.isArray(value) && value.every(single) && !notes(value);
function view(name, value, top) {
  if (long(value)) return el('div', { className: 'text' }, el('span', { textContent: name }), el('p', { textContent: value }));
  if (single(value) || (series(value) && value.length <= 8)) {
    return el('div', { className: 'row' }, el('span', { textContent: name }), el('b', { textContent: single(value) ? plain(value) : value.map(plain).join('; ') || '—' }));
  }
  // A long row of numbers stays closed until the user opens it. A day of 48 half hours shows the hour of each value above zero.
  if (series(value)) {
    const day = value.length === 48 && value.every((each) => typeof each === 'number');
    const hour = (i) => `${String(Math.floor(i / 2)).padStart(2, '0')}:${i % 2 ? '30' : '00'}`;
    const text = day ? value.map((each, i) => each && `${hour(i)} ${plain(each)}`).filter(Boolean).join('; ') || '0 en todo el día' : value.map(plain).join('; ');
    return el('details', {}, el('summary', { textContent: `${name} (${value.length})` }), el('p', { textContent: text }));
  }
  // Inside a section: single values first, long notes last.
  const entries = parts(value);
  if (!Array.isArray(value)) entries.sort((a, b) => inner(a) - inner(b));
  return el('details', { open: top && short(value) },
    el('summary', { textContent: Array.isArray(value) ? `${name} (${entries.length})` : name }),
    ...entries.slice(0, 30).map(([key, each]) => view(key, each, false)),
    ...(entries.length > 30 ? [el('div', { textContent: `… ${entries.length - 30} más` })] : []));
}
const rank = ([, value]) => short(value) ? 0 : notes(value) ? 3 : single(value) ? 1 : 2;
const inner = ([, value]) => notes(value) ? 3 : single(value) ? 0 : series(value) && value.length > 8 ? 2 : 1;
const showReply = (reply) => {
  const entries = Object.entries(reply).sort((a, b) => rank(a) - rank(b));
  $('#result').replaceChildren(...entries.map(([name, value]) => view(name, value, true)));
  $('#result').hidden = false;
  $('#result').scrollIntoView({ behavior: 'smooth', block: 'start' });
};

// One line for a person who sees the form for the first time. The full text of the MCP server is behind "?".
const hints = {
  set_scene: 'Lleva la cámara a un lugar y a una hora.',
  simulate_city: 'Agrega cambios con + y envía. La lista vacía vuelve al día normal.',
};

// Each tool keeps its form, so a change of tab does not clear what the user wrote.
const forms = new Map(tools.map((each) => [each.name, control(each.inputSchema)]));
$('#input').append(...[...forms.values()].map(([form]) => form));
let chosen;
const open = (name) => {
  chosen = name;
  for (const tab of $('#tabs').children) tab.className = tab.textContent === name ? '' : 'blue';
  for (const [each, [form]] of forms) form.hidden = each !== name;
  $('#about').textContent = tools.find((each) => each.name === name).description;
  $('#hint').textContent = hints[name];
};
for (const each of tools) $('#tabs').append(small(each.name, () => open(each.name)));
// "?" beside the tabs shows the text that the MCP server gives an agent for the tool.
$('#tabs').append(small('?', () => { $('#about').hidden = !$('#about').hidden; }));
open(tools[0].name);
$('#tools').onsubmit = async (event) => {
  event.preventDefault();
  $('#send').disabled = true;
  note('El motor calcula el cambio en esta pestaña…');
  // An error from the form or from the tool stays in the band until the next call.
  await Promise.resolve().then(() => tool(chosen, forms.get(chosen)[1]())).then((reply) => {
    // A change that the engine did not apply stays in the band with its reason.
    note(reply.not_modeled?.map((change) => change.reason).filter(Boolean).join(' ') || undefined);
    showReply(reply);
  }, (error) => note(error.message));
  $('#send').disabled = false;
};
$('#tools').hidden = false;
note();
