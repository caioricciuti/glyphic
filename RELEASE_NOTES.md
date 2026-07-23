## Glyphic v0.20.1

Patch release fixing a Windows regression and the missing macOS Intel build.

### Fixes

- **Windows project folders parsed incorrectly again ([#2](https://github.com/caioricciuti/glyphic/issues/2)).** Newer Claude Code versions start session `.jsonl` files with metadata entries (`last-prompt`, `mode`) that carry no `cwd`, so the v0.16.0 resolver silently fell back to naive dash-to-slash decoding and paths like `C:\Development\TestProject` showed up mangled. The resolver now scans the first 30 lines of each session file for the authoritative `cwd` instead of only the first line. Thanks @mcbyte-it for catching the regression.
- **macOS Intel builds restored.** The x86_64 release job was pinned to GitHub's retired `macos-13` runner, so it never started and v0.20.0 shipped without an Intel DMG. The build now cross-compiles on `macos-latest`.
