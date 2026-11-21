import { inboxDirectory, localPage, newMessages, snapshot, watchedRoots } from "./inbox.mjs";
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
    const guild = state?.guilds.find(g => watchedRoots(g.channels).some(c => c.id === parent));
    const local = guild?.threads.find(c => c.id === threadId);
    if (local) known.delete(threadId);
    const thread = local ?? known.get(threadId) ?? await discord(`/channels/${threadId}`);
    if (thread.parent_id !== parent || ![10, 11].includes(thread.type) || thread.name === "secrets") {
      throw new Error("The issue thread must be a public thread in #github.");
    }
    if (!local) known.set(threadId, thread);
    if (local) {
      const page = localPage(state, local, { after, limit: 100 });
      if (page) return page;
    }
    // Dormant archived threads can be absent from the initial Gateway snapshot.
    const observable = guild && (local || thread.thread_metadata?.archived);
    const previous = covered.get(threadId);
    if (observable && previous?.session === state.session
      && previous.baseline === local?.last_message_id && BigInt(after) >= previous.after) {
      return newMessages(state, thread, after).slice(0, 100);
    }
    const rows = await discord(`/channels/${threadId}/messages?after=${after}&limit=100`);
    const page = rows.filter(m => BigInt(m.id) > BigInt(after))
      .sort((a, b) => a.id.length - b.id.length || a.id.localeCompare(b.id));
    if (observable && rows.length < 100) {
      // Only use the cache once the caller has consumed every REST-only row.
      covered.set(threadId, { session: state.session, baseline: local?.last_message_id,
        after: BigInt(page.at(-1)?.id ?? after) });
    }
    return page;
  };
}
