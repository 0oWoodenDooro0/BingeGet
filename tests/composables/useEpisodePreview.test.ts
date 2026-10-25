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

  it('handles video decode error (code 3) gracefully with friendly message mentioning codecs/GStreamer', async () => {
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
  });

  it('handles video src not supported error (code 4) gracefully with friendly message', async () => {
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
  });

  it('handles HLS fatal media error gracefully with informative error message', async () => {
    vi.spyOn(Hls, 'isSupported').mockReturnValue(true);
    let capturedErrorHandler: ((event: any, data: any) => void) | null = null;

    vi.spyOn(Hls.prototype, 'on').mockImplementation((event: string, handler: any) => {
      if (event === Hls.Events.ERROR) {
        capturedErrorHandler = handler;
      }
      return Hls.prototype;
    });

    const { bindVideoElement, loadPreview, error } = useEpisodePreview();
    const mockVideo = createMockVideoElement();
    bindVideoElement(mockVideo);

    await loadPreview('drama1', 'ep1', 1, 'ch1');

    expect(capturedErrorHandler).not.toBeNull();
    capturedErrorHandler!(Hls.Events.ERROR, {
      fatal: true,
      type: Hls.ErrorTypes.MEDIA_ERROR,
      details: 'bufferAddCodecError',
    });

    expect(error.value).toMatch(/解碼|GStreamer|bufferAddCodecError/i);
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
  });
});
