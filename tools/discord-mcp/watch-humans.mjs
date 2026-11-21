// One receipt owner per computer. The agent's background monitor consumes stdout.
import { existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { inboxDirectory, localPage, snapshot, watched } from "./inbox.mjs";
import { ASK_FOR_TOKEN, discord, savedToken } from "./rest.mjs";

const [emoji, ...channels] = process.argv.slice(2);
if (!emoji || !channels.length || channels.some(c => !/^\d{1,20}(:\d{1,20})?$/.test(c))) {
  throw new Error("Usage: node tools/discord-mcp/watch-humans.mjs EMOJI CHANNEL_ID[:LAST_ID] ...");
}
const token = savedToken();
if (!token) throw new Error(ASK_FOR_TOKEN);
const directory = inboxDirectory(token);
mkdirSync(directory, { recursive: true, mode: 0o700 });
const reaction = encodeURIComponent(emoji);
const now = ((BigInt(Date.now()) - 1420070400000n) << 22n).toString();
const cursors = new Map();
function save(file, value) {
  writeFileSync(file + ".next", value, { mode: 0o600 });
  renameSync(file + ".next", file);
}

// A slow or denied reaction must not hold up the local message reader.
async function deliverReceipts(id, cursor) {
  if (cursor.delivering) return;
  cursor.delivering = true;
  try {
    for (const message of cursor.pending) {
      for (;;) {
        try {
          await discord(`/channels/${id}/messages/${message}/reactions/${reaction}/@me`, { method: "PUT" });
          break;
        } catch (error) {
          console.error(`Receipt failed for ${message}: ${error.message}`);
          if (error.discordCode === 10008) break; // Discord deleted the message.
          await sleep(60000);
        }
      }
      cursor.pending.delete(message);
      save(cursor.file + ".pending", JSON.stringify([...cursor.pending]));
    }
  } finally {
    cursor.delivering = false;
  }
}

for (const spec of channels) {
  const [id, initial = now] = spec.split(":");
  const file = join(directory, `watch-${reaction}-${id}.cursor`);
  const after = existsSync(file) ? readFileSync(file, "utf8").trim() : initial;
  if (!existsSync(file)) writeFileSync(file, after, { mode: 0o600 });
  const pending = existsSync(file + ".pending") ? JSON.parse(readFileSync(file + ".pending", "utf8")) : [];
  const cursor = { after, file, pending: new Set(pending), delivering: false };
  cursors.set(id, cursor);
  void deliverReceipts(id, cursor);
}

for (;;) {
  const state = await snapshot();
  const localChannels = state?.guilds.flatMap(watched) ?? [];
  await Promise.all([...cursors].map(async ([id, cursor]) => {
    const local = localChannels.find(c => c.id === id);
    try {
      let page;
      do {
        page = local && localPage(state, local, { after: cursor.after, limit: 20 });
        page ??= (await discord(`/channels/${id}/messages?after=${cursor.after}&limit=20`)).reverse();
        for (const message of page) {
          if (!message.author.bot && !message.webhook_id) {
            cursor.pending.add(message.id);
            save(cursor.file + ".pending", JSON.stringify([...cursor.pending]));
            console.log(`HUMAN ${id} ${message.id}`);
            void deliverReceipts(id, cursor);
          }
          save(cursor.file, message.id);
          cursor.after = message.id;
        }
      } while (page.length === 20);
    } catch (error) {
      console.error(`Channel ${id}: ${error.message}`);
    }
  }));
  await sleep(3000);
}
