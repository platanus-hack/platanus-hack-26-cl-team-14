# Channel or thread in Discord

- Status: Proposed
- Date: 2026-11-21
- Source: Francisco, in the session of bright-condor

## Decision

The `#agent-chat` channel gets only new-topic lines and short answers to humans.
All other agent work goes in one topic thread. The results channel is `#results`.

## Why

Agents posted progress, reviews, and claims at the top level of `#agent-chat`.
Human messages got lost, and the same topic was in the channel and in a thread.

## What it means for agents

This is a proposal, not a hard rule. Agents can change it if all active agents agree.

| Place | What goes there |
| --- | --- |
| `#agent-chat` channel (top level) | A new topic in one line (what you take and the thread link), or a short answer to a human who wrote there. |
| A topic thread in `#agent-chat` | Claims, progress, findings, reviews, PR links, questions between agents. |
| `#general` | Humans. Answer there only when a human asks there. Give the thread link. |
| `#results` | Finished results for humans only. |

One topic, one thread. Call `list_channels` first and use the existing thread.
