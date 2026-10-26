import { ref, computed, shallowRef } from 'vue';
import Hls from 'hls.js';
import type {
  SeriesMetadata,
  SearchResult,
  StreamDetails,
  StreamSession,
  PreviewStreamInfo,
} from '../types';
import { FALLBACK_HINT, getMediaErrorMessage } from '../types';
import { isTauri, getErrorMessage } from './useSystemEnvironment';

export interface UseSeriesSessionOptions {
  autoPlay?: boolean;
}

export function useSeriesSession(options: UseSeriesSessionOptions = {}) {
  const autoPlay = options.autoPlay === true;

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

  // 5. Active Preview Playback state
  const activePreviewEp = ref<number>(1);
  const streamSession = ref<StreamSession | null>(null);
  const streamDetails = ref<StreamDetails | null>(null);
  const previewStreamInfo = ref<PreviewStreamInfo | null>(null);
  const currentStreamSource = ref<string>('');
  const isMediaAttached = ref<boolean>(false);
  const loadingStream = ref<boolean>(false);
  const streamError = ref<string>('');
  const isPlaying = ref<boolean>(false);
  const isMuted = ref<boolean>(true);
  const currentTime = ref<number>(0);
  const duration = ref<number>(0);
  const volume = ref<number>(1);

  const hlsInstance = shallowRef<Hls | null>(null);
  const videoElement = shallowRef<HTMLVideoElement | null>(null);

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
    return streamDetails.value ?? null;
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
      streamDetails.value = channelProbeMap.value[channelId];
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

  // Media Playback & Deferred Stream Attachment
  const cleanup = () => {
    if (hlsInstance.value) {
      hlsInstance.value.destroy();
      hlsInstance.value = null;
    }
    if (videoElement.value) {
      if (typeof videoElement.value.pause === 'function') {
        videoElement.value.pause();
      }
      if (typeof videoElement.value.removeAttribute === 'function') {
        videoElement.value.removeAttribute('src');
      }
      if (typeof videoElement.value.load === 'function') {
        videoElement.value.load();
      }
    }
    isMediaAttached.value = false;
    isPlaying.value = false;
  };

  const safePlay = async () => {
    if (!videoElement.value) return;
    try {
      await videoElement.value.play();
    } catch (err: unknown) {
      streamError.value = `播放失敗: ${getErrorMessage(err)}。${FALLBACK_HINT}`;
    }
  };

  const attachMediaSource = () => {
    if (!videoElement.value || !currentStreamSource.value || isMediaAttached.value) {
      return;
    }
    const streamSource = currentStreamSource.value;

    if (Hls.isSupported()) {
      const hls = new Hls({
        enableWorker: false,
      });
      hls.loadSource(streamSource);
      hls.attachMedia(videoElement.value);
      hls.on(Hls.Events.MANIFEST_PARSED, () => {
        safePlay();
      });
      hls.on(Hls.Events.ERROR, (_, data) => {
        if (data.fatal) {
          switch (data.type) {
            case Hls.ErrorTypes.NETWORK_ERROR:
              streamError.value = `串流網路錯誤 (${data.details})，請檢查網路連線`;
              break;
            case Hls.ErrorTypes.MEDIA_ERROR:
              streamError.value = `媒體解碼或處理錯誤 (${data.details})，可能是系統缺少解碼器 (如 Linux GStreamer 外掛程式)。${FALLBACK_HINT}`;
              break;
            default:
              streamError.value = `串流播放失敗: ${data.details}。${FALLBACK_HINT}`;
              break;
          }
        }
      });
      hlsInstance.value = hls;
      isMediaAttached.value = true;
    } else {
      videoElement.value.src = streamSource;
      isMediaAttached.value = true;
      safePlay();
    }
  };

  const startPlayback = () => {
    if (!videoElement.value) return;
    if (currentStreamSource.value && !isMediaAttached.value) {
      attachMediaSource();
    } else if (videoElement.value.paused) {
      safePlay();
    }
  };

  const togglePlay = () => {
    if (!videoElement.value) return;
    if (currentStreamSource.value && !isMediaAttached.value) {
      startPlayback();
      return;
    }
    if (videoElement.value.paused) {
      safePlay();
    } else {
      videoElement.value.pause();
    }
  };

  const bindVideoElement = (el: HTMLVideoElement | null) => {
    videoElement.value = el;
    if (el) {
      el.muted = isMuted.value;
      el.volume = volume.value;
      el.onplay = () => {
        isPlaying.value = true;
      };
      el.onpause = () => {
        isPlaying.value = false;
      };
      el.ontimeupdate = () => {
        currentTime.value = el.currentTime;
      };
      el.onloadedmetadata = () => {
        if (!isNaN(el.duration) && isFinite(el.duration)) {
          duration.value = el.duration;
        }
        if (el.videoWidth > 0 && el.videoHeight > 0) {
          const w = el.videoWidth;
          const h = el.videoHeight;
          const tier = h >= 2160 ? '4K' : h >= 1080 ? '1080p' : h >= 720 ? '720p' : h >= 480 ? '480p' : '';
          const resStr = tier ? `${w}x${h} (${tier})` : `${w}x${h}`;
          if (streamDetails.value) {
            if (!streamDetails.value.resolution || streamDetails.value.resolution.includes('未知') || streamDetails.value.resolution.includes('需 ffprobe')) {
              const updated = {
                ...streamDetails.value,
                resolution: resStr,
              };
              streamDetails.value = updated;
              if (selectedChannel.value) {
                setCachedProbe(selectedChannel.value, updated);
              }
            }
          }
        }
      };
      el.ondurationchange = () => {
        if (!isNaN(el.duration) && isFinite(el.duration)) {
          duration.value = el.duration;
        }
      };
      el.onvolumechange = () => {
        volume.value = el.volume;
        isMuted.value = el.muted;
      };
      el.onerror = () => {
        const mediaErr = el.error;
        if (!mediaErr) {
          streamError.value = `播放器發生未知錯誤。${FALLBACK_HINT}`;
          return;
        }
        streamError.value = getMediaErrorMessage(mediaErr.code);
      };

      if (autoPlay && currentStreamSource.value && !isMediaAttached.value) {
        startPlayback();
      }
    }
  };

  const toggleMute = () => {
    if (!videoElement.value) return;
    videoElement.value.muted = !videoElement.value.muted;
    isMuted.value = videoElement.value.muted;
  };

  const seekTo = (seconds: number) => {
    if (!videoElement.value) return;
    const target = Math.max(0, Math.min(seconds, duration.value || seconds));
    videoElement.value.currentTime = target;
    currentTime.value = target;
  };

  const setVolume = (val: number) => {
    if (!videoElement.value) return;
    const clamped = Math.max(0, Math.min(1, val));
    videoElement.value.volume = clamped;
    volume.value = clamped;
    if (clamped > 0 && videoElement.value.muted) {
      videoElement.value.muted = false;
      isMuted.value = false;
    }
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
    let details: StreamDetails;
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      const session = await invoke<StreamSession>('prepare_episode_stream', {
        dramaId,
        epId,
        channel,
        sourceId: sourceId || metadata.value?.source_id || 'chinaq',
        source_id: sourceId || metadata.value?.source_id || 'chinaq',
      });
      details = session.stream_details;
    } else {
      details = {
        resolution: '1920x1080 (1080p)',
        bitrate: '3,200 kbps',
        video_codec: 'H.264 / AVC',
        audio_codec: 'AAC',
        fps: 30,
        duration_secs: 2700,
      };
    }
    setCachedProbe(channel, details);
    return details;
  };

  const loadPreview = async (
    arg1?: number | string,
    arg2?: string,
    arg3?: number,
    arg4?: string,
    arg5?: string
  ) => {
    let epNumber: number;
    let dramaId: string;
    let epId: string;
    let channel: string;
    let sourceId: string | undefined;

    if (typeof arg1 === 'number') {
      epNumber = arg1;
      channel = arg2 || selectedChannel.value;
      dramaId = metadata.value?.drama_id || '';
      const ep = metadata.value?.episodes.find((e) => e.ep_number === epNumber);
      epId = ep ? ep.ep_id : `ep${epNumber}`;
      sourceId = metadata.value?.source_id;
    } else if (typeof arg1 === 'string') {
      dramaId = arg1;
      epId = arg2 || '';
      epNumber = typeof arg3 === 'number' ? arg3 : activePreviewEp.value;
      channel = arg4 || selectedChannel.value;
      sourceId = arg5 || metadata.value?.source_id;
    } else {
      epNumber = activePreviewEp.value;
      channel = selectedChannel.value;
      dramaId = metadata.value?.drama_id || '';
      const ep = metadata.value?.episodes.find((e) => e.ep_number === epNumber);
      epId = ep ? ep.ep_id : `ep${epNumber}`;
      sourceId = metadata.value?.source_id;
    }

    activePreviewEp.value = epNumber;
    loadingStream.value = true;
    streamError.value = '';
    currentTime.value = 0;
    duration.value = 0;
    streamSession.value = null;
    streamDetails.value = null;

    cleanup();

    try {
      let session: StreamSession;

      if (isTauri()) {
        const { invoke } = await import('@tauri-apps/api/core');
        session = await invoke<StreamSession>('prepare_episode_stream', {
          dramaId,
          epId,
          channel,
          sourceId: sourceId || 'chinaq',
          source_id: sourceId || 'chinaq',
        });
      } else {
        const referer = sourceId === 'juqing'
          ? `https://tw.juqing.app/vod/play/id/${dramaId}-${channel}-${epId}.html`
          : `https://chinaq.fun/tv-cn/${dramaId}-${epId}.html`;
        session = {
          m3u8_url: `https://mock.stream/${dramaId}/${epId}.m3u8`,
          referer,
          proxy_url: `http://127.0.0.1:8899/proxy?url=https://mock.stream/${dramaId}/${epId}.m3u8`,
          stream_details: {
            resolution: '1920x1080 (1080p)',
            bitrate: '3,200 kbps',
            video_codec: 'H.264 / AVC',
            audio_codec: 'AAC',
            fps: 30,
            duration_secs: 2700,
          },
        };
      }

      streamSession.value = session;
      previewStreamInfo.value = {
        m3u8_url: session.m3u8_url,
        referer: session.referer,
        proxy_url: session.proxy_url,
      };

      // Preserve channel quality probe metrics in session cache across channel switching without requiring re-probing
      if (channel && channelProbeMap.value[channel]) {
        streamDetails.value = channelProbeMap.value[channel];
      } else {
        streamDetails.value = session.stream_details;
        if (channel) {
          setCachedProbe(channel, session.stream_details);
        }
      }

      if (streamDetails.value.duration_secs && streamDetails.value.duration_secs > 0) {
        duration.value = streamDetails.value.duration_secs;
      }

      const streamSource = session.proxy_url || session.m3u8_url;
      currentStreamSource.value = streamSource;

      if (autoPlay) {
        startPlayback();
      }
    } catch (err: unknown) {
      streamError.value = getErrorMessage(err);
    } finally {
      loadingStream.value = false;
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
          source_id: 'chinaq',
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
          url: 'https://chinaq.fun/tv-kr/202639642/',
          source_id: 'chinaq',
          source_name: 'ChinaQ',
          cover_url: 'https://chinaq.fun/img/202639642.jpg',
        },
      ];
    }
  };

  const resetSession = () => {
    cleanup();
    metadata.value = null;
    selectedChannel.value = '';
    selectedEpisodes.value = [];
    clearProbeCache();
    error.value = '';
    activePreviewEp.value = 1;
    streamSession.value = null;
    streamDetails.value = null;
    previewStreamInfo.value = null;
    currentStreamSource.value = '';
    streamError.value = '';
    currentTime.value = 0;
    duration.value = 0;
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
    activePreviewEp,
    streamSession,
    streamDetails,
    previewStreamInfo,
    currentStreamSource,
    isMediaAttached,
    loadingStream,
    streamError,
    isPlaying,
    isMuted,
    currentTime,
    duration,
    volume,
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
    startPlayback,
    togglePlay,
    toggleMute,
    seekTo,
    setVolume,
    cleanup,
  };
}

export type SeriesSession = ReturnType<typeof useSeriesSession>;
