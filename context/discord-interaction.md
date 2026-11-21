# Discord interaction

## Acknowledge and answer

- Each session reacts to human messages in #general, #agent-chat, #resultados, and their threads. React within five seconds of each message. Emerson's session uses 👀, Bruno's session uses 🤝, and Juan's session uses 🫡.
- Name one receipt owner and one backup per session. The reaction means only that the message arrived. The spokesperson reads the message and replies separately.
- Name one spokesperson and one backup per session. One of them answers for that session. If a request is open to all sessions, the first spokesperson to claim it answers.
- Check message and receipt times. If a receipt is late or the listener fails, tell the humans in #agent-chat and name who will take over.

## Use the main channel and threads

- Use Discord for all communication between computers. On one computer, the orchestrator and its subagents can discuss a task their session already claimed. Keep those internal decisions local if desired.
- In the main #agent-chat channel, post claims, handoffs, decisions that affect other sessions, blockers, and short progress updates. For each topic, post a short summary with a direct link that shows humans where to find the thread. Say what review passed, what ran, and what is in main.
- Use one thread per topic. Put technical discussion, evidence, and questions for agents there. Limit each agent message in a thread to 1,400 characters. If a human cannot open the thread, post the answer where they asked.
- Write messages for humans in plain Spanish. Use at most 1,200 characters. Give the answer or next action first. Keep required information in the message body.
- Post finished, checked, and independently reviewed results once in #resultados. Follow the release format in AGENTS.md.

## Divide and review work

- Before starting, read the topic thread. Announce the piece, owner, and thread in the main channel. Record the scope and dependencies in the thread.
- Give each piece one owner. If claims overlap, pause that piece until the sessions agree on the owner. Record the decision where both sessions can see it.
- Ask another session to challenge the work. Name the claim, give evidence, and propose a correction. The owner replies with the change or the evidence for keeping it.
- Before merging, get a review from another session on the pull request. The reviewer posts the exact commit, verdict, and each blocking defect. Review the new commit after a fix.
