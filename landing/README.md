# Deploy Santiaguito

The Cloudflare Worker `santiaguito-landing` serves the landing page and the browser simulator on `santiaguito.cl`. Use the account that owns that Worker.

From the repository root, build the browser files and upload a Worker version.

```sh
sh tools/wasm/build.sh
npx wrangler versions upload --config landing/wrangler.jsonc
```

Use the version ID from the upload to switch traffic to that version. This keeps the existing custom-domain binding in place.

```sh
npx wrangler versions deploy --config landing/wrangler.jsonc VERSION_ID@100 --yes
```

On the shared demo laptop, run each command through the build lock. Set `PORT=8961` for the lock.

Wrangler runs `pack-assets.py`. It puts the landing and the public simulator dependencies in `landing/dist`. The script splits files above Cloudflare's 25 MiB static asset limit. The Worker streams the parts at the original URL. It serves the precompressed viewer Wasm to clients that accept gzip.

After deployment, open the landing, the replay gallery, and one replay. Confirm that the city opens with the saved experiment.
