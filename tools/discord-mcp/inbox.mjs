// One Gateway writer per laptop. MCP readers use the same bot's private temp directory.
import { createHash } from "node:crypto";
import { appendFileSync, mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync, statSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";

const heartbeatMs = 5000;
let directory;
export function inboxDirectory(token) {
  const key = createHash("sha256").update(token).digest("hex").slice(0, 20);
  directory = join(tmpdir(), `team-discord-${process.getuid()}-${key}`);
}

export function snapshot() {
  if (!directory) return;
  try {
    if (Date.now() - statSync(join(directory, "alive")).mtimeMs > 3 * heartbeatMs) return;
    return JSON.parse(readFileSync(join(directory, "channels.json"), "utf8"));
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
}

const thread = (c) => [10, 11, 12].includes(c.type);
export function watched(guild) {
  const categories = new Set(guild.channels.filter(c => c.type === 4 && c.name === "Agents").map(c => c.id));
  const roots = guild.channels.filter(c => [0, 5, 15].includes(c.type) && c.name !== "secrets"
    && (c.name === "general" || categories.has(c.parent_id)));
  const ids = new Set(roots.map(c => c.id));
  return [...roots, ...guild.threads.filter(c => c.type !== 12 && c.name !== "secrets" && ids.has(c.parent_id))];
}

const snowflakeAt = (ms) => BigInt(ms - 1420070400000) << 22n;
const pages = new Map();
let pageSession;
function messagesIn(state, channel) {
  if (pageSession !== state.session) {
    pageSession = state.session;
    pages.clear();
  }
  const file = join(state.session, channel.id + ".jsonl");
  const size = statSync(file, { throwIfNoEntry: false })?.size ?? 0;
  const cached = pages.get(channel.id);
  if (cached?.size === size) return cached.messages;
  const rows = new Map();
  if (size) {
    const data = readFileSync(file, "utf8");
    for (const line of data.slice(0, data.lastIndexOf("\n")).split("\n")) {
      if (!line) continue;
      const [type, m] = JSON.parse(line);
      if (type === "MESSAGE_CREATE") rows.set(m.id, m);
      if (type === "MESSAGE_UPDATE" && rows.has(m.id)) Object.assign(rows.get(m.id), m);
      if (type === "MESSAGE_DELETE") rows.delete(m.id);
      if (type === "MESSAGE_DELETE_BULK") for (const id of m.ids) rows.delete(id);
    }
  }
  const messages = [...rows.values()].sort((a, b) => a.id.length - b.id.length || a.id.localeCompare(b.id));
  pages.set(channel.id, { size, messages });
  return messages;
}

// Undefined means Discord has older history that could change this page.
export function localPage(state, channel, { before, after, around, limit }) {
  const messages = messagesIn(state, channel);
  const first = channel.last_message_id ? BigInt(channel.last_message_id) : snowflakeAt(state.since);
  if (after) {
    if (BigInt(after) < first) return;
    return messages.filter(m => BigInt(m.id) > BigInt(after)).slice(0, limit);
  }
  if (around) {
    const pos = messages.findIndex(m => m.id === around);
    if (pos < Math.floor(limit / 2)) return;
    const start = pos - Math.floor(limit / 2);
    return messages.slice(start, start + limit);
  }
  const page = before ? messages.filter(m => BigInt(m.id) < BigInt(before)) : messages;
  if (page.length >= limit) return page.slice(-limit);
}

export function newMessages(state, channel, cursor) {
  return messagesIn(state, channel).filter(m => BigInt(m.id) > BigInt(cursor));
}

function channelInfo(c) {
  return { id: c.id, name: c.name, type: c.type, parent_id: c.parent_id,
    last_message_id: c.last_message_id, archived: c.thread_metadata?.archived,
    applied_tags: c.applied_tags, available_tags: c.available_tags?.map(({ id, name }) => ({ id, name })) };
}

function scrub(text) {
  return text.replace(/[\w-]{18,}\.[\w-]{4,}\.[\w-]{15,}/g, "[redacted]")
    .replace(/https?:\/\/(?:[\w-]+\.)?discord(?:app)?\.com\/api\/(?:v\d+\/)?webhooks\/\d+\/[^\s"'<>]+/gi, "[redacted webhook]");
}

export async function startInbox(token, discord) {
  // The OS releases the listening port when the writer exits, also after a crash.
  const lock = createServer(socket => socket.destroy());
  await new Promise((resolve, reject) => {
    lock.once("error", reject).listen(43871, "127.0.0.1", resolve);
  }).catch(() => { throw new Error("Cannot start inbox: local port 43871 is in use."); });
  mkdirSync(directory, { recursive: true, mode: 0o700 });
  const alive = join(directory, "alive");
  const offline = () => rmSync(alive, { force: true });
  offline();
  const { url } = await discord("/gateway/bot");
  function connect() {
    // Heartbeat and Identify follow the listener in PR 110.
    const socket = new WebSocket(`${url}/?v=10&encoding=json`);
    const guilds = new Map();
    const subscribed = new Set();
    const session = mkdtempSync(join(directory, "messages-"));
    let expected = [];
    let since;
    let sequence = null;
    let acknowledged = true;
    let firstBeat, beats, touch;
    const send = (op, d) => socket.send(JSON.stringify({ op, d }));
    function publish() {
      subscribed.clear();
      for (const guild of guilds.values()) for (const c of watched(guild)) subscribed.add(c.id);
      if (!expected.length || !expected.every(id => guilds.has(id))) {
        offline();
        return;
      }
      const next = join(directory, "channels.next");
      writeFileSync(next, JSON.stringify({ session, since, guilds: [...guilds.values()] }), { mode: 0o600 });
      renameSync(next, join(directory, "channels.json"));
      writeFileSync(alive, "", { mode: 0o600 });
    }
    const heartbeat = () => {
      if (!acknowledged) return socket.close(4000);
      acknowledged = false;
      send(1, sequence);
    };
    socket.addEventListener("message", ({ data }) => {
      const { op, s, t, d } = JSON.parse(data);
      if (s != null) sequence = s;
      if (op === 10) {
        firstBeat = setTimeout(() => {
          heartbeat();
          beats = setInterval(heartbeat, d.heartbeat_interval);
        }, Math.random() * d.heartbeat_interval);
        send(2, { token, intents: 1 | 512 | 32768, properties: {
          os: process.platform, browser: "team-inbox", device: "team-inbox",
        } });
      }
      if (op === 11) acknowledged = true;
      if (op === 1) send(1, sequence);
      if (op === 7 || op === 9) socket.close(4000);
      if (t === "READY") {
        expected = d.guilds.map(g => g.id);
        console.error("Discord accepted the Gateway connection with Message Content Intent.");
        touch = setInterval(() => {
          if (expected.length && expected.every(id => guilds.has(id))) writeFileSync(alive, "", { mode: 0o600 });
        }, heartbeatMs);
      }
      if (t === "GUILD_CREATE") {
        since = Date.now();
        guilds.set(d.id, { id: d.id, name: d.name, channels: d.channels.map(channelInfo), threads: d.threads.map(channelInfo) });
        publish();
      }
      if (t === "GUILD_DELETE") {
        guilds.delete(d.id);
        if (!d.unavailable) expected = expected.filter(id => id !== d.id);
        publish();
      }
      const guild = guilds.get(d?.guild_id);
      if (!guild) return;
      if (/^(CHANNEL|THREAD)_(CREATE|UPDATE|DELETE)$/.test(t)) {
        const key = thread(d) || t.startsWith("THREAD_") ? "threads" : "channels";
        const existing = guild[key].find(c => c.id === d.id);
        guild[key] = guild[key].filter(c => c.id !== d.id);
        if (!t.endsWith("_DELETE")) guild[key].push({ ...channelInfo(d), last_message_id: existing ? existing.last_message_id : d.last_message_id });
        publish();
      }
      if (t === "THREAD_LIST_SYNC") {
        const parents = d.channel_ids && new Set(d.channel_ids);
        guild.threads = guild.threads.filter(c => c.archived || (parents && !parents.has(c.parent_id)));
        guild.threads.push(...d.threads.map(channelInfo));
        publish();
      }
      if (subscribed.has(d.channel_id) && ["MESSAGE_CREATE", "MESSAGE_UPDATE", "MESSAGE_DELETE", "MESSAGE_DELETE_BULK"].includes(t)) {
        const { id, ids, author, content, timestamp, embeds, attachments, message_reference } = d;
        const row = JSON.stringify([t, { id, ids, author, content, timestamp, embeds, attachments, message_reference }],
          (_, value) => typeof value === "string" ? scrub(value) : value);
        appendFileSync(join(session, d.channel_id + ".jsonl"), row + "\n", { mode: 0o600 });
      }
    });
    socket.addEventListener("error", () => console.error("Discord Gateway connection error."));
    socket.addEventListener("close", ({ code }) => {
      clearTimeout(firstBeat); clearInterval(beats); clearInterval(touch);
      offline();
      console.error(`Discord Gateway closed (${code}).`);
      if ([4004, 4010, 4011, 4012, 4013, 4014].includes(code)) process.exit(1);
      setTimeout(connect, 5000);
    });
  }
  connect();
}
