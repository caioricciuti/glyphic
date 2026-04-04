## Glyphic v0.14.0 — Command Palette, Keybindings, Onboarding & Community

### New Features

- **Command Palette (Cmd+K)** — Fuzzy-search all pages and actions. Arrow keys to navigate, Enter to select. Also supports Cmd+1-9 for direct page shortcuts.
- **Keybindings Editor** — Visual table editor for `~/.claude/keybindings.json`. Customize all Claude Code keyboard shortcuts with key combos, commands, and conditions. One-click reset to defaults.
- **CLAUDE.local.md Support** — New "Local (gitignored)" tab in Instructions for personal project instructions that won't be committed to version control.
- **First-Run Onboarding** — Guided welcome screen for new users with 5 setup steps covering Settings, Instructions, MCP Servers, Skills, and Terminal.

### Community & Infrastructure

- **CI Pipeline** — Automated svelte-check + cargo check + clippy on every PR
- **Contributing Guide** — Development setup, conventions, and PR workflow
- **Issue Templates** — Structured bug report and feature request forms
- **PR Template** — Checklist for pull request submissions
- **Code of Conduct** — Contributor Covenant v2.1
- **Security Policy** — Vulnerability disclosure process
- **Changelog** — Full version history from v0.1.0

### Downloads

| Platform | File |
|----------|------|
| macOS (Apple Silicon) | `Glyphic_0.14.0_aarch64.dmg` |
| macOS (Intel) | `Glyphic_0.14.0_x64.dmg` |
| Windows | `.msi` installer |
| Linux | `.deb`, `.AppImage`, `.rpm` |

All macOS builds are signed and notarized. The app includes auto-updates — existing users will be notified automatically.
