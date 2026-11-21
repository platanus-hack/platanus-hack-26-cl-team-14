import { execFile } from "node:child_process";

const issues = "repos/platanus-hack/platanus-hack-26-cl-team-14/issues";
const markerPattern = /\r?\n\r?\n<!-- team14-discord-message:([1-9]\d{0,19}) -->$/;

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
  const marker = `<!-- team14-discord-message:${messageId} -->`;
  if (recovering) {
    const writer = JSON.stringify(await githubLogin());
    const suffix = JSON.stringify("\n\n" + marker);
    const filter = `.[] | select(.user.login == ${writer} and (.body | gsub("\\r\\n"; "\\n") | endswith(${suffix}))) | .node_id`;
    const matches = await ghApi([endpoint + "?per_page=100", "--paginate", "--jq", filter]);
    if (matches) return matches.split("\n")[0];
  }
  const name = author.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
  const body = `<code>${name}</code> (Discord)\n\n${content}\n\n${marker}`;
  return ghApi([endpoint, "--method", "POST", "--input", "-", "--jq", ".node_id"], JSON.stringify({ body }));
}
