const panels = new Set();

function showPanelState() {
  const open = panels.size > 0;
  document.documentElement.toggleAttribute('data-santiaguito-assistant', open);
  window.dispatchEvent(new CustomEvent('santiaguito:assistant', { detail: { open } }));
}

chrome.runtime.onConnect.addListener((port) => {
  if (port.name !== 'santiaguito-panel') return;
  panels.add(port);
  showPanelState();
  const relay = ({ source, origin, data }) => {
    if (source === window && origin === location.origin && data?.source === 'santiaguito-page' &&
        (data.type === 'reply' || data.type === 'ready')) port.postMessage(data);
  };
  window.addEventListener('message', relay);
  port.onMessage.addListener((request) => {
    window.postMessage({ ...request, source: 'santiaguito-assistant' }, location.origin);
  });
  port.onDisconnect.addListener(() => {
    window.removeEventListener('message', relay);
    panels.delete(port);
    showPanelState();
  });
});
