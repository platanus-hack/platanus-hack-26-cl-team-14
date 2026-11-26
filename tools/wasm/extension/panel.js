const $ = (id) => document.getElementById(id);
const controls = $('controls');
const status = $('status');
const resultView = $('result');
const ask = $('ask');
const pending = new Map();
let port;
let tools = [];
let busy = false;
let tabId;
let panelOpen = true;

function request(type, name, args) {
  const id = crypto.randomUUID();
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    port.postMessage({ type, id, name, args });
  });
}

function setTab(id) {
  if (port) {
    port.disconnect();
    clearConnection();
  }
  tabId = id;
  if (id === undefined) return;
  const connection = chrome.tabs.connect(id, { frameId: 0, name: 'santiaguito-panel' });
  port = connection;

  async function loadTools() {
    try {
      const available = await request('list');
      if (port !== connection) return;
      tools = available.filter(({ name }) => ['simulate_city', 'set_scene', 'query_result'].includes(name));
      controls.hidden = false;
      ask.hidden = !('LanguageModelToolCall' in globalThis);
      status.textContent = 'Conectado al simulador.';
    } catch (error) {
      if (port === connection) status.textContent = error.message;
    }
  }

  connection.onMessage.addListener(({ type, id, result, error }) => {
    if (type === 'ready') {
      void loadTools();
      return;
    }
    const reply = pending.get(id);
    if (!reply) return;
    pending.delete(id);
    if (error) reply.reject(new Error(error));
    else reply.resolve(result);
  });
  connection.onDisconnect.addListener(() => {
    // Chrome reports a disconnect when the active tab has no bridge.
    void chrome.runtime.lastError;
    if (port === connection) clearConnection();
  });
  void loadTools();
}

function clearConnection() {
  port = undefined;
  controls.hidden = true;
  status.textContent = 'Abre el simulador en esta ventana.';
  resultView.textContent = '';
  for (const { reject } of pending.values()) reject(new Error('Se cerró la conexión con el simulador.'));
  pending.clear();
}

async function run(action) {
  if (busy) return;
  busy = true;
  const connection = port;
  resultView.textContent = 'Ejecutando…';
  try {
    const reply = await action(connection);
    if (port === connection) {
      const result = reply.result ?? reply;
      resultView.textContent = JSON.stringify(result.summary ?? result, null, 2);
    }
  } catch (error) {
    if (port === connection) resultView.textContent = error.message;
  } finally {
    busy = false;
  }
}

$('ferry').onclick = () => run(() => request('call', 'run_replay', { id: 'mapocho-muelle-ferry' }));
$('reset').onclick = () => run(() => request('call', 'simulate_city', { changes: [] }));
ask.onsubmit = (event) => {
  event.preventDefault();
  void run(async (connection) => {
    const prompt = $('prompt').value.trim();
    const options = {
      tools,
      expectedInputs: [{ type: 'text', languages: ['es'] }],
      expectedOutputs: [{ type: 'text', languages: ['es'] }, { type: 'tool-call' }],
    };
    if (await LanguageModel.availability(options) === 'unavailable') {
      throw new Error('Gemini Nano no está disponible. Usa los controles del simulador.');
    }
    const session = await LanguageModel.create({
      ...options,
      initialPrompts: [{
        role: 'system',
        content: 'Controla la simulación de Santiago. Usa una herramienta solo si la petición incluye los datos necesarios.',
      }],
    });
    try {
      const response = await session.prompt(prompt);
      const calls = response.filter(({ type }) => type === 'tool-call');
      const call = calls[0]?.value;
      if (calls.length !== 1 || !tools.some(({ name }) => name === call?.name)) {
        throw new Error('Indica un cambio con sus datos o un lugar concreto.');
      }
      if (port !== connection) throw new Error('Vuelve a la pestaña del simulador.');
      return await request('call', call.name, call.arguments ?? {});
    } finally {
      session.destroy();
    }
  });
};

const { id: windowId } = await chrome.windows.getCurrent();
async function selectTab() {
  const [active] = await chrome.tabs.query({ active: true, windowId });
  if (panelOpen) setTab(active?.id);
}

chrome.sidePanel.onOpened.addListener((panel) => {
  if (panel.windowId !== windowId) return;
  panelOpen = true;
  if (!port) void selectTab();
});
chrome.sidePanel.onClosed.addListener((panel) => {
  if (panel.windowId !== windowId) return;
  panelOpen = false;
  setTab(undefined);
});
chrome.tabs.onActivated.addListener((active) => {
  if (panelOpen && active.windowId === windowId) setTab(active.tabId);
});
chrome.tabs.onUpdated.addListener((id, change) => {
  if (panelOpen && id === tabId && change.status === 'complete') setTab(id);
});
void selectTab();
