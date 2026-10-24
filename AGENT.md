# Agent Guidelines for BingeGet

## 1. T3 Code Worktree & Branch Management

- **Sandbox Boundary**: Do NOT attempt to run shell commands or `cd` into paths under `~/.t3-desktop/worktrees/...`. The agent workspace sandbox will block access (`denied by pre-tool hook`).
- **Resolving Branch Lock in Root**:
  - If a branch cannot be checked out because an external worktree is using it (`fatal: '<branch>' is already used by worktree at ...`):
    - Inspect existing worktrees: `npm run worktrees` (or `git worktree list`).
    - Remove the stale worktree directly from the repository root: `git worktree remove --force <worktreePath>` or `npm run worktree:prune`.
- **Working in an Isolated Worktree**:
  - When child tasks or PR stacks need an isolated worktree checkout, use T3 Code's native tooling:
    - To move the current thread into a worktree: call `t3_worktree_handoff`.
    - To launch a new agent/thread in a worktree: call `t3_thread_launch` with `workspaceStrategy: { "type": "worktree", "branch": "...", "startFromOrigin": false }`.
    - Do NOT manually run `git worktree add` in shell prompts; let T3 bind the workspace and permissions automatically.

## 2. Quality & Validation Commands

- **Frontend checks**: `npm run test:fe` (runs theme guardrails check + `vue-tsc --noEmit`).
- **Backend checks**: `npm run test:be` (`cargo test --manifest-path src-tauri/Cargo.toml`).
- **Full test suite**: `npm test` or `npm run check`.
- **Theme Guardrails**: Native `<select>` and `<option>` elements in WebView require `color-scheme: dark` and explicit dark background classes (`bg-slate-900 text-slate-100`) to avoid white background artifacts.
