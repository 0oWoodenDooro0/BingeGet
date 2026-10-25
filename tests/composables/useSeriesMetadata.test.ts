import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useSeriesMetadata } from '../../src/composables/useSeriesMetadata';
import type { SeriesMetadata, SearchResult } from '../../src/types';

describe('useSeriesMetadata composable', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  afterEach(() => {
    // @ts-expect-error Mock cleanup
    delete window.__TAURI_INTERNALS__;
  });

  it('initializes with null metadata and empty selections', () => {
    const { metadata, selectedChannel, selectedEpisodes, loading, error } = useSeriesMetadata();
    expect(metadata.value).toBeNull();
    expect(selectedChannel.value).toBe('');
    expect(selectedEpisodes.value).toEqual([]);
    expect(loading.value).toBe(false);
    expect(error.value).toBe('');
  });

  it('parses url in non-tauri mode with fallback metadata and auto-selects default channel and all episodes', async () => {
    const { metadata, selectedChannel, selectedEpisodes, parseUrl } = useSeriesMetadata();

    const data = await parseUrl('https://chinaq.fun/tv-cn/123456/');
    expect(data.drama_id).toBe('123456');
    expect(data.source_id).toBe('chinaq');
    expect(data.episodes.length).toBe(36);
    expect(metadata.value).toEqual(data);
    expect(selectedChannel.value).toBe(data.channels[0].id);
    expect(selectedEpisodes.value).toEqual(data.episodes.map(e => e.ep_number));
  });

  it('parses url via tauri invoke when available', async () => {
    const mockMetadata: SeriesMetadata = {
      drama_id: '999',
      source_id: 'chinaq',
      category_slug: 'tv-kr',
      channels: [
        { id: 'ch1', name: 'Channel 1' },
        { id: 'ch2', name: 'Channel 2' },
      ],
      episodes: [
        { ep_number: 1, ep_id: 'ep1', available_channels: ['ch1'] },
        { ep_number: 2, ep_id: 'ep2', available_channels: ['ch1', 'ch2'] },
      ],
    };

    vi.doMock('@tauri-apps/api/core', () => ({
      invoke: vi.fn().mockResolvedValue(mockMetadata),
    }));

    // @ts-expect-error Mock Tauri
    window.__TAURI_INTERNALS__ = {};

    const { metadata, selectedChannel, selectedEpisodes, parseUrl } = useSeriesMetadata();
    const data = await parseUrl('https://chinaq.fun/tv-kr/999/');

    expect(data).toEqual(mockMetadata);
    expect(metadata.value).toEqual(mockMetadata);
    expect(metadata.value?.source_id).toBe('chinaq');
    expect(selectedChannel.value).toBe('ch1');
    expect(selectedEpisodes.value).toEqual([1, 2]);
  });

  it('updates selectedChannel via selectChannel', () => {
    const { selectedChannel, selectChannel } = useSeriesMetadata();
    expect(selectedChannel.value).toBe('');

    selectChannel('hnzy');
    expect(selectedChannel.value).toBe('hnzy');
  });

  it('supports episode toggling, selecting all, clearing, selecting first N, and range', async () => {
    const {
      selectedEpisodes,
      parseUrl,
      toggleEpisode,
      clearSelectedEpisodes,
      selectAllEpisodes,
      selectFirstNEpisodes,
      selectEpisodeRange,
      setSelectedEpisodes,
    } = useSeriesMetadata();

    await parseUrl('https://chinaq.fun/tv-cn/123456/');
    expect(selectedEpisodes.value.length).toBe(36);

    // Toggle off ep 1
    toggleEpisode(1);
    expect(selectedEpisodes.value.includes(1)).toBe(false);

    // Toggle on ep 1
    toggleEpisode(1);
    expect(selectedEpisodes.value.includes(1)).toBe(true);

    // Clear all
    clearSelectedEpisodes();
    expect(selectedEpisodes.value).toEqual([]);

    // Select all
    selectAllEpisodes();
    expect(selectedEpisodes.value.length).toBe(36);

    // Select first 5
    selectFirstNEpisodes(5);
    expect(selectedEpisodes.value).toEqual([1, 2, 3, 4, 5]);

    // Select range 3 to 7
    selectEpisodeRange(3, 7);
    expect(selectedEpisodes.value).toEqual([3, 4, 5, 6, 7]);

    // Select reversed range 6 to 4
    selectEpisodeRange(6, 4);
    expect(selectedEpisodes.value).toEqual([4, 5, 6]);

    // Set custom episodes
    setSelectedEpisodes([2, 4, 6]);
    expect(selectedEpisodes.value).toEqual([2, 4, 6]);
  });

  it('searches series with fallback or tauri invoke', async () => {
    const { searchSeries } = useSeriesMetadata();

    // Empty keyword returns empty
    const empty = await searchSeries('   ');
    expect(empty).toEqual([]);

    // Non-tauri fallback
    const results = await searchSeries('繁花');
    expect(results.length).toBeGreaterThan(0);
    expect(results[0].title).toContain('繁花');

    // Tauri invoke
    const mockResults: SearchResult[] = [
      {
        title: '繁花',
        year: '2023',
        category: '陸劇',
        url: 'https://chinaq.fun/tv-cn/202312271/',
        source_id: 'chinaq',
        source_name: 'ChinaQ',
      },
    ];

    vi.doMock('@tauri-apps/api/core', () => ({
      invoke: vi.fn().mockResolvedValue(mockResults),
    }));

    // @ts-expect-error Mock Tauri
    window.__TAURI_INTERNALS__ = {};

    const tauriResults = await searchSeries('繁花');
    expect(tauriResults).toEqual(mockResults);
  });

  it('resets metadata and state cleanly via resetMetadata', async () => {
    const { metadata, selectedChannel, selectedEpisodes, parseUrl, resetMetadata } = useSeriesMetadata();

    await parseUrl('https://chinaq.fun/tv-cn/123456/');
    expect(metadata.value).not.toBeNull();
    expect(selectedChannel.value).not.toBe('');
    expect(selectedEpisodes.value.length).toBeGreaterThan(0);

    resetMetadata();
    expect(metadata.value).toBeNull();
    expect(selectedChannel.value).toBe('');
    expect(selectedEpisodes.value).toEqual([]);
  });
});
