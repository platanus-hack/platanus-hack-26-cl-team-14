import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { homedir } from "node:os";
import { join } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { issueActivity } from "./issue-activity.mjs";
import { discord } from "./rest.mjs";

const parent = "1573561159955648553"; // #agent-chat
const issueBase = "https://github.com/platanus-hack/platanus-hack-26-cl-team-14/issues/";
const minute = 60_000;
const directory = join(homedir(), ".local", "state", "team14-discord-issue-mirror");
const file = join(directory, "state.json");
const lock = createServer(socket => socket.destroy());
await new Promise((resolve, reject) => lock.once("error", reject).listen(43872, "127.0.0.1", resolve));

mkdirSync(directory, { recursive: true, mode: 0o700 });
const start = Date.now();
const state = existsSync(file) ? JSON.parse(readFileSync(file, "utf8")) : {
  start, cursor: start, seen: {}, threads: {}, batches: [], sending: null,
};
function save() {
  writeFileSync(file + ".next", JSON.stringify(state), { mode: 0o600 });
  renameSync(file + ".next", file);
}
save();

function excerpt(value, limit) {
  const clean = value.replace(/https?:\/\/(?:[\w.-]+\.)?discord(?:app)?\.com\/api\/(?:v\d+\/)?webhooks\/\S+|(?:mfa\.[\w-]+|[\w-]{20,}\.[\w-]{5,}\.[\w-]{20,})|(?:gh[pousr]_|github_pat_)[\w]+/gi, "[redactado]")
    .replace(/\s+/gu, " ").trim();
  return clean.length > limit ? clean.slice(0, limit - 1) + "…" : clean;
}

function makeBatches(events) {
  const groups = new Map();
  const labels = { opened: "abrió", commented: "comentó", closed: "cerró" };
  for (const event of events) {
    if (state.seen[event.key]) continue;
    const group = groups.get(event.number) ?? [];
    groups.set(event.number, group);
    const detail = excerpt(event.kind === "opened" ? event.title : event.body, 200);
    const line = `${event.author} ${labels[event.kind]} #${event.number}${detail ? " · " + detail : ""} <${event.url}>`;
    let batch = group.at(-1);
    if (!batch || batch.content.length + line.length + 1 > 2000) {
      batch = { number: event.number, title: excerpt(`#${event.number} ${event.title}`, 100), content: "", keys: [] };
      group.push(batch);
    }
    batch.content += (batch.content ? "\n" : "") + line;
    batch.keys.push(event.key);
    state.seen[event.key] = Date.parse(event.at);
  }
  return [...groups.values()].flat();
}

// After an uncertain send, check Discord history before repeating the saved batch.
async function send(channel, content, key) {
  const nonce = createHash("sha256").update(key).digest("hex").slice(0, 24);
  if (state.sending) {
    let after = state.sending;
    for (;;) {
      const page = await discord(`/channels/${channel}/messages?after=${after}&limit=100`);
      const found = page.find(m => m.author.id === botId && m.content === content);
      if (found) return found.id;
      if (page.length < 100) break;
      after = page.reduce((id, m) => BigInt(m.id) > BigInt(id) ? m.id : id, after);
    }
  } else {
    state.sending = ((BigInt(Date.now() - 60_000) - 1420070400000n) << 22n).toString();
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
    const anchor = await send(parent, `GitHub · <${issueBase}${batch.number}>`, `issue:${batch.number}`);
    thread = state.threads[batch.number] = { id: anchor, ready: false };
    state.sending = null;
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
  await send(thread.id, batch.content, batch.keys.join(","));
  state.batches.shift();
  state.sending = null;
  save();
}

let botId;
for (;;) {
  try {
    botId ??= (await discord("/users/@me")).id;
    if (!state.batches.length) {
      const since = Math.max(state.start, state.cursor - minute);
      const until = Date.now();
      const events = await issueActivity(new Date(since).toISOString(), new Date(until).toISOString());
      state.seen = Object.fromEntries(Object.entries(state.seen).filter(([, at]) => at >= since));
      state.batches = makeBatches(events);
      state.cursor = until;
      save();
    }
    while (state.batches.length) {
      await deliver(state.batches[0]);
      await sleep(1000);
    }
  } catch (error) {
    // gh errors can include command arguments. Only log Discord's sanitized status or the exit code.
    console.error("Issue mirror will retry:", error.discordCode ?? error.code ?? error.name);
  }
  await sleep(minute);
}
