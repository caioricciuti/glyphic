## Glyphic v0.21.0

Feature release landing community PR [#6](https://github.com/caioricciuti/glyphic/pull/6) plus a security hardening pass across the Rust backend.

### New

- **Resume and delete sessions** (thanks @mohamed-chebbi-adeo). Resume any past session straight into the built-in terminal, or delete its `.jsonl` file with confirmation. Deletion is restricted to session files inside the Claude projects directory.
- **Move / copy MCP servers between scopes.** Move or copy servers between Desktop, Global, Local, and Project scopes, individually or in bulk with multi-select.
- **Open in Terminal on Linux and Windows.** Previously macOS-only; Linux picks the first available terminal emulator, Windows opens cmd.

### Security

- Closed a path traversal in memory file reads (unsanitized filename could reach arbitrary files).
- The token-optimizer hook no longer double-evaluates Bash commands: rewritten commands travel base64-encoded and run in exactly one shell, so `$(…)` and backticks can't execute twice.
- Closed AppleScript injection in pipeline notifications, where upstream node output (including HTTP responses) could reach `do shell script`.
- Scheduled pipeline scripts no longer embed raw node labels, pipeline names, or unvalidated pipeline ids.

### Fixes

- Settings writes are atomic (temp file + rename), so a crash mid-write can't truncate `settings.json` anymore.
- The pipeline runner recovers from panics instead of staying locked until app restart.
- Terminal input handles emoji and CJK; several UTF-8 boundary panics removed.
- Pipeline `git clone` with a branch uses `-b` correctly.

### Changed

- Frontend and Rust dependencies updated within semver ranges.
