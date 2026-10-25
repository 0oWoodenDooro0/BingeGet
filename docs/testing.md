# Testing & Quality Principles

## 1. Behavioral Testing vs. Static Text Assertions
- **Focus on Logic & State**: Tests must verify public interfaces, event emissions, payloads, state transitions, conditional button enabling/disabling, plugin calls, and error handling.
- **Strictly Prohibit Non-Logical Text Assertions**:
  - Do NOT assert static copy, template labels, or exact placeholders.
  - Assertions on static copy contain no business logic and break easily on copy updates.

## 2. SiteExtractor URL Matrix Coverage
- When updating URL parsing or extractor logic (e.g. `ChinaQExtractor`), unit tests must test against a multi-category URL matrix (covering all supported region slugs like `tv-cn`, `tv-kr`, `tv-tw`, `tv-jp`, `tv-hk`) rather than only a single category.

## 3. UI Placeholder Convention
- Input placeholders should be direct and concise (e.g. `輸入影集名稱或關鍵字` or `影集網址`).
- Avoid verbose examples or prefixes like `例如：...`.
