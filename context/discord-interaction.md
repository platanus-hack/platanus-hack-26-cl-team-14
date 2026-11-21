# Discord swarm playbook

A new orchestrator follows this file from top to bottom. The most important section for the humans is first ([decision](decisions/20261121-1745-swarm-playbook-from-first-iteration.md)). Announce a change to these rules in #agent-chat only after it merges.

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

Name one receipt owner and one backup per session. If a receipt is late or the listener fails, tell the humans in #agent-chat: which span of messages is pending and who takes reception.

**If the inbox stalls.** The inbox touches the file `alive` in `$TMPDIR/team-discord-<uid>-<key>/` every five seconds. When that file is older than 15 seconds, each reader uses REST, which is rate-limited. On each heartbeat, check the age of `alive`. If it is stale, stop the old inbox, start one inbox again (port 43871 permits one writer per laptop), restart the watcher, and confirm that a new human message arrives. The watcher continues from its saved IDs, thus no message is lost. [Source](https://discord.com/channels/1573514576400752640/1573561159955648553/1573795537587798067).

## 2. How to talk

- **One spokesperson per computer.** Name one spokesperson and one backup per session. Announce both names in the main #agent-chat channel. Agent names change on restart: check existing names before you choose one. Present your session and role one time. If you replace an agent, say whom, and from which message and commit you resume.
- **One answer per human question, across all computers.** Only the spokesperson of the asking human’s computer answers. Other agents send their data to that spokesperson in a thread. The first spokesperson to claim a request for all sessions answers it. If no answer comes in 3 minutes, another spokesperson can write "tomo por X" and answer. Source: [Mac, 16:39 UTC](https://discord.com/channels/1573514576400752640/1573561159955648553/1573739676236251157).
- **Read before you reply.** Read the full message and the message it replies to. If an attachment is missing, ask the sender to resend it. Open each image or link before you give an opinion. For a repository, read its README. Check recent replies.
- **Reply to humans in plain Spanish.** Reply to the human’s message. Start with their name and the answer. Use at most 1,200 characters. State what is done, what remains, and what you need. Ask at most one question. Answer a #general request in #agent-chat and link the request. If a human cannot open the thread, answer where they asked.
- **Agent messages.** Write Simplified Technical English, within 1,400 characters, with the full commit SHA. If a peer must act, address it by name and say what it must do. Agent messages are silent. Only a direct answer to a human request can notify the requester ([decision](decisions/20261121-1727-silent-discord-messages.md)).
- **Main channel and threads.** Use Discord for all communication between computers. An orchestrator and its subagents can talk locally. In the main #agent-chat channel, post claims, handoffs, blockers, shared decisions, and one line with a link for each thread. Every 30 minutes, report what merged, what runs, what is blocked, and what you need from humans. Put discussion and evidence in the thread. Post a finished, checked, and independently reviewed result one time in #results, in the AGENTS.md format.
- **Verify before you claim.** Run the work or open its evidence yourself. Give the source and unit for each figure. A second agent challenges each figure before publication. Report limits in the same message. Mark each unverified claim. Attribute each check to the agent that made it: "Rosa reported two black captures", not "I opened the captures". Correct an error in the same channel and edit the original message.

## 3. How to organise work

- **One thread per topic is the record.** GitHub issues are not used ([decision](decisions/20261121-1745-swarm-playbook-from-first-iteration.md)). Each piece of work has one thread in #agent-chat. The first post states the goal, a "Done when" line, the owner, the current result, and its limits. Claims, progress, evidence, and the PR link go in that thread. Start the thread on the human’s original message. For a #general request, start it in #agent-chat and link the original. Read the thread before you start, and reuse it.
- **Message to order in one minute.** Post the assignment in the work thread. Quote the request, its author, and its time. Name one owner, the deliverable, and the evidence to post: a command, a full commit SHA, an inspected screenshot, or a measurement. Codex agents obey their first message literally.
- **Order change.** Quote the exact instruction it replaces. Say whom it affects, what stops, and what continues. If two human orders conflict, pause the affected part and ask the humans one question.
- **Interface contract.** The producer and the consumer write it together: base revision, input, output, units, identifiers, the rule for absent data, one example, and one case that the two can run. Keep it in the thread and in the file that consumers read.
- **One writer per file.** If two sessions claim the same work, the later claimant withdraws in the thread and names the first owner. For a handoff, record the new owner, scope, last message handled, branch, SHA, state of local changes, and pending proof. The previous writer confirms they stopped before the new writer edits. If that writer does not answer, escalate to the overviewer and wait for an explicit release.
- **Roles.** Name a coordinator, an integrator (the only agent that merges), a critic (challenges each figure), a runner (runs the product end to end and takes captures), and one owner per piece. A proposed name is pending until that agent accepts.
- **Review.** Deliver the PR, full commit SHA, and evidence in the thread. Before merge, get a review from another session on that exact commit. The reviewer posts the SHA and a verdict on the PR: Ship, Fix first, or Hold. For Fix first or Hold, give the defect, the case that shows it, and the condition to lift it. State what you did: a source review, a simulated test, and a real run prove different things. A changed commit needs a new verdict. The owner answers each challenge with a change or with evidence.
- **Close.** The integrator confirms the approved head before merge. If "Done when" needs a run or a visual proof, a merge does not close the work. Keep the thread open until that evidence is posted.
- **Blocked or resumed.** When blocked, link the dependency and name who can resolve it. When you resume, read from the last message handled, and report that message in a handoff.

## 4. The overviewer

- The swarm has exactly one overviewer at a time. It runs on a Fable, Opus, or Astra model. It announces itself in the main #agent-chat channel. It writes no product code.
- Every ten minutes, it reads each open thread. It looks for: no owner, no progress since the last sweep, a blocker, two agents on the same file or task, a PR that waits for review, and a human who waits for an answer. It unblocks in the thread: it names an owner, or it moves the work to another agent. It asks humans only about taste or direction.
- Before its session ends, it posts a briefing in #agent-chat: each open thread with its owner and blocker, and the name of the next overviewer. If the overviewer misses two sweeps, another session on one of those models writes "tomo overview" and continues.

Basis: the lead coordinates and the workers do the work ([Anthropic](https://www.anthropic.com/engineering/multi-agent-research-system)); manage items in progress actively and unblock them ([Kanban Guide](https://kanbanguides.org/english/)); one command, and a briefing to all when it transfers ([NIMS, pages 22–23](https://www.fema.gov/sites/default/files/2020-07/fema_nims_doctrine-2017.pdf)).

## 5. Keep the swarm ready and cheap

- Each laptop runs the inbox and the watcher of section 1. There are no exceptions.
- Each session keeps at least 4 subagents ready to take tasks from Discord threads. Ready means four named slots, each with a role. When a task appears, the orchestrator starts the agent for a free slot (Claude Code starts a subagent per task; Paseo uses `create_agent`) or sends the task to an idle Paseo agent with `send_agent_prompt`. A slot with no task runs no turns and uses no quota. Call `report_usage` as AGENTS.md says.
- Do not put end times, deadlines, or time estimates in agent prompts. Order work by sequence and condition. The first message to a subagent gives the human’s words, what to read first, its files, the evidence to post, its own worktree, and its first action. Tell it to read its thread every 60 to 90 seconds between steps and to continue after an empty read.
- Assign by confirmed capacity. Ask who can compile, open the window, use the GPU, write, or integrate. Check disk, battery, quota, and permission before costly work. If a laptop has no capacity, move the run to a session that accepts it. Write "cuota desconocida" when no quota measurement exists.
- Before you start another worker, name the independent deliverable that it unblocks.
- If another computer stops, tell its owner the time of its last message and the effect on the work.
- Use one viewer or capture slot per laptop. The holder releases the slot by name in the thread. Other agents queue.

## 6. Discord limits and secrets

- Read known channels and threads with `read_messages(after=last_id)`. Save the last ID for each channel or thread. Do not call `list_channels` or `poll_messages` in a loop.
- Read the service’s rules before you propose a way around a limit. Follow those rules. Do not rotate bots or tokens to pass a rate limit: Discord’s [Developer Policy](https://support-dev.discord.com/hc/en-us/articles/8563934450327-Discord-Developer-Policy) prohibits it. Before you use a person’s account, personal files, or browser, describe the exact scope and get permission. A local permission is valid only for the scope and the session that received it. Record the confirmation without credentials or personal data.
- Keep credentials, proxy addresses, and `.env` files out of Discord. If a credential appears, its owner resets it and the message is deleted. Agents do not read or post in #secrets.

## 7. Examples

Adapt these. Replace each placeholder before you send.

- **Take a human request:** "Ana, tomo el arranque de la maqueta. El motor ya responde. Falta comprobar que la ventana abra con la misma revisión. Le pedí a Diego la prueba en Linux. Dejo el caso y el comando en [hilo]."
- **Assign a worker:** "Diego, take the clean startup check at `<full SHA>`. Run `./run.sh` from the root. Post the command, the engine revision and a screenshot you opened. Julia keeps `run.sh`. Reply here when you have the GPU slot."
- **Handoff:** "Esto reemplaza «Julia escribe el lanzador». Julia entregó su rama en `<full SHA>` y confirmó que paró. Diego toma solo `run.sh` desde ese commit. La escena sigue con Marta."
- **Conflicting orders:** "Ana, el panel se puede hacer. El pedido de hoy choca con la pausa de desarrollo que dejó Luis en [mensaje]. El panel queda en espera y seguimos con las revisiones abiertas. ¿El panel queda autorizado como excepción?"
- **Verdicts:** "Fix first at `<full SHA>`, source review. The viewer opens before the base result exists. Start it after the engine publishes that result." Then: "Ship at `<new full SHA>` from the Linux session. I ran a clean launch and opened the first frame. I checked startup only."

## 8. Failures to avoid

- Agent prompts had end times. Eight agents stopped at the same time.
- A swarm of 23 agents had no owner per piece and used too much of the account limits.
- Loops of `list_channels` and `poll_messages` got HTTP 429 on all computers, because the computers share one bot.
- Agents talked in #general. Juan sent them to #agent-chat.

## What these rules do

These are coordination conventions. A thread, reaction, or task record does not reserve files, prevent an old writer from editing, or wake an agent. Atomic reservations, durable cursors, duplicate suppression, and host wakeup need technical mechanisms. This document does not install them.

Sources: [Board proposals](https://discord.com/channels/1573514576400752640/1573674856971636839/1573676010732523652), [cross-session findings](https://discord.com/channels/1573514576400752640/1573674856971636839/1573676123882127451), [vertical slices](https://discord.com/channels/1573514576400752640/1573687618971902153), and [human instruction to refine and announce](https://discord.com/channels/1573514576400752640/1573673014002712579/1573690273500893275).

Orchestrator patterns: [monitoring and replies](https://discord.com/channels/1573514576400752640/1573718111574564865/1573718231980703745), [assignments and threads](https://discord.com/channels/1573514576400752640/1573718111574564865/1573718263546912939), [verification and access](https://discord.com/channels/1573514576400752640/1573718111574564865/1573718301060890705), [subagent prompts, roles, and Paseo](https://discord.com/channels/1573514576400752640/1573703632115990640/1573703930423021732), and [rate limits](https://discord.com/channels/1573514576400752640/1573710243765162045). Historical record of Emerson's session, with the full examples: [communication guide](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/blob/fcba6efde975f2465c7aa0cb2de08da11f70fad4/context/temporal/guia-comunicacion-agentes-emerson.md).
