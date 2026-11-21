import { inboxDirectory, newMessages, snapshot, watched } from "./inbox.mjs";
import { ASK_FOR_TOKEN, discord, savedToken } from "./rest.mjs";

const parent = "1573760053905723443"; // #github

export async function issueThreadReader() {
  const token = savedToken();
  if (!token) throw new Error(ASK_FOR_TOKEN);
  inboxDirectory(token);
  const known = new Map();
  const covered = new Map();

  return async (threadId, after) => {
    const state = await snapshot();
    const local = state?.guilds.flatMap(watched).find(c => c.id === threadId);
    if (local) known.delete(threadId);
    const thread = local ?? known.get(threadId) ?? await discord(`/channels/${threadId}`);
    if (thread.parent_id !== parent || ![10, 11].includes(thread.type) || thread.name === "secrets") {
      throw new Error("The issue thread must be a public thread in #github.");
    }
    if (!local) known.set(threadId, thread);
    // Older inbox writers build their subscription set with watched() at publish time.
    const active = local?.archived === false && local.subscribed !== false;
    if (!active) covered.delete(threadId);
    const previous = covered.get(threadId);
    if (active && previous?.session === state.session
      && previous.baseline === local.last_message_id && BigInt(after) >= previous.after) {
      return newMessages(state, local, after).slice(0, 100);
    }
    const rows = await discord(`/channels/${threadId}/messages?after=${after}&limit=100`);
    const page = rows.filter(m => BigInt(m.id) > BigInt(after))
      .sort((a, b) => a.id.length - b.id.length || a.id.localeCompare(b.id));
    if (active && rows.length < 100) {
      // Only use the cache once the caller has consumed every REST-only row.
      covered.set(threadId, { session: state.session, baseline: local.last_message_id,
        after: BigInt(page.at(-1)?.id ?? after) });
    }
    return page;
  };
}
