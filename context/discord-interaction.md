# Discord swarm playbook

A new orchestrator follows this file from top to bottom. The most important section for the humans is first ([decision](decisions/20261121-1745-swarm-playbook-from-first-iteration.md)).

## 1. Answer humans fast

Humans liked this most. Juan asked each session to copy it: ["tú respondes muy rápido"](https://discord.com/channels/1573514576400752640/1573718111574564865/1573718373521563720). A wake once per minute is too slow. Juan called the sessions that used it ["super lento"](https://discord.com/channels/1573514576400752640/1573718111574564865/1573722952308891699).

1. Start the shared inbox one time per laptop and leave it running: `node tools/discord-mcp/server.mjs --inbox`. See the [setup](../tools/discord-mcp/README.md).
2. Run the watcher from the repository root under the agent’s background monitor. Replace 👀 with the emoji of your computer: Emerson 👀, Bruno 🤝, Juan 🫡, Francisco 🥝. The two IDs select #agent-chat and #general.

   ```sh
   node tools/discord-mcp/watch-humans.mjs 👀 1573561159955648553 1573514577109581907
   ```

3. Configure the monitor to wake the orchestrator on each stdout line. With Claude Code, re-arm Monitor when its 30-minute session ends.
4. The watcher adds the receipt reaction within five seconds. The reaction means only that the message arrived.
5. Within one minute, the spokesperson replies with an owner and the next step. That reply means the message is being handled. A result with evidence means the request is resolved.
6. Run a separate ten-minute heartbeat. Use it only to check the work threads, the agents, the inbox process, free disk space, and battery. Replace each agent that stopped.

The watcher checks every three seconds. It reads messages after the saved ID for each channel, from local inbox files when they cover the page, otherwise through REST. It skips bots and webhooks. It keeps pending reactions across restarts and retries them. On first start, it watches from the current time. To start after a known message, pass `CHANNEL_ID:LAST_ID`. The watcher reads only the IDs in the command. It does not find threads. Add the ID of #results and of each thread where a human writes.

Name one receipt owner and one backup per session. If a receipt is late or the listener fails, tell the humans in #agent-chat and name who takes over.

## 2. How to talk

- **One spokesperson per computer.** Name one spokesperson and one backup per session. Announce both names in the main #agent-chat channel. Agent names change on restart: check existing names before you choose one, and announce a change one time.
- **One answer per human question, across all computers.** Only the spokesperson of the asking human’s computer answers. Other agents send their data to that spokesperson in a thread. The first spokesperson to claim a request for all sessions answers it. If no answer comes in 3 minutes, another spokesperson can write "tomo por X" and answer. Source: [Mac, 16:39 UTC](https://discord.com/channels/1573514576400752640/1573561159955648553/1573739676236251157).
- **Read before you reply.** Read the full message and the message it replies to. If an attachment is missing, ask the sender to resend it. Open each image or link before you give an opinion. For a repository, read its README. Check recent replies.
- **Reply to humans in plain Spanish.** Reply to the human’s message. Start with their name and the answer. Use at most 1,200 characters. State what is done, what remains, and what you need. Ask at most one question. Answer a #general request in #agent-chat and link the request. If a human cannot open the thread, answer where they asked.
- **Agent messages.** Write Simplified Technical English, within 1,400 characters, with the full commit SHA. Agent messages are silent. Only a direct answer to a human request can notify the requester ([decision](decisions/20261121-1727-silent-discord-messages.md)).
- **Main channel and threads.** Use Discord for all communication between computers. An orchestrator and its subagents can talk locally. In the main #agent-chat channel, post claims, handoffs, blockers, shared decisions, and one line with a link for each thread. Every 30 minutes, report what merged, what runs, what is blocked, and what you need from humans. Put discussion and evidence in the thread. Post a finished, checked, and independently reviewed result one time in #results, in the AGENTS.md format.
- **Verify before you claim.** Run the work or open its evidence yourself. Give the source and unit for each figure. A second agent challenges each figure before publication. Report limits in the same message. Mark each unverified claim. Correct an error in the same channel and edit the original message.

## 3. How to organise work

- **One GitHub issue is the record.** The title names the owner. The body has a "Done when" line. The [mirror](../tools/discord-mcp/README.md#sync-github-issues-and-discord-threads) copies each issue to a thread in #github and copies thread replies back.
- **Vertical slices.** Use one thread per slice: one result that works from input to visible output. Start the thread on the human’s original message. For a #general request, start it in #agent-chat and link the original. The first post states the goal, completion criteria, current result, limits, and one agent with its role from each computer. Read the thread before you start, and reuse it.
- **Message to order in one minute.** Post the assignment in the work thread. Quote the request, its author, and its time. Name one owner, the deliverable, and the evidence to post: a command, a full commit SHA, an inspected screenshot, or a measurement. When an order changes, quote the exact instruction it replaces. Codex agents obey their first message literally.
- **One writer per file.** If two sessions claim the same work, the later claimant withdraws in the thread and names the first owner. For a handoff, record the new owner and starting commit. The previous writer confirms they stopped before the new writer edits.
- **Roles.** Name a coordinator, an integrator (the only agent that merges), a critic (challenges each figure), a runner (runs the product end to end and takes captures), and one owner per piece.
- **Closer.** One agent owns the list of open issues and PRs. For each one it names the owner, the exact SHA, and the missing review or evidence. It closes an issue at an intermediate point only with the missing items written in it. It does not force a merge. With a closer, open issues went from 17 to 2 in two hours ([thread](https://discord.com/channels/1573514576400752640/1573781903075709020)).
- **Review.** Deliver the PR, full commit SHA, and evidence in the thread. Before merge, get a review from another session on that exact commit. The reviewer posts the SHA, verdict, and blocking defects on the PR. A changed commit needs a new verdict. The owner answers each challenge with a change or with evidence.
- **Blocked or resumed.** When blocked, link the dependency and name who can resolve it. When you resume, read from the last message handled, and report that message in a handoff.
- Announce agreed changes to these rules in #agent-chat only after they merge. Link the decision and merged change.

## 4. Keep the swarm alive and cheap

- Use 4 subagents or fewer per session ([decision](decisions/20261121-0235-data-goal-and-agent-budget.md)). Call `report_usage` as AGENTS.md says.
- Do not put end times, deadlines, or time estimates in agent prompts. Order work by sequence and condition.
- The first message to a subagent gives the human’s words, what to read first, its files, the evidence to post, its own worktree, and its first action. Tell it to read its thread every 60 to 90 seconds between steps and to continue after an empty read.
- Use one viewer or capture slot per laptop. The holder releases the slot by name in the thread. Other agents queue.

## 5. Discord limits and secrets

- Read known channels and threads with `read_messages(after=last_id)`. Save the last ID for each channel or thread. Do not call `list_channels` or `poll_messages` in a loop.
- Read the service’s rules before you propose a way around a limit. Follow those rules. Do not rotate bots or tokens to pass a rate limit: Discord’s [Developer Policy](https://support-dev.discord.com/hc/en-us/articles/8563934450327-Discord-Developer-Policy) prohibits it.
- Keep credentials, proxy addresses, and `.env` files out of Discord. If a credential appears, its owner resets it and the message is deleted. Agents do not read or post in #secrets.
- Before you use a person’s account, personal files, or browser, describe the exact scope and get permission.

## 6. Failures to avoid

- Agent prompts had end times. Eight agents stopped at the same time.
- A swarm of 23 agents had no owner per piece and used too much of the account limits.
- Many agents compiled on one laptop on battery. Disk and battery went down quickly.
- Loops of `list_channels` and `poll_messages` got HTTP 429 on all computers, because the computers share one bot.
- Agents talked in #general. Juan sent them to #agent-chat.

## What these rules do

These are coordination conventions. A thread, reaction, or task record does not reserve files, prevent an old writer from editing, or wake an agent. Atomic reservations, durable cursors, duplicate suppression, and host wakeup need technical mechanisms. This document does not install them.

Sources: [Board proposals](https://discord.com/channels/1573514576400752640/1573674856971636839/1573676010732523652), [cross-session findings](https://discord.com/channels/1573514576400752640/1573674856971636839/1573676123882127451), [vertical slices](https://discord.com/channels/1573514576400752640/1573687618971902153), and [human instruction to refine and announce](https://discord.com/channels/1573514576400752640/1573673014002712579/1573690273500893275).

Orchestrator patterns: [monitoring and replies](https://discord.com/channels/1573514576400752640/1573718111574564865/1573718231980703745), [assignments and threads](https://discord.com/channels/1573514576400752640/1573718111574564865/1573718263546912939), [verification and access](https://discord.com/channels/1573514576400752640/1573718111574564865/1573718301060890705), [subagent prompts and roles](https://discord.com/channels/1573514576400752640/1573703632115990640/1573703930423021732), and [rate limits](https://discord.com/channels/1573514576400752640/1573710243765162045).
