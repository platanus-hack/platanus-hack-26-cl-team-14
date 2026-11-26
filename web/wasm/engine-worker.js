// The Rust engine in a worker. A message {id, path, body} gets the answer {id, status, body}:
// the same paths and JSON as the engine HTTP API.
import { engine } from './wasi.js';

const root = new URL('../../', import.meta.url);
// The engine reads its data files on demand. A worker can wait for a download; a page cannot.
const read = (path) => {
  const request = new XMLHttpRequest();
  request.open('GET', new URL(path, root), false);
  request.responseType = 'arraybuffer';
  request.send();
  if (request.status === 200) return new Uint8Array(request.response);
  // Only "not found" is a missing file. Another failure must stop the engine: a file it skips changes the numbers.
  if (request.status === 404) return null;
  throw new Error(`Download of ${path} failed with status ${request.status}`);
};
const call = await engine(await (await fetch(new URL('engine.wasm', import.meta.url))).arrayBuffer(), read);
onmessage = ({ data: { id, path, body } }) => postMessage({ id, ...call(path, body) });
postMessage({ ready: true });
