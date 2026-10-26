import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useSeriesSession } from '../../src/composables/useSeriesSession';
import type { SeriesMetadata, StreamDetails, StreamSession } from '../../src/types';

describe('useSeriesSession composable', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  afterEach(() => {
    // @ts-expect-error Mock cleanup
    delete window.__TAURI_INTERNALS__;
  });

  describe('initial state and parsing (Slice 1)', () => {
    it('initializes with clean session state', () => {
      const session = useSeriesSession();
      expect(session.metadata.value).toBeNull();
      expect(session.selectedChannel.value).toBe('');
      expect(session.selectedEpisodes.value).toEqual([]);
      expect(session.channelProbeMap.value).toEqual({});
      expect(session.loading.value).toBe(false);
      expect(session.error.value).toBe('');
      expect(session.isChannelLocked.value).toBe(true);
    });

    it('parses series url in fallback mode and auto-selects default channel, all episodes, and clears prior probe cache', async () => {
      const session = useSeriesSession();
      session.setCachedProbe('ikzy', {
        resolution: '1080p',
        bitrate: '3000 kbps',
        video_codec: 'H.264',
        audio_codec: 'AAC',
        fps: 30,
        duration_secs: 1200,
      });
      expect(session.channelProbeMap.value['ikzy']).toBeDefined();

      const data = await session.parseUrl('https://chinaq.fun/tv-cn/123456/');
      expect(data.drama_id).toBe('123456');
      expect(session.metadata.value).toEqual(data);
      expect(session.selectedChannel.value).toBe('ikzy');
      expect(session.selectedEpisodes.value).toEqual(data.episodes.map(e => e.ep_number));
      // Prior probe cache is cleared on new series parse
      expect(session.channelProbeMap.value).toEqual({});
    });

    it('parses series url via tauri invoke when tauri is available', async () => {
      const mockMetadata: SeriesMetadata = {
        drama_id: '8888',
        source_id: 'chinaq',
        category_slug: 'tv-cn',
        channels: [
          { id: 'ch_alpha', name: 'Alpha' },
          { id: 'ch_beta', name: 'Beta' },
        ],
        episodes: [
          { ep_number: 1, ep_id: 'ep1', available_channels: ['ch_alpha', 'ch_beta'] },
          { ep_number: 2, ep_id: 'ep2', available_channels: ['ch_beta'] },
        ],
      };

      vi.doMock('@tauri-apps/api/core', () => ({
        invoke: vi.fn().mockResolvedValue(mockMetadata),
      }));

      // @ts-expect-error Mock Tauri
      window.__TAURI_INTERNALS__ = {};

      const session = useSeriesSession();
      const data = await session.parseUrl('https://chinaq.fun/tv-cn/8888/');
      expect(data).toEqual(mockMetadata);
      expect(session.metadata.value).toEqual(mockMetadata);
      expect(session.selectedChannel.value).toBe('ch_alpha');
      expect(session.selectedEpisodes.value).toEqual([1, 2]);
    });
  });

  describe('channel selection and Channel Lock enforcement (Slice 2)', () => {
    it('updates selectedChannel and validates against metadata channels', async () => {
      const session = useSeriesSession();
      await session.parseUrl('https://chinaq.fun/tv-cn/123456/');

      expect(session.selectedChannel.value).toBe('ikzy');
      session.selectChannel('hnzy');
      expect(session.selectedChannel.value).toBe('hnzy');
    });

    it('enforces Channel Lock and detects missing episodes for semi-automatic fallback (ADR-0001)', () => {
      const session = useSeriesSession();
      session.metadata.value = {
        drama_id: '123',
        channels: [
          { id: 'ikzy', name: 'ikzy' },
          { id: 'hnzy', name: 'hnzy' },
        ],
        episodes: [
          { ep_number: 1, ep_id: 'ep1', available_channels: ['ikzy', 'hnzy'] },
          { ep_number: 2, ep_id: 'ep2', available_channels: ['ikzy'] }, // lacks hnzy
          { ep_number: 3, ep_id: 'ep3', available_channels: ['ikzy', 'hnzy'] },
          { ep_number: 4, ep_id: 'ep4', available_channels: ['ikzy'] }, // lacks hnzy
        ],
      };

      session.selectChannel('ikzy');
      expect(session.isChannelLocked.value).toBe(true);
      expect(session.missingEpisodesForLockedChannel.value).toEqual([]);
      expect(session.channelLockStatus.value.hasMissingEpisodes).toBe(false);

      // Switch to hnzy where ep 2 and ep 4 are missing
      session.selectChannel('hnzy');
      expect(session.missingEpisodesForLockedChannel.value).toEqual([2, 4]);
      expect(session.channelLockStatus.value.hasMissingEpisodes).toBe(true);
      expect(session.channelLockStatus.value.missingEpisodes).toEqual([2, 4]);

      // Can toggle channel lock
      session.toggleChannelLock(false);
      expect(session.isChannelLocked.value).toBe(false);
      expect(session.channelLockStatus.value.isLocked).toBe(false);
    });

    it('supports episode selection operations: toggle, select all, clear, first N, and range', async () => {
      const session = useSeriesSession();
      await session.parseUrl('https://chinaq.fun/tv-cn/123456/');
      expect(session.selectedEpisodes.value.length).toBe(36);

      session.toggleEpisode(1);
      expect(session.selectedEpisodes.value.includes(1)).toBe(false);

      session.toggleEpisode(1);
      expect(session.selectedEpisodes.value.includes(1)).toBe(true);

      session.clearSelectedEpisodes();
      expect(session.selectedEpisodes.value).toEqual([]);

      session.selectAllEpisodes();
      expect(session.selectedEpisodes.value.length).toBe(36);

      session.selectFirstNEpisodes(5);
      expect(session.selectedEpisodes.value).toEqual([1, 2, 3, 4, 5]);

      session.selectEpisodeRange(3, 6);
      expect(session.selectedEpisodes.value).toEqual([3, 4, 5, 6]);

      session.setSelectedEpisodes([2, 4]);
      expect(session.selectedEpisodes.value).toEqual([2, 4]);
    });
  });

  describe('telemetry cache and channel switching (Slice 3)', () => {
    const ikzyDetails: StreamDetails = {
      resolution: '3840x2160 (4K)',
      bitrate: '8,500 kbps',
      video_codec: 'HEVC / H.265',
      audio_codec: 'E-AC-3',
      fps: 60,
      duration_secs: 3600,
    };

    const hnzyDetails: StreamDetails = {
      resolution: '1920x1080 (1080p)',
      bitrate: '3,200 kbps',
      video_codec: 'H.264 / AVC',
      audio_codec: 'AAC',
      fps: 30,
      duration_secs: 2700,
    };

    it('preserves channel quality probe metrics across channel switching without requiring re-probing', () => {
      const session = useSeriesSession();
      session.metadata.value = {
        drama_id: '123',
        channels: [
          { id: 'ikzy', name: 'ikzy' },
          { id: 'hnzy', name: 'hnzy' },
        ],
        episodes: [
          { ep_number: 1, ep_id: 'ep1', available_channels: ['ikzy', 'hnzy'] },
        ],
      };

      session.selectChannel('ikzy');
      session.setCachedProbe('ikzy', ikzyDetails);
      expect(session.currentChannelProbe.value).toEqual(ikzyDetails);

      // Switch to hnzy
      session.selectChannel('hnzy');
      expect(session.currentChannelProbe.value).toBeNull();
      session.setCachedProbe('hnzy', hnzyDetails);
      expect(session.currentChannelProbe.value).toEqual(hnzyDetails);

      // Switch back to ikzy - cached metrics are immediately retained without re-probing
      session.selectChannel('ikzy');
      expect(session.currentChannelProbe.value).toEqual(ikzyDetails);
      expect(session.getCachedProbe('ikzy')).toEqual(ikzyDetails);
      expect(session.getCachedProbe('hnzy')).toEqual(hnzyDetails);
    });

    it('resets telemetry cache on resetSession', () => {
      const session = useSeriesSession();
      session.setCachedProbe('ikzy', ikzyDetails);
      expect(session.channelProbeMap.value['ikzy']).toEqual(ikzyDetails);

      session.resetSession();
      expect(session.channelProbeMap.value).toEqual({});
      expect(session.currentChannelProbe.value).toBeNull();
    });
  });

  describe('preview playback and atomic prepare_episode_stream (Slice 4)', () => {
    it('loads preview and automatically populates channelProbeMap with probed stream details', async () => {
      const session = useSeriesSession();
      await session.parseUrl('https://chinaq.fun/tv-cn/123456/');

      expect(session.channelProbeMap.value['ikzy']).toBeUndefined();

      await session.loadPreview(1);

      expect(session.activePreviewEp.value).toBe(1);
      expect(session.streamSession.value).not.toBeNull();
      expect(session.streamDetails.value).not.toBeNull();
      expect(session.currentStreamSource.value).toContain('http');
      // Automatically cached in channelProbeMap for the channel!
      expect(session.channelProbeMap.value['ikzy']).toEqual(session.streamDetails.value);
      expect(session.currentChannelProbe.value).toEqual(session.streamDetails.value);
    });

    it('invokes prepare_episode_stream via Tauri and populates telemetry cache', async () => {
      const mockStreamSession: StreamSession = {
        m3u8_url: 'https://cdn.example.com/stream.m3u8',
        referer: 'https://example.com/play/1',
        proxy_url: 'http://127.0.0.1:8899/proxy?url=https://cdn.example.com/stream.m3u8',
        stream_details: {
          resolution: '1920x1080 (1080p)',
          bitrate: '4,500 kbps',
          video_codec: 'H.264',
          audio_codec: 'AAC',
          fps: 60,
          duration_secs: 2400,
        },
      };

      const invokeMock = vi.fn().mockResolvedValue(mockStreamSession);
      vi.doMock('@tauri-apps/api/core', () => ({
        invoke: invokeMock,
      }));

      // @ts-expect-error Mock Tauri
      window.__TAURI_INTERNALS__ = {};

      const session = useSeriesSession();
      session.metadata.value = {
        drama_id: '999',
        source_id: 'juqing',
        channels: [{ id: 'hnzy', name: 'hnzy' }],
        episodes: [{ ep_number: 1, ep_id: 'ep1' }],
      };
      session.selectChannel('hnzy');

      await session.loadPreview(1);

      expect(invokeMock).toHaveBeenCalledWith('prepare_episode_stream', expect.objectContaining({
        dramaId: '999',
        epId: 'ep1',
        channel: 'hnzy',
        sourceId: 'juqing',
      }));

      expect(session.streamSession.value).toEqual(mockStreamSession);
      expect(session.streamDetails.value).toEqual(mockStreamSession.stream_details);
      expect(session.currentStreamSource.value).toBe(mockStreamSession.proxy_url);
      expect(session.channelProbeMap.value['hnzy']).toEqual(mockStreamSession.stream_details);
      expect(session.currentChannelProbe.value).toEqual(mockStreamSession.stream_details);
    });

    it('manages video element binding and deferred stream attachment', () => {
      const session = useSeriesSession();
      const mockVideo = document.createElement('video');
      const pauseSpy = vi.spyOn(mockVideo, 'pause');

      session.bindVideoElement(mockVideo);
      expect(session.isPlaying.value).toBe(false);

      session.toggleMute();
      expect(mockVideo.muted).toBe(false);
      expect(session.isMuted.value).toBe(false);

      session.setVolume(0.5);
      expect(session.volume.value).toBe(0.5);
      expect(mockVideo.volume).toBe(0.5);

      session.cleanup();
      expect(pauseSpy).toHaveBeenCalled();
      expect(session.isMediaAttached.value).toBe(false);
    });
  });
});
