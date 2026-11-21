# Swarm playbook from the first iteration

- Status: Accepted
- Date: 2026-11-21
- Source: Juan’s instructions in the agent session, 2026-11-21, about 17:45 to 18:00 Chile time

## Decision

`context/discord-interaction.md` is one playbook for the agent swarm in Discord. Its first section is the fast answer to humans.
All coordination occurs in Discord threads, one per topic. The team does not use GitHub issues.
Each session has Discord notifications on and at least 4 subagents ready. One overviewer watches the full swarm. The command `/enjambre` starts a session in this mode.

## Why

Juan asked to improve the swarm orchestration docs from what worked well in the first 18 hours, and to insist on what the users liked, such as the fast Discord polling. He then said:

- "todos tenemos que tener las notificacion de discord y AL MENOS 4 agentes ready to go y tomar tareas que se comentan en el discord"
- "mataron las issues de github toda la comunicacion van a ocurrir en discord y escopearse por hilos separados por temas que se requiran"
- "vamos a crear un comando aca para claude y codex que sea /enjambre que de este contexto para tener los 4 agentes atentos al discord y operar en nuestra software factory"
- "uno tiene que tomar el rol del overviewer y ir viendo que los agents no se traben entre si, que las cosas se vayan moviendo etc /research-first mejores practicas de esto que se defina como sea pero tiene que ser un fable, opus o astra"

The old file repeated itself. `AGENTS.md` asked for a poll once per minute, which the humans found too slow.
The overviewer role uses three sources: the lead and worker pattern ([Anthropic](https://www.anthropic.com/engineering/multi-agent-research-system)), active management of items in progress ([Kanban Guide, May 2025](https://kanbanguides.org/english/)), and one command with a transfer briefing ([NIMS 2017, pages 22–23](https://www.fema.gov/sites/default/files/2020-07/fema_nims_doctrine-2017.pdf)). The ten-minute sweep and the takeover after two missed sweeps are our choice.

## What it means for agents

- Run `/enjambre` (Codex: `$enjambre`) at the start of a session. The text is in `.agents/skills/enjambre/SKILL.md`.
- Run the inbox and the three-second watcher on each laptop. Do not poll once per minute.
- Track work in one topic thread in `#agent-chat`. Do not open GitHub issues. Pull requests and reviews on the exact commit stay.
- Keep at least 4 subagents ready. This replaces "4 subagents or fewer" in `20261121-0235-data-goal-and-agent-budget.md`. The other points of that decision stay. A ready slot with no task uses no quota. Report usage as before.
- One session is the overviewer. Its model is Fable, Opus, or Astra. It writes no product code.
- The results channel is `#results`. Older files call it `#resultados`.
- The playbook does not list spokesperson names. Each session announces them in `#agent-chat`.
