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
- **Branch Lock**: If git checkout fails due to a stale worktree lock, run `npm run worktree:prune` or `git worktree remove --force <path>`.

## Development & Testing Rules

- **Behavioral Tests Only**: Verify public interfaces, state transitions, and event emissions. Strictly avoid asserting static copy or template labels.
- **Extractor Matrix**: Verify all supported regional slugs (`tv-cn`, `tv-kr`, `tv-tw`, `tv-jp`, `tv-hk`) when testing extractors.
- **Architecture Boundaries**: Decouple state presentation (e.g. search results) from input controls (e.g. URL bar); route referer and header logic through `SiteExtractor` methods rather than hardcoding.

## Agent skills

### Issue tracker

Issues and specs live as GitHub issues. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five-role triage vocabulary. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context layout (`GLOSSARY.md` + `docs/adr/`). See `docs/agents/domain.md`.
