// The 14 WASI calls the Rust engine imports, over read-only files. `read(path)` returns the
// file bytes or null; it must answer at once, so the browser worker uses a synchronous request.
// shortcut: no writes, seeks or directories, add them when the engine starts to use them.
export function wasi(read) {
  let memory;
  const view = () => new DataView(memory.buffer);
  const bytes = (ptr, len) => new Uint8Array(memory.buffer, ptr, len);
  const text = (ptr, len) => new TextDecoder().decode(bytes(ptr, len));
  // "data/mvp/../../web/data/x" -> "web/data/x"
  const clean = (path) => path.split('/').reduce((parts, part) => {
    if (part === '..') parts.pop();
    else if (part && part !== '.') parts.push(part);
    return parts;
  }, []).join('/');
  const files = new Map(); // path -> bytes, kept only between the first look and the close
  const file = (ptr, len) => {
    const path = clean(text(ptr, len));
    if (!files.has(path)) files.set(path, read(path));
    return [path, files.get(path)];
  };
  const open = new Map(); // fd -> { path, data, at }
  let nextFd = 4;
  const stat = (buf, type, size) => {
    bytes(buf, 64).fill(0);
    view().setUint8(buf + 16, type);
    view().setBigUint64(buf + 32, BigInt(size), true);
    return 0;
  };
  let line = '';
  const imports = {
    environ_sizes_get: (count, size) => (view().setUint32(count, 0, true), view().setUint32(size, 0, true), 0),
    environ_get: () => 0,
    clock_time_get: (_id, _precision, out) => (view().setBigUint64(out, BigInt(Math.round(performance.now() * 1e6)), true), 0),
    random_get: (ptr, len) => {
      for (let at = 0; at < len; at += 65536) crypto.getRandomValues(bytes(ptr + at, Math.min(65536, len - at)));
      return 0;
    },
    proc_exit: (code) => { throw new Error(`The engine stopped with code ${code}.`); },
    // File descriptor 3 is the one folder the engine sees: the site root.
    fd_prestat_get: (fd, buf) => fd === 3 ? (view().setUint8(buf, 0), view().setUint32(buf + 4, 1, true), 0) : 8,
    fd_prestat_dir_name: (_fd, ptr) => (bytes(ptr, 1)[0] = 46, 0),
    fd_fdstat_get: (fd, buf) => {
      bytes(buf, 24).fill(0);
      view().setUint8(buf, fd === 3 ? 3 : fd < 3 ? 2 : 4);
      view().setBigUint64(buf + 8, ~0n & 0xffffffffffffffffn, true);
      view().setBigUint64(buf + 16, ~0n & 0xffffffffffffffffn, true);
      return 0;
    },
    path_filestat_get: (_fd, _flags, ptr, len, buf) => {
      const [, data] = file(ptr, len);
      return data ? stat(buf, 4, data.length) : 44;
    },
    path_open: (_fd, _dirflags, ptr, len, _oflags, _rights, _inheriting, _fdflags, out) => {
      const [path, data] = file(ptr, len);
      if (!data) return 44;
      open.set(nextFd, { path, data, at: 0 });
      view().setUint32(out, nextFd++, true);
      return 0;
    },
    fd_filestat_get: (fd, buf) => stat(buf, 4, open.get(fd).data.length),
    fd_read: (fd, iovs, count, out) => {
      const f = open.get(fd);
      let total = 0;
      for (let i = 0; i < count; i++) {
        const ptr = view().getUint32(iovs + i * 8, true);
        const part = f.data.subarray(f.at, f.at + view().getUint32(iovs + i * 8 + 4, true));
        bytes(ptr, part.length).set(part);
        f.at += part.length;
        total += part.length;
      }
      view().setUint32(out, total, true);
      return 0;
    },
    fd_write: (_fd, iovs, count, out) => {
      let total = 0;
      for (let i = 0; i < count; i++) {
        const len = view().getUint32(iovs + i * 8 + 4, true);
        line += text(view().getUint32(iovs + i * 8, true), len);
        total += len;
      }
      const lines = line.split('\n');
      line = lines.pop();
      lines.forEach((l) => console.log(l));
      view().setUint32(out, total, true);
      return 0;
    },
    fd_close: (fd) => (files.delete(open.get(fd)?.path), open.delete(fd), 0),
  };
  return { imports: { wasi_snapshot_preview1: imports }, start: (instance) => { memory = instance.exports.memory; instance.exports._start(); } };
}

// Start the engine and return `call(path, body)`: the same paths and JSON as the HTTP API.
export async function engine(wasm, read) {
  const host = wasi(read);
  const { instance } = await WebAssembly.instantiate(wasm, host.imports);
  host.start(instance);
  const { memory, santiago_alloc, santiago_call } = instance.exports;
  return (path, body) => {
    const request = new TextEncoder().encode(JSON.stringify({ path, body }));
    const ptr = santiago_alloc(request.length);
    new Uint8Array(memory.buffer, ptr, request.length).set(request);
    const reply = santiago_call(ptr, request.length);
    return JSON.parse(new TextDecoder().decode(new Uint8Array(memory.buffer, Number(reply >> 32n), Number(reply & 0xffffffffn))));
  };
}
