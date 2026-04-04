# Contributing to Glyphic

Thanks for your interest in contributing to Glyphic! This guide will help you get started.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) 1.70+
- [Bun](https://bun.sh/) 1.0+ (or Node.js 18+)
- [Tauri CLI v2](https://v2.tauri.app/start/prerequisites/)
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) installed

### Setup

```bash
git clone https://github.com/caioricciuti/glyphic.git
cd glyphic
bun install
bun run tauri dev
```

### Verify your setup

```bash
bun run check          # TypeScript + Svelte diagnostics
bun run tauri build    # Full production build
```

## Development Workflow

1. **Fork** the repo and create a branch from `main`
2. **Make changes** — follow the conventions below
3. **Test manually** — run `bun run tauri dev` and verify your changes
4. **Run checks** — `bun run check` must pass with zero errors
5. **Commit** — use [Conventional Commits](https://www.conventionalcommits.org/)
6. **Open a PR** — fill in the PR template

## Conventions

### Code Style

- **TypeScript strict mode** — no `any` types
- **Svelte 5 runes** — use `$state`, `$derived`, `$effect`. No legacy `$:` or writable stores
- **Named exports** over default exports
- **Tailwind CSS** for styling — no custom CSS unless necessary
- **Lucide Svelte** for icons
- `$lib` path alias maps to `src/lib/`

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(dashboard): add weekly activity chart
fix(hooks): prevent duplicate hook creation
refactor(settings): extract scope selector component
docs: update README with new screenshots
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `perf`, `style`

### Adding a New Feature Page

1. Create a component folder in `src/lib/components/<feature>/`
2. Add the page ID to `src/lib/stores/navigation.svelte.ts`
3. Add the route case in `src/App.svelte`
4. Add a Sidebar entry with an icon from Lucide

### Adding a New Rust Command

1. Create or extend a module in `src-tauri/src/commands/`
2. Register the command in `src-tauri/src/lib.rs`
3. Add a typed wrapper in `src/lib/tauri/commands.ts`

## Pull Requests

- Keep PRs focused — one feature or fix per PR
- Include screenshots for UI changes
- Update the README if you're adding a visible feature
- All CI checks must pass

## Reporting Bugs

Use the [Bug Report](https://github.com/caioricciuti/glyphic/issues/new?template=bug_report.md) issue template. Include:

- OS and version
- Glyphic version
- Steps to reproduce
- Expected vs actual behavior
- Screenshots if applicable

## Feature Requests

Use the [Feature Request](https://github.com/caioricciuti/glyphic/issues/new?template=feature_request.md) issue template. Describe:

- The problem you're trying to solve
- Your proposed solution
- Any alternatives you've considered

## License

By contributing, you agree that your contributions will be licensed under the [AGPL-3.0 License](LICENSE).
