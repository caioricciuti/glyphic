## Glyphic v0.7.0 — The Pipeline Power-Up

Major overhaul of the visual pipeline builder, turning it into a proper mini workflow automation engine.

### Pipeline Engine

- **Fixed Claude Prompt in production builds** — Resolved "No such file or directory" by resolving the `claude` binary via zsh login shell + explicit path fallback. Added `enriched_path()` to fix PATH for all subprocess spawns in GUI apps.
- **Node & edge deletion** — Backspace key works on Mac. Right-click edges to delete connections.
- **Right-side config drawer** — Full-height tabbed panel (Config | Results | Logs) replaces the old cramped bottom popup.
- **CodeMirror 6 editors** — Syntax-highlighted code editing for Bash, GitHub CLI, Claude prompts, and HTTP body fields.
- **HTTP node upgrade** — Dynamic headers (key-value pairs), body editor with JSON mode, PATCH and HEAD methods.
- **Test individual nodes** — "Test" button in the config panel runs a single node and shows the result inline.
- **Pipeline scheduler** — Cron-based scheduling via macOS launchd. Pipelines run even when the app is closed. Preset schedules or custom cron expressions. View execution logs in the Logs tab.

### 6 New Node Types (12 total)

| Node | Description |
|------|------------|
| **Git** | Git operations (status, log, diff, pull, push, commit, checkout, clone) on any local repo |
| **Filter (If)** | Conditional pass-through — contains, equals, regex, empty/not_empty checks |
| **Read File** | Read file contents into the pipeline |
| **Write File** | Write pipeline data to a file (overwrite or append) |
| **Notification** | Send macOS notifications with custom title and body |
| **JSON Extract** | Extract values from JSON using dot-path notation (e.g. `data.items[0].name`) |

### CI/CD

- Release workflow now preserves custom release notes instead of overwriting them with a template
