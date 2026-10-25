import { ref } from 'vue';
import type { SeriesMetadata, SearchResult } from '../types';
import { isTauri, getErrorMessage } from './useSystemEnvironment';

export function useSeriesMetadata() {
  const metadata = ref<SeriesMetadata | null>(null);
  const selectedChannel = ref<string>('');
  const selectedEpisodes = ref<number[]>([]);
  const loading = ref<boolean>(false);
  const error = ref<string>('');

  const selectChannel = (channelId: string) => {
    selectedChannel.value = channelId;
  };

  const toggleEpisode = (epNum: number) => {
    const current = [...selectedEpisodes.value];
    const idx = current.indexOf(epNum);
    if (idx >= 0) {
      current.splice(idx, 1);
    } else {
      current.push(epNum);
    }
    selectedEpisodes.value = current;
  };

  const selectAllEpisodes = () => {
    if (!metadata.value) {
      selectedEpisodes.value = [];
      return;
    }
    selectedEpisodes.value = metadata.value.episodes.map(e => e.ep_number);
  };

  const clearSelectedEpisodes = () => {
    selectedEpisodes.value = [];
  };

  const selectFirstNEpisodes = (n: number) => {
    if (!metadata.value) {
      selectedEpisodes.value = [];
      return;
    }
    selectedEpisodes.value = metadata.value.episodes.slice(0, n).map(e => e.ep_number);
  };

  const selectEpisodeRange = (start: number, end: number) => {
    if (!metadata.value) {
      selectedEpisodes.value = [];
      return;
    }
    const min = Math.min(start, end);
    const max = Math.max(start, end);
    selectedEpisodes.value = metadata.value.episodes
      .filter(e => e.ep_number >= min && e.ep_number <= max)
      .map(e => e.ep_number);
  };

  const setSelectedEpisodes = (episodes: number[]) => {
    selectedEpisodes.value = [...episodes];
  };

  const parseUrl = async (url: string): Promise<SeriesMetadata> => {
    loading.value = true;
    error.value = '';

    try {
      let data: SeriesMetadata;

      if (isTauri()) {
        const { invoke } = await import('@tauri-apps/api/core');
        data = await invoke<SeriesMetadata>('parse_series_metadata', { url });
      } else {
        // Direct parse for preview / development fallback
        const dramaIdMatch = url.match(/\/(?:tv-[a-zA-Z0-9]+|[a-zA-Z0-9_-]+)\/([a-zA-Z0-9]+)/);
        const dramaId = dramaIdMatch ? dramaIdMatch[1] : '202614235';

        data = {
          drama_id: dramaId,
          category_slug: 'tv-cn',
          cover_url: `https://chinaq.fun/img/${dramaId}.jpg`,
          episodes: Array.from({ length: 36 }, (_, i) => ({
            ep_number: i + 1,
            ep_id: `ep${i + 1}`,
            available_channels: ['ikzy', 'hnzy', 'jszy'],
          })),
          channels: [
            { id: 'ikzy', name: 'ikzy' },
            { id: 'hnzy', name: 'hnzy' },
            { id: 'jszy', name: 'jszy' },
            { id: 'jyzy', name: 'jyzy' },
          ],
        };
      }

      metadata.value = data;

      if (data.channels && data.channels.length > 0) {
        selectedChannel.value = data.channels[0].id;
      } else {
        selectedChannel.value = '';
      }

      if (data.episodes) {
        selectedEpisodes.value = data.episodes.map(e => e.ep_number);
      } else {
        selectedEpisodes.value = [];
      }

      return data;
    } catch (err: unknown) {
      const msg = getErrorMessage(err);
      error.value = msg;
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const searchSeries = async (keyword: string): Promise<SearchResult[]> => {
    const kw = keyword.trim();
    if (!kw) return [];

    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<SearchResult[]>('search_series', { keyword: kw });
    } else {
      // Mock fallback for browser / development
      return [
        {
          title: `${kw} (示範影集)`,
          year: '2024',
          category: '韓劇',
          url: 'https://chinaq.fun/tv-kr/202639642/',
          source_id: 'chinaq',
          source_name: 'ChinaQ',
          cover_url: 'https://chinaq.fun/img/202639642.jpg',
        },
      ];
    }
  };

  const resetMetadata = () => {
    metadata.value = null;
    selectedChannel.value = '';
    selectedEpisodes.value = [];
    error.value = '';
  };

  return {
    metadata,
    selectedChannel,
    selectedEpisodes,
    loading,
    error,
    parseUrl,
    searchSeries,
    selectChannel,
    toggleEpisode,
    selectAllEpisodes,
    clearSelectedEpisodes,
    selectFirstNEpisodes,
    selectEpisodeRange,
    setSelectedEpisodes,
    resetMetadata,
  };
}
