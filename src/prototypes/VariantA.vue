<script setup lang="ts">
import { ref, computed } from 'vue';
import {
  Layers,
  Sliders,
  Folder,
  Radio,
  Play,
  Pause,
  Volume2,
  VolumeX,
  Tv,
  Check,
  CheckSquare,
  Square,
  Copy,
  CheckCheck,
} from 'lucide-vue-next';
import type { SeriesMetadata, SeriesConfigModel, StreamDetails } from '../types';

const props = defineProps<{
  metadata: SeriesMetadata;
  config: SeriesConfigModel;
  selectedChannel: string;
  selectedEpisodes: number[];
  channelProbeMap: Record<string, StreamDetails>;
}>();

const emit = defineEmits<{
  (e: 'update:config', val: SeriesConfigModel): void;
  (e: 'update:selectedChannel', val: string): void;
  (e: 'update:selectedEpisodes', val: number[]): void;
}>();

// Local active preview state for throwaway exploration
const activePreviewEp = ref(1);
const isPlaying = ref(false);
const isMuted = ref(false);
const currentTime = ref(38);
const duration = ref(2700);
const volume = ref(0.8);
const copiedUrl = ref(false);

const currentChannel = computed(() => {
  return props.metadata.channels.find(c => c.id === props.selectedChannel) || props.metadata.channels[0];
});

const currentStreamDetails = computed<StreamDetails>(() => {
  return (
    props.channelProbeMap[props.selectedChannel] || {
      resolution: '1080p',
      bitrate: '3,200 kbps',
      video_codec: 'H.264',
      audio_codec: 'AAC',
      fps: 30,
      duration_secs: 2700,
    }
  );
});

const updateField = <K extends keyof SeriesConfigModel>(field: K, value: SeriesConfigModel[K]) => {
  emit('update:config', {
    ...props.config,
    [field]: value,
  });
};

const toggleEpisode = (epNum: number) => {
  const current = [...props.selectedEpisodes];
  const idx = current.indexOf(epNum);
  if (idx >= 0) {
    current.splice(idx, 1);
  } else {
    current.push(epNum);
  }
  emit('update:selectedEpisodes', current);
};

const selectAll = () => {
  emit('update:selectedEpisodes', props.metadata.episodes.map(e => e.ep_number));
};

const clearAll = () => {
  emit('update:selectedEpisodes', []);
};

const selectFirstN = (n: number) => {
  emit('update:selectedEpisodes', props.metadata.episodes.slice(0, n).map(e => e.ep_number));
};



const formatTime = (secs: number) => {
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${m < 10 ? '0' + m : m}:${s < 10 ? '0' + s : s}`;
};

const copyStreamUrl = () => {
  copiedUrl.value = true;
  setTimeout(() => (copiedUrl.value = false), 2000);
};
</script>

<template>
  <div class="space-y-5">
    <!-- 1. Series Config (Zero Artwork, Full Width Inputs) & Download Settings -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <!-- Naming Configuration (Item 1: 影集設定無圖片) -->
      <div class="rounded-xl border border-slate-800 bg-slate-900/40 p-4 space-y-3">
        <div class="flex items-center justify-between border-b border-slate-800/80 pb-2">
          <h2 class="text-xs font-semibold uppercase tracking-wider text-slate-400 flex items-center gap-1.5">
            <Layers class="h-4 w-4 text-indigo-400" />
            影集設定
          </h2>
          <span class="text-xs font-medium text-slate-400">
            共 {{ metadata.episodes.length }} 集
          </span>
        </div>

        <!-- Inputs take 100% width with no thumbnail -->
        <div class="space-y-2">
          <div class="grid grid-cols-3 gap-2">
            <div class="col-span-2">
              <label class="block text-xs text-slate-400 mb-1">
                劇名 <span class="text-rose-400">*</span>
              </label>
              <input
                :value="config.customTitle"
                @input="updateField('customTitle', ($event.target as HTMLInputElement).value)"
                type="text"
                placeholder="請輸入劇名"
                class="w-full rounded-lg border border-slate-700 bg-slate-800/80 px-3 py-1.5 text-sm text-slate-100 placeholder-slate-500 focus:border-indigo-500 focus:outline-none"
              />
            </div>
            <div>
              <label class="block text-xs text-slate-400 mb-1">季數</label>
              <div class="relative">
                <span class="absolute inset-y-0 left-0 flex items-center pl-2.5 text-xs text-slate-400">S</span>
                <input
                  :value="config.season"
                  @input="updateField('season', parseInt(($event.target as HTMLInputElement).value) || 1)"
                  type="number"
                  min="1"
                  class="w-full rounded-lg border border-slate-700 bg-slate-800/80 py-1.5 pl-6 pr-2 text-sm text-slate-100 focus:border-indigo-500 focus:outline-none"
                />
              </div>
            </div>
          </div>

          <div class="text-[11px] text-slate-400 font-mono bg-slate-950/40 px-2 py-1 rounded border border-slate-800/50">
            <span class="text-indigo-400 font-medium truncate">
              {{ config.customTitle.trim() || '劇名' }} - S{{ config.season < 10 ? '0' + config.season : config.season }}E01.mp4
            </span>
          </div>
        </div>
      </div>

      <!-- Download Settings -->
      <div class="rounded-xl border border-slate-800 bg-slate-900/40 p-4 space-y-3">
        <div class="flex items-center justify-between border-b border-slate-800/80 pb-2">
          <h2 class="text-xs font-semibold uppercase tracking-wider text-slate-400 flex items-center gap-1.5">
            <Sliders class="h-4 w-4 text-indigo-400" />
            下載設定
          </h2>
        </div>
        <div>
          <label class="block text-xs text-slate-400 mb-1">儲存目錄</label>
          <div class="flex gap-2">
            <input
              :value="config.saveDir"
              @input="updateField('saveDir', ($event.target as HTMLInputElement).value)"
              type="text"
              class="w-full rounded-lg border border-slate-700 bg-slate-800/80 px-3 py-1.5 text-sm text-slate-100 focus:border-indigo-500 focus:outline-none"
            />
            <button
              type="button"
              class="rounded-lg border border-slate-700 bg-slate-800 px-3 py-1.5 text-xs font-medium text-slate-300 hover:bg-slate-700 flex items-center gap-1"
            >
              <Folder class="h-3.5 w-3.5" />
              瀏覽
            </button>
          </div>
        </div>
        <div class="grid grid-cols-2 gap-3 pt-1">
          <div>
            <div class="flex justify-between text-xs text-slate-400 mb-1">
              <span>分片線程</span>
              <span class="text-indigo-400 font-medium">{{ config.threads }}</span>
            </div>
            <input
              :value="config.threads"
              @input="updateField('threads', parseInt(($event.target as HTMLInputElement).value))"
              type="range"
              min="4"
              max="32"
              step="4"
              :style="{
                background: `linear-gradient(to right, #6366f1 0%, #6366f1 ${((config.threads - 4) / 28) * 100}%, #334155 ${((config.threads - 4) / 28) * 100}%, #334155 100%)`
              }"
              class="h-1.5 w-full cursor-pointer appearance-none rounded-lg accent-indigo-500"
            />
          </div>
          <div>
            <div class="flex justify-between text-xs text-slate-400 mb-1">
              <span>同時下載集數</span>
              <span class="text-indigo-400 font-medium">{{ config.maxConcurrent }} 集</span>
            </div>
            <select
              :value="config.maxConcurrent"
              @change="updateField('maxConcurrent', parseInt(($event.target as HTMLSelectElement).value))"
              class="w-full rounded-lg border border-slate-700 bg-slate-800/80 px-2 py-1.5 text-xs text-slate-100 focus:border-indigo-500 focus:outline-none"
            >
              <option :value="1">1 集</option>
              <option :value="2">2 集</option>
              <option :value="3">3 集</option>
            </select>
          </div>
        </div>
      </div>
    </div>

    <!-- 2. Channel Selector (Item 2 & 3: 無括號副標題，無高畫質優先徽章) -->
    <div class="rounded-xl border border-slate-800 bg-slate-900/40 p-4 space-y-3">
      <div class="flex items-center justify-between border-b border-slate-800/80 pb-2">
        <div class="flex items-center gap-2">
          <Radio class="h-4 w-4 text-indigo-400" />
          <span class="text-xs font-semibold uppercase tracking-wider text-slate-300">
            播放頻道
          </span>
        </div>
        <span class="text-xs text-slate-400">
          共 {{ metadata.channels.length }} 個頻道
        </span>
      </div>

      <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
        <div
          v-for="c in metadata.channels"
          :key="c.id"
          @click="emit('update:selectedChannel', c.id)"
          :class="[
            'flex flex-col rounded-lg border p-2.5 text-left transition-all cursor-pointer',
            selectedChannel === c.id
              ? 'border-indigo-500 bg-indigo-600/20 text-white shadow-sm shadow-indigo-500/10'
              : 'border-slate-800 bg-slate-800/30 text-slate-400 hover:border-slate-700 hover:text-slate-200'
          ]"
        >
          <span class="font-medium text-xs truncate">{{ c.name }}</span>
          <div class="mt-1 flex items-center gap-1.5 text-[11px] text-slate-400 font-mono">
            <span>{{ channelProbeMap[c.id]?.resolution || '1080p' }}</span>
            <span>·</span>
            <span>{{ channelProbeMap[c.id]?.bitrate || '實時探測' }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 3. Dual Deck: Episode Picker & Episode Preview (Item 4: 左右獨立兩卡並列) -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-5 items-start">
      <!-- LEFT CARD: Episode Picker (Independent) -->
      <div class="rounded-xl border border-slate-800 bg-slate-900/40 p-4 space-y-3.5 shadow-sm">
        <div class="flex flex-wrap items-center justify-between gap-2 border-b border-slate-800/80 pb-2.5">
          <div class="flex items-center gap-2">
            <span class="text-xs font-semibold uppercase tracking-wider text-slate-300">
              集數選擇
            </span>
            <span class="text-xs text-indigo-400 font-mono font-medium">
              ({{ selectedEpisodes.length }} / {{ metadata.episodes.length }} 已選)
            </span>
          </div>

          <!-- Batch Action Toolbar -->
          <div class="flex items-center gap-1.5 text-xs">
            <button
              type="button"
              @click="selectAll"
              class="rounded border border-slate-700 bg-slate-800/80 px-2 py-0.5 text-slate-300 hover:bg-slate-700 flex items-center gap-1 transition-colors"
            >
              <CheckSquare class="h-3 w-3" /> 全選
            </button>
            <button
              type="button"
              @click="clearAll"
              class="rounded border border-slate-700 bg-slate-800/80 px-2 py-0.5 text-slate-300 hover:bg-slate-700 flex items-center gap-1 transition-colors"
            >
              <Square class="h-3 w-3" /> 清空
            </button>
            <button
              type="button"
              @click="selectFirstN(10)"
              class="rounded border border-slate-700 bg-slate-800/80 px-2 py-0.5 text-slate-300 hover:bg-slate-700 transition-colors"
            >
              前 10 集
            </button>
          </div>
        </div>

        <div class="flex items-center justify-between text-[11px] text-slate-400 pb-1">
          <span>點選集數進行試看，點擊「✓」切換下載</span>
          <span class="text-violet-400 font-medium">正在試看：第 {{ activePreviewEp }} 集</span>
        </div>

        <!-- Episode Grid with zero-overlap design (Item 4: 絕不擋字) -->
        <div class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 gap-2 max-h-[380px] overflow-y-auto pr-1">
          <div
            v-for="ep in metadata.episodes"
            :key="ep.ep_number"
            @click="activePreviewEp = ep.ep_number"
            :class="[
              'group flex flex-col justify-between rounded-lg border p-2 text-xs cursor-pointer transition-all min-h-[58px]',
              activePreviewEp === ep.ep_number
                ? 'border-violet-500 bg-violet-950/30 ring-2 ring-violet-500/50 shadow-md'
                : selectedEpisodes.includes(ep.ep_number)
                  ? 'border-indigo-500/50 bg-indigo-950/20 text-slate-200'
                  : 'border-slate-800 bg-slate-800/30 text-slate-400 hover:border-slate-700 hover:text-slate-200'
            ]"
          >
            <!-- Top: Centered clean label -->
            <div class="flex items-center justify-between">
              <span class="font-mono font-medium text-slate-200">
                第 {{ ep.ep_number < 10 ? '0' + ep.ep_number : ep.ep_number }} 集
              </span>
              <!-- Active Preview Indicator -->
              <Play
                v-if="activePreviewEp === ep.ep_number"
                class="h-3 w-3 fill-violet-400 text-violet-400 animate-pulse"
              />
            </div>

            <!-- Bottom: Download Status Button Tag (Zero text overlap) -->
            <button
              type="button"
              @click.stop="toggleEpisode(ep.ep_number)"
              :class="[
                'mt-1.5 flex items-center justify-center gap-1 rounded py-0.5 px-1.5 text-[10px] font-medium transition-all',
                selectedEpisodes.includes(ep.ep_number)
                  ? 'bg-indigo-600 text-white shadow-xs'
                  : 'bg-slate-800 text-slate-400 hover:bg-slate-700 hover:text-slate-200 border border-slate-700/60'
              ]"
            >
              <Check v-if="selectedEpisodes.includes(ep.ep_number)" class="h-2.5 w-2.5 stroke-[3]" />
              <span>{{ selectedEpisodes.includes(ep.ep_number) ? '已選下載' : '選取' }}</span>
            </button>
          </div>
        </div>
      </div>

      <!-- RIGHT CARD: Episode Preview (Independent Deck, Zero HUD overlap, No add/remove buttons) -->
      <div class="rounded-xl border border-slate-800 bg-slate-900/40 p-4 space-y-3.5 shadow-sm">
        <div class="flex items-center justify-between border-b border-slate-800/80 pb-2.5">
          <div class="flex items-center gap-2">
            <Tv class="h-4 w-4 text-violet-400" />
            <span class="text-xs font-semibold text-slate-200">
              集數預覽 (第 {{ activePreviewEp }} 集)
            </span>
          </div>
          <span class="text-[11px] text-slate-400 font-mono truncate max-w-[150px]">
            {{ currentChannel.name }}
          </span>
        </div>

        <!-- Clean Video Frame (Item 5: 畫面 100% 無浮動 HUD 遮蔽) -->
        <div class="relative aspect-video w-full overflow-hidden rounded-lg bg-black border border-slate-800 flex items-center justify-center shadow-inner group">
          <div class="text-center text-slate-500 text-xs flex flex-col items-center gap-1">
            <Tv class="h-8 w-8 text-slate-700" />
            <span>串流播放視窗 (第 {{ activePreviewEp }} 集)</span>
          </div>

          <!-- Standby Play Trigger -->
          <div
            v-if="!isPlaying"
            @click="isPlaying = !isPlaying"
            class="absolute inset-0 flex flex-col items-center justify-center bg-black/40 hover:bg-black/50 transition-colors cursor-pointer"
          >
            <div class="flex h-12 w-12 items-center justify-center rounded-full bg-violet-600/90 text-white shadow-lg transition-transform group-hover:scale-110">
              <Play class="h-6 w-6 fill-current translate-x-0.5" />
            </div>
            <span class="mt-2 text-xs font-medium text-slate-200 drop-shadow">點擊預覽播放</span>
          </div>
        </div>

        <!-- Item 8: 動態進度填充條 (Gradient Progress Bar) -->
        <div class="flex items-center gap-2 pt-1">
          <input
            type="range"
            min="0"
            :max="duration"
            step="1"
            :value="currentTime"
            @input="currentTime = Number(($event.target as HTMLInputElement).value)"
            :style="{
              background: `linear-gradient(to right, #8b5cf6 0%, #8b5cf6 ${(currentTime / duration) * 100}%, #334155 ${(currentTime / duration) * 100}%, #334155 100%)`
            }"
            class="h-1.5 w-full cursor-pointer appearance-none rounded-lg accent-violet-500"
          />
          <span class="text-[10px] font-mono text-slate-400 shrink-0 select-none">
            {{ formatTime(currentTime) }} / {{ formatTime(duration) }}
          </span>
        </div>

        <!-- Player Controls (Item 7: 已移除加入/移出下載按鈕) -->
        <div class="flex items-center justify-between gap-2 text-xs">
          <div class="flex items-center gap-2">
            <button
              type="button"
              @click="isPlaying = !isPlaying"
              class="flex h-7 w-7 items-center justify-center rounded-md border border-slate-700 bg-slate-800 text-slate-300 hover:bg-slate-700 transition-colors"
            >
              <Pause v-if="isPlaying" class="h-3.5 w-3.5" />
              <Play v-else class="h-3.5 w-3.5 fill-current" />
            </button>
            <button
              type="button"
              @click="isMuted = !isMuted"
              class="flex h-7 w-7 items-center justify-center rounded-md border border-slate-700 bg-slate-800 text-slate-300 hover:bg-slate-700 transition-colors"
            >
              <VolumeX v-if="isMuted" class="h-3.5 w-3.5 text-slate-400" />
              <Volume2 v-else class="h-3.5 w-3.5 text-violet-400" />
            </button>

            <!-- Item 8: 動態音量填充條 -->
            <input
              type="range"
              min="0"
              max="1"
              step="0.05"
              :value="volume"
              @input="volume = Number(($event.target as HTMLInputElement).value)"
              :style="{
                background: `linear-gradient(to right, #8b5cf6 0%, #8b5cf6 ${volume * 100}%, #334155 ${volume * 100}%, #334155 100%)`
              }"
              class="h-1.5 w-16 cursor-pointer appearance-none rounded-lg accent-violet-500"
            />
          </div>

          <button
            type="button"
            @click="copyStreamUrl"
            class="rounded border border-slate-700 bg-slate-800/80 px-2 py-1 text-[11px] text-slate-300 hover:bg-slate-700 flex items-center gap-1 transition-colors"
          >
            <CheckCheck v-if="copiedUrl" class="h-3 w-3 text-emerald-400" />
            <Copy v-else class="h-3 w-3" />
            <span>{{ copiedUrl ? '已複製' : '複製串流' }}</span>
          </button>
        </div>

        <!-- Item 5 & 6: 獨立串流規格列 (移至播放器下方獨立容器，徹底杜絕遮擋與需 ffprobe 字樣) -->
        <div class="flex flex-wrap items-center justify-between gap-2 rounded-lg bg-slate-950/60 border border-slate-800/80 px-3 py-2 text-xs">
          <div class="flex items-center gap-1.5">
            <span class="rounded bg-emerald-500/20 border border-emerald-500/30 px-1.5 py-0.5 font-medium text-emerald-300 text-[11px]">
              {{ currentStreamDetails.resolution }}
            </span>
            <span class="rounded bg-sky-500/20 border border-sky-500/30 px-1.5 py-0.5 font-medium text-sky-300 text-[11px]">
              {{ currentStreamDetails.bitrate }}
            </span>
          </div>
          <span class="text-[11px] text-slate-400 font-mono">
            {{ currentStreamDetails.video_codec }} / {{ currentStreamDetails.audio_codec }} ({{ currentStreamDetails.fps }} fps)
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
