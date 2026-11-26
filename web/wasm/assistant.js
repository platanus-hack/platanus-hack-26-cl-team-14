// The companion reports its own panel state. WebMCP can report when a tool is used.
export function connectAssistant(tools, call) {
  const panel = document.querySelector('#tools');
  const button = document.querySelector('#toggle-tools');
  const status = document.querySelector('#assistant-status');
  const arrow = document.querySelector('#assistant-arrow');
  let manual = false;
  let connected = document.documentElement.hasAttribute('data-santiaguito-assistant');

  const update = () => {
    const open = manual || connected;
    panel.hidden = !open;
    document.body.classList.toggle('controls-open', open);
    button.hidden = connected;
    button.setAttribute('aria-expanded', String(open));
    button.textContent = open ? 'Ocultar cambios' : 'Cambios';
    status.textContent = connected ? 'Extensión conectada' : 'Abre la extensión de Santiaguito en Chrome';
    arrow.hidden = connected;
  };
  const panelChanged = ({detail}) => {
    connected = detail?.open === true;
    update();
  };
  window.addEventListener('santiaguito:assistant', panelChanged);
  button.disabled = false;
  button.addEventListener('click', () => { manual = !manual; update(); });
  update();

  const allowed = new Set(tools.map(({name}) => name));
  const reply = (id, result, error) => window.postMessage({
    source: 'santiaguito-page', type: 'reply', id, result, error,
  }, location.origin);
  window.addEventListener('message', async (event) => {
    const request = event.data;
    if (event.source !== window || event.origin !== location.origin || !connected ||
        request?.source !== 'santiaguito-assistant') return;
    if (request.type === 'list') reply(request.id, tools);
    if (request.type === 'call') {
      if (!allowed.has(request.name)) {
        reply(request.id, undefined, 'Unknown Santiaguito tool.');
        return;
      }
      try {
        reply(request.id, await call(request.name, request.args));
      } catch (error) {
        reply(request.id, undefined, error.message);
      }
    }
  });
  window.postMessage({source: 'santiaguito-page', type: 'ready'}, location.origin);

  const modelContext = document.modelContext ?? navigator.modelContext;
  if (!modelContext?.registerTool) return;
  for (const descriptor of tools) {
    Promise.resolve().then(() => modelContext.registerTool({
      ...descriptor,
      async execute(args, {signal} = {}) {
        signal?.throwIfAborted();
        manual = true;
        update();
        try {
          return await call(descriptor.name, args);
        } catch (error) {
          return {error: error.message};
        }
      },
    })).catch(error => console.warn(`Could not register ${descriptor.name}: ${error.message}`));
  }
}
