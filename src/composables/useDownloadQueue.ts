import { ref, onMounted, onUnmounted, getCurrentInstance } from 'vue';
import type {
  QueueItem,
  SeriesConfigModel,
  SeriesMetadata,
  DownloadQueuePayload,
  ProgressPayload,
  EpisodeStatusPayload,
  DownloadProgress,
} from '../types';
import { isTauri, getErrorMessage } from './useSystemEnvironment';

export interface StartDownloadParams {
  metadata: SeriesMetadata;
  selectedEpisodes: number[];
  selectedChannel: string;
  config: SeriesConfigModel;
}

export interface UseDownloadQueueOptions {
  autoListen?: boolean;
}

export function useDownloadQueue(options: UseDownloadQueueOptions = {}) {
  const { autoListen = true } = options;

  const queue = ref<QueueItem[]>([]);
  const error = ref<string>('');

  const unlistenFns: Array<() => void> = [];
  let simulationInterval: ReturnType<typeof setInterval> | null = null;
  const timeoutIds: Array<ReturnType<typeof setTimeout>> = [];

  const updateProgress = (payload: ProgressPayload) => {
    const item = queue.value.find(q => q.spec.episode === payload.episode);
    if (item) {
      const progressData: DownloadProgress = {
        percent: payload.percent,
        speed: payload.speed,
        eta: payload.eta,
      };
      item.progress = progressData;
      item.state = {
        type: 'Progress',
        data: progressData,
      };
    }
  };

  const updateStatus = (payload: { episode: number; status: string; error?: string | null }) => {
    const item = queue.value.find(q => q.spec.episode === payload.episode);
    if (!item) return;

    const status = payload.status.toLowerCase();
    if (status === 'completed') {
      item.state = { type: 'Completed' };
      if (item.progress) {
        item.progress.percent = 100;
      }
    } else if (status === 'downloading') {
      item.state = { type: 'Downloading' };
    } else if (status === 'failed') {
      item.state = { type: 'Failed', error: payload.error || '下載失敗' };
    } else if (status === 'queued') {
      item.state = { type: 'Queued' };
    } else if (status === 'muxing') {
      item.state = { type: 'Muxing' };
    } else if (status === 'cancelled') {
      item.state = { type: 'Cancelled' };
    }
  };

  const setupListeners = async (): Promise<void> => {
    if (!isTauri()) {
      return;
    }

    try {
      const { listen } = await import('@tauri-apps/api/event');

      const unlistenProgress = await listen<ProgressPayload>(
        'download-progress',
        (event) => {
          if (event?.payload) {
            updateProgress(event.payload);
          }
        }
      );
      unlistenFns.push(unlistenProgress);

      const unlistenEpisodeStatus = await listen<EpisodeStatusPayload>(
        'episode-status',
        (event) => {
          if (event?.payload) {
            updateStatus(event.payload);
          }
        }
      );
      unlistenFns.push(unlistenEpisodeStatus);

      const unlistenDownloadStatus = await listen<EpisodeStatusPayload>(
        'download-status',
        (event) => {
          if (event?.payload) {
            updateStatus(event.payload);
          }
        }
      );
      unlistenFns.push(unlistenDownloadStatus);
    } catch (err: unknown) {
      console.warn('Tauri event listener initialization note:', getErrorMessage(err));
    }
  };

  const simulateQueueProgress = () => {
    if (simulationInterval !== null) {
      clearInterval(simulationInterval);
      simulationInterval = null;
    }

    let activeIndex = 0;
    simulationInterval = setInterval(() => {
      if (activeIndex >= queue.value.length) {
        if (simulationInterval !== null) {
          clearInterval(simulationInterval);
          simulationInterval = null;
        }
        return;
      }

      const current = queue.value[activeIndex];
      if (!current) return;

      if (current.state.type === 'Queued') {
        const prog: DownloadProgress = { percent: 20, speed: '18.4 MB/s', eta: '00:01:05' };
        current.progress = prog;
        current.state = {
          type: 'Progress',
          data: prog,
        };
      } else if (current.state.type === 'Progress') {
        const nextPct = current.state.data.percent + 40;
        if (nextPct >= 100) {
          current.state = { type: 'Muxing' };
          const timer = setTimeout(() => {
            current.state = { type: 'Completed' };
            if (current.progress) current.progress.percent = 100;
            activeIndex++;
          }, 800);
          timeoutIds.push(timer);
        } else {
          current.state.data.percent = nextPct;
          if (current.progress) current.progress.percent = nextPct;
        }
      }
    }, 1000);
  };

  const startDownload = async (params: StartDownloadParams): Promise<void> => {
    const { metadata, selectedEpisodes, selectedChannel, config } = params;

    if (!config?.customTitle?.trim()) {
      const msg = '請輸入劇名！';
      error.value = msg;
      throw new Error(msg);
    }

    if (!selectedEpisodes || selectedEpisodes.length === 0) {
      const msg = '請至少選擇一集！';
      error.value = msg;
      throw new Error(msg);
    }

    if (!selectedChannel || !selectedChannel.trim()) {
      const msg = '請選擇來源線路！';
      error.value = msg;
      throw new Error(msg);
    }

    error.value = '';

    const categorySlug = metadata?.category_slug || 'tv-cn';
    const newItems: QueueItem[] = selectedEpisodes.map((ep, idx) => ({
      id: idx + 1,
      spec: {
        m3u8_url: '',
        custom_title: config.customTitle.trim(),
        season: config.season,
        episode: ep,
        save_dir: config.saveDir,
        referer: `https://chinaq.fun/${categorySlug}/${metadata?.drama_id}/ep${ep}.html`,
      },
      state: { type: 'Queued' },
    }));

    queue.value = newItems;

    if (isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const payload: DownloadQueuePayload = {
          drama_id: metadata?.drama_id || '',
          category_slug: categorySlug,
          channel_id: selectedChannel,
          episodes: selectedEpisodes,
          custom_title: config.customTitle.trim(),
          season: config.season,
          save_dir: config.saveDir,
          threads: config.threads,
          max_concurrent: config.maxConcurrent,
          binary_path: config.binaryPath || null,
        };
        await invoke('start_download_queue', { req: payload });
      } catch (err: unknown) {
        const msg = `下載啟動失敗: ${getErrorMessage(err)}`;
        error.value = msg;
        throw new Error(msg);
      }
    } else {
      simulateQueueProgress();
    }
  };

  const retryDownload = (ep: number) => {
    const item = queue.value.find(q => q.spec.episode === ep);
    if (item) {
      item.state = { type: 'Queued' };
      if (item.progress) {
        item.progress.percent = 0;
      }
    }
  };

  const cancelDownload = (ep: number) => {
    const item = queue.value.find(q => q.spec.episode === ep);
    if (item) {
      item.state = { type: 'Cancelled' };
    }
  };

  const clearQueue = () => {
    queue.value = [];
  };

  const cleanup = () => {
    while (unlistenFns.length > 0) {
      const fn = unlistenFns.pop();
      try {
        fn?.();
      } catch (err) {
        console.warn('Error during unlisten:', err);
      }
    }

    if (simulationInterval !== null) {
      clearInterval(simulationInterval);
      simulationInterval = null;
    }

    while (timeoutIds.length > 0) {
      const tid = timeoutIds.pop();
      clearTimeout(tid);
    }
  };

  if (autoListen && getCurrentInstance()) {
    onMounted(async () => {
      await setupListeners();
    });

    onUnmounted(() => {
      cleanup();
    });
  }

  return {
    queue,
    error,
    startDownload,
    retryDownload,
    cancelDownload,
    clearQueue,
    setupListeners,
    cleanup,
    simulateQueueProgress,
    handleRetry: retryDownload,
    handleCancel: cancelDownload,
    updateProgress,
    updateStatus,
  };
}
