## Glyphic v0.22.0

Feature release: full plugin management, MCP live testing, and visual editors for Claude Code's sandbox and auto-mode settings.

### New

- **Plugins v2.** Enable/disable, update, and prune plugins; see each plugin's component inventory and projected token cost; install to user, project, or local scope; and manage marketplace sources (add, refresh, remove) from a new Sources tab.
- **MCP live testing, all transports.** Connect to any MCP server from the MCP page (stdio, streamable HTTP with auth headers, or legacy SSE), list its tools, and run tool calls with JSON arguments, without leaving Glyphic.
- **Chat with an MCP server through Claude.** One click opens a Claude session in the built-in terminal scoped to exactly that server, so you can explore your MCP conversationally.
- **Headers editing** for URL-type MCP servers (put your auth tokens in the config from the UI).
- **Sandbox editor.** Visual editing for Claude Code's Bash sandbox: network domain allowlist, filesystem allow/deny paths, and credential protection (deny or mask) for files and env vars.
- **Auto-mode rules editor.** Manage the plain-language rules that decide which commands run, ask, or get blocked in auto mode.
- **Auto-approve is now your call.** The token optimizer's hook auto-approving rewritten Bash commands is now a toggle (on by default). Turn it off to keep the token savings while Claude Code's normal permission prompts apply.
- Move/copy of MCP servers warns before overwriting a same-name server at the destination, and MEMORY.md is pinned as the auto-memory index.

### Fixes

- Rapid project or scope switching no longer shows stale data on the Settings, Hooks, Rules, and MCP pages.

### Under the hood

- First Rust unit tests (path sanitization, atomic writes, script generation, MCP client roundtrip), fully typed pipeline canvas, and a build-tool refresh: Vite 8, vite-plugin-svelte 7, marked 18.
