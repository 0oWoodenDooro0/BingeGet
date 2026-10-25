import { ref, shallowRef } from 'vue';
import Hls from 'hls.js';
import type { StreamDetails, PreviewStreamInfo } from '../types';
import { isTauri, getErrorMessage } from './useSystemEnvironment';

export interface UseEpisodePreviewOptions {
  autoPlay?: boolean;
}

export function useEpisodePreview(options: UseEpisodePreviewOptions = {}) {
  const autoPlay = options.autoPlay === true;
  const activePreviewEp = ref<number>(1);
  const streamDetails = ref<StreamDetails | null>(null);
  const previewStreamInfo = ref<PreviewStreamInfo | null>(null);
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
          error.value = '播放器發生未知錯誤';
          return;
        }
        const code = mediaErr.code;
        if (code === 1) {
          error.value = '影片載入已中止';
        } else if (code === 2) {
          error.value = '網路連線異常，無法載入影片串流';
        } else if (code === 3) {
          error.value = '影片解碼失敗，可能是系統缺少解碼器 (如 Linux GStreamer 插件)';
        } else if (code === 4) {
          error.value = '不支援的影片格式或系統缺少相應解碼器 (如 Linux GStreamer 插件)';
        } else {
          error.value = `影片播放錯誤: ${mediaErr.message || '未知錯誤'}`;
        }
      };
    }
  };

  const togglePlay = () => {
    if (!videoElement.value) return;
    if (videoElement.value.paused) {
      videoElement.value.play().catch((err) => {
        error.value = `播放失敗: ${getErrorMessage(err)}`;
      });
    } else {
      videoElement.value.pause();
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

      if (videoElement.value && streamSource) {
        if (Hls.isSupported()) {
          const hls = new Hls({
            enableWorker: false,
          });
          hls.loadSource(streamSource);
          hls.attachMedia(videoElement.value);
          hls.on(Hls.Events.MANIFEST_PARSED, () => {
            if (autoPlay) {
              videoElement.value?.play().catch((err) => {
                error.value = `播放失敗: ${getErrorMessage(err)}`;
              });
            }
          });
          hls.on(Hls.Events.ERROR, (_, data) => {
            if (data.fatal) {
              switch (data.type) {
                case Hls.ErrorTypes.NETWORK_ERROR:
                  error.value = `串流網路錯誤 (${data.details})，請檢查網路連線`;
                  break;
                case Hls.ErrorTypes.MEDIA_ERROR:
                  error.value = `媒體解碼或處理錯誤 (${data.details})，可能是系統缺少解碼器 (如 Linux GStreamer 插件)`;
                  break;
                default:
                  error.value = `串流播放失敗: ${data.details}`;
                  break;
              }
            }
          });
          hlsInstance.value = hls;
        } else if (
          typeof videoElement.value.canPlayType === 'function' &&
          videoElement.value.canPlayType('application/vnd.apple.mpegurl')
        ) {
          videoElement.value.src = streamSource;
          if (autoPlay) {
            videoElement.value.play().catch((err) => {
              error.value = `播放失敗: ${getErrorMessage(err)}`;
            });
          }
        }
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
    loading,
    error,
    isPlaying,
    isMuted,
    currentTime,
    duration,
    volume,
    bindVideoElement,
    loadPreview,
    togglePlay,
    toggleMute,
    seekTo,
    setVolume,
    cleanup,
  };
}
