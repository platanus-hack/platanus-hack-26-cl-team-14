---
name: enjambre
description: >
  Join the team's agent swarm in Discord: start the fast human watcher, announce
  the session, keep 4 subagents ready, and take tasks from Discord threads. Use
  when the user says "/enjambre", "$enjambre", "enjambre", or tells the session
  to join the swarm.
---

# Enjambre

The team works as a swarm: one orchestrator session per laptop, coordinated in Discord.
This command makes your session a working member of that swarm.
Read `AGENTS.md` first. Then read `context/discord-interaction.md` (the playbook). If this file and the playbook disagree, the playbook is correct.

Do these steps in order.

1. Apply the hackathon work and review rules in `AGENTS.md` to the session and its workers.
2. Make sure that Discord works on this laptop, as section 1 of the playbook says:
   - Call `list_channels` one time. Save the IDs of `#agent-chat`, `#agent-status`, and `#results`. Do not call it again.
   - Start the inbox in the background: `node tools/discord-mcp/server.mjs --inbox`. The error "local port 43871 is in use" means that an inbox already runs. That is correct.
   - Run `watch-humans.mjs` under your background monitor, with the receipt emoji of this computer. If your host has no background monitor, tell your human.
3. Read the last messages of `#agent-chat` with `read_messages`. Find the spokesperson names in use and the current overviewer.
4. Post one message in the main `#agent-chat` channel, in Spanish: your name, your human, your spokesperson and backup, your receipt emoji, and your four ready slots with their roles.
5. In the same message, state the overviewer. Take the role only if your model is Fable, Opus, or Astra and no other session holds it. Then follow section 4 of the playbook.
6. Keep 4 subagent slots ready, as section 5 of the playbook says. Take tasks that appear in Discord threads. Start or wake an agent only when a slot has a task.
   - As orchestrator, coordinate each task and delegate its execution to a worker.
   - Workers inspect files, logs, and detailed evidence in their own context. They return only the result, a link or path to the evidence, and any blockers to the orchestrator.
   - Keep all required orchestrator checks. Follow the playbook's verification before claims and its review requirements before merge.
7. Call `report_usage` in `#agent-status`.
8. Continue to listen. Re-arm the monitor when it ends. An empty read is not a reason to stop. Stop only when a human tells you.
