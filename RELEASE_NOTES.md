## Glyphic v0.15.0 — Context Engine

Every tool call Claude Code makes eats your context window. Every new session starts from scratch — prior debugging sessions, the bug you found yesterday, the `curl` output from Tuesday, all gone. Glyphic v0.15.0 changes that.

### The Context Engine

A new sidecar (`glyphic-ctx`) wired into Claude Code hooks does three things:

- **Virtualizes large tool outputs.** Oversized `Bash`, `Grep`, `WebFetch`, … results are stored locally in SQLite and replaced in-context with a tiny `ref tr_xxxx` pointer + summary. Claude can expand any ref on demand via `glyphic-ctx expand <id>`. A 50 KB test output becomes ~200 bytes in the prompt.
- **Retrieves relevant prior results on every prompt.** When you send a message, the engine runs a hybrid search — BM25 full-text recall from SQLite FTS5, reranked by cosine similarity on BGE-Small-EN-v1.5 embeddings (384-dim, runs locally on CPU, no API calls). Past turns and tool outputs semantically close to your prompt get injected as `<glyphic-context>`. "login broken" surfaces a prior "auth failing" result.
- **Short-circuits expand calls.** `glyphic-ctx expand tr_xxxx` never leaves your machine — served straight from the local store via a `PreToolUse` hook.

All local. Zero network except the one-time ~130 MB embedding model download on first use.

### Context Engine page

New page under the sidebar:

- **Enable / Disable** — installs the sidecar, writes the hook config, toggles the engine.
- **Semantic coverage card** — % of stored rows that have embeddings. Reindex button backfills any row missing one, in batches.
- **Clean legacy** — one-click purge of rows stored before the current extractors (raw JSON envelopes) and rows for tools now excluded from indexing.
- **Recent tool results** — last 30 virtualized outputs with size, line count, and copy-the-expand-command button.
- Kill switch: `GLYPHIC_CTX_DISABLED=1` in your shell env, no app changes needed.

### Why it matters

Less noise in the context window, more relevant signal from prior work, cross-session memory that actually works — without paying for a cloud vector DB or trusting your code to a third party.

### Downloads

| Platform | File |
|----------|------|
| macOS (Apple Silicon) | `Glyphic_0.15.0_aarch64.dmg` |
| macOS (Intel) | `Glyphic_0.15.0_x64.dmg` |
| Windows | `.msi` installer |
| Linux | `.deb`, `.AppImage`, `.rpm` |

macOS builds are signed and notarized. Auto-update is wired — existing users will be prompted automatically.
