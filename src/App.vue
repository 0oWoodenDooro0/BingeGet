<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Download, AlertTriangle, CheckCircle2, ShieldAlert } from 'lucide-vue-next';
import HeaderBar from './components/HeaderBar.vue';
import UrlBar from './components/UrlBar.vue';
import SeriesConfig from './components/SeriesConfig.vue';
import ChannelSelector from './components/ChannelSelector.vue';
import EpisodePicker from './components/EpisodePicker.vue';
import DownloadQueue from './components/DownloadQueue.vue';
import { useSystemEnvironment, useSeriesMetadata, isTauri, getErrorMessage } from './composables';
import type { QueueItem, SeriesConfigModel, SearchResult } from './types';

const loading = ref(false);
const errorMsg = ref('');
const successMsg = ref('');

// System environment detection composable
const { envInfo } = useSystemEnvironment();

// Series metadata & channel/episode selection composable
const {
  metadata,
  selectedChannel,
  selectedEpisodes,
  parseUrl,
  searchSeries,
} = useSeriesMetadata();

// User inputs model (bundled to avoid data clumps)
const config = ref<SeriesConfigModel>({
  customTitle: '',
  season: 1,
  saveDir: '/home/user/Downloads',
  threads: 16,
  maxConcurrent: 2,
});

// Queue state
const queue = ref<QueueItem[]>([]);

// Setup download event listeners on mount
onMounted(async () => {
  if (isTauri()) {
    try {
      const { listen } = await import('@tauri-apps/api/event');

      // Listen for download progress updates from Rust backend
      await listen<{ episode: number; percent: number; speed: string; eta: string }>(
        'download-progress',
        (event) => {
          const item = queue.value.find(q => q.spec.episode === event.payload.episode);
          if (item) {
            item.state = {
              type: 'Progress',
              data: {
                percent: event.payload.percent,
                speed: event.payload.speed,
                eta: event.payload.eta,
              },
            };
          }
        }
      );

      // Listen for download status updates from Rust backend
      await listen<{ episode: number; status: string; error?: string }>(
        'download-status',
        (event) => {
          const item = queue.value.find(q => q.spec.episode === event.payload.episode);
          if (item) {
            if (event.payload.status === 'Completed') {
              item.state = { type: 'Completed' };
            } else if (event.payload.status === 'Downloading') {
              item.state = { type: 'Downloading' };
            } else if (event.payload.status === 'Failed') {
              item.state = { type: 'Failed', error: event.payload.error || '下載失敗' };
            }
          }
        }
      );
    } catch (err: unknown) {
      console.warn('Tauri initialization note:', getErrorMessage(err));
    }
  }
});

// Handle URL parse
const handleParse = async (url: string) => {
  loading.value = true;
  errorMsg.value = '';
  successMsg.value = '';

  try {
    const data = await parseUrl(url);
    successMsg.value = `解析成功，共 ${data.episodes.length} 集。請確認劇名與集數後即可開始下載。`;
  } catch (err: unknown) {
    errorMsg.value = `解析失敗: ${getErrorMessage(err)}`;
  } finally {
    loading.value = false;
  }
};

// Handle series search result selection
const handleSelectResult = async (result: SearchResult) => {
  if (result.title) {
    config.value.customTitle = result.title;
  }
  await handleParse(result.url);
};

// Start download
const startDownload = async () => {
  if (!config.value.customTitle.trim()) {
    errorMsg.value = '請輸入劇名！';
    return;
  }

  if (selectedEpisodes.value.length === 0) {
    errorMsg.value = '請至少選擇一集！';
    return;
  }

  errorMsg.value = '';

  const categorySlug = metadata.value?.category_slug || 'tv-cn';
  const newItems: QueueItem[] = selectedEpisodes.value.map((ep, idx) => ({
    id: idx + 1,
    spec: {
      m3u8_url: '',
      custom_title: config.value.customTitle.trim(),
      season: config.value.season,
      episode: ep,
      save_dir: config.value.saveDir,
      referer: `https://chinaq.fun/${categorySlug}/${metadata.value?.drama_id}/ep${ep}.html`,
    },
    state: { type: 'Queued' },
  }));

  queue.value = newItems;

  if (isTauri()) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('start_download_queue', {
        req: {
          drama_id: metadata.value?.drama_id,
          category_slug: categorySlug,
          channel_id: selectedChannel.value,
          episodes: selectedEpisodes.value,
          custom_title: config.value.customTitle.trim(),
          season: config.value.season,
          save_dir: config.value.saveDir,
          threads: config.value.threads,
          max_concurrent: config.value.maxConcurrent,
          binary_path: config.value.binaryPath || null,
        }
      });
    } catch (err: unknown) {
      errorMsg.value = `下載啟動失敗: ${getErrorMessage(err)}`;
    }
  } else {
    // Simulated queue progress for web preview
    simulateQueueProgress();
  }
};

const simulateQueueProgress = () => {
  let activeIndex = 0;
  const interval = setInterval(() => {
    if (activeIndex >= queue.value.length) {
      clearInterval(interval);
      return;
    }

    const current = queue.value[activeIndex];
    if (current.state.type === 'Queued') {
      current.state = {
        type: 'Progress',
        data: { percent: 20, speed: '18.4 MB/s', eta: '00:01:05' }
      };
    } else if (current.state.type === 'Progress') {
      const nextPct = current.state.data.percent + 40;
      if (nextPct >= 100) {
        current.state = { type: 'Muxing' };
        setTimeout(() => {
          current.state = { type: 'Completed' };
          activeIndex++;
        }, 800);
      } else {
        current.state.data.percent = nextPct;
      }
    }
  }, 1000);
};

const handleRetry = (ep: number) => {
  const item = queue.value.find(q => q.spec.episode === ep);
  if (item) {
    item.state = { type: 'Queued' };
  }
};

const handleCancel = (ep: number) => {
  const item = queue.value.find(q => q.spec.episode === ep);
  if (item) {
    item.state = { type: 'Cancelled' };
  }
};
</script>

<template>
  <div class="min-h-screen bg-slate-950 text-slate-100 flex flex-col font-sans">
    <HeaderBar />

    <main class="flex-1 max-w-5xl w-full mx-auto p-4 sm:p-6 space-y-5">
      <!-- Dependency Alert Banner (only visible if environment has missing dependencies) -->
      <div v-if="envInfo && !envInfo.is_ready" class="flex items-center justify-between rounded-lg border border-amber-500/30 bg-amber-950/20 px-3.5 py-2 text-xs text-amber-300">
        <div class="flex items-center gap-2">
          <ShieldAlert class="h-4 w-4 text-amber-400 shrink-0" />
          <span>
            缺少必要依賴：請安裝
            <template v-if="!envInfo.nm3u8dl_path && !envInfo.ffmpeg_path">
              <code class="font-mono">n-m3u8dl-re</code> 與 <code class="font-mono">ffmpeg</code>
            </template>
            <template v-else-if="!envInfo.nm3u8dl_path">
              <code class="font-mono">n-m3u8dl-re</code>
            </template>
            <template v-else-if="!envInfo.ffmpeg_path">
              <code class="font-mono">ffmpeg</code>
            </template>
            以進行下載。
          </span>
        </div>
        <span class="font-medium">
          環境未就緒
        </span>
      </div>

      <!-- Error / Alert Banner -->
      <div v-if="errorMsg" class="flex items-center gap-2 rounded-lg border border-rose-500/30 bg-rose-500/10 p-3 text-sm text-rose-300">
        <AlertTriangle class="h-4 w-4 shrink-0 text-rose-400" />
        <span>{{ errorMsg }}</span>
      </div>

      <!-- Success Alert Banner -->
      <div v-if="successMsg" class="flex items-center gap-2 rounded-lg border border-emerald-500/30 bg-emerald-500/10 p-3 text-sm text-emerald-300">
        <CheckCircle2 class="h-4 w-4 shrink-0 text-emerald-400" />
        <span>{{ successMsg }}</span>
      </div>

      <!-- URL & Search Input Bar -->
      <UrlBar
        :loading="loading"
        :on-search="searchSeries"
        @parse="handleParse"
        @select-result="handleSelectResult"
      />

      <!-- Metadata & Configuration Section -->
      <template v-if="metadata">
        <SeriesConfig
          v-model:config="config"
          :totalEpisodes="metadata.episodes.length"
        />

        <ChannelSelector
          :channels="metadata.channels"
          v-model:selectedChannel="selectedChannel"
        />

        <EpisodePicker
          :episodes="metadata.episodes"
          v-model:selectedEpisodes="selectedEpisodes"
        />

        <!-- Primary Action -->
        <div class="flex items-center justify-end pt-2">
          <button
            type="button"
            @click="startDownload"
            :disabled="selectedEpisodes.length === 0 || !config.customTitle.trim()"
            class="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-indigo-600 to-violet-600 px-8 py-3.5 text-base font-semibold text-white shadow-xl shadow-indigo-600/25 transition-all hover:from-indigo-500 hover:to-violet-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 disabled:cursor-not-allowed disabled:opacity-50"
          >
            <Download class="h-5 w-5" />
            <span>開始下載 (已選 {{ selectedEpisodes.length }} 集)</span>
          </button>
        </div>
      </template>

      <!-- Download Queue -->
      <DownloadQueue
        :queue="queue"
        @retry="handleRetry"
        @cancel="handleCancel"
      />
    </main>
  </div>
</template>
