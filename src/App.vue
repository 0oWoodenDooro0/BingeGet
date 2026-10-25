<script setup lang="ts">
import { ref } from 'vue';
import { Download, AlertTriangle, CheckCircle2, ShieldAlert } from 'lucide-vue-next';
import HeaderBar from './components/HeaderBar.vue';
import UrlBar from './components/UrlBar.vue';
import SeriesConfig from './components/SeriesConfig.vue';
import ChannelSelector from './components/ChannelSelector.vue';
import EpisodePicker from './components/EpisodePicker.vue';
import DownloadQueue from './components/DownloadQueue.vue';
import {
  useSystemEnvironment,
  useSeriesMetadata,
  useDownloadQueue,
  getErrorMessage,
} from './composables';
import type { SeriesConfigModel, SearchResult } from './types';

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

// Download queue state and event coordination composable
const {
  queue,
  startDownload: triggerStartDownload,
  retryDownload,
  cancelDownload,
} = useDownloadQueue();

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

  try {
    if (!metadata.value) return;
    await triggerStartDownload({
      metadata: metadata.value,
      selectedEpisodes: selectedEpisodes.value,
      selectedChannel: selectedChannel.value,
      config: config.value,
    });
  } catch (err: unknown) {
    errorMsg.value = getErrorMessage(err);
  }
};

const handleRetry = (ep: number) => {
  retryDownload(ep);
};

const handleCancel = (ep: number) => {
  cancelDownload(ep);
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
