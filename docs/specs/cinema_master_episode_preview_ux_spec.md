# Specification: Cinema Master 7:5 Deck Layout (Variant B) and Episode Preview UX Refactor

## Problem Statement

Users interacting with the drama configuration, channel selection, and episode preview workflow in BingeGet face several visual clutter, legibility, and interaction frictions:

1. **Header Clutter and Compressed Inputs**: In the drama configuration header, the drama poster thumbnail duplicates the visual identification already performed in the search results list, unnecessarily compressing the custom drama title and season inputs.
2. **Channel Header Redundancy**: The playback channel selector header displays redundant explanatory subtitles (`（依畫質高至低排序）`) and an extraneous "高畫質優先" badge that clutter the header row without adding actionable value.
3. **Cramped Episode Cards and Obscured Episode Numbers**: In the current layout, the video preview dock is nested inside the episode selection card. Episode buttons cram episode numbers, preview play states, and download checkboxes into a single tight button. Episode numbers frequently get clipped or obscured by badges, and users frequently trigger stream preview switches when intending to toggle download checkboxes.
4. **Video HUD Obstruction**: Stream resolution, bitrate, and codec badges are superimposed as an overlay directly over the active preview video window. This heads-up display (HUD) blocks the lower third of the video, obscuring subtitles, watermarks, and critical video footage during quality inspection.
5. **Leaking Internal Probe Debug States**: Telemetry readouts occasionally leak unparsed internal strings (such as "需 ffprobe") instead of displaying clean, aligned stream specifications matching the selected channel.
6. **Muddled Download Actions in Video Player**: The preview video player includes a redundant "加入下載清單 / 移出下載清單" button that duplicates the checkbox functionality in the episode picker, confusing the mental boundary between previewing video and configuring the batch download queue.
7. **Unstyled Player Sliders**: The timeline scrubber and volume control use unstyled native slider tracks that lack colored progress fill, making it difficult to discern current playback position and audio levels at a glance against the dark theme.

## Solution

Adopt the prototype-validated **Variant B (Cinema Master 7:5 Deck Layout)** across the series workflow:

1. **Full-Width Drama Configuration**: Remove the cover art thumbnail from the drama configuration card (`SeriesConfig`). Expand drama title, season number, and download directory inputs across full width with balanced grid spacing.
2. **Streamlined Channel Header**: Remove the subtitle text `（依畫質高至低排序）` and eliminate redundant quality priority badges from `ChannelSelector`. Display solely the clean channel count (e.g. `共 N 個可用頻道`).
3. **Decoupled 7:5 Cinema Master Workspace**:
   - Separate episode selection and video preview monitoring into two independent, top-level cards arranged in a responsive 7:5 grid.
   - **Episode Deck (7 Columns)**: Organize episodes into a multi-column matrix. Each episode item features a clear 3-zone structure:
     - Left: Dedicated checkbox gutter for batch download selection.
     - Center: Dedicated episode title and number (`第 01 集`) with guaranteed whitespace, immune to clipping or label overlap.
     - Right: Dedicated "預覽" / "播放中" preview button for switching the monitor feed.
     - Top Toolbar: Quick-action bulk selectors (Select All, Clear All, First N, Range).
   - **Preview Deck (5 Columns)**: Host a dedicated preview monitoring box with zero HUD obstruction inside the video viewport.
4. **Dedicated Stream Telemetry Dashboard**: Extract all stream metrics (resolution, bitrate, video codec, audio codec, frame rate, duration) into a standalone telemetry panel placed directly beneath the player controls.
5. **Clean Player Controls with Dynamic Gradient Fill**:
   - Remove the redundant download toggle button from the player control bar.
   - Implement dynamic gradient fill on the timeline scrubber and volume slider (`linear-gradient` reflecting percentage value).

## User Stories

1. As a desktop user, I want the drama configuration card to have full-width inputs without a redundant poster thumbnail, so that I have ample space to edit custom series titles and file paths.
2. As a desktop user, I want the channel selector header to display a clean channel count without redundant sorting subtitles or priority badges, so that the interface remains minimalist and uncluttered.
3. As a desktop user, I want the episode selector and the video preview monitor to be separated into two distinct cards side-by-side, so that episode management and media inspection feel like independent, focused tools.
4. As a desktop user, I want each episode item to have a dedicated checkbox gutter on the left, so that I can click checkboxes to toggle download selection without accidentally triggering stream playback.
5. As a desktop user, I want episode numbers (e.g., "第 01 集") to have dedicated center alignment and fixed padding, so that the numbers are never hidden, wrapped, or obscured by other badges.
6. As a desktop user, I want each episode item to have an explicit "預覽" button on the right, so that I have a distinct target to switch the active preview stream.
7. As a desktop user, I want the active preview episode's "預覽" button to visually highlight and display "播放中", so that I immediately know which episode is loaded in the monitor.
8. As a desktop user, I want the video preview window to have no status overlays or HUD badges covering the video frame, so that subtitles and video details are 100% visible during playback.
9. As a desktop user, I want the stream's resolution, bitrate, codecs, frame rate, and duration to appear in a dedicated telemetry card below the player, so that I can analyze technical specifications without blocking video playback.
10. As a desktop user, I want the telemetry metrics to align with the active channel probe data and never display internal debug placeholders (such as "需 ffprobe"), so that telemetry remains professional and trustworthy.
11. As a desktop user, I want the timeline scrubber to display a violet dynamic progress fill that tracks playback time, so that I can see current progress at a glance.
12. As a desktop user, I want the volume slider to display a dynamic fill that tracks volume level, so that audio volume is visually recognizable.
13. As a desktop user, I want the preview player controls to exclude redundant download toggles, so that the player focuses exclusively on playback inspection.
14. As a desktop user, I want to use quick-selection buttons (Select All, Clear All, First 10, First 20) in the episode deck toolbar, so that I can rapidly configure my batch download queue.
15. As a desktop user, I want to copy the active M3U8 stream URL with a single click in the player bar, so that I can test the stream in external media players (e.g., mpv, VLC) if needed.
16. As a desktop user, I want switching channels in the channel selector to immediately update the preview monitor and telemetry panel to that channel's stream, so that I can cross-compare video quality across channels.
17. As a desktop user on smaller displays, I want the 7:5 deck layout to collapse gracefully into a stacked layout, so that the controls remain fully usable on compact screens.
18. As a desktop user, I want video preview playback to remain paused by default until I explicitly click play, preventing WebKitGTK decoder deadlocks on Linux systems.

## Implementation Decisions

### Layout & Component Architecture

1. **`SeriesConfig.vue` Header Clean-up**:
   - Remove the `series-header-cover` container and thumbnail element.
   - Adjust the grid layout of custom title, season number, and save directory inputs to occupy full 12-column width.
   - Maintain the `v-model:config` public contract and directory browser dialog integration.

2. **`ChannelSelector.vue` Header Simplification**:
   - Strip the `（依畫質高至低排序）` subtitle from the section header.
   - Remove the redundant `channel-quality-priority-badge` element.
   - Display a clean summary counter (e.g. `共 N 個可用頻道`).

3. **`EpisodePicker.vue` & Video Monitor (Variant B 7:5 Architecture)**:
   - Split the component into a responsive 12-column grid (`lg:grid-cols-12 gap-5`):
     - **Left Episode Deck (`lg:col-span-7`)**:
       - Quick-action toolbar with Select All, Clear, First 10, First 20 buttons.
       - Matrix of episode cards with a 3-zone structure:
         ```html
         <div class="flex items-center justify-between p-2 rounded-lg border ...">
           <!-- Zone 1: Dedicated Checkbox Gutter -->
           <button type="button" @click.stop="toggleEpisode(ep.ep_number)">
             <CheckSquare v-if="isSelected" />
             <Square v-else />
           </button>

           <!-- Zone 2: Unobscured Episode Title -->
           <span class="font-medium text-xs">第 {{ epNumber }} 集</span>

           <!-- Zone 3: Dedicated Preview Trigger Button -->
           <button type="button" @click.stop="setPreviewEp(ep.ep_number)">
             <Play />
             <span>{{ isActivePreview ? '播放中' : '預覽' }}</span>
           </button>
         </div>
         ```
     - **Right Monitor Deck (`lg:col-span-5`)**:
       - Video box container (`aspect-video`) with clean standby overlay, deferred stream attachment, and zero HUD elements inside the viewport.
       - Player control toolbar with Play/Pause, Mute/Unmute, dynamic gradient scrubber, dynamic volume slider, and Copy Stream URL button.
       - Standalone **Stream Telemetry Dashboard** container directly below player controls:
         ```html
         <div class="rounded-xl border border-slate-800 bg-slate-900/40 p-3.5 space-y-2.5">
           <!-- Grid of Resolution, Bitrate, Video/Audio Codecs, FPS/Duration -->
         </div>
         ```

4. **Dynamic Gradient Scrubber & Volume (Prototype Pattern)**:
   - Use dynamic inline style for range inputs derived from the prototype:
     ```ts
     const scrubberStyle = computed(() => ({
       background: `linear-gradient(to right, #8b5cf6 0%, #8b5cf6 ${(currentTime.value / duration.value) * 100}%, #334155 ${(currentTime.value / duration.value) * 100}%, #334155 100%)`
     }));
     ```

5. **Interface and Contract Stability**:
   - `EpisodePicker.vue` props remain backwards-compatible:
     ```ts
     props: {
       episodes: EpisodeInfo[];
       selectedEpisodes: number[];
       selectedChannel: string;
       metadata: SeriesMetadata;
       channelProbeMap?: Record<string, StreamDetails>;
     }
     ```
   - Emits remain:
     ```ts
     emits: {
       'update:selectedEpisodes': (episodes: number[]) => void;
       'change-preview': (episodeNumber: number) => void;
     }
     ```

## Testing Decisions

### Behavioral Testing Guidelines
- Tests must strictly verify public component interfaces, emitted events, user click interactions, and observable DOM states (`data-testid`).
- Avoid testing static text copy or CSS class names that do not reflect behavioral contracts.

### Test Seam Definition
- **Primary Seam**: Component Unit & Integration Seam (`EpisodePicker.test.ts`, `SeriesConfig.test.ts`, `ChannelSelector.test.ts`) executed via Vitest and Vue Test Utils.
- **Composable Seam**: `useEpisodePreview.test.ts` verifying player lifecycle, deferred stream attachment, play/pause toggling, and error recovery.

### Test Coverage Plan
1. `SeriesConfig.test.ts`:
   - Verify that cover thumbnail element `[data-testid="series-header-cover"]` does not exist.
   - Verify that updating title, season, and directory inputs emits `update:config`.
2. `ChannelSelector.test.ts`:
   - Verify that the channels are rendered and clickable.
   - Verify that sorting subtitle and priority badge elements are absent from the DOM.
3. `EpisodePicker.test.ts`:
   - Verify the 7:5 deck layout: episode list container (`[data-testid="episode-deck"]`) and preview monitor container (`[data-testid="preview-dock"]`).
   - Verify that clicking the checkbox toggles episode selection without changing the active preview episode.
   - Verify that clicking the preview action button updates the preview episode and emits `change-preview` without altering `selectedEpisodes`.
   - Verify that the video element contains zero HUD overlay children.
   - Verify that the telemetry container (`[data-testid="telemetry-dashboard"]`) renders resolution and bitrate from `channelProbeMap`.
   - Verify that the video player does NOT contain an "add/remove download" action button.
   - Verify that autoplay is disabled by default on mount.
4. `useEpisodePreview.test.ts`:
   - Verify play, pause, volume, and time scrubbing operations.

## Out of Scope

- Modifying the underlying Rust backend download engine (`src-tauri/src/downloader.rs`) or `n-m3u8dl-re` invocation.
- Changing search result cover artwork in `SearchResultList.vue`.
- Multi-channel concurrent downloading (Channel Lock ADR-0001 constraint remains strictly in effect).
- Fullscreen video playback modal.

## Further Notes

- Prototype validation: The 7:5 Cinema Master layout (Variant B) was evaluated and selected over Variant A (Side-by-side vertical flow) and Variant C (Segmented dual-state card) for its optimal separation of concerns and prevention of text clipping.
- Adheres to the Linux Tauri WebKitGTK guidelines (`docs/architecture/media-preview-lifecycle.md`): Deferred stream attachment and user-gesture-initiated playback prevent GStreamer pipeline hangs.
