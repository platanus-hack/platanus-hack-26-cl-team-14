# V1 needs, preferences, and life constraints

- Status: Accepted
- Date: 2026-11-21
- Source: [Bruno's clarification in the V1 thread](https://discord.com/channels/1573514576400752640/1573809708047798364/1573822209481244824)

## Decision

Model each person with needs and preferences that guide their choices. Account for stable life constraints, such as where the person works.

## Why

Keep the model simple enough to explain how a person decides to go to a stadium or travel to work. A plausible general model is enough. Use scientific evidence where it helps and make reasonable assumptions for the remaining details.

## What it means for agents

- Needs give a person a reason to act. Preferences guide the person's choices. Stable life constraints limit those choices.
- When the Metro closes, allow a person to change how they travel or choose another destination for a flexible activity. Keep their workplace if the scenario leaves it unchanged.
- The team can choose how to represent these concepts internally. For example, it can represent a stable commitment as a need.
