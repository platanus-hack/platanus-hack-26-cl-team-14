# Context

This folder is the shared memory of the team. Humans and agents read it and write to it.
Keep each file short. If a file does not change what an agent does, delete it.

## What is in this folder

| Path | Contents | Who writes |
| --- | --- | --- |
| `pitch.md` | The idea, the story, and the demo moments. It sets the scope. | Humans (agents propose) |
| `decisions/` | One file for each decision, with the reason. | Anyone |
| `open-questions.md` | Questions that do not have an answer yet. | Anyone |
| `checkpoints.md` | When humans review releases. | Humans |
| `research/` | Briefs from `research-first`: sources, datasets, libraries. | Agents |
| `../meetings/` | Meeting summaries. Do not edit. Raw transcripts stay in `meetings/raw/`, which git ignores. | Humans |

## How to add context

1. **Somebody made a decision** (in a meeting, in Discord, or in a chat with your agent):
   copy `decisions/TEMPLATE.md` to `decisions/YYYYMMDD-HHMM-short-name.md`
   (use the current date and time). Fill it in. Post the link in Discord.
2. **A decision changes:** write a new decision file. In the old file,
   change only the `Status` line to `Superseded by <new file name>`.
3. **A question does not have an answer:** add it to `open-questions.md`.
   When it gets an answer, remove it from that file and write a decision.
4. **You did research:** save the brief in `research/short-name.md`, with links to the sources.
5. **A new meeting occurred:** put the summary in `meetings/YYYY-MM-DD-name.md`
   and the raw transcript in `meetings/raw/`. Then write the decisions and the questions in this folder.
6. **A new rule for all agents:** change `AGENTS.md`. Keep it short.
   Long rules make agents worse. Add a rule only if agents make a mistake without it.
7. **A new skill:** add a folder `.agents/skills/<name>/` with a `SKILL.md`
   (frontmatter `name` and `description`). Claude Code, Codex, and OpenCode find it there.

To see all decisions, list `decisions/`. The file names sort by date.

## Status of a decision

- `Accepted`: agents must obey it.
- `Proposed`: context only. Agents do not have to obey it until a human accepts it.
- `Superseded by <file>`: old. Read the new file.
