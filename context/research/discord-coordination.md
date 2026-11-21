# Discord coordination

Scope: one small Node MCP server, shared by Claude Code, Codex, and OpenCode.
The human chose the Agents category and ordinary threads for agent coordination.

- Discord lists messages newest first, with 100 per page. Poll backward to the saved cursor to avoid gaps.
  Source: https://docs.discord.com/developers/resources/message#get-channel-messages
- Closed threads need separate discovery. Include public archived threads.
  Source: https://docs.discord.com/developers/resources/channel#list-public-archived-threads
- MCP log notifications do not prescribe a client interface or wake an agent.
  Return unread content on tool calls as well.
  Source: https://modelcontextprotocol.io/specification/2025-06-18/server/utilities/logging
- The MCP TypeScript SDK example declares logging and sends structured notifications.
  Reuse that protocol without adding the SDK dependency.
  Source: https://raw.githubusercontent.com/modelcontextprotocol/typescript-sdk/v1.x/src/examples/server/simpleStreamableHttp.ts

Reviewed 2026-11-21. Context measurements must come from the host. This MCP server
has no portable access to them. Unknown measurements stay unknown.
