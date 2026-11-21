# Immediate human receipts

Task: acknowledge new human messages within 5 seconds in general and agent-chat.
Bruno authorized receipt reactions in the active agent session.

- [Discord Gateway](https://docs.discord.com/developers/events/gateway) sends message events without REST polling. Heartbeats need acknowledgements.
- [Create Reaction](https://docs.discord.com/developers/resources/message#create-reaction) adds a reaction as the current bot. It requires Read Message History and Add Reactions.
- [discord.js WebSocketShard](https://github.com/discordjs/discord.js/blob/main/packages/ws/src/ws/WebSocketShard.ts) uses heartbeat acknowledgement tracking and a randomized first heartbeat.

Use Node's existing WebSocket and the MCP's reaction helper. No dependency is needed.
Use the same bot's 👀 reaction so multiple hosts cannot create multiple visible receipts.
Scope to two channel IDs in the hack guild. Exclude bots and webhooks. Do not request message content.
Only the standalone listener uses the Gateway; normal MCP behavior stays unchanged.
5 seconds is a measured target, not an outage guarantee. Log elapsed time after the reaction succeeds.
Read sources on 2026-11-21. The small listener reconnects with a fresh session and does not recover missed messages.
