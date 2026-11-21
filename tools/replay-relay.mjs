// Serve one cached replay to one viewer, with no Rust engine: node tools/replay-relay.mjs <port> <replay-id>
// The viewer reads only these two routes (viz/src/net.rs).
import { readFileSync } from 'node:fs';
import { createServer } from 'node:http';

const [port, id] = process.argv.slice(2);
const { result } = JSON.parse(readFileSync(new URL(`../data/replays/${id}.json`, import.meta.url), 'utf8'));
result.revision = 1;
result.scene.revision = 1;
const body = JSON.stringify(result);

createServer((request, response) => {
  if (request.url === '/api/scenario-latest') {
    response.writeHead(200, { 'Content-Type': 'application/json' }).end(body);
  } else if (request.url === '/api/scenario-events') {
    // Keep the stream open: the viewer stops reading when it closes.
    response.writeHead(200, { 'Content-Type': 'text/event-stream' }).write(`event: scenario\nid: 1\ndata: ${body}\n\n`);
  } else {
    response.writeHead(404).end();
  }
}).listen(+port, '127.0.0.1', () => console.error(`Replay ${id} on http://127.0.0.1:${port}`));
