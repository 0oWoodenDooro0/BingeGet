# Agent Guidelines for BingeGet

## Quick Commands

| Task | Command |
|---|---|
| Setup & Git hooks | `npm run setup` |
| Frontend checks | `npm run test:fe` |
| Test rule check | `npm run check:tests` |
| Backend checks | `npm run test:be` |
| Full test suite | `npm test` |
| Prune stale worktrees | `npm run worktree:prune` |

## Critical Guardrails

- **Dependencies**: If `node_modules` or CLI tools (`vue-tsc`) are missing, run `npm run setup` first.
- **Lockfile Sync**: Always keep `package-lock.json` in sync with `package.json` (`npm run check:lockfile`). Modifying `package.json` without updating the lockfile fails pre-commit and CI.
- **Branch Lock**: If git checkout fails due to a stale worktree lock, run `npm run worktree:prune` or `git worktree remove --force <path>`.

## Development & Testing Rules

- **Behavioral Tests Only**: Verify public interfaces, state transitions, and event emissions. Strictly avoid asserting static copy or template labels. Enforced by `npm run check:tests`.
- **Media Player Lifecycle**: Enforce Deferred Stream Attachment for `<video>` previews to avoid WebKitGTK/GStreamer pipeline deadlocks. See `docs/architecture/media-preview-lifecycle.md`.
- **Extractor Matrix**: Verify all supported regional slugs (`tv-cn`, `tv-kr`, `tv-tw`, `tv-jp`, `tv-hk`) when testing extractors.
- **Architecture Boundaries**: Decouple state presentation (e.g. search results) from input controls (e.g. URL bar); route referer and header logic through `SiteExtractor` methods rather than hardcoding.

## Orchestration & Delegation

- **Coordinator Discipline**: When executing workflow skills (`implement-spec`, `implement`), the coordinator agent manages the task graph, monitors subagent status, and merges integration branches. It must NOT modify code or tests directly in the coordinator workspace.
- **Worktree Isolation**: All implementation tasks must be delegated via `delegate_task` with `isolation: "worktree"` to protect the primary checkout and sandbox code changes.
- **Scoped Task Boundaries**: The coordinator brief MUST declare `Allowed Paths` (the specific components, composables, or tests being modified). The implementation agent must NOT touch files outside this boundary, and review agents must report changes outside allowed paths as hard scope creep.
- **Mandatory Review Gate**: An integration branch or PR cannot be marked ready until two-axis code review (Standards & Spec) completes via `delegate_task` and all hard findings are resolved.

## Agent skills

### Issue tracker

Issues and specs live as GitHub issues. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five-role triage vocabulary. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context layout (`GLOSSARY.md` + `docs/adr/`). See `docs/agents/domain.md`.

### Media Preview Architecture

Video preview lifecycle and WebKitGTK degradation guide. See `docs/architecture/media-preview-lifecycle.md`.
