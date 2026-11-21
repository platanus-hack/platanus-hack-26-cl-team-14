# Discord MCP server

This server connects the agents (Claude Code, Codex, OpenCode) to the team Discord.
It is one file, `server.mjs`. It has no dependencies. It needs Node 22 or later and the `gh` CLI.

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
| `report_usage` | Share measured context usage, remaining tokens, and memory notes in agent-status. |

Each message shows the agent name, for example `quiet-heron (v4rgas@thinkpad)`.
The name is random for each agent session. The GitHub user comes from `gh`. The machine name comes from the hostname.
The bot posts through one webhook for each channel, named `agents`. Discord shows the tag `APP` on these messages.

## Setup (one person, one time)

1. Go to <https://discord.com/developers/applications>. Click **New Application**.
2. Open **Bot**. Click **Reset Token** and copy the token. Keep it secret. Do not put it in the repo.
3. In **Bot**, turn on these two privileged intents:
   - **Message Content Intent** (to read messages)
   - **Server Members Intent** (for `find_member`)
4. Open this URL. Replace `APP_ID` with the Application ID from **General Information**. Select the team server.

   ```
   https://discord.com/oauth2/authorize?client_id=APP_ID&scope=bot&permissions=326954503248
   ```

   The permissions are: View Channels, Send Messages, Send Messages in Threads, Create Public Threads,
   Read Message History, Embed Links, Attach Files, Add Reactions, Manage Webhooks, Manage Threads, Manage Channels.
5. Create the `Agents` category with `agent-chat` and `agent-status` text channels.
   Use ordinary threads in `agent-chat` for work. Keep `general` for human direction.
6. Send the token to the team in a private way (not in Discord, not in the repo).

## Setup (each person, on each laptop)

1. Make sure that `node --version` shows 22 or later, and that `gh auth status` shows your GitHub user.
2. Put the token in the file `.env` at the repo root: `DISCORD_BOT_TOKEN=<token>`. Git ignores this file. The server reads it each time.
3. Start your agent from the root of the repo:
   - **Claude Code:** approve the `discord` server when Claude Code asks.
   - **Codex:** trust the project when Codex asks. Codex reads `.codex/config.toml` only in trusted projects.
   - **OpenCode:** no step.
4. Tell your agent: "list the Discord channels". It must show `#tasks`, `#releases`, `#general`.

## Run by hand

```
node tools/discord-mcp/server.mjs
```

The server speaks MCP on stdin and stdout. Usually the agent tool starts it.


## Notifications and usage

Restart the agent's Discord MCP connection after updating this server.
Each connection polls all text channels under `Agents`, plus `general`, every 15 seconds.
It discovers new channels and their public threads on each poll.
It includes threads archived between polls. It never polls `secrets`.
New messages arrive in an inbox. Each tool result includes up to 50 unread messages.
Call `poll_messages` again if more remain. All authors are included, including bots.
The MCP client also receives a log notification with the unread count.
Clients decide whether to show it. This cannot guarantee that an idle agent wakes.
Follow the continuous listening rule in `AGENTS.md`, including while waiting after a task ends.
Keep the connection and session active. Poll at least once per minute, even when the inbox is empty.

The inbox starts when the server starts. It is held in memory and resets on restart.
Read the topic history when starting or resuming work. Polling reports new messages,
not edits or deletions. A message deleted before the poll cannot be recovered.

Call `report_usage` with the `agent-status` `channel_id`, `source`, and `memory` notes.
Add `context_used` and `context_limit` only when the host provides real measurements.
The tool calculates remaining tokens. Missing measurements display `unknown`.
The MCP server cannot inspect the host's context or extra memory automatically.
Report at start, checkpoints, and before compaction. Never reply to a usage report
with another usage report. Memory notes should name saved context and handoff files.
