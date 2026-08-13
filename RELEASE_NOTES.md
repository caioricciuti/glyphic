## Glyphic v0.21.1

Patch release.

### Fixes

- **Plugin uninstall actually uninstalls.** The Plugins page was running `claude plugin install "uninstall <name>"`, which the CLI rejects with "not found in marketplace". Uninstall now calls `claude plugin uninstall` properly.
