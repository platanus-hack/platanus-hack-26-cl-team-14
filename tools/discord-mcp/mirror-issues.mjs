import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { homedir } from "node:os";
import { join } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { issueActivity } from "./issue-activity.mjs";
import { discord } from "./rest.mjs";

const parent = "1573760053905723443"; // #github
const issueBase = "https://github.com/platanus-hack/platanus-hack-26-cl-team-14/issues/";
const minute = 60_000;
const directory = join(homedir(), ".local", "state", "team14-discord-issue-mirror");
const file = join(directory, "state.json");
const lock = createServer(socket => socket.destroy());
await new Promise((resolve, reject) => lock.once("error", reject).listen(43872, "127.0.0.1", resolve));

mkdirSync(directory, { recursive: true, mode: 0o700 });
const start = Math.floor(Date.now() / 1000) * 1000;
const state = existsSync(file) ? JSON.parse(readFileSync(file, "utf8")) : {
  start, parent, cursor: start, seen: {}, threads: {}, batches: [],
};
function save() {
  writeFileSync(file + ".next", JSON.stringify(state), { mode: 0o600 });
  renameSync(file + ".next", file);
}
// Older state files belong to #agent-chat. Resolve queued events again from GitHub
// so pending excerpts are replaced by full text in the new destination.
if (state.parent !== parent) {
  for (const batch of state.batches) {
    for (const key of batch.keys) {
      state.cursor = Math.min(state.cursor, state.seen[key] ?? state.start);
      delete state.seen[key];
    }
  }
  state.parent = parent;
  state.threads = {};
  state.batches = [];
  delete state.sending;
}
save();

function redact(value) {
  return value
    .replace(/https?:\/\/(?:[\w.-]+\.)?discord(?:app)?\.com\/api\/(?:v\d+\/)?webhooks\/\S+|(?:mfa\.[\w-]+|[\w-]{20,}\.[\w-]{5,}\.[\w-]{20,})|(?:gh[pousr]_|github_pat_)[\w]+|sk-[a-zA-Z0-9_-]{16,}/gi, "[redactado]")
    .replace(/(\b(?:[a-z_]*(?:TOKEN|PASSWORD|SECRET|API_KEY|PROXY_URL|BASE_URL)|HTTPS?_PROXY)\s*=\s*)(?:"[^"\n]*"|'[^'\n]*'|[^\s`]+)/gi, "$1[redactado]")
    .replace(/(\bAuthorization:\s*(?:Bearer|Bot|Basic)\s+)\S+/gi, "$1[redactado]");
}

function* messageParts(event) {
  const labels = { opened: "abrió", commented: "comentó", closed: "cerró" };
  const header = `${event.author} ${labels[event.kind]} #${event.number}`;
  const footer = `\n<${event.url}>`;
  const body = redact(event.kind === "opened" ? `${event.title}\n${event.body}` : event.body);
  if (header.length + 1 + body.length + footer.length <= 2000) {
    yield header + "\n" + body + footer;
    return;
  }
  let offset = 0;
  for (let part = 1; offset < body.length; part++) {
    const prefix = `${header} · parte ${part}\n`;
    let end = Math.min(body.length, offset + 2000 - prefix.length - footer.length);
    if (end < body.length && /[\uD800-\uDBFF]/.test(body[end - 1])) end--;
    yield prefix + body.slice(offset, end) + footer;
    offset = end;
  }
}

function makeBatches(events) {
  const groups = new Map();
  for (const event of events) {
    if (state.seen[event.key]) continue;
    const group = groups.get(event.number) ?? [];
    groups.set(event.number, group);
    const title = redact(`#${event.number} ${event.title}`).replace(/\s+/g, " ").slice(0, 100);
    let part = 0;
    for (const content of messageParts(event)) {
      let batch = group.at(-1);
      if (!batch || batch.content.length + 2 + content.length > 2000) {
        batch = { number: event.number, title, content: "", keys: [] };
        group.push(batch);
      }
      batch.content += (batch.content ? "\n\n" : "") + content;
      batch.keys.push(`${event.key}:${++part}`);
    }
    state.seen[event.key] = Date.parse(event.at);
  }
  return [...groups.values()].flat();
}

// After an uncertain send, check Discord history before repeating the saved batch.
async function send(batch, channel, content, key) {
  const nonce = createHash("sha256").update(key).digest("hex").slice(0, 24);
  if (batch.sending) {
    let after = batch.sending;
    for (;;) {
      const page = await discord(`/channels/${channel}/messages?after=${after}&limit=100`);
      const found = page.find(m => m.author.id === botId && m.content === content);
      if (found) return found.id;
      if (page.length < 100) break;
      after = page.reduce((id, m) => BigInt(m.id) > BigInt(id) ? m.id : id, after);
    }
  } else {
    batch.sending = ((BigInt(Date.now() - 60_000) - 1420070400000n) << 22n).toString();
    save();
  }
  const message = await discord(`/channels/${channel}/messages`, {
    method: "POST", json: { content, flags: 4096 | 4, allowed_mentions: { parse: [] }, nonce, enforce_nonce: true },
  });
  return message.id;
}

async function deliver(batch) {
  let thread = state.threads[batch.number];
  if (!thread) {
    const anchor = await send(batch, parent, `GitHub · <${issueBase}${batch.number}>`, `issue:${batch.number}`);
    thread = state.threads[batch.number] = { id: anchor, ready: false };
    batch.sending = null;
    save();
  }
  if (!thread.ready) {
    try {
      await discord(`/channels/${parent}/messages/${thread.id}/threads`, { method: "POST", json: { name: batch.title } });
    } catch (error) {
      if (error.discordCode !== 160004) throw error; // This starter message already has its thread.
    }
    thread.ready = true;
    save();
  }
  await send(batch, thread.id, batch.content, batch.keys.join(","));
  state.batches.splice(state.batches.indexOf(batch), 1);
  save();
}

let botId;
for (;;) {
  try {
    botId ??= (await discord("/users/@me")).id;
    const since = Math.max(state.start, state.cursor - minute);
    const until = Date.now();
    const events = await issueActivity(new Date(since).toISOString(), new Date(until).toISOString());
    state.seen = Object.fromEntries(Object.entries(state.seen).filter(([, at]) => at >= since));
    state.batches.push(...makeBatches(events));
    state.cursor = until;
    save();

    const blocked = new Set();
    for (const batch of [...state.batches]) {
      if (blocked.has(batch.number)) continue;
      try {
        await deliver(batch);
      } catch (error) {
        if (!error.discordCode) throw error;
        blocked.add(batch.number);
        console.error(`Issue #${batch.number} will retry: Discord ${error.discordCode}`);
      }
      await sleep(1000);
    }
  } catch (error) {
    // gh errors can include command arguments. Only log Discord's sanitized status or the exit code.
    console.error("Issue mirror will retry:", error.discordCode ?? error.code ?? error.name);
  }
  await sleep(minute);
}
