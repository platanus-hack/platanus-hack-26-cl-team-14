# Discord channel rules

- Status: Accepted
- Date: 2026-11-20
- Source: request from v4rgas, 2026-11-20

## Decision

Each channel has one use (table in `AGENTS.md`, section "Coordination").
All work and all discussion that ends in a choice is a task in the `#tasks` forum.
`#decisions` has final decisions only. `#releases` has releases only. `#general` is for humans.

## Why

An agent opened the idea discussion first in `#general`, then in `#decisions`. No rule said where a discussion goes before it is a decision.
Practice in multi-agent teams: coordinate on the task board, not in chat. A decision made in a chat thread is lost.
A task thread already has a state tag: `blocked` shows that the task waits for a human answer. Thus we do not add a `#proposals` channel.

## What it means for agents

- Discuss a task only in its thread. Do not open threads in `#general` or `#decisions`.
- To get a human choice: post the proposals in the task thread, then use `update_task` with `blocked` and the question.
- When a human chooses: write the decision file, post one message in `#decisions`, and use `done` on the task.
