# Agent Guidelines for BingeGet

## Quick Commands

| Task | Command |
|---|---|
| Setup & Git hooks | `npm run setup` |
| Frontend checks | `npm run test:fe` |
| Backend checks | `npm run test:be` |
| Full test suite | `npm test` |
| Test rule check | `npm run check:tests` |
| Prune stale worktrees | `npm run worktree:prune` |

## Codebase Navigation

- `src/components/`: Vue 3 UI views and components ([`EpisodePicker.vue`](file:///home/user/IdeaProjects/BingeGet/src/components/EpisodePicker.vue), [`ChannelSelector.vue`](file:///home/user/IdeaProjects/BingeGet/src/components/ChannelSelector.vue), [`DownloadQueue.vue`](file:///home/user/IdeaProjects/BingeGet/src/components/DownloadQueue.vue)).
- `src/composables/`: Client state and session composables ([`useSeriesSession.ts`](file:///home/user/IdeaProjects/BingeGet/src/composables/useSeriesSession.ts), [`useEpisodePreview.ts`](file:///home/user/IdeaProjects/BingeGet/src/composables/useEpisodePreview.ts), [`useDownloadQueue.ts`](file:///home/user/IdeaProjects/BingeGet/src/composables/useDownloadQueue.ts)).
- `src-tauri/src/engine/`: Download queue (`queue.rs`), streaming proxy (`proxy.rs`), atomic stream sessions (`stream.rs`), and stream probing (`probe.rs`).
- `src-tauri/src/extractors/`: Unified catalog facade (`catalog.rs`), registry (`registry.rs`), and site extractor plugins (`chinaq.rs`, `juqing.rs`).
- *Exploration tip*: When searching files via shell, avoid dumping large trees; always exclude `node_modules` and `src-tauri/target`.

## Critical Guardrails

- **Dependencies**: If `node_modules` or CLI tools (`vue-tsc`) are missing, run `npm run setup` first.
- **Lockfile Sync**: Keep `package-lock.json` in sync with `package.json` (`npm run check:lockfile`). Modifying `package.json` without updating the lockfile fails pre-commit and CI.
- **Branch Lock**: If git checkout fails due to a stale worktree lock, run `npm run worktree:prune` or `git worktree remove --force <path>`.

## Standards & Architecture

Review-time rules, Fowler smells, architecture boundaries, and testing standards are maintained in [`CODING_STANDARDS.md`](file:///home/user/IdeaProjects/BingeGet/CODING_STANDARDS.md).
- **Behavioral Tests Only**: Verify public interfaces and state transitions; avoid asserting static copy. Enforced by `npm run check:tests`.
- **Media Preview**: Follow the deferred attachment lifecycle in [`docs/architecture/media-preview-lifecycle.md`](file:///home/user/IdeaProjects/BingeGet/docs/architecture/media-preview-lifecycle.md).
- **Domain Vocabulary**: Strictly conform to [`GLOSSARY.md`](file:///home/user/IdeaProjects/BingeGet/GLOSSARY.md) (no banned terms like `ep`).

## Orchestration & Delegation

- **Coordinator Discipline**: When executing workflow skills (`implement-spec`, `implement`), the coordinator agent manages the task graph, monitors subagent status, and merges integration branches. It must NOT modify code or tests directly in the coordinator workspace.
- **Worktree Isolation**: All implementation tasks must be delegated via `delegate_task` with `isolation: "worktree"` to protect the primary checkout and sandbox code changes.
- **Scoped Task Boundaries**: The coordinator brief MUST declare `Allowed Paths` (the specific components, composables, or tests being modified). The implementation agent must NOT touch files outside this boundary, and review agents must report changes outside allowed paths as hard scope creep.
- **Mandatory Review Gate**: An integration branch or PR cannot be marked ready until two-axis code review (Standards & Spec) completes via `delegate_task` and all hard findings are resolved.

## Context Pointers

- **Coding Standards**: See [`CODING_STANDARDS.md`](file:///home/user/IdeaProjects/BingeGet/CODING_STANDARDS.md).
- **Domain Language**: See [`GLOSSARY.md`](file:///home/user/IdeaProjects/BingeGet/GLOSSARY.md) and [`docs/adr/`](file:///home/user/IdeaProjects/BingeGet/docs/adr/).
- **Issue Tracker & Triage**: See [`docs/agents/issue-tracker.md`](file:///home/user/IdeaProjects/BingeGet/docs/agents/issue-tracker.md) and [`docs/agents/triage-labels.md`](file:///home/user/IdeaProjects/BingeGet/docs/agents/triage-labels.md).
- **Media Preview Architecture**: See [`docs/architecture/media-preview-lifecycle.md`](file:///home/user/IdeaProjects/BingeGet/docs/architecture/media-preview-lifecycle.md).
