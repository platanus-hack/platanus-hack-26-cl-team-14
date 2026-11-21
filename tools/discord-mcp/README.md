# Discord MCP server

This server connects the agents (Claude Code, Codex, OpenCode) to the team Discord.
The MCP server and its local inbox have no package dependencies. It needs Node 22 or later and the `gh` CLI.

The repo loads it for all three tools: `.mcp.json` (Claude Code), `.codex/config.toml` (Codex), `opencode.json` (OpenCode).

## What agents can do

| Tool | What it does |
| --- | --- |
| `list_channels` | List channels, forums, and active threads, with IDs. |
| `read_messages` | Read a channel or a thread. |
| `send_message` | Post in a channel, or reply in a thread (use the thread ID). Can attach files. |
| `edit_message`, `add_reaction`, `create_thread`, `find_member` | Edit, react, start a thread, find a person to mention. |
| `create_channel` | Create a text channel or a forum (with tags). |
| `poll_messages` | Drain unread messages from Agents channels, general, and their public threads. |
| `report_usage` | Share measured 5-hour and weekly account quotas, reset times, and memory notes in agent-status. |

Each message shows the agent name, for example `Clorinda-47 (v4rgas@thinkpad)`.
Each agent session keeps one name and number. The names come from [Chilean Civil Registry data for 1940 to 1975](https://rivaquiroga.github.io/guaguas/). The GitHub user comes from `gh`. The machine name comes from the hostname.
The bot posts through one webhook for each channel, named `agents`. Discord shows the tag `APP` on these messages.

### Local name and quiet replies

The MCP reads `discord-agent-names.json` from the system temp directory at startup. Use the parent PID of the MCP server as the name key. List numeric parent PIDs in `quiet` for agents that read unread messages on demand. For example:

```json
{"12345": "Lautaro", "quiet": [12345]}
```

Reconnect that agent’s Discord MCP after editing the file. A quiet agent can call `poll_messages` to collect unread messages. Its other tool replies omit the unread list.

## Create or replace the team bot

1. Go to <https://discord.com/developers/applications>. Click **New Application**, enter a name, and click **Create**.
2. Open **Bot**. Click **Reset Token** and copy the token into your private secret manager.
3. In **Bot**, turn on these two privileged intents:
   - **Message Content Intent** (to read messages)
   - **Server Members Intent** (for `find_member`)

   Click **Save Changes**.
4. Open this URL. Replace `APP_ID` with the Application ID from **General Information**. Select the team server and click **Authorize**. Your Discord account needs Manage Server.

   ```
   https://discord.com/oauth2/authorize?client_id=APP_ID&scope=bot&permissions=326954503248
   ```

   The permissions are: View Channels, Send Messages, Send Messages in Threads, Create Public Threads,
   Read Message History, Embed Links, Attach Files, Add Reactions, Manage Webhooks, Manage Threads, Manage Channels.
5. Create the `Agents` category with `agent-chat` and `agent-status` text channels.
   Use ordinary threads in `agent-chat` for work. Keep `general` for human direction.
6. Share the token with the laptop owners through your private secret manager. If a token appears in Discord, reset it in the portal and delete the message.

## Setup (each person, on each laptop)

1. Make sure that `node --version` shows 22 or later, and that `gh auth status` shows your GitHub user.
2. Set `DISCORD_BOT_TOKEN` in the ignored `.env` at the repo root. Each laptop uses the same bot.
3. Start your agent from the root of the repo:
   - **Claude Code:** approve the `discord` server when Claude Code asks.
   - **Codex:** trust the project when Codex asks. Codex reads `.codex/config.toml` only in trusted projects.
   - **OpenCode:** no step.
4. Start the shared inbox in a terminal and leave it running. Run this command once per laptop.

   ```sh
   node tools/discord-mcp/server.mjs --inbox
   ```

5. After updating the Discord MCP code, restart the shared inbox and reconnect each agent’s Discord MCP server. Stop the old inbox before starting its replacement. Running processes keep the code they loaded at startup.

## Run by hand

```
node tools/discord-mcp/server.mjs
```

The server speaks MCP on stdin and stdout. Usually the agent tool starts it.


## Notifications and usage

The inbox connects to the Discord Gateway and stores messages in private JSONL files in the system temp directory. It watches the Agents channels, `general`, and their public threads. It excludes `secrets`. Port 43871 on loopback reserves one writer on this laptop.

In Juan’s session, a Paseo heartbeat wakes the orchestrator once per minute. The orchestrator reads the local inbox files, keeps one last-seen message ID for the entire mailbox, and uses Discord MCP to post receipt reactions. The [Discord interaction rules](../../context/discord-interaction.md#patterns-for-an-orchestrator) call for a three-second inbox check and a separate last-seen ID for each channel or thread. The current heartbeat falls short of both targets.

Each MCP client keeps its own unread cursor. It checks local files every 15 seconds and returns up to 50 unread messages per tool call. Call `poll_messages` to drain the rest. The client also sends an MCP log notification. The host decides when to wake an idle agent.

Channel listings and recent reads use the local files. `read_messages` preserves `before`, `after`, `around`, and `limit`. The client uses REST if the selected page reaches beyond local history. After a disconnect, the inbox asks Discord to replay the missing events. Clients allow 12 seconds for recovery before they use REST. If Discord rejects the session, the inbox starts a new local history window.

Set `DISCORD_REST_METRICS=1` when starting a process to write one JSON line per REST response to stderr. Each line includes the route, cumulative route count and status. A 429 line also includes the bucket and retry delay. The client waits for Discord’s delay and stops a request after three attempts.

Discord’s Developer Policy says, “You agree to, and will not attempt to circumvent, such limitations we set for each API.” Exceeding a limit requires Discord’s express written consent. This tool uses a single bot identity. [Developer Policy](https://support-dev.discord.com/hc/en-us/articles/8563934450327-Discord-Developer-Policy) · [Rate limits](https://docs.discord.com/developers/topics/rate-limits) · [Gateway](https://docs.discord.com/developers/events/gateway)

Call `report_usage` with the `agent-status` `channel_id` and `memory` notes.
Codex agents add `provider: "codex"`. The server starts a short-lived local
`codex app-server`, reads `account/rateLimits/read`, and closes it. It uses the
existing Codex login. It does not start a model turn or read credentials itself.
The report covers that local account's `codex` quota bucket, shared across its sessions.
It identifies 5-hour and weekly windows by durations of 300 and 10080 minutes.
Primary/secondary order does not identify the duration. Missing windows display `unavailable`.

Other clients use `provider: "manual"` and supply `quota_windows` from their host:
`[{"window_minutes":300,"used_percent":25,"resets_at":1795212000}]`.
This is a schema example, not a measurement. Include `measured_at` (ISO timestamp)
and `source` (provider, account/quota scope, and measurement source).
Omit missing windows or reset times. Without host measurements, omit the array and
state why in `source`. The server does not collect Claude or OpenCode quotas automatically.

Reports show percent used, percent remaining, observation time, and UTC reset times.
An unavailable 5-hour window does not mean 0% used or unlimited usage.
Context tokens and cumulative token counts cannot substitute for quota measurements.
Report at start, checkpoints, and before compaction. Never answer a usage report
with another usage report. Memory notes name saved context and handoff files.
