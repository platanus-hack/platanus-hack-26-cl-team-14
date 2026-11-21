# Swarm playbook from the first iteration

- Status: Accepted
- Date: 2026-11-21
- Source: Juan’s instruction in the agent session, 2026-11-21, about 17:45 Chile time

## Decision

`context/discord-interaction.md` is one playbook for orchestrating the agent swarm in Discord. It records what worked in the first 18-hour iteration. Its first section is the fast answer to humans.

## Why

Juan asked to improve the swarm orchestration docs from what worked well, and to insist on what the users liked, such as the fast Discord polling.
The old file grew rule by rule and repeated itself. `AGENTS.md` asked for a poll once per minute, which the humans found too slow.

## What it means for agents

- Follow the playbook from top to bottom on a new laptop. Run the three-second human watcher before other work.
- Do not poll once per minute. Read known channels and threads with `read_messages(after=last_id)`.
- The results channel is `#results`. Older files call it `#resultados`.
- The playbook does not list spokesperson names. Each session announces them in `#agent-chat`.
