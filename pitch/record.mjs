// Export frames of a draw(t) page through Chrome DevTools Protocol. Usage: node rec.mjs <url> <outdir> <seconds> <fps>
import { spawn } from 'node:child_process'; import { writeFileSync, mkdirSync } from 'node:fs';
const [url, out, secs, fps] = process.argv.slice(2); mkdirSync(out, { recursive: true });
const ch = spawn('/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', ['--headless=new', '--remote-debugging-port=9517', `--user-data-dir=${out}/../rec-profile`, '--window-size=1920,1080', '--hide-scrollbars', 'about:blank']);
await new Promise(r => setTimeout(r, 2000));
const tabs = await (await fetch('http://127.0.0.1:9517/json')).json();
const ws = new WebSocket(tabs.find(t => t.type === 'page').webSocketDebuggerUrl); await new Promise(r => ws.onopen = r);
let id = 0; const wait = {}; ws.onmessage = e => { const m = JSON.parse(e.data); if (wait[m.id]) { wait[m.id](m.result); delete wait[m.id]; } };
const send = (method, params = {}) => new Promise(r => { wait[++id] = r; ws.send(JSON.stringify({ id, method, params })); });
await send('Emulation.setDeviceMetricsOverride', { width: 1920, height: 1080, deviceScaleFactor: 1, mobile: false });
await send('Page.enable'); await send('Page.navigate', { url }); await new Promise(r => setTimeout(r, 2000));
for (let f = 0; f < secs * fps; f++) {
  await send('Runtime.evaluate', { expression: `draw(${f / fps})` });
  const { data } = await send('Page.captureScreenshot', { format: 'png' });
  writeFileSync(`${out}/f${String(f).padStart(4, '0')}.png`, Buffer.from(data, 'base64'));
}
ws.close(); ch.kill();
