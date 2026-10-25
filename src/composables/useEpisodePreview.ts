import { ref, shallowRef } from 'vue';
import Hls from 'hls.js';
import type { StreamDetails, PreviewStreamInfo } from '../types';
import { isTauri, getErrorMessage } from './useSystemEnvironment';

export interface UseEpisodePreviewOptions {
  autoPlay?: boolean;
}

const MEDIA_ERR_ABORTED = typeof MediaError !== 'undefined' ? MediaError.MEDIA_ERR_ABORTED : 1;
const MEDIA_ERR_NETWORK = typeof MediaError !== 'undefined' ? MediaError.MEDIA_ERR_NETWORK : 2;
const MEDIA_ERR_DECODE = typeof MediaError !== 'undefined' ? MediaError.MEDIA_ERR_DECODE : 3;
const MEDIA_ERR_SRC_NOT_SUPPORTED = typeof MediaError !== 'undefined' ? MediaError.MEDIA_ERR_SRC_NOT_SUPPORTED : 4;

const FALLBACK_HINT = '建議檢查本機解碼器或複製串流網址至外部播放器 (如 VLC / mpv) 播放';

const MEDIA_ERROR_MESSAGES: Record<number, string> = {
  [MEDIA_ERR_ABORTED]: '影片載入已中止',
  [MEDIA_ERR_NETWORK]: `網路連線異常，無法載入影片串流。${FALLBACK_HINT}`,
  [MEDIA_ERR_DECODE]: `影片解碼失敗，可能是系統缺少解碼器 (如 Linux GStreamer 外掛程式)。${FALLBACK_HINT}`,
  [MEDIA_ERR_SRC_NOT_SUPPORTED]: `不支援的影片格式或系統缺少相應解碼器 (如 Linux GStreamer 外掛程式)。${FALLBACK_HINT}`,
};

export function useEpisodePreview(options: UseEpisodePreviewOptions = {}) {
  const autoPlay = options.autoPlay === true;
  const activePreviewEp = ref<number>(1);
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
        const matchedMsg = MEDIA_ERROR_MESSAGES[mediaErr.code];
        if (matchedMsg) {
          error.value = matchedMsg;
        } else {
          error.value = `影片播放錯誤: ${mediaErr.message || '未知錯誤'}。${FALLBACK_HINT}`;
        }
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

  const loadPreview = async (
    dramaId: string,
    epId: string,
    epNumber: number,
    channel: string
  ) => {
    activePreviewEp.value = epNumber;
    loading.value = true;
    error.value = '';
    currentTime.value = 0;
    duration.value = 0;

    cleanup();

    try {
      let info: PreviewStreamInfo;
      let details: StreamDetails;

      if (isTauri()) {
        const { invoke } = await import('@tauri-apps/api/core');
        const [streamRes, probeRes] = await Promise.all([
          invoke<PreviewStreamInfo>('get_episode_preview_stream', {
            dramaId,
            epId,
            channel,
          }),
          invoke<StreamDetails>('probe_episode_stream', {
            dramaId,
            epId,
            channel,
          }),
        ]);
        info = streamRes;
        details = probeRes;
      } else {
        // Fallback for preview / dev / tests
        info = {
          m3u8_url: `https://mock.stream/${dramaId}/${epId}.m3u8`,
          referer: `https://chinaq.fun/tv-cn/${dramaId}-${epId}.html`,
          proxy_url: `http://127.0.0.1:8899/proxy?url=https://mock.stream/${dramaId}/${epId}.m3u8`,
        };
        details = {
          resolution: '1920x1080 (1080p)',
          bitrate: '3,200 kbps',
          video_codec: 'H.264 / AVC',
          audio_codec: 'AAC',
          fps: 30,
          duration_secs: 2700,
        };
      }

      previewStreamInfo.value = info;
      streamDetails.value = details;
      if (details.duration_secs && details.duration_secs > 0) {
        duration.value = details.duration_secs;
      }

      const streamSource = info.proxy_url || info.m3u8_url;
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
    startPlayback,
    togglePlay,
    toggleMute,
    seekTo,
    setVolume,
    cleanup,
  };
}
