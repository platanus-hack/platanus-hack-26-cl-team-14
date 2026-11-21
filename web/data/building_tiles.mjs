// Read an inflated city building file. All tile arrays share the input buffer.
export function buildingTiles(buffer) {
  const header = new DataView(buffer);
  const magic = String.fromCharCode(...new Uint8Array(buffer, 0, 4));
  const columns = header.getUint16(4, true);
  const rows = header.getUint16(6, true);
  const west = header.getInt32(8, true);
  const south = header.getInt32(12, true);
  const east = header.getInt32(16, true);
  const north = header.getInt32(20, true);
  const step = header.getUint32(24, true);
  const offsets = new Uint32Array(buffer, 28, columns * rows + 1);

  function tile(column, row) {
    const index = row * columns + column;
    const start = offsets[index];
    if (start === offsets[index + 1]) return null;
    const count = header.getUint32(start, true);
    if (magic === 'SGBX') {
      const boxes = new Uint16Array(buffer, start + 4, count * 6);
      const flags = new Uint8Array(buffer, boxes.byteOffset + boxes.byteLength, count);
      return {count, boxes, flags};
    }
    const ringCount = header.getUint32(start + 4, true);
    const vertexCount = header.getUint32(start + 8, true);
    let at = start + 12;
    const polygonOffsets = new Uint32Array(buffer, at, count + 1);
    at += polygonOffsets.byteLength;
    const ringOffsets = new Uint32Array(buffer, at, ringCount + 1);
    at += ringOffsets.byteLength;
    const positions = new Uint16Array(buffer, at, vertexCount * 2);
    at += positions.byteLength;
    const heights = new Uint16Array(buffer, at, count);
    const flags = new Uint8Array(buffer, at + heights.byteLength, count);
    return {count, polygonOffsets, ringOffsets, positions, heights, flags};
  }

  return {magic, columns, rows, west, south, east, north, step, tile};
}

// Some hosts serve .gz as an archive. Others send Content-Encoding: gzip.
export async function loadBuildingTiles(url) {
  const response = await fetch(url);
  let bytes = await response.arrayBuffer();
  if (new Uint8Array(bytes)[0] === 0x1f) {
    const unzip = new Response(bytes).body.pipeThrough(new DecompressionStream('gzip'));
    bytes = await new Response(unzip).arrayBuffer();
  }
  return buildingTiles(bytes);
}
