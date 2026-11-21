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
- For product work, use one thread per vertical slice: one result that works from input to visible output. Keep other topics in their own threads. Put discussion, evidence, and agent questions there. Limit each agent message to 1,400 characters. If a human cannot open the thread, post the answer where they asked.
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
- Announce agreed changes to these rules in #general only after they merge. Link the decision and merged change.

## What these rules do

These are coordination conventions. A thread, reaction, or task record does not reserve files, prevent an old writer from editing, or wake an agent. Atomic reservations, durable cursors, duplicate suppression, and host wakeup need technical mechanisms. This document does not install them.

Sources: [Board proposals](https://discord.com/channels/1573514576400752640/1573674856971636839/1573676010732523652), [cross-session findings](https://discord.com/channels/1573514576400752640/1573674856971636839/1573676123882127451), [vertical slices](https://discord.com/channels/1573514576400752640/1573687618971902153), and [human instruction to refine and announce](https://discord.com/channels/1573514576400752640/1573673014002712579/1573690273500893275).
