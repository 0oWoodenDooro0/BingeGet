import { ref, shallowRef } from 'vue';
import Hls from 'hls.js';
import type { StreamDetails, PreviewStreamInfo, StreamSession } from '../types';
import { FALLBACK_HINT, getMediaErrorMessage } from '../types';
import { isTauri, getErrorMessage } from './useSystemEnvironment';

export interface UseEpisodePreviewOptions {
  autoPlay?: boolean;
}

export function useEpisodePreview(options: UseEpisodePreviewOptions = {}) {
  const autoPlay = options.autoPlay === true;
  const activePreviewEp = ref<number>(1);
  const streamSession = ref<StreamSession | null>(null);
  const streamDetails = ref<StreamDetails | null>(null);
  const previewStreamInfo = ref<PreviewStreamInfo | null>(null);
  const currentStreamSource = ref<string>('');
  const isMediaAttached = ref<boolean>(false);
  const loading = ref<boolean>(false);
  const error = ref<string>('');
  const isPlaying = ref<boolean>(false);
  const isMuted = ref<boolean>(true);
  const currentTime = ref<number>(0);
  const duration = ref<number>(0);
  const volume = ref<number>(1);

  const hlsInstance = shallowRef<Hls | null>(null);
  const videoElement = shallowRef<HTMLVideoElement | null>(null);

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
      error.value = `播放失敗: ${getErrorMessage(err)}。${FALLBACK_HINT}`;
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
              error.value = `串流網路錯誤 (${data.details})，請檢查網路連線`;
              break;
            case Hls.ErrorTypes.MEDIA_ERROR:
              error.value = `媒體解碼或處理錯誤 (${data.details})，可能是系統缺少解碼器 (如 Linux GStreamer 外掛程式)。${FALLBACK_HINT}`;
              break;
            default:
              error.value = `串流播放失敗: ${data.details}。${FALLBACK_HINT}`;
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
              streamDetails.value = {
                ...streamDetails.value,
                resolution: resStr,
              };
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
          error.value = `播放器發生未知錯誤。${FALLBACK_HINT}`;
          return;
        }
        error.value = getMediaErrorMessage(mediaErr.code);
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
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      const session = await invoke<StreamSession>('prepare_episode_stream', {
        dramaId,
        epId,
        channel,
        sourceId: sourceId || 'chinaq',
        source_id: sourceId || 'chinaq',
      });
      return session.stream_details;
    }
    return {
      resolution: '1920x1080 (1080p)',
      bitrate: '3,200 kbps',
      video_codec: 'H.264 / AVC',
      audio_codec: 'AAC',
      fps: 30,
      duration_secs: 2700,
    };
  };

  const loadPreview = async (
    dramaId: string,
    epId: string,
    epNumber: number,
    channel: string,
    sourceId?: string
  ) => {
    activePreviewEp.value = epNumber;
    loading.value = true;
    error.value = '';
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
        // Fallback for preview / dev / tests
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
      streamDetails.value = session.stream_details;
      if (session.stream_details.duration_secs && session.stream_details.duration_secs > 0) {
        duration.value = session.stream_details.duration_secs;
      }

      const streamSource = session.proxy_url || session.m3u8_url;
      currentStreamSource.value = streamSource;

      if (autoPlay) {
        startPlayback();
      }
    } catch (err: unknown) {
      error.value = getErrorMessage(err);
    } finally {
      loading.value = false;
    }
  };

  return {
    activePreviewEp,
    streamSession,
    streamDetails,
    previewStreamInfo,
    currentStreamSource,
    isMediaAttached,
    loading,
    error,
    isPlaying,
    isMuted,
    currentTime,
    duration,
    volume,
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
