<script setup lang="ts">
import { ref } from 'vue';
import { Download, AlertTriangle, CheckCircle2 } from 'lucide-vue-next';
import HeaderBar from './components/HeaderBar.vue';
import UrlBar from './components/UrlBar.vue';
import SeriesConfig from './components/SeriesConfig.vue';
import ChannelSelector from './components/ChannelSelector.vue';
import EpisodePicker from './components/EpisodePicker.vue';
import DownloadQueue from './components/DownloadQueue.vue';
import type { SeriesMetadata, QueueItem } from './types';

const loading = ref(false);
const errorMsg = ref('');
const successMsg = ref('');

// Series state
const metadata = ref<SeriesMetadata | null>(null);
const selectedChannel = ref('');
const selectedEpisodes = ref<number[]>([]);

// User inputs
const customTitle = ref('');
const season = ref(1);
const saveDir = ref('/home/user/Downloads/BingeGet');
const threads = ref(16);
const maxConcurrent = ref(2);

// Queue state
const queue = ref<QueueItem[]>([]);

// Tauri invoke check
const isTauri = () => {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
};

// Handle URL parse
const handleParse = async (url: string) => {
  loading.value = true;
  errorMsg.value = '';
  successMsg.value = '';

  try {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      const data = await invoke<SeriesMetadata>('parse_series_metadata', { url });
      metadata.value = data;
    } else {
      // Direct API or mock fallback for browser preview mode
      const dramaIdMatch = url.match(/\/tv-cn\/([a-zA-Z0-9]+)/);
      const dramaId = dramaIdMatch ? dramaIdMatch[1] : '202614235';
      
      // Simulate/fallback parsing
      metadata.value = {
        drama_id: dramaId,
        episodes: Array.from({ length: 36 }, (_, i) => ({
          ep_number: i + 1,
          ep_id: `ep${i + 1}`,
        })),
        channels: [
          { id: 'ikzy', name: 'iKun 資源 (ikzy) - 推薦' },
          { id: 'hnzy', name: '紅牛 資源 (hnzy)' },
          { id: 'jszy', name: '極速 資源 (jszy)' },
          { id: 'jyzy', name: '金鷹 資源 (jyzy)' },
        ],
      };
    }

    if (metadata.value && metadata.value.channels.length > 0) {
      selectedChannel.value = metadata.value.channels[0].id;
    }

    if (metadata.value) {
      selectedEpisodes.value = metadata.value.episodes.map(e => e.ep_number);
      if (!customTitle.value) {
        customTitle.value = '花開錦繡'; // default prompt for user to edit
      }
    }

    successMsg.value = `成功解析影集，共找到 ${metadata.value?.episodes.length || 0} 集！請確認劇名後開始下載。`;
  } catch (err: any) {
    errorMsg.value = `解析失敗: ${err.message || String(err)}`;
  } finally {
    loading.value = false;
  }
};

// Start download
const startDownload = async () => {
  if (!customTitle.value.trim()) {
    errorMsg.value = '請填寫自訂劇名，避免儲存為未命名檔案！';
    return;
  }

  if (selectedEpisodes.value.length === 0) {
    errorMsg.value = '請至少勾選一集！';
    return;
  }

  errorMsg.value = '';

  const newItems: QueueItem[] = selectedEpisodes.value.map((ep, idx) => ({
    id: idx + 1,
    spec: {
      m3u8_url: '',
      custom_title: customTitle.value.trim(),
      season: season.value,
      episode: ep,
      save_dir: saveDir.value,
      referer: `https://chinaq.fun/tv-cn/${metadata.value?.drama_id}/ep${ep}.html`,
    },
    state: { type: 'Queued' },
  }));

  queue.value = newItems;

  if (isTauri()) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('start_download_queue', {
        dramaId: metadata.value?.drama_id,
        channelId: selectedChannel.value,
        episodes: selectedEpisodes.value,
        customTitle: customTitle.value.trim(),
        season: season.value,
        saveDir: saveDir.value,
        threads: threads.value,
        maxConcurrent: maxConcurrent.value,
      });
    } catch (err: any) {
      errorMsg.value = `下載啟動失敗: ${err.message || String(err)}`;
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
        data: { percent: 15, speed: '16.4 MB/s', eta: '00:01:20' }
      };
    } else if (current.state.type === 'Progress') {
      const nextPct = current.state.data.percent + 30;
      if (nextPct >= 100) {
        current.state = { type: 'Muxing' };
        setTimeout(() => {
          current.state = { type: 'Completed' };
          activeIndex++;
        }, 1000);
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

      <!-- URL Input Bar -->
      <UrlBar :loading="loading" @parse="handleParse" />

      <!-- Metadata & Configuration Section -->
      <template v-if="metadata">
        <SeriesConfig
          v-model:customTitle="customTitle"
          v-model:season="season"
          v-model:saveDir="saveDir"
          v-model:threads="threads"
          v-model:maxConcurrent="maxConcurrent"
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
            :disabled="selectedEpisodes.length === 0 || !customTitle.trim()"
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
