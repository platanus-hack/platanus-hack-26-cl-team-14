import { readFileSync, writeFileSync } from 'node:fs';
import { engine } from '../../web/wasm/wasi.js';

const read = (path) => {
  try {
    return readFileSync(path);
  } catch (error) {
    if (error.code === 'ENOENT') return null;
    throw error;
  }
};

const call = await engine(readFileSync('web/wasm/engine.wasm'), read);
const { status, body } = call('/api/scenario', { changes: [] });
if (status !== 200) throw new Error(String(body));
writeFileSync('web/wasm/baseline.json', JSON.stringify(body));
