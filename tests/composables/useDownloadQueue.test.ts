import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useDownloadQueue } from '../../src/composables/useDownloadQueue';
import type { SeriesMetadata, SeriesConfigModel, ProgressPayload, EpisodeStatusPayload } from '../../src/types';

describe('useDownloadQueue composable', () => {
  const dummyMetadata: SeriesMetadata = {
    drama_id: 'drama-123',
    category_slug: 'tv-tw',
    channels: [{ id: 'ikzy', name: 'iKun' }],
    episodes: [
      { ep_number: 1, ep_id: 'ep1', available_channels: ['ikzy'] },
      { ep_number: 2, ep_id: 'ep2', available_channels: ['ikzy'] },
    ],
  };

  const dummyConfig: SeriesConfigModel = {
    customTitle: '測試劇集',
    season: 1,
    saveDir: '/downloads',
    threads: 8,
    maxConcurrent: 2,
    binaryPath: '/custom/bin',
  };

  beforeEach(() => {
    vi.restoreAllMocks();
  });

  afterEach(() => {
    // @ts-expect-error Mock cleanup
    delete window.__TAURI_INTERNALS__;
    vi.useRealTimers();
  });

  it('initializes with empty queue and empty error', () => {
    const { queue, error } = useDownloadQueue({ autoListen: false });
    expect(queue.value).toEqual([]);
    expect(error.value).toBe('');
  });

  it('validates title, episodes, and channel on startDownload', async () => {
    const { startDownload, error } = useDownloadQueue({ autoListen: false });

    // Missing customTitle
    await expect(
      startDownload({
        metadata: dummyMetadata,
        selectedEpisodes: [1],
        selectedChannel: 'ikzy',
        config: { ...dummyConfig, customTitle: '   ' },
      })
    ).rejects.toThrow('請輸入劇名！');
    expect(error.value).toBe('請輸入劇名！');

    // Missing selectedEpisodes
    await expect(
      startDownload({
        metadata: dummyMetadata,
        selectedEpisodes: [],
        selectedChannel: 'ikzy',
        config: dummyConfig,
      })
    ).rejects.toThrow('請至少選擇一集！');
    expect(error.value).toBe('請至少選擇一集！');

    // Missing selectedChannel
    await expect(
      startDownload({
        metadata: dummyMetadata,
        selectedEpisodes: [1, 2],
        selectedChannel: '',
        config: dummyConfig,
      })
    ).rejects.toThrow('請選擇來源線路！');
    expect(error.value).toBe('請選擇來源線路！');
  });

  it('builds initial queue items correctly with proper spec and Queued state', async () => {
    const { queue, startDownload } = useDownloadQueue({ autoListen: false });

    await startDownload({
      metadata: dummyMetadata,
      selectedEpisodes: [1, 2],
      selectedChannel: 'ikzy',
      config: dummyConfig,
    });

    expect(queue.value.length).toBe(2);
    expect(queue.value[0]).toEqual({
      id: 1,
      spec: {
        m3u8_url: '',
        custom_title: '測試劇集',
        season: 1,
        episode: 1,
        save_dir: '/downloads',
        referer: 'https://chinaq.fun/tv-tw/drama-123/ep1.html',
      },
      state: { type: 'Queued' },
    });
    expect(queue.value[1].spec.episode).toBe(2);
    expect(queue.value[1].state).toEqual({ type: 'Queued' });
  });

  it('uses fallback category_slug when metadata category_slug is missing', async () => {
    const { queue, startDownload } = useDownloadQueue({ autoListen: false });

    const metadataWithoutSlug: SeriesMetadata = {
      ...dummyMetadata,
      category_slug: null,
    };

    await startDownload({
      metadata: metadataWithoutSlug,
      selectedEpisodes: [3],
      selectedChannel: 'ikzy',
      config: dummyConfig,
    });

    expect(queue.value[0].spec.referer).toBe('https://chinaq.fun/tv-cn/drama-123/ep3.html');
  });

  it('simulates queue progress in non-tauri environment', async () => {
    vi.useFakeTimers();
    const { queue, startDownload, cleanup } = useDownloadQueue({ autoListen: false });

    await startDownload({
      metadata: dummyMetadata,
      selectedEpisodes: [1],
      selectedChannel: 'ikzy',
      config: dummyConfig,
    });

    expect(queue.value[0].state).toEqual({ type: 'Queued' });

    // Tick 1s -> state becomes Progress at 20%
    vi.advanceTimersByTime(1000);
    expect(queue.value[0].state.type).toBe('Progress');
    if (queue.value[0].state.type === 'Progress') {
      expect(queue.value[0].state.data.percent).toBe(20);
    }
    expect(queue.value[0].progress?.percent).toBe(20);

    // Tick 1s -> state progress increases to 60%
    vi.advanceTimersByTime(1000);
    if (queue.value[0].state.type === 'Progress') {
      expect(queue.value[0].state.data.percent).toBe(60);
    }

    // Tick 1s -> nextPct = 100% -> state becomes Muxing
    vi.advanceTimersByTime(1000);
    expect(queue.value[0].state.type).toBe('Muxing');

    // Tick 800ms -> state becomes Completed
    vi.advanceTimersByTime(800);
    expect(queue.value[0].state.type).toBe('Completed');

    cleanup();
  });

  it('invokes start_download_queue with DownloadQueuePayload in Tauri mode', async () => {
    const mockInvoke = vi.fn().mockResolvedValue(undefined);
    vi.doMock('@tauri-apps/api/core', () => ({
      invoke: mockInvoke,
    }));

    // @ts-expect-error Mock Tauri
    window.__TAURI_INTERNALS__ = {};

    const { startDownload } = useDownloadQueue({ autoListen: false });

    await startDownload({
      metadata: dummyMetadata,
      selectedEpisodes: [1, 2],
      selectedChannel: 'ikzy',
      config: dummyConfig,
    });

    expect(mockInvoke).toHaveBeenCalledWith('start_download_queue', {
      req: {
        drama_id: 'drama-123',
        category_slug: 'tv-tw',
        channel_id: 'ikzy',
        episodes: [1, 2],
        custom_title: '測試劇集',
        season: 1,
        save_dir: '/downloads',
        threads: 8,
        max_concurrent: 2,
        binary_path: '/custom/bin',
      },
    });
  });

  it('catches Tauri invoke failure, sets error ref and throws', async () => {
    vi.doMock('@tauri-apps/api/core', () => ({
      invoke: vi.fn().mockRejectedValue(new Error('Process launch failed')),
    }));

    // @ts-expect-error Mock Tauri
    window.__TAURI_INTERNALS__ = {};

    const { startDownload, error } = useDownloadQueue({ autoListen: false });

    await expect(
      startDownload({
        metadata: dummyMetadata,
        selectedEpisodes: [1],
        selectedChannel: 'ikzy',
        config: dummyConfig,
      })
    ).rejects.toThrow('下載啟動失敗: Process launch failed');

    expect(error.value).toBe('下載啟動失敗: Process launch failed');
  });

  it('attaches Tauri event listeners and correctly updates item states and progress', async () => {
    type ListenerCallback = (event: { event: string; id: number; payload: unknown }) => void;
    const listeners: Record<string, ListenerCallback> = {};
    const unlistenProgressMock = vi.fn();
    const unlistenEpisodeStatusMock = vi.fn();
    const unlistenDownloadStatusMock = vi.fn();

    vi.doMock('@tauri-apps/api/event', () => ({
      listen: vi.fn().mockImplementation((eventName: string, cb: ListenerCallback) => {
        listeners[eventName] = cb;
        if (eventName === 'download-progress') return Promise.resolve(unlistenProgressMock);
        if (eventName === 'episode-status') return Promise.resolve(unlistenEpisodeStatusMock);
        return Promise.resolve(unlistenDownloadStatusMock);
      }),
    }));

    // @ts-expect-error Mock Tauri
    window.__TAURI_INTERNALS__ = {};

    const { queue, setupListeners, cleanup, updateStatus } = useDownloadQueue({ autoListen: false });

    // Seed queue with an item
    queue.value = [
      {
        id: 1,
        spec: {
          m3u8_url: '',
          custom_title: '測試劇集',
          season: 1,
          episode: 1,
          save_dir: '/downloads',
          referer: 'https://chinaq.fun/tv-tw/drama-123/ep1.html',
        },
        state: { type: 'Queued' },
      },
    ];

    await setupListeners();

    expect(listeners['download-progress']).toBeDefined();
    expect(listeners['episode-status']).toBeDefined();
    expect(listeners['download-status']).toBeDefined();

    // 1. Episode status event: Downloading
    listeners['episode-status']({
      event: 'episode-status',
      id: 1,
      payload: { episode: 1, status: 'Downloading', error: null } as EpisodeStatusPayload,
    });
    expect(queue.value[0].state).toEqual({ type: 'Downloading' });

    // 2. Download progress event: Progress update
    listeners['download-progress']({
      event: 'download-progress',
      id: 2,
      payload: {
        episode: 1,
        percent: 45.5,
        speed: '12.3 MB/s',
        eta: '00:00:30',
      } as ProgressPayload,
    });
    expect(queue.value[0].progress).toEqual({
      percent: 45.5,
      speed: '12.3 MB/s',
      eta: '00:00:30',
    });
    expect(queue.value[0].state).toEqual({
      type: 'Progress',
      data: {
        percent: 45.5,
        speed: '12.3 MB/s',
        eta: '00:00:30',
      },
    });

    // 3. Episode status event: Completed
    listeners['episode-status']({
      event: 'episode-status',
      id: 3,
      payload: { episode: 1, status: 'Completed', error: null } as EpisodeStatusPayload,
    });
    expect(queue.value[0].state).toEqual({ type: 'Completed' });
    expect(queue.value[0].progress?.percent).toBe(100);

    // 4. Download status event: Failed
    listeners['download-status']({
      event: 'download-status',
      id: 4,
      payload: { episode: 1, status: 'Failed', error: 'Network timeout' } as EpisodeStatusPayload,
    });
    expect(queue.value[0].state).toEqual({
      type: 'Failed',
      error: 'Network timeout',
    });

    // 5. Additional statuses via updateStatus
    updateStatus({ episode: 1, status: 'Muxing' });
    expect(queue.value[0].state).toEqual({ type: 'Muxing' });

    updateStatus({ episode: 1, status: 'Cancelled' });
    expect(queue.value[0].state).toEqual({ type: 'Cancelled' });

    // 6. Cleanup calls unlisten functions
    cleanup();
    expect(unlistenProgressMock).toHaveBeenCalled();
    expect(unlistenEpisodeStatusMock).toHaveBeenCalled();
    expect(unlistenDownloadStatusMock).toHaveBeenCalled();
  });

  it('supports retryDownload, cancelDownload, and clearQueue', () => {
    const { queue, retryDownload, cancelDownload, clearQueue } = useDownloadQueue({ autoListen: false });

    queue.value = [
      {
        id: 1,
        spec: {
          m3u8_url: '',
          custom_title: '測試劇集',
          season: 1,
          episode: 1,
          save_dir: '/downloads',
          referer: 'https://chinaq.fun/tv-tw/drama-123/ep1.html',
        },
        state: { type: 'Failed', error: 'Some error' },
      },
      {
        id: 2,
        spec: {
          m3u8_url: '',
          custom_title: '測試劇集',
          season: 1,
          episode: 2,
          save_dir: '/downloads',
          referer: 'https://chinaq.fun/tv-tw/drama-123/ep2.html',
        },
        state: { type: 'Downloading' },
      },
    ];

    // Retry episode 1
    retryDownload(1);
    expect(queue.value[0].state).toEqual({ type: 'Queued' });

    // Cancel episode 2
    cancelDownload(2);
    expect(queue.value[1].state).toEqual({ type: 'Cancelled' });

    // Clear queue
    clearQueue();
    expect(queue.value).toEqual([]);
  });
});
