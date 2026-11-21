import { discord } from "./rest.mjs";

const snowflake = /^[1-9]\d{0,19}$/;
const replyHeader = /^-# ↪ <@\d+> https:\/\/discord\.com\/channels\/\d+\/(\d+)\/\d+$/;

// The repo webhook moves long text to the last attachment and leaves only its reply header inline.
function textFile(threadId, message) {
  const inline = message.content ?? "";
  if (inline && inline.match(replyHeader)?.[1] !== threadId) return;
  const file = message.attachments?.at(-1);
  if (file?.filename !== "message.txt" || file.content_type?.split(";")[0] !== "text/plain") return;
  if (!snowflake.test(file.id) || file.size + (inline ? inline.length + 1 : 0) <= 2000) return;
  if (!URL.canParse(file.url)) return;
  const url = new URL(file.url);
  if (url.origin !== "https://cdn.discordapp.com" || url.username || url.password ||
      url.pathname !== `/attachments/${threadId}/${file.id}/message.txt`) return;
  return file;
}

export async function issueReplyText(threadId, message) {
  if (!snowflake.test(threadId) || !snowflake.test(message.id)) return message.content ?? "";
  let file = textFile(threadId, message);
  if (!file || (!message.webhook_id && !message.author?.bot)) return message.content ?? "";
  async function refresh() {
    try {
      message = await discord(`/channels/${threadId}/messages/${message.id}`);
    } catch (error) {
      if (error.discordCode !== 10008) throw error;
      message = {}; // The message was deleted before this poll.
    }
    file = message.webhook_id && textFile(threadId, message);
  }
  // Older local inboxes omit webhook_id. Read it from the API before downloading anything.
  if (!message.webhook_id) await refresh();
  if (!file) return message.content ?? "";
  const options = { redirect: "error", credentials: "omit" };
  let response = await fetch(file.url, options);
  if ([401, 403, 404, 410].includes(response.status)) {
    await response.body?.cancel();
    await refresh();
    if (!file) return message.content ?? "";
    response = await fetch(file.url, options);
  }
  if (!response.ok) throw new Error(`Discord attachment download returned HTTP ${response.status}.`);
  const text = new TextDecoder("utf-8", { ignoreBOM: true }).decode(await response.arrayBuffer());
  const inline = message.content ?? "";
  const full = inline ? inline + "\n" + text : text;
  return full.length > 2000 ? full : inline;
}
