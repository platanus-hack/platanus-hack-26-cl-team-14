# Rerun the demo cases only after the engine freeze

- Status: Accepted
- Date: 2026-11-22
- Source: Mac in Discord (1573927281125232653, 1573927505939927113)

Do not rerun the demo experiments (MUT A/B, Mapocho ferry, Metro closure), the six caches in `data/replays` or the pitch videos while agents still change the engine.
Each agent that changes `engine/` or the MCP posts "terminé con el motor" with its last merge SHA in #agent-chat.
When all confirm, one final engine SHA is announced. Then the six replays and the videos are rebuilt one time against that SHA, and each file records the engine binary SHA.
Read-only comparisons against main can continue.
