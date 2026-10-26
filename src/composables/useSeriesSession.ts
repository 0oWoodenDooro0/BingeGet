import { ref, computed } from 'vue';
import type {
  SeriesMetadata,
  SearchResult,
  StreamDetails,
} from '../types';
import { useEpisodePreview } from './useEpisodePreview';
import { isTauri, getErrorMessage } from './useSystemEnvironment';

export const DEFAULT_FALLBACK_SOURCE_ID = 'chinaq';
export const DEFAULT_FALLBACK_BASE_URL = 'https://chinaq.fun';
export const DEFAULT_FALLBACK_COVER_URL = (dramaId: string) => `${DEFAULT_FALLBACK_BASE_URL}/img/${dramaId}.jpg`;
export const DEFAULT_FALLBACK_SEARCH_URL = `${DEFAULT_FALLBACK_BASE_URL}/tv-kr/202639642/`;
export const DEFAULT_FALLBACK_SEARCH_COVER = `${DEFAULT_FALLBACK_BASE_URL}/img/202639642.jpg`;

export interface UseSeriesSessionOptions {
  autoPlay?: boolean;
}

export interface LoadPreviewOptions {
  episodeNumber?: number;
  channel?: string;
  dramaId?: string;
  epId?: string;
  sourceId?: string;
}

export function useSeriesSession(options: UseSeriesSessionOptions = {}) {
  // 1. Series Metadata & Search state
  const metadata = ref<SeriesMetadata | null>(null);
  const loading = ref<boolean>(false);
  const error = ref<string>('');

  // 2. Channel & Channel Lock state (ADR-0001)
  const selectedChannel = ref<string>('');
  const isChannelLocked = ref<boolean>(true);

  // 3. Episode Selection state
  const selectedEpisodes = ref<number[]>([]);

  // 4. Quality Telemetry Cache
  const channelProbeMap = ref<Record<string, StreamDetails>>({});

  // 5. Compose with dedicated episode preview player
  const player = useEpisodePreview(options);

  // Telemetry Cache Accessors & Computations
  const getCachedProbe = (channel: string): StreamDetails | undefined => {
    return channelProbeMap.value[channel];
  };

  const setCachedProbe = (channel: string, details: StreamDetails): void => {
    channelProbeMap.value = {
      ...channelProbeMap.value,
      [channel]: details,
    };
  };

  const clearProbeCache = (): void => {
    channelProbeMap.value = {};
  };

  const currentChannelProbe = computed<StreamDetails | null>(() => {
    if (selectedChannel.value && channelProbeMap.value[selectedChannel.value]) {
      return channelProbeMap.value[selectedChannel.value];
    }
    return player.streamDetails.value ?? null;
  });

  // Channel Lock & Missing Episode Fallback Computations (ADR-0001)
  const missingEpisodesForLockedChannel = computed<number[]>(() => {
    if (!metadata.value || !selectedChannel.value) return [];
    return metadata.value.episodes
      .filter((ep) => {
        if (!ep.available_channels || ep.available_channels.length === 0) return false;
        return !ep.available_channels.includes(selectedChannel.value);
      })
      .map((ep) => ep.ep_number);
  });

  const channelLockStatus = computed(() => {
    const missing = missingEpisodesForLockedChannel.value;
    return {
      channel: selectedChannel.value,
      isLocked: isChannelLocked.value,
      missingEpisodes: missing,
      hasMissingEpisodes: missing.length > 0,
    };
  });

  const selectChannel = (channelId: string) => {
    selectedChannel.value = channelId;
    if (channelProbeMap.value[channelId]) {
      player.streamDetails.value = channelProbeMap.value[channelId];
    } else {
      player.streamDetails.value = null;
    }
  };

  const toggleChannelLock = (locked?: boolean) => {
    isChannelLocked.value = locked ?? !isChannelLocked.value;
  };

  // Episode Selection helpers
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
    selectedEpisodes.value = metadata.value.episodes.map((e) => e.ep_number);
  };

  const clearSelectedEpisodes = () => {
    selectedEpisodes.value = [];
  };

  const selectFirstNEpisodes = (n: number) => {
    if (!metadata.value) {
      selectedEpisodes.value = [];
      return;
    }
    selectedEpisodes.value = metadata.value.episodes.slice(0, n).map((e) => e.ep_number);
  };

  const selectEpisodeRange = (start: number, end: number) => {
    if (!metadata.value) {
      selectedEpisodes.value = [];
      return;
    }
    const min = Math.min(start, end);
    const max = Math.max(start, end);
    selectedEpisodes.value = metadata.value.episodes
      .filter((e) => e.ep_number >= min && e.ep_number <= max)
      .map((e) => e.ep_number);
  };

  const setSelectedEpisodes = (episodes: number[]) => {
    selectedEpisodes.value = [...episodes];
  };

  const probeStream = async (
    dramaId: string,
    epId: string,
    channel: string,
    sourceId?: string
  ): Promise<StreamDetails> => {
    if (channelProbeMap.value[channel]) {
      return channelProbeMap.value[channel];
    }
    const details = await player.probeStream(
      dramaId,
      epId,
      channel,
      sourceId || metadata.value?.source_id
    );
    setCachedProbe(channel, details);
    return details;
  };

  const loadPreview = async (
    param?: number | LoadPreviewOptions,
    channelParam?: string
  ): Promise<void> => {
    let episodeNumber: number;
    let channel: string;
    let dramaId: string;
    let epId: string;
    let sourceId: string | undefined;

    if (typeof param === 'number') {
      episodeNumber = param;
      channel = channelParam || selectedChannel.value;
      dramaId = metadata.value?.drama_id || '';
      const ep = metadata.value?.episodes.find((e) => e.ep_number === episodeNumber);
      epId = ep ? ep.ep_id : `ep${episodeNumber}`;
      sourceId = metadata.value?.source_id;
    } else if (typeof param === 'object' && param !== null) {
      episodeNumber = param.episodeNumber ?? player.activePreviewEp.value;
      channel = param.channel || channelParam || selectedChannel.value;
      dramaId = param.dramaId || metadata.value?.drama_id || '';
      const ep = metadata.value?.episodes.find((e) => e.ep_number === episodeNumber);
      epId = param.epId || (ep ? ep.ep_id : `ep${episodeNumber}`);
      sourceId = param.sourceId || metadata.value?.source_id;
    } else {
      episodeNumber = player.activePreviewEp.value;
      channel = channelParam || selectedChannel.value;
      dramaId = metadata.value?.drama_id || '';
      const ep = metadata.value?.episodes.find((e) => e.ep_number === episodeNumber);
      epId = ep ? ep.ep_id : `ep${episodeNumber}`;
      sourceId = metadata.value?.source_id;
    }

    const cachedBefore = channel ? channelProbeMap.value[channel] : undefined;

    await player.loadPreview(dramaId, epId, episodeNumber, channel, sourceId);

    // Preserve channel quality probe metrics in session cache across channel switching without requiring re-probing
    if (cachedBefore) {
      player.streamDetails.value = cachedBefore;
    } else if (player.streamDetails.value && channel) {
      setCachedProbe(channel, player.streamDetails.value);
    }
  };

  const bindVideoElement = (el: HTMLVideoElement | null) => {
    player.bindVideoElement(el);
    if (el) {
      const origOnLoadedMetadata = el.onloadedmetadata;
      el.onloadedmetadata = (ev: Event) => {
        if (typeof origOnLoadedMetadata === 'function') {
          origOnLoadedMetadata.call(el, ev);
        }
        if (selectedChannel.value && player.streamDetails.value) {
          setCachedProbe(selectedChannel.value, player.streamDetails.value);
        }
      };
    }
  };

  // Parsing & Searching
  const parseUrl = async (url: string): Promise<SeriesMetadata> => {
    loading.value = true;
    error.value = '';
    clearProbeCache();

    try {
      let data: SeriesMetadata;

      if (isTauri()) {
        const { invoke } = await import('@tauri-apps/api/core');
        data = await invoke<SeriesMetadata>('parse_series_metadata', { url });
      } else {
        const dramaIdMatch = url.match(/\/(?:tv-[a-zA-Z0-9]+|[a-zA-Z0-9_-]+)\/([a-zA-Z0-9]+)/);
        const dramaId = dramaIdMatch ? dramaIdMatch[1] : '202614235';

        data = {
          drama_id: dramaId,
          source_id: DEFAULT_FALLBACK_SOURCE_ID,
          category_slug: 'tv-cn',
          cover_url: DEFAULT_FALLBACK_COVER_URL(dramaId),
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
        selectedEpisodes.value = data.episodes.map((e) => e.ep_number);
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
      return [
        {
          title: `${kw} (示範影集)`,
          year: '2024',
          category: '韓劇',
          url: DEFAULT_FALLBACK_SEARCH_URL,
          source_id: DEFAULT_FALLBACK_SOURCE_ID,
          source_name: 'ChinaQ',
          cover_url: DEFAULT_FALLBACK_SEARCH_COVER,
        },
      ];
    }
  };

  const resetSession = () => {
    player.cleanup();
    metadata.value = null;
    selectedChannel.value = '';
    selectedEpisodes.value = [];
    clearProbeCache();
    error.value = '';
    player.activePreviewEp.value = 1;
    player.streamSession.value = null;
    player.streamDetails.value = null;
    player.previewStreamInfo.value = null;
    player.currentStreamSource.value = '';
    player.error.value = '';
    player.currentTime.value = 0;
    player.duration.value = 0;
  };

  return {
    metadata,
    loading,
    error,
    selectedChannel,
    isChannelLocked,
    channelLockStatus,
    missingEpisodesForLockedChannel,
    selectedEpisodes,
    channelProbeMap,
    currentChannelProbe,
    activePreviewEp: player.activePreviewEp,
    streamSession: player.streamSession,
    streamDetails: player.streamDetails,
    previewStreamInfo: player.previewStreamInfo,
    currentStreamSource: player.currentStreamSource,
    isMediaAttached: player.isMediaAttached,
    loadingStream: player.loading,
    streamError: player.error,
    isPlaying: player.isPlaying,
    isMuted: player.isMuted,
    currentTime: player.currentTime,
    duration: player.duration,
    volume: player.volume,
    getCachedProbe,
    setCachedProbe,
    clearProbeCache,
    selectChannel,
    toggleChannelLock,
    toggleEpisode,
    selectAllEpisodes,
    clearSelectedEpisodes,
    selectFirstNEpisodes,
    selectEpisodeRange,
    setSelectedEpisodes,
    parseUrl,
    searchSeries,
    resetSession,
    resetMetadata: resetSession,
    bindVideoElement,
    loadPreview,
    probeStream,
    startPlayback: player.startPlayback,
    togglePlay: player.togglePlay,
    toggleMute: player.toggleMute,
    seekTo: player.seekTo,
    setVolume: player.setVolume,
    cleanup: player.cleanup,
  };
}

export type SeriesSession = ReturnType<typeof useSeriesSession>;
