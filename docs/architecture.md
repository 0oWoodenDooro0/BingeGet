# Architecture & Component Modularity

## 1. UI Component Modularity (SRP)
- **Decouple Input Coordination from Presentation**: Compound UI components (e.g. dual input modes, dropdowns) must separate user input controls from stateful or heavy render trees (search results, skeleton loading, empty states).
- **Focused Child Components**: Keep presenter components focused (e.g. `SearchResultList.vue` alongside `UrlBar.vue`). This prevents divergent change and simplifies unit testing of state transitions.

## 2. Extractor & Network Seams
- **SiteExtractor Boundaries**: Avoid hardcoding site-specific URL patterns, query formats, or referer paths outside extractor modules.
- **Canonical Parameters**: All download engines, queue dispatchers, and IPC commands must query the corresponding `SiteExtractor` method (e.g. `build_episode_referer`) to obtain canonical network parameters.
