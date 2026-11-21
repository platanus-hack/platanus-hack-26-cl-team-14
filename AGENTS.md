# AGENTS.md

Rules for each agent in this repo (Claude Code, Codex, OpenCode, and others).
Read this file first. Then read `context/README.md`.

## Required before any work

You cannot work without these two things. If one is missing, stop and tell your human.

1. **Ponytail is on.** At the start of each session, load the skill `ponytail`
   (`.agents/skills/ponytail/`) and keep it on for the full session.
2. **Discord access.** The `discord` MCP server works: `list_channels` shows
   `#general`. Setup: `tools/discord-mcp/README.md`.

## Mission

We are in Platanus Hack 26 (Santiago), track **Simulations**.
We win with the pitch. The pitch is 3 minutes or less.
The pitch tells a short, strong, emotional story with one surprising moment.

The order of work is:

1. Choose the idea (the problem and the simulation).
2. Write the pitch. Build a full mock of the demo frontend.
3. Build backwards only what the demo needs.

We want 90% of the pitch ready on day one.

**How the simulation gets deep.** Our coding agents make the simulation deeper
during the hackathon, one layer per task. Each layer adds one mechanism or one
real data source, and must improve a fixed score against real data.
The demo shows the finished, deep simulation. The demo does NOT show AI agents
that improve the model live. 

## The golden rule

**If the demo does not show it, and nothing in the demo needs it, do not build it.**

- Make the simulation deeper. Do not add features.
- Before you start a task, find the moment in `context/pitch.md` that needs it.
  If no moment needs it, do not do the task.
- While `context/pitch.md` is empty, work only on: research for the problem,
  the mock, and the pitch.

## Work with minimum human input

Humans give input on only two things:

- **Taste:** what looks good and what goes in the pitch.
- **Direction:** if the product and the pitch go the correct way.

You decide all other things: tasks, technical design, code, data, tools.

- Do not ask humans technical questions. Decide, and write your reason
  in the pull request or in `context/decisions/`.
- Create, split, and take tasks yourself. Coordinate with other agents yourself.
- Ask humans only for taste or direction, and only in a release or at a checkpoint.

## Work rules

- **No tests.** It is a hackathon. All the effort goes into the product.
  Before a release, run the demo by hand and make sure that it works.
- **No defensive code.** Write only the happy path that the demo uses.
  Do not add fallbacks, retries, validation, or error handling for cases
  that the demo does not show.
- **Orchestrator and subagents.** In each session, the main model plans and coordinates.
  It gives the work of each task to subagents.
- **Smallest code.** Ponytail is mandatory on every task.
  Use `ponytail-review` to review a change. Our rules win over ponytail: do not write tests or self-checks, and do not
  add validation or error handling that the demo does not need.
- **Research first.** Before you build, look for papers, libraries,
  open source code, and datasets (for example, open georeferenced data).
  Use what exists. Use the skill `research-first`.
- **Simple English.** Write all messages, pull requests, commits, and reports in
  Simplified Technical English. Use the skill `writing-simplified-technical-english`.
  Be short. Say only what the reader needs. No intro, no summary, no filler.
- **Frontend for a projector.** Use very little text. Do not use small text.
  Use large shapes, strong contrast, and clear motion.
- **Real data, not fake data.** The mock shows the look and the story.
  It is not the final product. If real data does not exist, tell the team
  in Discord and propose the nearest real option.

## Coordination

All work goes through Discord. Use the `discord` MCP tools.
Your messages show your agent name, for example `quiet-heron (v4rgas@thinkpad)`.

Use the `Agents` category for agent coordination. Use `#agent-chat` threads for work
and decisions. Use `#agent-status` for usage. Create ordinary channels there when needed.
Read `#general` for human direction. Start an ordinary thread for each topic.
Read the thread before work. Post what you take, progress, decisions, and the PR there.
Use plain messages. There are no forum tasks or state tags.
Agents do not read or post in `#secrets`.
Use `#resultados` (category `Agents`) only to show finished results to the humans.
Do not use it for progress, questions between agents, or drafts. See "Results for humans".

- Call `poll_messages` at work checkpoints and at least once per minute while working.
  Read all unread messages, including messages from other agents.
  Polling runs in the MCP server, but clients may not wake an idle agent.
- Call `report_usage` in `#agent-status` at start, checkpoints, and before compaction.
  Give measured context tokens and their source. Omit unavailable measurements.
  Include durable memory paths and handoff notes. Never invent a context percentage.
  Do not answer usage reports with another usage report.
- If a human gives you a decision in your session, record it in `context/decisions/`.
- `main` is the shared branch. Do one task on one branch. Open small
  pull requests to `main`. Pull `main` frequently.
- If many agents run on one machine, use one git worktree and one dev server
  port for each agent.

## Releases

A release is a **version of the presentation**, not of the code.
It shows the pitch with the current state of the product.

Post each release in its topic thread in `#agent-chat` in this format:

```
Release vN: <one line, what changed in the story>
How to see it: <link or one command>
Screenshots: 1 to 3, only the key moments
Changed: 3 short lines or fewer
Question for humans: <one taste or direction question, or "none">
```

Do not post many screenshots or long logs.
Also post each release in `#resultados`, in the Spanish format below.

## Results for humans (`#resultados`)

Humans read only `#resultados`. They do not read agent threads.
Post there only when a result is final: you finished it, you checked it
yourself, and another agent reviewed it. Post each result one time.

Before you post:
- The work is merged in `main`, or it runs with one command.
- You opened the files, the data, or the screen that you show.
- You removed what is not verified, or you marked it "sin verificar".

Write the message in Spanish, simple and direct, for a human who did not
follow the thread:

```
**<Qué logramos, en una línea>**
Qué es: <2 o 3 líneas, sin jerga>
Cómo revisarlo: <link al PR, archivo o comando>
Evidencia: <screenshot si hay UI; si no, 1 dato concreto verificado>
Falta o no sabemos: <1 o 2 líneas, o "nada">
Pregunta para ustedes: <una decisión de gusto o dirección, o "ninguna">
```

- If there is UI, attach 1 to 3 screenshots. No logs.
- Humans answer in a thread on the message. Read that thread and act on it.
- Do not post in `#resultados` to answer another agent.

## Checkpoints

At each checkpoint, humans review the last release and change the direction
if necessary. See `context/checkpoints.md`.

## Shared context

All durable context is in `context/`. `context/README.md` tells you how to
read it and how to add to it.
Skills are in `.agents/skills/`. `.claude/skills` points to the same folder.
