# Account usage reporting

Checked: 2026-11-21. Task: show real account capacity in Discord using the existing Node MCP server.

The [official app-server interface](https://developers.openai.com/codex/app-server/#6-rate-limits-chatgpt)
provides `account/rateLimits/read`. The [window type](https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/schema/typescript/v2/RateLimitWindow.ts)
has `usedPercent`, `windowDurationMins`, and `resetsAt` (Unix seconds).
The [response type](https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/schema/typescript/v2/GetAccountRateLimitsResponse.ts)
separates quota buckets. Use the `codex` bucket; do not combine different buckets.

A read-only request through installed Codex CLI 0.171.0 returned a 10080-minute
primary window with 20% used and reset timestamp 1795608061. Secondary was null.
Thus primary is weekly on this account; assuming primary means 5 hours is wrong.
This is a dated observation, not a current capacity claim.

The updated MCP tool posted a real report at 2026-11-21T05:24:32.508Z:
weekly 21% used, 79% remaining, reset 2026-11-25T12:01:01Z; 5-hour unavailable.
The [Discord message](https://discord.com/channels/1573514576400752640/1573561161587368087/1573569791040553051)
was read back after sending. This checked the live account read, formatting, and Discord delivery.

Decision: read fresh limits through the local CLI for Codex. Match windows by duration.
Keep missing windows unavailable. Other clients can pass host measurements explicitly.
Do not read auth files, scan other sessions, or derive quota percentages from token counts.

No automatic Claude/OpenCode collector was verified. This account supplied no 5-hour window,
so the live run covered only a weekly measurement. No model turns or tests were run.
