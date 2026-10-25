# BingeGet

Core domain vocabulary for BingeGet, a desktop streaming media scraper and batch downloader.

## Language

**Drama / Series**:
A multi-episode episodic video program identified by title and season number.
_Avoid_: Show, title, program

**Episode**:
A single numbered installment or video segment within a drama or series.
_Avoid_: Video, part, clip, ep

**Channel**:
A specific upstream release source and streaming provider (such as `ikzy` or `hnzy`) hosting encoded media segments for a drama.
_Avoid_: Source site, provider, route, line

**Channel Lock**:
A locking mechanism that constrains all episode downloads of a drama to a single upstream channel to ensure uniform bitrate, encoding quality, and subtitle styling.
_Avoid_: Source locking, channel sync, stream pin

**Semi-Auto Fallback**:
A fallback workflow that detects when a locked channel lacks specific episodes and prompts for manual per-episode channel assignment before downloading.
_Avoid_: Missing episode fallback, auto channel switch, fallback route

**SiteExtractor**:
A plugin trait and interface for scraping drama metadata and resolving playback channels into direct M3U8 stream playlists.
_Avoid_: Parser, scraper, resolver, crawler

**TaskQueue**:
A concurrency-controlled queue engine that throttles active episode downloads to prevent upstream rate limiting and process exhaustion.
_Avoid_: Download manager, process pool, job scheduler
