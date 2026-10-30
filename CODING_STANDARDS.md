# Coding Standards for BingeGet

This document defines the coding standards, architectural boundaries, and code smell heuristics for BingeGet. It is the primary reference during code reviews (Standards axis) and when designing module seams.

---

## 1. Architecture Boundaries & Seams

- **Catalog Seam Purity**:
  - All download engines, preview commands, and UI consumers must interact strictly with the [`Catalog`](file:///home/user/IdeaProjects/BingeGet/src-tauri/src/extractors/catalog.rs) interface or [`useSeriesSession`](file:///home/user/IdeaProjects/BingeGet/src/composables/useSeriesSession.ts).
  - Do not import or access `global_registry` directly from `src-tauri/src/engine/` or frontend composables.
  - Keep site-specific parsing and extraction logic isolated inside [`src-tauri/src/extractors/`](file:///home/user/IdeaProjects/BingeGet/src-tauri/src/extractors/).

- **Header & Referer Routing**:
  - Never hardcode upstream domain URLs, referer headers, or client user-agent strings in commands, download engines, or frontend mocks.
  - Always generate referers and query headers through [`SiteExtractor`](file:///home/user/IdeaProjects/BingeGet/src-tauri/src/extractors/mod.rs) and [`Catalog`](file:///home/user/IdeaProjects/BingeGet/src-tauri/src/extractors/catalog.rs) methods.

- **Presentation & State Decoupling**:
  - Keep search and series state decoupled from input controls (e.g. URL bar vs search results list).
  - UI components bind directly to session state; avoid prop drilling and unnecessary event bubbling chains.

---

## 2. Media Player Lifecycle & Deferred Stream Attachment

- **Deferred Stream Attachment**:
  - Never attach upstream or proxy stream URLs to `<video>` elements on mount or series load. Stream attachment must occur only when the user explicitly triggers an episode preview.
  - Follow the lifecycle rules in [`docs/architecture/media-preview-lifecycle.md`](file:///home/user/IdeaProjects/BingeGet/docs/architecture/media-preview-lifecycle.md) to prevent WebKitGTK / GStreamer pipeline deadlocks.
- **Player Composition**:
  - When wrapping or extending video preview functionality, compose [`useEpisodePreview`](file:///home/user/IdeaProjects/BingeGet/src/composables/useEpisodePreview.ts) directly. Do not duplicate Hls event handling, detach, or cleanup logic.

---

## 3. Domain Modeling & Vocabulary

- **Strict Glossary Adherence**:
  - All identifiers, types, variable names, and documentation must conform to [`GLOSSARY.md`](file:///home/user/IdeaProjects/BingeGet/GLOSSARY.md).
  - **Banned shorthand terms**: Never use `ep`, `ep_str`, `activePreviewEp` (use `episode`, `episode_id`, `episode_number`), `show` (use `drama` or `series`), `clip` / `video` (for an episode), or `download manager` (use `TaskQueue`).
- **Channel Lock Consistency**:
  - Ingested series must enforce Channel Lock across all episodes per [`docs/adr/0001-channel-lock-architecture.md`](file:///home/user/IdeaProjects/BingeGet/docs/adr/0001-channel-lock-architecture.md).
  - Semi-Auto Fallback must prompt the user before cross-channel episode assignment.

---

## 4. Testing Standards & Test Isolation

- **Behavioral Assertions Only**:
  - Verify public interfaces, state transitions, and event emissions.
  - Strictly avoid asserting static copy or template labels. Enforced by `npm run check:tests` via [`scripts/check-test-rules.mjs`](file:///home/user/IdeaProjects/BingeGet/scripts/check-test-rules.mjs).
- **Backend Test Isolation & Shared Singletons**:
  - Integration tests interacting with global shared state (such as the background [`TaskQueue`](file:///home/user/IdeaProjects/BingeGet/src-tauri/src/engine/queue.rs) singleton in [`ipc_commands_test.rs`](file:///home/user/IdeaProjects/BingeGet/src-tauri/tests/ipc_commands_test.rs)) must isolate execution with an async mutex lock or run with `--test-threads=1` to prevent parallel test state collisions.
- **Extractor Matrix Verification**:
  - Extractor test suites must verify all supported regional categories (`tv-cn`, `tv-kr`, `tv-tw`, `tv-jp`, `tv-hk`).

---

## 5. Code Smell Heuristics (Fowler Baseline)

- **Mysterious Name / Speculative Generality**:
  - Function parameters must clearly declare their domain roles. Avoid polymorphic positional signatures like `fn(arg1, arg2, arg3, arg4)`. Use typed options objects or dedicated structs.
- **Duplicated Code**:
  - Consolidate identical lifecycle, cleanup, or serialization routines into shared utilities or composables.
- **Data Clumps**:
  - When parameter tuples (e.g. `drama_id`, `episode_id`, `channel`, `source_id`) repeatedly travel together across module seams, bundle them into an explicit request struct.
- **Middle Man**:
  - Avoid pass-through functions whose sole purpose is forwarding parameters to another service without translation or validation.
