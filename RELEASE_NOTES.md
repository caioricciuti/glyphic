## Glyphic v0.23.0

Quality and hardening release for the Token Savings optimizer and the Context Engine: 132 new tests and five real bugs they caught.

### Fixes

- **The output filter can no longer hide errors.** Output containing both an error and a success marker used to be summarized as success; the error guard now always wins. This was the most important bug in the filter and it's covered by regression tests.
- `python -m pytest` output is now actually filtered, and the cargo test filter works again (both rules were dead).
- The Context Engine hook no longer panics on multi-byte output (emoji, CJK, box-drawing in build logs), and re-stored tool results no longer produce duplicate search hits.
- Savings percentages can't underflow anymore.

### Changed

- **Honest Windows behavior**: the optimizer and Context Engine need `sh`/`bash`, so on Windows commands now pass through untouched and enabling either feature says so plainly, instead of silently half-working.
- **Better token accounting**: savings are estimated from the actual text content (CJK and emoji count ~1 token per character) rather than bytes divided by 4.
- `~/.glyphic` is created with private permissions (0700), since savings logs and the context database can contain secrets from command output.
- The Context Engine page flags when Claude Code's native auto memory is also injecting context, so combined session growth isn't a surprise.
- 154 tests now guard the codebase, including a retrieval-quality eval for the Context Engine's hybrid search.
