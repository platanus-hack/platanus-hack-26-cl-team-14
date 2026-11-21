# Discord MCP server

This server connects the agents (Claude Code, Codex, OpenCode) to the team Discord.
It is one file, `server.mjs`. It has no dependencies. It needs Node 18 or later and the `gh` CLI.

The repo loads it for all three tools: `.mcp.json` (Claude Code), `.codex/config.toml` (Codex), `opencode.json` (OpenCode).

## What agents can do

| Tool | What it does |
| --- | --- |
| `list_channels` | List channels, forums, and active threads, with IDs. Task threads show their state tag. |
| `read_messages` | Read a channel or a thread. |
| `send_message` | Post in a channel, or reply in a thread (use the thread ID). Can attach files. |
| `edit_message`, `add_reaction`, `create_thread`, `find_member` | Edit, react, start a thread, find a person to mention. |
| `create_task` | Create a post in the `tasks` forum with the tag `open`. |
| `update_task` | Set the state `open`, `claimed`, `blocked`, or `done`, and post a note. `done` closes the thread. |
| `set_token` | Save the bot token on this machine. |

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
   https://discord.com/oauth2/authorize?client_id=APP_ID&scope=bot&permissions=326954503232
   ```

   The permissions are: View Channels, Send Messages, Send Messages in Threads, Create Public Threads,
   Read Message History, Embed Links, Attach Files, Add Reactions, Manage Webhooks, Manage Threads.
5. In the server, create these channels:
   - `general` (text)
   - `decisions` (text)
   - `releases` (text)
   - `tasks` (**forum**). In the forum settings, add four tags: `open`, `claimed`, `blocked`, `done`.
6. Send the token to the team in a private way (not in Discord, not in the repo).

## Setup (each person, on each laptop)

1. Make sure that `node --version` shows 18 or later, and that `gh auth status` shows your GitHub user.
2. Give the token to the server. Do one of these:
   - Put `export DISCORD_TOKEN=...` in your shell profile (`~/.zshrc` or `~/.bashrc`). Open a new terminal.
   - Or start your agent and paste the token. The agent saves it with `set_token` in `~/.config/hackathon-discord/token`.
3. Start your agent from the root of the repo:
   - **Claude Code:** approve the `discord` server when Claude Code asks.
   - **Codex:** trust the project when Codex asks. Codex reads `.codex/config.toml` only in trusted projects.
   - **OpenCode:** no step.
4. Tell your agent: "list the Discord channels". It must show `#tasks`, `#decisions`, `#releases`, `#general`.

## Run by hand

```
node tools/discord-mcp/server.mjs
```

The server speaks MCP on stdin and stdout. Usually the agent tool starts it.
