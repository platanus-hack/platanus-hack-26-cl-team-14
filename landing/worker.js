import largeAssets from './large-assets.json';

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const asset = Object.hasOwn(largeAssets, url.pathname) ? largeAssets[url.pathname] : undefined;
    if (!asset) return env.ASSETS.fetch(request);
    if (request.method !== 'GET' && request.method !== 'HEAD') {
      return new Response(null, { status: 405, headers: { Allow: 'GET, HEAD' } });
    }

    const headers = new Headers({
      'Content-Type': asset.type,
      'Cache-Control': 'no-cache',
    });
    const fetchPart = part => env.ASSETS.fetch(new URL(part, url));
    if (asset.gzip) {
      headers.set('Vary', 'Accept-Encoding');
      const encodings = (request.headers.get('Accept-Encoding') ?? '').split(',');
      const acceptsGzip = encodings.some(encoding => {
        const [name, quality] = encoding.toLowerCase().trim().split(/\s*;\s*/, 2);
        return name === 'gzip' && (quality === undefined || Number(quality.slice(2)) > 0);
      });
      if (acceptsGzip) {
        const compressed = await fetchPart(asset.gzip);
        if (!compressed.ok) return new Response('Simulator asset is unavailable.', { status: 502 });
        headers.set('Content-Encoding', 'gzip');
        if (request.method === 'HEAD') await compressed.body?.cancel();
        return new Response(request.method === 'HEAD' ? null : compressed.body, { headers, encodeBody: 'manual' });
      }
    }

    if (request.method === 'HEAD') {
      headers.set('Content-Length', String(asset.bytes));
      return new Response(null, { headers });
    }

    const { readable, writable } = new FixedLengthStream(asset.bytes);
    ctx.waitUntil((async () => {
      for (const part of asset.parts) {
        const chunk = await fetchPart(part);
        if (!chunk.ok) throw new Error(`Missing simulator asset part ${part}.`);
        await chunk.body.pipeTo(writable, { preventClose: true });
      }
      await writable.close();
    })().catch(error => writable.abort(error)));
    return new Response(readable, { headers });
  },
};
