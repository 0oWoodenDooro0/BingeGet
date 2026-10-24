# Agent Guidelines for BingeGet

## 1. T3 Code Worktree & Branch Management

- **Sandbox Boundary**: Do NOT attempt to run shell commands or `cd` into paths outside the active workspace directory (such as other worktrees or the root repo if running in a worktree). The agent workspace sandbox will block access (`denied by pre-tool hook`). Always execute commands relative to the current working tree.
- **Resolving Branch Lock in Root**:
  - If a branch cannot be checked out because an external worktree is using it (`fatal: '<branch>' is already used by worktree at ...`):
    - Inspect existing worktrees: `npm run worktrees` (or `git worktree list`).
    - Remove the stale worktree directly from the repository root: `git worktree remove --force <worktreePath>` or `npm run worktree:prune`.
- **Working in an Isolated Worktree**:
  - When child tasks or PR stacks need an isolated worktree checkout, use T3 Code's native tooling:
    - To move the current thread into a worktree: call `t3_worktree_handoff`.
    - To launch a new agent/thread in a worktree: call `t3_thread_launch` with `workspaceStrategy: { "type": "worktree", "branch": "...", "startFromOrigin": false }`.
    - Do NOT manually run `git worktree add` in shell prompts; let T3 bind the workspace and permissions automatically.
  - T3 Code project script `setup` (`npm run setup`) is configured with `runOnWorktreeCreate: true` to automatically install dependencies and configure git hooks upon worktree creation.

## 2. Quality & Validation Commands

- **Setup & Hooks**: `npm run setup` (installs node modules and wires `.githooks`).
- **Frontend checks**: `npm run test:fe` (runs theme guardrails check + `vue-tsc --noEmit` + `vitest run`).
- **Backend checks**: `npm run test:be` (`cargo test --manifest-path src-tauri/Cargo.toml`).
- **Full test suite**: `npm test` or `npm run check`.
- **Pre-commit**: Automatically validates theme guardrails, TypeScript types, and ts-rs schema generation (`export_schema_test`).
- **Theme Guardrails**: Native `<select>` and `<option>` elements in WebView require `color-scheme: dark` and explicit dark background classes (`bg-slate-900 text-slate-100`) to avoid white background artifacts.

## 3. Testing Principles

- **Focus on Behavioral Logic and State Transitions**: Tests must verify public interfaces and critical behaviors (such as event emission and payloads, state transitions, conditional button disabling/enabling, external API / Plugin calls, and cancellation/error handling) rather than static DOM content or hardcoded copy.
- **Strictly Prohibit Non-Logical Text Assertions**:
  - Do not write non-logical tests asserting static text, placeholders, or template labels.
  - Assertions on static copy contain no business logic and are fragile to minor copy tweaks.
  - Test value lies in verifying functional correctness and boundary conditions, not literal copy.
