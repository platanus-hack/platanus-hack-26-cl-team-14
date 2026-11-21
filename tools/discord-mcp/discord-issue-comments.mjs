import { execFile } from "node:child_process";
import { setTimeout as sleep } from "node:timers/promises";

const issues = "repos/platanus-hack/platanus-hack-26-cl-team-14/issues";
const markerPattern = /\r?\n\r?\n<!-- team14-discord-message:([1-9]\d{0,19})(?::[1-9]\d*)? -->$/;
const commentLimit = 65_536;

export function discordMessageId(body) {
  return body.match(markerPattern)?.[1];
}

function ghApi(args, input) {
  return new Promise((resolve, reject) => {
    const child = execFile("gh", ["api", ...args], (error, stdout) => {
      if (error) reject(new Error(`GitHub API request failed (${error.code}).`));
      else resolve(stdout.trim());
    });
    child.stdin.on("error", reject);
    child.stdin.end(input);
  });
}

let login;
export async function githubLogin() {
  login ??= await ghApi(["user", "--jq", ".login"]);
  return login;
}

export async function ensureIssueComment(number, messageId, author, content, recovering) {
  const endpoint = `${issues}/${number}/comments`;
  const source = `<!-- team14-discord-message:${messageId}`;
  const previous = new Map();
  if (recovering) {
    const writer = JSON.stringify(await githubLogin());
    const suffix = JSON.stringify(`\n\n${source}(:[1-9][0-9]*)? -->$`);
    const filter = `.[] | select(.user.login == ${writer}) | (.body | gsub("\\r\\n"; "\\n")) as $body
      | select($body | test(${suffix})) | [($body | split("\\n")[-1]), .node_id] | @tsv`;
    const matches = await ghApi([endpoint + "?per_page=100", "--paginate", "--jq", filter]);
    for (const row of matches.split("\n")) {
      const [marker, nodeId] = row.split("\t");
      if (nodeId) previous.set(marker, nodeId);
    }
  }
  const name = author.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
  const shortHeader = `<code>${name}</code> (Discord)\n\n`;
  const multipart = shortHeader.length + content.length + `\n\n${source} -->`.length > commentLimit;
  const nodeIds = [];
  let offset = 0;
  do {
    const part = nodeIds.length + 1;
    const header = multipart ? `<code>${name}</code> (Discord, parte ${part})\n\n` : shortHeader;
    const marker = multipart ? `${source}:${part} -->` : `${source} -->`;
    const footer = "\n\n" + marker;
    let end = Math.min(content.length, offset + commentLimit - header.length - footer.length);
    if (end < content.length && /[\uD800-\uDBFF]/.test(content[end - 1])) end--;
    let nodeId = previous.get(marker);
    if (!nodeId) {
      if (part > 1) await sleep(1000);
      const body = header + content.slice(offset, end) + footer;
      nodeId = await ghApi([endpoint, "--method", "POST", "--input", "-", "--jq", ".node_id"], JSON.stringify({ body }));
    }
    nodeIds.push(nodeId);
    offset = end;
  } while (offset < content.length);
  return nodeIds;
}
