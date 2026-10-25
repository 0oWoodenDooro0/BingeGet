<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import {
  CheckSquare,
  Square,
  Check,
  Play,
  Pause,
  Volume2,
  VolumeX,
  Tv,
  Loader2,
  AlertCircle,
  Plus,
  Trash2,
} from 'lucide-vue-next';
import type { EpisodeInfo, SeriesMetadata } from '../types';
import { useEpisodePreview } from '../composables/useEpisodePreview';

const props = withDefaults(
  defineProps<{
    episodes: EpisodeInfo[];
    selectedEpisodes: number[];
    dramaId?: string;
    metadata?: SeriesMetadata | null;
    selectedChannel?: string;
  }>(),
  {
    dramaId: '',
    metadata: null,
    selectedChannel: '',
  }
);

const emit = defineEmits<{
  (e: 'update:selectedEpisodes', val: number[]): void;
  (e: 'change-preview', epNum: number): void;
}>();

const videoRef = ref<HTMLVideoElement | null>(null);
const rangeStart = ref(1);
const rangeEnd = ref(10);

const currentDramaId = computed(() => {
  return props.dramaId || props.metadata?.drama_id || '';
});

const currentChannelName = computed(() => {
  if (!props.selectedChannel) return '預設頻道';
  const found = props.metadata?.channels?.find(c => c.id === props.selectedChannel);
  return found ? found.name : props.selectedChannel;
});

const {
  activePreviewEp,
  streamDetails,
  loading: loadingStream,
  error: streamError,
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
} = useEpisodePreview({ autoPlay: false });

const formatTime = (secs: number) => {
  if (isNaN(secs) || secs <= 0) return '00:00';
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  const mm = m < 10 ? '0' + m : m;
  const ss = s < 10 ? '0' + s : s;
  return `${mm}:${ss}`;
};

// Bind video ref
onMounted(() => {
  if (videoRef.value) {
    bindVideoElement(videoRef.value);
  }
  // Initialize preview with the first episode
  if (props.episodes.length > 0) {
    const firstEp = props.episodes[0];
    switchPreview(firstEp.ep_number);
  }
});

onUnmounted(() => {
  cleanup();
});

// Watch video ref for template mounting
watch(videoRef, (newEl) => {
  if (newEl) {
    bindVideoElement(newEl);
  }
});

// Trigger preview for a specific episode
const switchPreview = (epNum: number) => {
  const ep = props.episodes.find(e => e.ep_number === epNum);
  if (!ep) return;

  emit('change-preview', epNum);
  loadPreview(currentDramaId.value, ep.ep_id, epNum, props.selectedChannel);
};

// Reload preview if user switches Channel in ChannelSelector
watch(
  () => props.selectedChannel,
  (newChannel, oldChannel) => {
    if (newChannel && newChannel !== oldChannel && activePreviewEp.value) {
      const ep = props.episodes.find(e => e.ep_number === activePreviewEp.value);
      if (ep) {
        loadPreview(currentDramaId.value, ep.ep_id, activePreviewEp.value, newChannel);
      }
    }
  }
);

// Watch episodes list changes
watch(
  () => props.episodes,
  (newEps) => {
    if (newEps.length > 0) {
      if (!newEps.some(e => e.ep_number === activePreviewEp.value)) {
        switchPreview(newEps[0].ep_number);
      }
    }
  }
);

// Toggle download selection (Decoupled from preview)
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

// Toggle download state of currently previewed episode
const toggleCurrentPreviewDownload = () => {
  if (activePreviewEp.value) {
    toggleEpisode(activePreviewEp.value);
  }
};

const isCurrentInDownload = computed(() => {
  return props.selectedEpisodes.includes(activePreviewEp.value);
});

// Batch Selection Toolbar handlers
const selectAll = () => {
  emit('update:selectedEpisodes', props.episodes.map(e => e.ep_number));
};

const clearAll = () => {
  emit('update:selectedEpisodes', []);
};

const selectFirstN = (n: number) => {
  emit('update:selectedEpisodes', props.episodes.slice(0, n).map(e => e.ep_number));
};

const applyRange = () => {
  const start = Math.min(rangeStart.value, rangeEnd.value);
  const end = Math.max(rangeStart.value, rangeEnd.value);
  const inRange = props.episodes
    .filter(e => e.ep_number >= start && e.ep_number <= end)
    .map(e => e.ep_number);
  emit('update:selectedEpisodes', inRange);
};
</script>

<template>
  <div v-if="episodes.length > 0" class="rounded-xl border border-slate-800 bg-slate-900/40 p-4 space-y-4">
    <!-- Action Bar -->
    <div class="flex flex-wrap items-center justify-between gap-3 border-b border-slate-800/80 pb-3">
      <div class="flex items-center gap-2">
        <span class="text-xs font-semibold uppercase tracking-wider text-slate-400">
          集數選擇 (已選 {{ selectedEpisodes.length }} / {{ episodes.length }} 集)
        </span>
      </div>

      <div class="flex flex-wrap items-center gap-2 text-xs">
        <button
          type="button"
          @click="selectAll"
          class="rounded-md border border-slate-700 bg-slate-800/80 px-2.5 py-1 text-slate-300 hover:bg-slate-700 flex items-center gap-1 transition-colors"
        >
          <CheckSquare class="h-3 w-3" /> 全選
        </button>
        <button
          type="button"
          @click="clearAll"
          class="rounded-md border border-slate-700 bg-slate-800/80 px-2.5 py-1 text-slate-300 hover:bg-slate-700 flex items-center gap-1 transition-colors"
        >
          <Square class="h-3 w-3" /> 清空
        </button>
        <button
          type="button"
          @click="selectFirstN(10)"
          class="rounded-md border border-slate-700 bg-slate-800/80 px-2.5 py-1 text-slate-300 hover:bg-slate-700 transition-colors"
        >
          前 10 集
        </button>
        
        <div class="flex items-center gap-1 pl-2 border-l border-slate-800">
          <input
            v-model.number="rangeStart"
            type="number"
            class="w-12 rounded border border-slate-700 bg-slate-800 px-1.5 py-0.5 text-center text-xs text-slate-200"
          />
          <span class="text-slate-500">-</span>
          <input
            v-model.number="rangeEnd"
            type="number"
            class="w-12 rounded border border-slate-700 bg-slate-800 px-1.5 py-0.5 text-center text-xs text-slate-200"
          />
          <button
            type="button"
            @click="applyRange"
            class="rounded border border-indigo-500/30 bg-indigo-500/20 px-2 py-0.5 text-indigo-300 hover:bg-indigo-500/30 transition-colors"
          >
            套用範圍
          </button>
        </div>
      </div>
    </div>

    <!-- 2-Column Responsive Workspace Layout -->
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-5 items-start">
      <!-- Left Column: Episode Picker Grid -->
      <div class="lg:col-span-7 xl:col-span-7 space-y-2">
        <div class="flex items-center justify-between text-xs text-slate-400 pb-1">
          <span>點選集數即刻於右側預覽，勾選右上角加入下載</span>
          <span class="text-violet-400 font-medium">預覽中：第 {{ activePreviewEp }} 集</span>
        </div>

        <div class="grid grid-cols-4 sm:grid-cols-5 md:grid-cols-6 gap-2 max-h-[380px] overflow-y-auto pr-1 custom-scrollbar">
          <div
            v-for="ep in episodes"
            :key="ep.ep_number"
            data-testid="episode-item"
            :data-episode-number="ep.ep_number"
            @click="switchPreview(ep.ep_number)"
            :class="[
              'group relative flex flex-col items-center justify-center rounded-lg border p-2.5 text-xs font-medium cursor-pointer transition-all select-none',
              activePreviewEp === ep.ep_number
                ? 'border-violet-500 bg-violet-950/40 text-violet-100 ring-2 ring-violet-500/50 shadow-md shadow-violet-950/50'
                : selectedEpisodes.includes(ep.ep_number)
                  ? 'border-indigo-500/60 bg-indigo-950/20 text-slate-200 hover:border-indigo-400'
                  : 'border-slate-800 bg-slate-800/40 text-slate-400 hover:border-slate-700 hover:text-slate-200'
            ]"
          >
            <!-- Episode Label -->
            <span class="font-mono">第 {{ ep.ep_number < 10 ? '0' + ep.ep_number : ep.ep_number }} 集</span>

            <!-- Previewing Active Badge -->
            <div
              v-if="activePreviewEp === ep.ep_number"
              class="mt-1 flex items-center gap-1 text-[10px] text-violet-400 font-normal"
            >
              <Play class="h-2.5 w-2.5 fill-violet-400 text-violet-400 animate-pulse" />
              <span>預覽中</span>
            </div>

            <!-- Decoupled Download Checkbox Trigger (Top-Right) -->
            <button
              type="button"
              data-testid="episode-checkbox"
              :data-episode-checkbox="ep.ep_number"
              :aria-label="'勾選第 ' + ep.ep_number + ' 集'"
              @click.stop="toggleEpisode(ep.ep_number)"
              :class="[
                'absolute top-1.5 right-1.5 flex h-4 w-4 items-center justify-center rounded transition-all',
                selectedEpisodes.includes(ep.ep_number)
                  ? 'bg-indigo-600 text-white shadow-sm shadow-indigo-600/30'
                  : 'border border-slate-700 bg-slate-800/80 text-transparent hover:border-slate-500 hover:text-slate-500'
              ]"
            >
              <Check class="h-2.5 w-2.5 stroke-[3]" />
            </button>
          </div>
        </div>
      </div>

      <!-- Right Column: Episode Preview Dock -->
      <div
        data-testid="preview-dock"
        class="lg:col-span-5 xl:col-span-5 flex flex-col rounded-xl border border-slate-800 bg-slate-950/60 p-3.5 space-y-3"
      >
        <!-- Dock Header -->
        <div class="flex items-center justify-between border-b border-slate-800/80 pb-2.5">
          <div class="flex items-center gap-2">
            <Tv class="h-4 w-4 text-violet-400" />
            <span class="text-xs font-semibold text-slate-200">
              集數預覽 (第 {{ activePreviewEp }} 集)
            </span>
          </div>
          <span
            data-testid="channel-indicator"
            class="text-[11px] text-slate-400 font-mono truncate max-w-[150px]"
            :title="currentChannelName"
          >
            {{ currentChannelName }}
          </span>
        </div>

        <!-- Video Player Window -->
        <div class="relative aspect-video w-full overflow-hidden rounded-lg bg-black border border-slate-800 flex items-center justify-center shadow-inner">
          <video
            ref="videoRef"
            data-testid="preview-video"
            class="h-full w-full object-contain"
            playsinline
            muted
          ></video>

          <!-- Loading State Overlay -->
          <div
            v-if="loadingStream"
            data-testid="preview-loading-overlay"
            class="absolute inset-0 flex flex-col items-center justify-center bg-black/75 backdrop-blur-xs text-xs text-slate-300 gap-2 z-10"
          >
            <Loader2 class="h-6 w-6 animate-spin text-violet-400" />
            <span>串流與指標分析中...</span>
          </div>

          <!-- Error State Overlay -->
          <div
            v-else-if="streamError"
            data-testid="preview-error-overlay"
            class="absolute inset-0 flex flex-col items-center justify-center bg-black/85 p-4 text-center text-xs text-rose-300 gap-2 z-10"
          >
            <AlertCircle class="h-6 w-6 text-rose-400" />
            <span>{{ streamError }}</span>
            <button
              type="button"
              data-testid="btn-retry-preview"
              @click="switchPreview(activePreviewEp)"
              class="mt-1 rounded bg-rose-950/50 border border-rose-800 px-2.5 py-1 text-[11px] text-rose-200 hover:bg-rose-900/50"
            >
              重新重試
            </button>
          </div>

          <!-- Bottom Floating HUD: Resolution & Bitrate -->
          <div
            v-if="streamDetails"
            data-testid="preview-hud"
            class="absolute bottom-2 left-2 right-2 flex flex-wrap items-center justify-between gap-1.5 rounded-md bg-slate-950/85 backdrop-blur-md px-2.5 py-1.5 text-[11px] border border-slate-800/80 shadow-lg z-20 pointer-events-none"
          >
            <div class="flex items-center gap-1.5">
              <span
                data-testid="hud-resolution"
                class="rounded bg-emerald-500/20 border border-emerald-500/30 px-1.5 py-0.5 font-medium text-emerald-300"
              >
                {{ streamDetails.resolution }}
              </span>
              <span
                data-testid="hud-bitrate"
                class="rounded bg-sky-500/20 border border-sky-500/30 px-1.5 py-0.5 font-medium text-sky-300"
              >
                {{ streamDetails.bitrate }}
              </span>
            </div>
            <span
              data-testid="hud-codec"
              class="text-[10px] text-slate-400 font-mono truncate max-w-[140px]"
            >
              {{ streamDetails.video_codec }} / {{ streamDetails.audio_codec }}
            </span>
          </div>
        </div>

        <!-- Timeline Scrubber & Progress Display -->
        <div class="flex items-center gap-2 pt-1">
          <input
            data-testid="timeline-scrubber"
            type="range"
            min="0"
            :max="duration > 0 ? duration : 100"
            step="1"
            :value="currentTime"
            @input="seekTo(Number(($event.target as HTMLInputElement).value))"
            class="h-1.5 w-full cursor-pointer appearance-none rounded-lg bg-slate-800 accent-violet-500 hover:bg-slate-700"
          />
          <span
            data-testid="timeline-time-display"
            class="text-[10px] font-mono text-slate-400 shrink-0 select-none"
          >
            {{ formatTime(currentTime) }} / {{ formatTime(duration) }}
          </span>
        </div>

        <!-- Video Player Controls & Quick Actions -->
        <div class="flex items-center justify-between gap-2 text-xs">
          <!-- Player Controls -->
          <div class="flex items-center gap-2">
            <button
              type="button"
              data-testid="btn-play-pause"
              @click="togglePlay"
              class="flex h-7 w-7 items-center justify-center rounded-md border border-slate-700 bg-slate-800 text-slate-300 hover:bg-slate-700 transition-colors"
              :aria-label="isPlaying ? '暫停' : '播放'"
            >
              <Pause v-if="isPlaying" class="h-3.5 w-3.5" />
              <Play v-else class="h-3.5 w-3.5 fill-current" />
            </button>
            <button
              type="button"
              data-testid="btn-mute"
              @click="toggleMute"
              class="flex h-7 w-7 items-center justify-center rounded-md border border-slate-700 bg-slate-800 text-slate-300 hover:bg-slate-700 transition-colors"
              :aria-label="isMuted ? '解除靜音' : '靜音'"
            >
              <VolumeX v-if="isMuted" class="h-3.5 w-3.5 text-slate-400" />
              <Volume2 v-else class="h-3.5 w-3.5 text-violet-400" />
            </button>
            <!-- Continuous Volume Slider -->
            <input
              data-testid="volume-slider"
              type="range"
              min="0"
              max="1"
              step="0.05"
              :value="volume"
              @input="setVolume(Number(($event.target as HTMLInputElement).value))"
              class="h-1.5 w-14 cursor-pointer appearance-none rounded-lg bg-slate-800 accent-violet-500"
              aria-label="音量控制"
            />
          </div>

          <!-- Quick Toggle for Current Preview Episode in Download List -->
          <button
            type="button"
            data-testid="btn-toggle-download-current"
            @click="toggleCurrentPreviewDownload"
            :class="[
              'inline-flex items-center gap-1 rounded-md px-2.5 py-1 text-xs font-medium border transition-colors',
              isCurrentInDownload
                ? 'border-rose-900/60 bg-rose-950/30 text-rose-300 hover:bg-rose-900/40'
                : 'border-indigo-500/40 bg-indigo-500/20 text-indigo-300 hover:bg-indigo-500/30'
            ]"
          >
            <template v-if="isCurrentInDownload">
              <Trash2 class="h-3 w-3" />
              <span>移出下載</span>
            </template>
            <template v-else>
              <Plus class="h-3 w-3" />
              <span>加入下載</span>
            </template>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(15, 23, 42, 0.4);
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(51, 65, 85, 0.6);
  border-radius: 2px;
}
</style>
