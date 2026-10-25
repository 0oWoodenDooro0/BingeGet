import { ref, shallowRef } from 'vue';
import Hls from 'hls.js';
import type { StreamDetails, PreviewStreamInfo } from '../types';
import { isTauri, getErrorMessage } from './useSystemEnvironment';

export interface UseEpisodePreviewOptions {
  autoPlay?: boolean;
}

export function useEpisodePreview(options: UseEpisodePreviewOptions = {}) {
  const activePreviewEp = ref<number>(1);
  const streamDetails = ref<StreamDetails | null>(null);
  const previewStreamInfo = ref<PreviewStreamInfo | null>(null);
  const loading = ref<boolean>(false);
  const error = ref<string>('');
  const isPlaying = ref<boolean>(false);
  const isMuted = ref<boolean>(true);

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
      el.onplay = () => {
        isPlaying.value = true;
      };
      el.onpause = () => {
        isPlaying.value = false;
      };
    }
  };

  const togglePlay = () => {
    if (!videoElement.value) return;
    if (videoElement.value.paused) {
      videoElement.value.play().catch(() => {});
    } else {
      videoElement.value.pause();
    }
  };

  const toggleMute = () => {
    if (!videoElement.value) return;
    videoElement.value.muted = !videoElement.value.muted;
    isMuted.value = videoElement.value.muted;
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
        };
        details = {
          resolution: '1920x1080 (1080p)',
          bitrate: '3,200 kbps',
          video_codec: 'H.264 / AVC',
          audio_codec: 'AAC',
          fps: 30,
          has_master_playlist: true,
        };
      }

      previewStreamInfo.value = info;
      streamDetails.value = details;

      if (videoElement.value && info.m3u8_url) {
        if (Hls.isSupported()) {
          const hls = new Hls({
            enableWorker: false,
            xhrSetup: (xhr) => {
              if (info.referer) {
                try {
                  xhr.setRequestHeader('X-Referer', info.referer);
                } catch {
                  // Browser security may ignore restricted headers
                }
              }
            },
          });
          hls.loadSource(info.m3u8_url);
          hls.attachMedia(videoElement.value);
          hls.on(Hls.Events.MANIFEST_PARSED, () => {
            if (options.autoPlay !== false) {
              videoElement.value?.play().catch(() => {});
            }
          });
          hls.on(Hls.Events.ERROR, (_, data) => {
            if (data.fatal) {
              error.value = `播放錯誤: ${data.details}`;
            }
          });
          hlsInstance.value = hls;
        } else if (
          typeof videoElement.value.canPlayType === 'function' &&
          videoElement.value.canPlayType('application/vnd.apple.mpegurl')
        ) {
          videoElement.value.src = info.m3u8_url;
          if (options.autoPlay !== false) {
            videoElement.value.play().catch(() => {});
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
    bindVideoElement,
    loadPreview,
    togglePlay,
    toggleMute,
    cleanup,
  };
}
