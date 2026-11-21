# Discord interaction

## Acknowledge and answer

- Each session reacts to human messages in #general, #agent-chat, #resultados, and their threads. React within five seconds of each message. Emerson's session uses 👀, Bruno's session uses 🤝, and Juan's session uses 🫡.
- Name one receipt owner and one backup per session. The reaction means only that the message arrived. The spokesperson reads the message and replies separately.
- A reply with an owner and next step means the message is being handled. A result that meets the closure criterion, with evidence, means the request is resolved.
- Name one spokesperson and one backup per session. One of them answers for that session. If a request is open to all sessions, the first spokesperson to claim it answers.
- Check message and receipt times. If a receipt is late or the listener fails, tell the humans in #agent-chat and name who will take over.

## Use the main channel and threads

- Use Discord for all communication between computers. On one computer, the orchestrator and its subagents can discuss a task their session already claimed. Keep those internal decisions local if desired.
- In the main #agent-chat channel, post claims, handoffs, decisions that affect other sessions, blockers, and short progress updates. For each topic, post a short summary with a direct link that shows humans where to find the thread. Say what review passed, what ran, and what is in main.
- For product work, use one thread per vertical slice: one result that works from input to visible output. Keep other topics in their own threads. Put discussion, evidence, and agent questions there. Limit each agent message to 1,400 characters. If a human cannot open the thread, answer where they asked. For a request from #general, answer in #agent-chat and link the request.
- Write messages for humans in plain Spanish. Use at most 1,200 characters. Give the answer or next action first. Keep required information in the message body.
- Post finished, checked, and independently reviewed results once in #resultados. Follow the release format in AGENTS.md.

## Divide and review work

- Before starting, read the slice thread. Announce its owner and link in the main channel. The thread records the outcome, scope, dependencies, next deliverable, and evidence needed to close it.
- Each slice team includes at least one subagent from each computer. Confirm their names and roles in the thread. Proposed names do not form a team until those agents accept. Check existing agent names before choosing a new one. Announce an identity change once.
- Name the coordinator, one writer per file, a reviewer from another session, and the integrator. If claims overlap, pause that piece until the sessions confirm one writer.
- Each next deliverable has an owner and evidence to inspect. When blocked, link the dependency and name who can resolve it. State what must happen before work resumes.
- When an instruction changes, the coordinator states which instruction it replaces before the writer continues. For a handoff, record the new owner and starting commit. The previous writer confirms they stopped before the new writer edits.
- Ask another session to challenge the work. Name the claim, give evidence, and propose a correction. The owner replies with the change or the evidence for keeping it.
- Deliver the PR, full commit SHA, and evidence in the slice thread. Before merging, get a review from another session on that exact commit. The reviewer posts the SHA, verdict, and blocking defects on the PR. A changed commit needs a new verdict.
- When resuming, read from the last message handled. Reuse the existing slice, delivery, and reply before starting another. Report the last message handled in a handoff.
- Announce agreed changes to these rules in #agent-chat only after they merge. Link the decision and merged change.

## Patterns for an orchestrator

- Wake the orchestrator on each new human message in #agent-chat or #general. If the monitor checks a local inbox, check it every three seconds. Use the assigned receipt emoji.
- Read known channels and threads with `read_messages(after=last_id)`. Save the last ID for each channel or thread. Skip `list_channels` and `poll_messages` for these reads.
- Every ten minutes, check the work threads, agent status, battery, and free disk space.
- Read the full message and the message it replies to. If an attachment seems missing, inspect the original. Ask the sender to resend it if it is missing.
- Open each relevant image or link before giving an opinion. For a repository, read its README.
- Check recent replies before you answer. Leave requests addressed to another session to that session.
- Reply to the human’s message in plain Spanish. Start with their name and the answer. For #general, link the message from #agent-chat.
- Keep the reply within 1,200 characters. State what is done, what remains, and what you need. If you need a decision, ask one question at the end.
- Within one minute of a request for work, reply and post the assignment in the work thread. Quote the request, its author, and its time.
- Name one owner, the deliverable, and the evidence to post. Give a command, a full commit SHA, an inspected screenshot, or a measurement as applicable.
- Assign one writer per file. If two sessions claim the same work, the later claimant withdraws in the thread and names the first owner.
- When an order changes, quote the exact instruction it replaces. Give the sequence and the condition for starting each task. Omit deadlines and time estimates.
- Reuse the topic thread. Start a new topic on the human’s original message. For a request in #general, start the thread in #agent-chat and link the original.
- In the first post, state the goal, completion criteria, current result, limits, and the name and role of an agent from each computer.
- Post one line in #agent-chat linking to the thread. Every 30 minutes, report what merged, what runs, what is blocked, and what you need from humans.
- Write agent messages in Simplified Technical English. Keep each within 1,400 characters. Give the full commit SHA when identifying a revision.
- Run the work or open its evidence yourself before saying it works. Give the source and unit for each measured figure. Ask a second agent to challenge each figure before publication.
- Report regressions and improvements with their limits in the same message. Mark each unverified claim. Correct errors in the same channel and edit the original message.
- Keep credentials, proxy addresses, and `.env` files out of Discord. If a credential appears, ask its owner to reset it and delete the message.
- Read the service’s rules before proposing a way around a limit. Follow those rules. Before using a person’s account, personal files, or browser, describe the exact scope and get permission.

## What these rules do

These are coordination conventions. A thread, reaction, or task record does not reserve files, prevent an old writer from editing, or wake an agent. Atomic reservations, durable cursors, duplicate suppression, and host wakeup need technical mechanisms. This document does not install them.

Sources: [Board proposals](https://discord.com/channels/1573514576400752640/1573674856971636839/1573676010732523652), [cross-session findings](https://discord.com/channels/1573514576400752640/1573674856971636839/1573676123882127451), [vertical slices](https://discord.com/channels/1573514576400752640/1573687618971902153), and [human instruction to refine and announce](https://discord.com/channels/1573514576400752640/1573673014002712579/1573690273500893275).

Orchestrator patterns: [monitoring and replies](https://discord.com/channels/1573514576400752640/1573718111574564865/1573718231980703745), [assignments and threads](https://discord.com/channels/1573514576400752640/1573718111574564865/1573718263546912939), and [verification and access](https://discord.com/channels/1573514576400752640/1573718111574564865/1573718301060890705).
