// The MCP server (tools/santiago-mcp) reads files with node:fs. The page import map sends it here.
// shortcut: a read waits for its download on the main thread, move the MCP to a worker when a read stops the view.
const root = new URL('../../../', import.meta.url);
/** Files the page already has: full address -> bytes. */
export const preloaded = new Map();
const written = new Map(); // replays saved in this tab
const get = (url) => {
  const request = new XMLHttpRequest();
  request.open('GET', url, false);
  request.send();
  if (request.status !== 200) throw Object.assign(new Error(`No file at ${url}`), { code: 'ENOENT' });
  return request.responseText;
};
const folders = JSON.parse(get(new URL('../folders.json', import.meta.url)));

export function readFileSync(url) {
  const address = String(url);
  if (written.has(address)) return written.get(address);
  const bytes = preloaded.get(address);
  return bytes ? new TextDecoder().decode(bytes) : get(address);
}
export function readdirSync(url) {
  const folder = String(url);
  const saved = [...written.keys()].filter((address) => address.startsWith(folder)).map((address) => address.slice(folder.length));
  return [...new Set([...folders[folder.slice(root.href.length)], ...saved])];
}
export function writeFileSync(url, text) { written.set(String(url), text); }
export function renameSync(from, to) { written.set(String(to), written.get(String(from))); written.delete(String(from)); }
export function rmSync(url) { written.delete(String(url)); }
/** The text of a file the MCP server wrote in this tab, or undefined. */
export const saved = (address) => written.get(address);
