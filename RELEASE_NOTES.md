## Glyphic v0.6.0 — The Pipeline Power-Up

Major overhaul of the visual pipeline builder, turning it into a proper mini workflow automation engine.

### Pipeline Engine Improvements

- **Fixed Claude Prompt execution** — Resolved "No such file or directory" error on macOS by properly resolving the `claude` CLI binary path for GUI apps
- **Node & edge deletion** — Backspace key now works on Mac; right-click edges to delete connections
- **Right-side config drawer** — Replaced the cramped bottom popup with a full-height tabbed panel (Config | Results | Logs)
- **CodeMirror 6 editors** — Syntax-highlighted code editing for Bash, GitHub CLI, Claude prompts, and HTTP body fields
- **HTTP node upgrade** — Dynamic headers (key-value pairs), body editor with JSON mode, PATCH and HEAD methods
- **Pipeline scheduler** — Cron-based scheduling via macOS launchd. Pipelines run even when the app is closed. Preset schedules (every 5 min, hourly, daily, weekly) or custom cron expressions. View execution logs in the Logs tab.

### 6 New Node Types (12 total)

| Node | Description |
|------|------------|
| **Git** | Run git operations (status, log, diff, pull, push, commit, checkout, clone) on any local repo |
| **Filter (If)** | Conditional pass-through — contains, equals, regex, empty/not_empty checks |
| **Read File** | Read file contents into the pipeline |
| **Write File** | Write pipeline data to a file (overwrite or append) |
| **Notification** | Send macOS notifications with custom title and body |
| **JSON Extract** | Extract values from JSON using dot-path notation (e.g. `data.items[0].name`) |

### CI/CD

- Release workflow now preserves custom release notes instead of overwriting them with a template
