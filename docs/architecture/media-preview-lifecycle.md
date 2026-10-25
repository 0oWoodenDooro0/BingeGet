# Media Preview Lifecycle Architecture

Reference for video preview lifecycle, deferred stream attachment, and media error degradation in BingeGet.

## Problem Context

In Linux environments (WebKitGTK inside Tauri), the web process delegates HTML5 `<video>` playback to the host system's GStreamer pipeline (`MediaPlayerPrivateGStreamer`).
If an M3U8 stream or video source is attached eagerly (`video.src = ...` or `hls.attachMedia(video)`), WebKitGTK immediately instantiates the demuxer and sink pipeline even when playback has not started (`paused` state). If the host lacks required GStreamer plugins (`gst-plugins-good`, `gst-libav`), WebKit encounters a NULL pointer instance and crashes or deadlocks `WebKitWebProcess`.

To eliminate this vulnerability, BingeGet enforces **Deferred Stream Attachment**.

## Lifecycle State Machine

```
[Unattached / Idle]
        │
        ▼ (Metadata Loaded)
[Standby (Poster)] ──────── (User clicks Play) ────────▶ [Stream Attached & Buffering]
        ▲                                                              │
        │                                                              ▼
        │ (Cleanup / Ep Switch)                                   [Playing]
        │                                                              │
        └─────────────────── (Playback Error) ◀────────────────────────┘
                                    │
                                    ▼
                         [Error Overlay / Fallback]
```

### 1. Unattached / Idle
- `<video>` is mounted with `muted` and `playsinline`.
- `video.src` is empty, `autoplay` attribute is absent, and `hls.attachMedia()` is NOT called.
- No media pipeline is initialized in the browser engine.

### 2. Standby (Poster State)
- The preview dock displays episode metadata and binds `:poster="metadata?.cover_url"`.
- Displays a `StandbyPlayOverlay` with a clear "點擊預覽播放" play button.
- Stream probe metrics (resolution, bitrate, codecs) are probed in the Rust background without attaching media to the DOM video element.

### 3. Stream Attached & Buffering
- Triggered **strictly** upon explicit user interaction (clicking the play button or switching preview with active playback).
- `attachMediaSource()` binds Hls.js (`hls.loadSource()`, `hls.attachMedia()`) or assigns `video.src`.
- Calls `safePlay()` asynchronously inside a try/catch block.

### 4. Playing / Paused
- User can pause/resume, seek, and adjust volume via standard player controls.
- Time updates and duration sync reactively to player state.

### 5. Error & Fallback Protocol
- Standardized via `src/types/media.ts` (`MediaErrorCode`, `getMediaErrorMessage`).
- If `video.onerror` or `hls.on(ERROR)` fires:
  1. Capture specific `MediaError` code or fatal HLS error details.
  2. Display the error overlay with actionable diagnostic instructions (checking GStreamer packages or network).
  3. Offer a **"複製串流網址" (Copy Stream URL)** button so the user can seamlessly paste into an external player (e.g. VLC, mpv).
  4. Offer a **"重新重試" (Retry)** button to reset the pipeline without page reload.
