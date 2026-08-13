# Changelog

All notable changes to Glyphic will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.21.0] - 2026-08-13

### Added
- **Resume and delete sessions** ([#6](https://github.com/caioricciuti/glyphic/pull/6), thanks @mohamed-chebbi-adeo). Any past session can now be resumed straight into the built-in terminal (`claude --resume` through the PTY), and session `.jsonl` files can be deleted from the Sessions page with a confirmation dialog. Deletion is restricted to `.jsonl` files inside the Claude projects directory
- **Move / copy MCP servers between scopes** ([#6](https://github.com/caioricciuti/glyphic/pull/6)). Servers can be moved or copied between Desktop, Global, Local, and Project scopes, one at a time or in bulk via multi-select, with per-server failure reporting
- **Open in Terminal now works on Linux and Windows.** Previously macOS-only; Linux picks the first available emulator (gnome-terminal, konsole, xfce4-terminal, x-terminal-emulator, xterm), Windows opens cmd

### Security
- **Memory file path traversal closed.** `read_memory_file` joined an unsanitized filename, so a crafted path could read arbitrary files; filename and project hash are now validated in all memory commands
- **Filter hook commands are evaluated exactly once.** The token-optimizer PreToolUse hook re-quoted the Bash command into a double-quoted string, so `$(…)`, backticks and `$VARs` were expanded by the outer shell and then executed again by `glyphic-filter exec`; side-effecting substitutions ran twice. The command now travels base64-encoded and runs only in the shell that `exec` spawns
- **AppleScript injection in notifications closed.** Pipeline notification bodies (which default to upstream node output, including HTTP responses) could break out of the AppleScript string and run arbitrary code via `do shell script`. The in-process runner now escapes properly; generated schedule scripts pass values as osascript argv instead of splicing them into code
- **Scheduled pipeline scripts no longer interpolate raw user input.** Node labels and pipeline names are stripped of shell metacharacters before embedding, and `pipeline_id` is validated before being used in script, log, and LaunchAgents plist paths

### Fixed
- **A crash can no longer truncate `settings.json`.** All settings, MCP config, hooks, and optimizer writes go through an atomic temp-file + rename, so a crash mid-write leaves the previous file intact instead of a half-written one
- **Pipeline runner no longer deadlocks after a panic.** The running flag is reset via a drop guard, so a panicked node can't leave "a pipeline run is already in progress" stuck until app restart
- **Typing emoji or CJK in the terminal no longer drops keystrokes.** Keystrokes are UTF-8 encoded before base64, where `btoa()` used to throw on non-Latin1 input
- **UTF-8 panics removed.** Context truncation, schedule log truncation, and `~` path expansion no longer panic on multi-byte characters or a bare `~`
- **Pipeline git clone with a branch now uses `-b`** instead of passing the branch as a positional argument

### Changed
- Frontend and Rust dependencies updated within semver ranges (codemirror, Tauri API/CLI/plugins, Svelte, Tailwind, DOMPurify, xyflow, and transitive Rust crates)

## [0.20.1] - 2026-07-23

### Fixed
- **Windows project folders parsed incorrectly again ([#2](https://github.com/caioricciuti/glyphic/issues/2)).** Newer Claude Code versions start session `.jsonl` files with metadata entries (`last-prompt`, `mode`) that carry no `cwd`, so the resolver from v0.16.0 silently fell back to naive dash-to-slash decoding. `project_hash_to_path` now scans the first 30 lines of each session file for the authoritative `cwd` instead of only the first line. Reported by @mcbyte-it
- **macOS Intel builds restored.** The x86_64 release job was pinned to the retired `macos-13` runner, so it never started and v0.20.0 shipped without an Intel DMG. The build now cross-compiles on `macos-latest`.

## [0.16.0] - 2026-04-16

### Fixed
- **Context Engine — turn refs now expand.** Retrieval surfaced both `tool ref=tr_…` and `turn ref=u_…` IDs but only tool results had an expand path; turn refs fell through `PreToolUse` and hit Bash with a missing binary. Both the hook and the `glyphic-ctx expand` subcommand now look up turns as a fallback, so every ref Claude sees is actually expandable
- **Context Engine — Reindex no longer freezes the UI.** `ctx_reindex_embeddings` was a sync Tauri command, which pins the main thread; the multi-second fastembed pass blocked every other invoke. Switched to `#[tauri::command(async)]` so embedding runs on a worker thread and the rest of the UI stays responsive
- **Windows project folders resolved correctly ([#2](https://github.com/caioricciuti/glyphic/issues/2)).** `project_hash_to_path` used to convert every `-` in a Claude Code project folder name to `/`, mangling Windows paths like `C--Development-convivo-invitation` into `C//Development/convivo/invitation`. It now reads the authoritative `cwd` from the first line of any session `.jsonl` and uses that; dash-decoding stays as the fallback for folders without sessions. Reported by @mcbyte-it

## [0.15.0] - 2026-04-15

### Added
- **Context Engine** — new sidecar binary `glyphic-ctx` wired into Claude Code hooks (`PreToolUse`, `PostToolUse`, `UserPromptSubmit`) for virtualizing tool outputs, indexing prompts, and injecting retrieved context on every turn
- **Tool-output virtualization** — oversized `Bash`/`Grep`/`WebFetch`/`Glob` results are stored in a local SQLite database and replaced in-context with a `ref tr_xxxx` pointer + summary; Claude can expand refs on demand via `glyphic-ctx expand <id>`
- **Hybrid retrieval** — BM25 full-text search via SQLite FTS5 reranked by cosine similarity on BGE-Small-EN-v1.5 embeddings (384-dim, CPU-only, model cached at `~/.glyphic/models/`); "auth failing" surfaces when you later ask about "login broken"
- **Context Engine page** with Enable/Disable toggle, live stats (tool results stored, prompts indexed, bytes stored), semantic coverage card, reindex button, and list of recent virtualized outputs
- **Reindex** — backfill embeddings for rows stored before embedding support, in 64-row batches with progress indicator
- **Clean legacy** — one-click purge of pre-extractor rows (raw JSON envelopes) and rows for tools now on the skip list
- `glyphic-ctx` CLI subcommands: `hook`, `query`, `reindex`, `expand`, `version`
- Kill switch — set `GLYPHIC_CTX_DISABLED=1` to disable the engine at the shell level

### Changed
- `Read` added to the skip-list — file contents live on disk, no value in storing them for retrieval
- Per-tool `extract_output` — `Bash` merges stdout/stderr, `Read` pulls `file.content`, `Grep`/`Glob` pull `content/output/results`; unknown shapes return empty instead of dumping raw JSON
- `dedup_key` delete-before-insert for `Read`/`Glob` rows, so repeatedly looking at the same file doesn't pile up duplicate rows
- Retrieval is session-scoped — the active conversation is excluded from the injected context block so prior sessions (not the current one) are where the signal comes from

## [0.14.0] - 2026-04-04

### Added
- **Command Palette** — press Cmd+K (Ctrl+K) to fuzzy-search all pages and actions with keyboard navigation
- **Page shortcuts** — Cmd+1 through Cmd+9 to jump directly to any page
- **Keybindings Editor** — visual editor for `~/.claude/keybindings.json` with table view, add/remove, and reset to defaults
- **CLAUDE.local.md support** — 4th tab in Instructions for personal project instructions (gitignored)
- **First-run onboarding** — guided welcome screen for new users with 5 setup steps
- **CI workflow** — GitHub Actions pipeline with svelte-check, cargo check, and clippy on PRs
- **Community files** — CONTRIBUTING.md, CODE_OF_CONDUCT.md, SECURITY.md
- **Issue templates** — structured bug report and feature request templates
- **PR template** — checklist for pull request submissions
- **Package metadata** — description, repository, keywords, author, homepage in package.json

## [0.13.0] - 2025-06-01

### Added
- **Multi-tool token optimization** — automatic output filtering with sidecar binary
- Token Savings page with daily savings trends, breakdown by command/tool type
- Opportunity discovery for identifying filterable outputs
- Custom filter rules editor
- Auto-upgrade sidecar binary to match app version

### Fixed
- Hook output format corrected for token optimizer
- Hook detection traversal fixed for nested project structures

## [0.12.0] - 2025-05-15

### Added
- **Token optimizer** with PreToolUse hook integration
- Filter pipeline for automatic command output reduction
- Built-in filter strategies for common tool outputs
- Savings tracking with per-command breakdowns

## [0.11.0] - 2025-05-01

### Added
- **Session replay** — browse and replay past Claude Code sessions step by step
- Full-text search across all sessions
- Session tagging (bug-fix, feature, refactor) and notes
- Export sessions as Markdown
- Live session detection with green pulse indicator
- Paginated session loading for large histories

### Added
- **Analytics page** — token usage and cost tracking per model
- Plan-aware labels (Max/Pro/API)
- Daily token trend and hourly activity charts
- Cache efficiency visualization
- Cost monitoring sidebar widget with budget alerts

## [0.10.0] - 2025-04-15

### Added
- **Pipelines** — visual workflow builder powered by Svelte Flow
- 15 node types (Claude Prompt, Bash, GitHub Action, HTTP, Transform, etc.)
- Node connection-based data flow with `{{input}}` and `{{NodeName}}` references
- Async execution with real-time status updates on canvas
- Pipeline scheduling with cron presets
- Run history with per-node input/output/duration tracking

## [0.9.0] - 2025-04-01

### Added
- **Terminal** — embedded Claude Code via PTY + xterm.js
- Multi-tab sessions with persistence across navigation
- Full ANSI rendering (colors, progress bars, formatting)

### Added
- **Git integration** — branch switcher, file status, conventional commits
- Commit timeline with hash copy
- Auto-refresh every 30 seconds

## [0.8.0] - 2025-03-15

### Added
- **Plugins marketplace** — browse 100+ plugins, install counts, search
- Installed plugin management with version and scope display

### Added
- **Rules editor** — contextual rules with path-based filtering
- 8 templates (TypeScript Strict, API Design, Testing, Security, etc.)
- Visual path filter badges and markdown preview

## [0.7.0] - 2025-03-01

### Added
- **Skills & Agents editor** — full SKILL.md and AGENT.md management
- Frontmatter parsing with visual config cards
- Connection visualization showing relationships
- 8 starter templates

### Added
- **MCP Servers** — manage cloud and local MCP configurations
- Templates for Filesystem, GitHub, PostgreSQL, Memory servers
- Multi-scope support (desktop, global, project)

## [0.6.0] - 2025-02-15

### Added
- **Memory browser** — card-based project memory management
- Frontmatter editor (type, name, description)
- Full CRUD operations

### Added
- **Instructions editor** — CLAUDE.md at global, project, and .claude/ scopes
- Edit/Preview toggle with dark-mode markdown rendering
- Clickable @import reference resolution

## [0.5.0] - 2025-02-01

### Added
- **Hooks manager** — 22 hook events with visual form editor
- Quick-add templates (Shell Command, HTTP Webhook, Prompt Guard, Log to File)
- Collapsible hook cards with type selector and matcher fields

## [0.4.0] - 2025-01-15

### Added
- **Settings editor** — global and project-scope settings management
- Model selector, effort levels, toggle switches
- Permissions editor (allow/ask/deny rules)
- Environment variables management

## [0.3.0] - 2025-01-01

### Added
- **Dashboard** — live stats from history.jsonl
- XP/leveling system with 19 achievements
- Activity heatmap and streak tracking
- Configuration completeness ring

## [0.2.0] - 2024-12-15

### Added
- Light/dark theme with persisted preference
- Auto-updates via Tauri updater plugin
- Cost monitoring sidebar widget
- About dialog with version info

## [0.1.0] - 2024-12-01

### Added
- Initial release
- Tauri 2 + Svelte 5 desktop application
- Basic project structure and navigation
- Multi-platform builds (macOS, Windows, Linux)
- Apple code signing and notarization
