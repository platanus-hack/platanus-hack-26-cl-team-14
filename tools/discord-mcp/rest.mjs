import { existsSync } from "node:fs";

const ENV_PATH = new URL("../../.env", import.meta.url);
const API_URL = "https://discord.com/api/v10";
const MAX_ATTEMPTS = 3;
export const ASK_FOR_TOKEN = "Put DISCORD_BOT_TOKEN=<token> in the .env file at the repo root.";

export function savedToken() {
  if (existsSync(ENV_PATH)) process.loadEnvFile(ENV_PATH);
  return process.env.DISCORD_BOT_TOKEN;
}

function routeName(method, path) {
  return method + " " + path.split("?")[0]
    .replace(/(\/webhooks\/\d+)\/[^/]+/, "$1/:token")
    .replace(/\/\d+(?=\/|$)/g, "/:id")
    .replace(/(\/reactions\/)[^/]+/, "$1:emoji");
}

const routeCounts = new Map();
function recordCall(route, response, body) {
  if (!process.env.DISCORD_REST_METRICS) return;
  const count = (routeCounts.get(route) ?? 0) + 1;
  routeCounts.set(route, count);
  console.error(JSON.stringify({
    time: Date.now(), route, count, status: response.status,
    ...(response.status === 429 && {
      bucket: response.headers.get("x-ratelimit-bucket"), retry_after: body.retry_after ?? response.headers.get("retry-after"),
    }),
  }));
}

const cooldowns = new Map();
const buckets = new Map();
const wait = ms => new Promise(resolve => setTimeout(resolve, ms));

export async function discord(path, { auth = true, token = auth ? savedToken() : undefined, json, ...request } = {}) {
  if (auth && !token) throw new Error(ASK_FOR_TOKEN);
  const route = routeName(request.method ?? "GET", path);
  const major = path.match(/^\/(?:channels|guilds|webhooks)\/(\d+)/)?.[1] ?? "";
  const global = auth ? "bot" : "webhooks";
  const key = () => `${buckets.get(route) ?? route}/${major}`;
  for (let attempt = 1; attempt <= MAX_ATTEMPTS; attempt++) {
    let delay;
    while ((delay = Math.max(cooldowns.get(global) ?? 0, cooldowns.get(key()) ?? 0) - Date.now()) > 0) await wait(delay);
    const response = await fetch(`${API_URL}${path}`, {
      ...request,
      ...(json && { body: JSON.stringify(json) }),
      headers: {
        ...(token && { Authorization: `Bot ${token}` }),
        "User-Agent": "DiscordBot (https://github.com, 1.0)",
        ...(json && { "Content-Type": "application/json" }),
      },
    });
    const body = response.status === 204 ? {} : await response.json().catch(() => ({}));
    recordCall(route, response, body);
    const bucket = response.headers.get("x-ratelimit-bucket");
    if (bucket) buckets.set(route, bucket);
    const pause = (scope, seconds) => {
      const until = Date.now() + Math.ceil(seconds * 1000);
      cooldowns.set(scope, Math.max(until, cooldowns.get(scope) ?? 0));
    };
    if (response.headers.get("x-ratelimit-remaining") === "0") {
      pause(key(), Number(response.headers.get("x-ratelimit-reset-after")) || 0);
    }
    if (response.ok) return body;
    if (response.status === 429) {
      const seconds = Math.max(Number(body.retry_after) || 0, Number(response.headers.get("retry-after")) || 0) || 1;
      pause(body.global || response.headers.has("x-ratelimit-global") ? global : key(), seconds);
      if (attempt === MAX_ATTEMPTS) throw new Error(`Discord limited ${route} after ${attempt} attempts. Retry after ${seconds} seconds.`);
      continue;
    }
    if (response.status === 401) throw new Error(`Discord rejected the token. ${ASK_FOR_TOKEN}`);
    throw Object.assign(new Error(`Discord answered ${response.status}: ${body.message ?? "no detail"}`), { discordCode: body.code });
  }
}
