---
status: accepted
---

# SiteExtractor Plugin Architecture

Streaming websites exhibit widely varying DOM structures, dynamic API endpoints, and request header requirements that frequently change independently of the core application. We decided to encapsulate all site-specific scraping and stream extraction behind an asynchronous `SiteExtractor` plugin trait and registry pattern. This decouples website-specific crawler logic from the download engine and user interface, enabling new video platforms and regional extractors to be added or maintained without destabilizing core download orchestration or concurrency management.
