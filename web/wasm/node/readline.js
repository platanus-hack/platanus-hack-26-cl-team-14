// The page sends JSON-RPC lines to the MCP server here, in place of stdin.
const lines = [];
let wake;
export function send(line) { lines.push(line); wake?.(); }
export function createInterface() {
  return { async *[Symbol.asyncIterator]() {
    for (;;) {
      while (lines.length) yield lines.shift();
      await new Promise((resolve) => { wake = resolve; });
    }
  } };
}
