# Specification: Series Cover Image, Episode Video Preview Dock, and Channel Quality Sorting

## Problem Statement

When users search for a drama in BingeGet, the search results currently present only text titles, years, and categories without cover artwork. This makes it difficult to visually identify the desired drama, particularly for series with identical or similar titles across different production years or adaptations.

Furthermore, multi-source streaming providers offer various upstream channels (such as `ikzy`, `hnzy`, `jszy`), each differing widely in resolution, encoding bitrate, subtitle styling, watermark placement, and completeness. Currently, users have no means to inspect video footage or examine stream metrics prior to committing to a full download. Users often discover that an episode has poor picture quality, garbled subtitles, or incorrect episode content only after an entire batch download completes, leading to wasted bandwidth, time, and storage.

Finally, available playback channels are currently listed in an arbitrary order without clear quality indicators, forcing users to guess which channel offers the best resolution and audio-visual fidelity.

## Solution

1. **Cover Artwork Presentation**: Extend drama search results and metadata with cover image URLs. Search results feature visual cover thumbnails alongside titles and tags with graceful lazy loading and placeholder fallbacks, enabling instant visual verification.
2. **Episode Picker with Right-Side Video Preview Dock**: Refactor the episode section of the current UI so that the episode selection checklist sits on the left while a persistent, live video preview player dock sits directly on the right. Clicking any episode instantaneously switches the preview player to stream that episode's video content without requiring popup dialogs.
3. **Channel Quality-Descending Sorting**: Automatically sort all available channels in descending order of quality score (from highest resolution and bitrate down to lower formats: e.g., 1080p optimal > 1080p standard > 720p medium). Quality tags and bitrate metrics are displayed on channel options.
4. **Detailed Stream Metrics Inspection**: Implement a two-stage stream probe engine (M3U8 Master Playlist tag parsing with an `ffprobe` fallback) that extracts video resolution, stream bitrate, video codec, frame rate, and audio track details. The active stream metrics are presented on the preview player's status overlay and channel selectors with in-memory caching to prevent upstream rate-limiting.

## User Stories

1. As a desktop user searching for a drama, I want to see cover posters in the search results list, so that I can immediately confirm I am selecting the correct drama adaptation.
2. As a desktop user, I want search result images to load asynchronously with placeholder fallbacks, so that search responsiveness is never blocked by slow image loading or broken image links.
3. As a desktop user, I want the loaded drama header to display the drama's cover artwork, so that I maintain visual context while configuring download options.
4. As a desktop user, I want the episode selection section to feature an integrated preview video player docked on the right side, so that I can inspect actual video playback side-by-side with episode numbers.
5. As a desktop user, I want clicking any episode button to instantly route that episode's M3U8 stream to the preview player, so that I can verify that the episode contents and subtitles match the story without opening modal dialogs.
6. As a desktop user, I want the preview player to support play, pause, volume control, and timeline scrubbing, so that I can sample different time points across the episode.
7. As a desktop user, I want to see real-time stream information (resolution, bitrate, video codec, audio codec) displayed on the preview player, so that I can make informed decisions about source quality.
8. As a desktop user, I want playback channels to be sorted from highest quality to lowest quality by default, so that the best available stream source is automatically placed first.
9. As a desktop user, I want channel options to display badges indicating their verified resolution and bitrate (e.g., "1080p · 3.2 Mbps"), so that I can quickly compare different upstream release sources.
10. As a desktop user, I want changing the selected channel in the channel selector to immediately refresh the preview player's stream, so that I can directly compare video quality and watermarks between different channels for the same episode.
11. As a desktop user, I want stream metadata to be cached once fetched for a specific episode and channel, so that switching back and forth does not trigger redundant network requests or slow down the interface.
12. As a desktop user, I want video preview playback to operate smoothly even when upstream streaming servers enforce strict HTTP Referer validation, so that preview requests are not blocked by 403 Forbidden errors.
13. As a desktop user, I want the preview player to default to muted playback, so that sudden audio bursts do not disrupt my environment when browsing episodes.
14. As a desktop user, I want to toggle episode download selection independently of previewing, so that clicking an episode to inspect it does not inadvertently alter my planned download queue.
15. As a desktop user, I want an explicit "Download / Skip" button or checkbox on each episode card, so that I can easily assemble my batch download queue while viewing previews.
16. As a desktop user with missing system dependencies (`ffmpeg` or `n-m3u8dl-re`), I want the preview feature to clearly explain any limitations if deep probing requires `ffprobe`, so that I understand why certain detailed metrics might fall back to basic playlist tags.

## Implementation Decisions

### Frontend Layout & Components
- **Current UI Architecture Preservation**: Retain the existing single-page layout structure (`HeaderBar`, `UrlBar`, `SeriesConfig`, `ChannelSelector`, `EpisodePicker`, `DownloadQueue`).
- **EpisodePicker Refactor**:
  - Restructure `EpisodePicker` into a responsive 2-column dock layout:
    - **Left Column**: Episode selector grid and quick-action toolbar (Select All, Clear, Range apply). Each episode card displays its episode number, an active playback indicator, and a download selection checkbox.
    - **Right Column**: Docked video preview player with interactive controls and a floating stream metrics HUD.
  - On compact screen widths, the layout stacks naturally with the preview dock placed above or below the episode buttons.
- **Preview Video Player Engine**:
  - Integrate `hls.js` within the frontend for native M3U8 stream demuxing and decoding across desktop platforms.
  - Video stream URLs subject to anti-hotlinking are routed through a lightweight Tauri backend streaming proxy endpoint that injects appropriate `Referer` and `User-Agent` headers.
  - Autoplay is disabled by default to prevent audio/video deadlocks or crashes in Linux WebKitGTK when GStreamer plugins are missing. Playback requires explicit user action.
- **Channel Sorting Pipeline**:
  - In `ChannelSelector` and channel models, calculate a quality score for each channel based on probed or declared resolution (1080p > 720p > 480p) and bitrate (higher kbps > lower kbps).
  - Sort channels in descending order of quality score, with highest-quality channels positioned at the front.
- **Search Result Presentation**:
  - Update `SearchResultList` to incorporate vertical cover image thumbnails (`w-10 h-14` / `w-12 h-16`) alongside drama title, year, and category pills.
  - Implement image `loading="lazy"` and `onError` fallback to a styled film placeholder icon.

### Data Models & Schema Evolution
- **SearchResult**:
  - Add `cover_url: Option<String>` to Rust `SearchResult` model and export to TypeScript types.
- **SeriesMetadata**:
  - Add `cover_url: Option<String>` to `SeriesMetadata` to propagate the cover URL to the drama configuration view.
- **StreamDetails Model**:
  - Define a new schema `StreamDetails` containing:
    ```ts
    interface StreamDetails {
      resolution: string;
      bitrate: string;
      video_codec: string;
      audio_codec: string;
      fps?: number;
      duration_secs?: number;
    }
    ```
- **ChannelOption Enhancement**:
  - Enhance channel representations to include optional verified quality metadata (`resolution`, `bitrate`, `quality_rank`) for sorting.

### Backend Stream Probe Engine
- **Two-Stage Stream Probe**:
  - **Stage 1 (M3U8 Master Tag Parsing)**: Fetch the initial M3U8 playlist via HTTP. Inspect `#EXT-X-STREAM-INF` tags for `BANDWIDTH` and `RESOLUTION`. If present, parse instantly in sub-second time without spawning child processes.
  - **Stage 2 (Local `ffprobe` Fallback)**: For stream sources lacking master playlist tags, invoke `/usr/bin/ffprobe` with JSON output flags (`-v quiet -print_format json -show_streams -show_format`) on the first 1~2 segments to resolve video and audio stream parameters.
- **In-Memory Probe Cache**:
  - Maintain a thread-safe LRU cache `RwLock<HashMap<(String, String, String), StreamDetails>>` keyed by `(drama_id, ep_id, channel_id)` with a 30-minute TTL to prevent redundant network and process overhead.
- **Tauri IPC Commands**:
  - Provide `probe_episode_stream(drama_id, ep_id, channel_id) -> Result<StreamDetails, String>`.
  - Provide `get_episode_preview_stream(drama_id, ep_id, channel_id) -> Result<PreviewStreamInfo, String>`.

## Testing Decisions

### Behavioral Testing Guidelines
- Strictly test public interfaces, observable state transitions, and user interactions.
- Avoid testing static text copy or internal template implementation details.

### Test Coverage Matrix
1. **Frontend Unit & Component Tests (`vitest`)**:
   - `SearchResultList.test.ts`: Verify that search items render image tags with `cover_url` and fall back gracefully on broken image events.
   - `ChannelSelector.test.ts`: Verify that channels are sorted in descending order of quality rank and that clicking a channel updates the selected channel state.
   - `EpisodePicker.test.ts`: Verify that clicking an episode updates the active preview episode state while preserving separate download selection toggles, and verify that default autoplay is disabled.
   - `useEpisodePreview.test.ts`: Verify that preview player defaults to non-autoplay, handles media decode/source errors gracefully, and surfaces actionable error messages.
   - `useSeriesMetadata.test.ts`: Verify that metadata parsing preserves `cover_url` and provides quality-sorted channel options.
2. **Backend Unit & Integration Tests (`cargo test`)**:
   - `chinaq_extractor_test.rs`: Verify that `ChinaQExtractor` populates `cover_url` matching `https://chinaq.fun/img/{drama_id}.jpg` across all regional categories (`tv-cn`, `tv-kr`, `tv-tw`, `tv-jp`, `tv-hk`).
   - `stream_probe_test.rs`: Test parsing of M3U8 playlists containing `#EXT-X-STREAM-INF` with bandwidth and resolution attributes.
   - `ipc_commands_test.rs`: Verify input validation and error handling for the new stream probe IPC command.

## Out of Scope

- Full-screen cinematic video playback or offline downloaded video library management.
- Downloading video cover art files to disk alongside video media.
- Automatic transcoding or re-encoding of streams to lower bitrates.
- Subtitle extraction, editing, or OCR burning.

## Further Notes

- ChinaQ serves cover images from `https://chinaq.fun/img/{drama_id}.jpg` with public CORS headers (`Access-Control-Allow-Origin: *`), allowing direct browser rendering without backend proxy overhead.
- The Channel Lock architectural constraint documented in ADR-0001 is maintained: all episodes continue to lock to the user's selected primary channel, while the preview player allows rapid verification before confirming batch downloads.
- **Linux Tauri WebKitGTK Video Dependencies**: Video playback within WebKitGTK relies on GStreamer plugins (`gst-plugins-base`, `gst-plugins-good`, `gst-plugins-bad`, `gst-libav`). When missing, WebKitWebProcess can fail decoding; BingeGet disables autoplay by default and catches media errors defensively to prevent UI freezes.
