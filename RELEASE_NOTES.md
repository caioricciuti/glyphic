## Glyphic v0.16.0 — Context Engine polish + Windows fix

A maintenance release focused on making v0.15.0's Context Engine reliable in practice, plus a long-standing Windows path bug.

### Fixes

- **Turn refs now expand.** The retrieval block advertised both tool-result (`tr_…`) and turn (`u_…`) refs, but the expand path only handled tool results — turn refs fell through to a real `glyphic-ctx expand` shell invocation and failed. Both the `PreToolUse` hook and the CLI now look up turns as a fallback, so every ref Claude sees is actually expandable.
- **Reindex no longer freezes the UI.** `ctx_reindex_embeddings` was running the multi-second fastembed pass on Tauri's main thread, pinning every other invoke. It now dispatches to a worker thread — progress updates and other UI interactions stay responsive while rows embed.
- **Windows project folders resolved correctly ([#2](https://github.com/caioricciuti/glyphic/issues/2)).** `project_hash_to_path` used to blindly convert every `-` in a Claude Code project folder name to `/`, which mangles Windows paths like `C--Development-convivo-invitation` into `C//Development/convivo/invitation`. It now reads the authoritative `cwd` from the first line of any session `.jsonl` and uses that; dash-decoding stays as the fallback. Thanks @mcbyte-it for the detailed report and the suggested approach.

### Downloads

| Platform | File |
|----------|------|
| macOS (Apple Silicon) | `Glyphic_0.16.0_aarch64.dmg` |
| macOS (Intel) | `Glyphic_0.16.0_x64.dmg` |
| Windows | `.msi` installer |
| Linux | `.deb`, `.AppImage`, `.rpm` |

macOS builds are signed and notarized. Auto-update will prompt existing users automatically.
