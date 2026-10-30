import { describe, it, expect, vi, beforeEach } from 'vitest';
import Hls from 'hls.js';
import { useEpisodePreview } from '../../src/composables/useEpisodePreview';

describe('useEpisodePreview composable', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  const createMockVideoElement = () => {
    const el = {
      muted: false,
      volume: 1,
      currentTime: 0,
      duration: 100,
      paused: true,
      error: null as { code: number; message: string } | null,
      src: '',
      onplay: null as (() => void) | null,
      onpause: null as (() => void) | null,
      ontimeupdate: null as (() => void) | null,
      onloadedmetadata: null as (() => void) | null,
      ondurationchange: null as (() => void) | null,
      onvolumechange: null as (() => void) | null,
      onerror: null as ((ev: any) => void) | null,
      play: vi.fn().mockResolvedValue(undefined),
      pause: vi.fn().mockImplementation(function (this: any) {
        this.paused = true;
        if (this.onpause) this.onpause();
      }),
      canPlayType: vi.fn().mockReturnValue('maybe'),
    };
    return el as unknown as HTMLVideoElement;
  };

  it('defaults autoPlay to false and does not call play() when preview loads', async () => {
    const { bindVideoElement, loadPreview } = useEpisodePreview();
    const mockVideo = createMockVideoElement();
    bindVideoElement(mockVideo);

    await loadPreview('drama1', 'ep1', 1, 'ch1');

    expect(mockVideo.play).not.toHaveBeenCalled();
  });

  it('calls play() when autoPlay is explicitly true', async () => {
    const { bindVideoElement, loadPreview } = useEpisodePreview({ autoPlay: true });
    const mockVideo = createMockVideoElement();
    bindVideoElement(mockVideo);

    await loadPreview('drama1', 'ep1', 1, 'ch1');

    expect(mockVideo.play).toHaveBeenCalled();
  });

  it('defers stream attachment until playback starts', async () => {
    vi.spyOn(Hls, 'isSupported').mockReturnValue(true);
    const attachMediaSpy = vi.spyOn(Hls.prototype, 'attachMedia');

    const { bindVideoElement, loadPreview, startPlayback, isMediaAttached } = useEpisodePreview();
    const mockVideo = createMockVideoElement();
    bindVideoElement(mockVideo);

    await loadPreview('drama1', 'ep1', 1, 'ch1');

    expect(isMediaAttached.value).toBe(false);
    expect(attachMediaSpy).not.toHaveBeenCalled();

    startPlayback();

    expect(isMediaAttached.value).toBe(true);
    expect(attachMediaSpy).toHaveBeenCalled();
  });

  it('handles video decode error (code 3) gracefully with friendly message mentioning codecs/GStreamer and fallback advice', async () => {
    const { bindVideoElement, error } = useEpisodePreview();
    const mockVideo = createMockVideoElement();
    bindVideoElement(mockVideo);

    // Simulate video element onerror with MEDIA_ERR_DECODE (code 3)
    (mockVideo as any).error = {
      code: 3,
      message: 'Demuxer or decoder failed',
    };
    if (mockVideo.onerror) {
      mockVideo.onerror(new Event('error'));
    }

    expect(error.value).toMatch(/解碼|GStreamer/i);
    expect(error.value).toMatch(/外部播放器/i);
  });

  it('handles video src not supported error (code 4) gracefully with friendly message and fallback advice', async () => {
    const { bindVideoElement, error } = useEpisodePreview();
    const mockVideo = createMockVideoElement();
    bindVideoElement(mockVideo);

    // Simulate video element onerror with MEDIA_ERR_SRC_NOT_SUPPORTED (code 4)
    (mockVideo as any).error = {
      code: 4,
      message: 'Format unsupported',
    };
    if (mockVideo.onerror) {
      mockVideo.onerror(new Event('error'));
    }

    expect(error.value).toMatch(/不支援|解碼|GStreamer/i);
    expect(error.value).toMatch(/外部播放器/i);
  });

  it('handles HLS fatal media error gracefully with informative error message and fallback advice', async () => {
    vi.spyOn(Hls, 'isSupported').mockReturnValue(true);
    let capturedErrorHandler: ((event: any, data: any) => void) | null = null;

    vi.spyOn(Hls.prototype, 'on').mockImplementation((event: string, handler: any) => {
      if (event === Hls.Events.ERROR) {
        capturedErrorHandler = handler;
      }
      return Hls.prototype;
    });

    const { bindVideoElement, loadPreview, startPlayback, error } = useEpisodePreview();
    const mockVideo = createMockVideoElement();
    bindVideoElement(mockVideo);

    await loadPreview('drama1', 'ep1', 1, 'ch1');
    startPlayback();

    expect(capturedErrorHandler).not.toBeNull();
    capturedErrorHandler!(Hls.Events.ERROR, {
      fatal: true,
      type: Hls.ErrorTypes.MEDIA_ERROR,
      details: 'bufferAddCodecError',
    });

    expect(error.value).toMatch(/解碼|GStreamer|bufferAddCodecError/i);
    expect(error.value).toMatch(/外部播放器/i);
  });

  it('toggles play/pause and catches play() rejection gracefully', async () => {
    const { bindVideoElement, togglePlay, error } = useEpisodePreview();
    const mockVideo = createMockVideoElement();
    bindVideoElement(mockVideo);

    // When play() rejects (e.g. browser policy or codec failure)
    (mockVideo.play as any).mockRejectedValueOnce(new Error('GStreamer pipeline initialization failed'));

    togglePlay();
    await new Promise((r) => setTimeout(r, 10));

    expect(error.value).toContain('GStreamer pipeline initialization failed');
    expect(error.value).toMatch(/外部播放器/i);
  });

  it('calls atomic prepare_episode_stream IPC command and eliminates duplicate parallel calls', async () => {
    const mockInvoke = vi.fn().mockImplementation((cmd: string) => {
      if (cmd === 'prepare_episode_stream') {
        return Promise.resolve({
          m3u8_url: 'https://tw.juqing.app/stream.m3u8',
          referer: 'https://tw.juqing.app/vod/play/id/drama1-ch1-1.html',
          proxy_url: 'http://127.0.0.1:8899/proxy?url=https://tw.juqing.app/stream.m3u8',
          stream_details: {
            resolution: '1920x1080 (1080p)',
            bitrate: '3,200 kbps',
            video_codec: 'H.264 / AVC',
            audio_codec: 'AAC',
            fps: 30,
            duration_secs: 2700,
          },
        });
      }
      return Promise.resolve(null);
    });

    vi.doMock('@tauri-apps/api/core', () => ({
      invoke: mockInvoke,
    }));
    // @ts-expect-error Mock Tauri
    window.__TAURI_INTERNALS__ = {};

    try {
      const { loadPreview, probeStream, streamDetails, previewStreamInfo } = useEpisodePreview();

      // Explicit sourceId / source_id
      await loadPreview('drama1', 'ep1', 1, 'ch1', 'juqing');

      // Verified: called atomic prepare_episode_stream
      expect(mockInvoke).toHaveBeenCalledWith('prepare_episode_stream', {
        dramaId: 'drama1',
        epId: 'ep1',
        channel: 'ch1',
        sourceId: 'juqing',
        source_id: 'juqing',
      });

      // Verified: did NOT call duplicate get_episode_preview_stream or probe_episode_stream
      expect(mockInvoke).not.toHaveBeenCalledWith('get_episode_preview_stream', expect.anything());
      expect(mockInvoke).not.toHaveBeenCalledWith('probe_episode_stream', expect.anything());

      // Verified state populated
      expect(previewStreamInfo.value?.m3u8_url).toBe('https://tw.juqing.app/stream.m3u8');
      expect(streamDetails.value?.resolution).toBe('1920x1080 (1080p)');

      // Default sourceId / source_id when omitted
      await loadPreview('drama1', 'ep1', 1, 'ch1');
      expect(mockInvoke).toHaveBeenCalledWith('prepare_episode_stream', {
        dramaId: 'drama1',
        epId: 'ep1',
        channel: 'ch1',
        sourceId: 'chinaq',
        source_id: 'chinaq',
      });

      // probeStream helper also calls atomic prepare_episode_stream
      const probed = await probeStream('drama1', 'ep1', 'ch1', 'juqing');
      expect(probed.resolution).toBe('1920x1080 (1080p)');
    } finally {
      // @ts-expect-error cleanup
      delete window.__TAURI_INTERNALS__;
    }
  });
});
