# Agent Guidelines for BingeGet

## Quick Commands

| Task | Command |
|---|---|
| Setup & Git hooks | `npm run setup` |
| Frontend checks | `npm run test:fe` |
| Backend checks | `npm run test:be` |
| Full test suite | `npm test` |
| Prune stale worktrees | `npm run worktree:prune` |

## Critical Guardrails

- **Dependencies**: If `node_modules` or CLI tools (`vue-tsc`) are missing, run `npm run setup` first.
- **Lockfile Sync**: Always keep `package-lock.json` in sync with `package.json` (`npm run check:lockfile`). Modifying `package.json` without updating the lockfile fails pre-commit and CI.
- **Dark Theme (WebView)**: Native `<select>` and `<option>` elements require `color-scheme: dark` and dark background classes (`bg-slate-900 text-slate-100`) to prevent white artifacts.
- **Branch Lock**: If git checkout fails due to a stale worktree lock, run `npm run worktree:prune` or `git worktree remove --force <path>`.

## Development & Testing Rules

- **Behavioral Tests Only**: Assert functional behavior, state changes, and event payloads. Do not assert static copy or labels.
- **Extractor Matrix**: Verify all supported regional slugs (`tv-cn`, `tv-kr`, `tv-tw`, `tv-jp`, `tv-hk`) when testing extractors.
- **UI Decoupling**: Separate state presentation (e.g. search results) from input controls (e.g. URL bar).

## Detailed References

- Architecture & Component Modularity: [`docs/architecture.md`](docs/architecture.md)
- Testing & Extractor Principles: [`docs/testing.md`](docs/testing.md)
