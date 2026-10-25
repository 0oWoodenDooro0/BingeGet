# BingeGet

A modern desktop drama & video downloader built with Tauri, Rust, Vue 3, and TypeScript.

## System Dependencies

### Linux (WebKitGTK Video Playback)

On Linux, BingeGet runs within a Tauri WebKitGTK environment. Embedded `<video>` playback uses GStreamer. To ensure smooth video preview playback and avoid media decoding errors or process aborts, install the required GStreamer plugin packages:

#### Debian / Ubuntu / Mint
```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  gstreamer1.0-plugins-base \
  gstreamer1.0-plugins-good \
  gstreamer1.0-plugins-bad \
  gstreamer1.0-libav
```

#### Arch Linux / Manjaro
```bash
sudo pacman -S --needed \
  webkit2gtk-4.1 \
  gst-plugins-base \
  gst-plugins-good \
  gst-plugins-bad \
  gst-libav
```

#### Fedora / RHEL
```bash
sudo dnf install \
  webkit2gtk4.1-devel \
  gstreamer1-plugins-base \
  gstreamer1-plugins-good \
  gstreamer1-plugins-bad-free \
  gstreamer1-libav
```

### Media Tools
- `ffmpeg` / `ffprobe`: Recommended for local stream probing and remuxing.
- `n-m3u8dl-re`: Recommended CLI backend for robust parallel segment downloading.

## Development & Testing

```bash
# Setup dependencies and git hooks
npm run setup

# Frontend checks (lockfile sync, vue-tsc typecheck, vitest)
npm run test:fe

# Backend checks (cargo test)
npm run test:be

# Full test suite
npm test
```
