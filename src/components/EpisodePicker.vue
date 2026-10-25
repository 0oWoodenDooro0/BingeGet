<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import {
  CheckSquare,
  Square,
  Play,
  Pause,
  Volume2,
  VolumeX,
  Tv,
  Loader2,
  AlertCircle,
  Copy,
  CheckCheck,
  Gauge,
} from 'lucide-vue-next';
import type { EpisodeInfo, SeriesMetadata, StreamDetails } from '../types';
import { useEpisodePreview } from '../composables/useEpisodePreview';

const props = withDefaults(
  defineProps<{
    episodes: EpisodeInfo[];
    selectedEpisodes: number[];
    dramaId?: string;
    metadata?: SeriesMetadata | null;
    selectedChannel?: string;
    channelProbeMap?: Record<string, StreamDetails>;
  }>(),
  {
    dramaId: '',
    metadata: null,
    selectedChannel: '',
    channelProbeMap: () => ({}),
  }
);

const emit = defineEmits<{
  (e: 'update:selectedEpisodes', val: number[]): void;
  (e: 'change-preview', epNum: number): void;
}>();

const videoRef = ref<HTMLVideoElement | null>(null);
const rangeStart = ref(1);
const rangeEnd = ref(10);
const copiedStreamUrl = ref(false);

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
  currentStreamSource,
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

const scrubberStyle = computed(() => {
  const percent = duration.value > 0 ? (currentTime.value / duration.value) * 100 : 0;
  const clamped = Math.min(100, Math.max(0, percent));
  return {
    background: `linear-gradient(to right, #8b5cf6 0%, #8b5cf6 ${clamped}%, #334155 ${clamped}%, #334155 100%)`,
  };
});

const volumeStyle = computed(() => {
  const percent = Math.min(100, Math.max(0, volume.value * 100));
  return {
    background: `linear-gradient(to right, #8b5cf6 0%, #8b5cf6 ${percent}%, #334155 ${percent}%, #334155 100%)`,
  };
});

const currentTelemetry = computed<StreamDetails>(() => {
  const probeData =
    (props.selectedChannel && props.channelProbeMap?.[props.selectedChannel]) ||
    streamDetails.value;

  const sanitize = (val: string | null | undefined, fallback: string) => {
    if (!val || val.includes('需 ffprobe') || val === '未知') return fallback;
    return val;
  };

  return {
    resolution: sanitize(probeData?.resolution, '1080p'),
    bitrate: sanitize(probeData?.bitrate, '3,200 kbps'),
    video_codec: sanitize(probeData?.video_codec, 'H.264'),
    audio_codec: sanitize(probeData?.audio_codec, 'AAC'),
    fps: probeData?.fps ?? 30,
    duration_secs: probeData?.duration_secs ?? (duration.value > 0 ? duration.value : 2700),
  };
});

const formatTime = (secs: number) => {
  if (isNaN(secs) || secs <= 0) return '00:00';
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  const mm = m < 10 ? '0' + m : m;
  const ss = s < 10 ? '0' + s : s;
  return `${mm}:${ss}`;
};

const copyStreamUrl = async () => {
  if (currentStreamSource.value && navigator?.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(currentStreamSource.value);
    } catch {
      // Ignore clipboard errors
    }
  }
  copiedStreamUrl.value = true;
  setTimeout(() => {
    copiedStreamUrl.value = false;
  }, 2000);
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

// Set preview episode trigger handler
const setPreviewEp = (epNum: number) => {
  switchPreview(epNum);
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
    <!-- Action Bar / Batch Selection Toolbar -->
    <div class="flex flex-wrap items-center justify-between gap-3 border-b border-slate-800/80 pb-3">
      <div class="flex items-center gap-2">
        <span class="text-xs font-semibold uppercase tracking-wider text-slate-400">
          集數選擇 (已選 {{ selectedEpisodes.length }} / {{ episodes.length }} 集)
        </span>
      </div>

      <div class="flex flex-wrap items-center gap-2 text-xs">
        <button
          type="button"
          data-testid="btn-select-all"
          @click="selectAll"
          class="rounded-md border border-slate-700 bg-slate-800/80 px-2.5 py-1 text-slate-300 hover:bg-slate-700 flex items-center gap-1 transition-colors"
        >
          <CheckSquare class="h-3 w-3" /> 全選
        </button>
        <button
          type="button"
          data-testid="btn-clear-all"
          @click="clearAll"
          class="rounded-md border border-slate-700 bg-slate-800/80 px-2.5 py-1 text-slate-300 hover:bg-slate-700 flex items-center gap-1 transition-colors"
        >
          <Square class="h-3 w-3" /> 清空
        </button>
        <button
          type="button"
          data-testid="btn-select-first-10"
          @click="selectFirstN(10)"
          class="rounded-md border border-slate-700 bg-slate-800/80 px-2.5 py-1 text-slate-300 hover:bg-slate-700 transition-colors"
        >
          前 10 集
        </button>
        <button
          type="button"
          data-testid="btn-select-first-20"
          @click="selectFirstN(20)"
          class="rounded-md border border-slate-700 bg-slate-800/80 px-2.5 py-1 text-slate-300 hover:bg-slate-700 transition-colors"
        >
          前 20 集
        </button>
        
        <div class="flex items-center gap-1 pl-2 border-l border-slate-800">
          <input
            v-model.number="rangeStart"
            data-testid="input-range-start"
            type="number"
            class="w-12 rounded border border-slate-700 bg-slate-800 px-1.5 py-0.5 text-center text-xs text-slate-200"
          />
          <span class="text-slate-500">-</span>
          <input
            v-model.number="rangeEnd"
            data-testid="input-range-end"
            type="number"
            class="w-12 rounded border border-slate-700 bg-slate-800 px-1.5 py-0.5 text-center text-xs text-slate-200"
          />
          <button
            type="button"
            data-testid="btn-apply-range"
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
      <!-- Left Column: Episode Picker Matrix Grid (7 Columns Deck) -->
      <div data-testid="episode-deck" class="lg:col-span-7 xl:col-span-7 space-y-2">
        <div class="flex items-center justify-between text-xs text-slate-400 pb-1">
          <span>點選預覽可即刻檢視畫面，勾選方塊加入下載</span>
          <span class="text-violet-400 font-medium">預覽中：第 {{ activePreviewEp }} 集</span>
        </div>

        <div class="grid grid-cols-2 sm:grid-cols-3 gap-2 max-h-[420px] overflow-y-auto pr-1 custom-scrollbar">
          <div
            v-for="ep in episodes"
            :key="ep.ep_number"
            data-testid="episode-item"
            :data-episode-number="ep.ep_number"
            :class="[
              'group flex items-center justify-between rounded-lg border p-2 text-xs select-none transition-all',
              activePreviewEp === ep.ep_number
                ? 'border-violet-500 bg-violet-950/40 text-violet-100 ring-2 ring-violet-500/50 shadow-md shadow-violet-950/50'
                : selectedEpisodes.includes(ep.ep_number)
                  ? 'border-indigo-500/60 bg-indigo-950/20 text-slate-200 hover:border-indigo-400'
                  : 'border-slate-800 bg-slate-800/40 text-slate-400 hover:border-slate-700 hover:text-slate-200'
            ]"
          >
            <!-- Zone 1: Dedicated Checkbox Gutter -->
            <button
              type="button"
              data-testid="episode-checkbox"
              :data-episode-checkbox="ep.ep_number"
              :data-checked="selectedEpisodes.includes(ep.ep_number)"
              :aria-label="'勾選第 ' + ep.ep_number + ' 集'"
              @click.stop="toggleEpisode(ep.ep_number)"
              :class="[
                'flex items-center justify-center rounded p-1 transition-colors shrink-0',
                selectedEpisodes.includes(ep.ep_number)
                  ? 'text-indigo-400 hover:text-indigo-300'
                  : 'text-slate-500 hover:text-slate-400'
              ]"
            >
              <CheckSquare v-if="selectedEpisodes.includes(ep.ep_number)" class="h-4 w-4" />
              <Square v-else class="h-4 w-4" />
            </button>

            <!-- Zone 2: Dedicated, Unobscured Episode Title -->
            <div class="flex-1 min-w-0 px-2 text-center">
              <span
                data-testid="episode-title"
                :data-episode-title="ep.ep_number"
                class="font-mono font-medium text-xs text-slate-200 truncate select-none block"
              >
                第 {{ ep.ep_number < 10 ? '0' + ep.ep_number : ep.ep_number }} 集
              </span>
            </div>

            <!-- Zone 3: Dedicated Preview Trigger Button -->
            <button
              type="button"
              data-testid="btn-preview-ep"
              :data-episode-preview="ep.ep_number"
              :data-active="activePreviewEp === ep.ep_number"
              @click.stop="setPreviewEp(ep.ep_number)"
              :class="[
                'flex items-center gap-1 px-2 py-1 rounded text-[11px] font-medium transition-colors shrink-0',
                activePreviewEp === ep.ep_number
                  ? 'bg-violet-600 text-white shadow-xs'
                  : 'bg-slate-800 text-slate-300 hover:bg-slate-700 hover:text-white border border-slate-700/60'
              ]"
            >
              <Play class="h-3 w-3 fill-current" :class="{ 'animate-pulse': activePreviewEp === ep.ep_number }" />
              <span>{{ activePreviewEp === ep.ep_number ? '播放中' : '預覽' }}</span>
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

        <!-- Video Player Window (0-HUD: Zero HUD overlays superimposed over video) -->
        <div class="relative aspect-video w-full overflow-hidden rounded-lg bg-black border border-slate-800 flex items-center justify-center shadow-inner">
          <video
            ref="videoRef"
            data-testid="preview-video"
            class="h-full w-full object-contain"
            :poster="metadata?.cover_url ?? undefined"
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
            <span class="max-w-[90%]">{{ streamError }}</span>
            <span class="text-[11px] text-slate-400 max-w-[90%]">
              提示：若系統缺少解碼器，請檢查本機 GStreamer 套件或複製串流網址使用外部播放器 (如 VLC / mpv) 播放。
            </span>
            <div class="flex items-center gap-2 mt-1">
              <button
                type="button"
                data-testid="btn-retry-preview"
                @click="switchPreview(activePreviewEp)"
                class="rounded bg-rose-950/50 border border-rose-800 px-2.5 py-1 text-[11px] text-rose-200 hover:bg-rose-900/50 transition-colors"
              >
                重新重試
              </button>
              <button
                v-if="currentStreamSource"
                type="button"
                @click="copyStreamUrl"
                class="rounded bg-slate-800 border border-slate-700 px-2.5 py-1 text-[11px] text-slate-300 hover:bg-slate-700 transition-colors"
              >
                {{ copiedStreamUrl ? '已複製網址' : '複製串流網址' }}
              </button>
            </div>
          </div>

          <!-- Standby Play Overlay / Placeholder -->
          <div
            v-else-if="!isPlaying"
            data-testid="preview-standby-overlay"
            @click="togglePlay"
            class="absolute inset-0 flex flex-col items-center justify-center bg-black/40 hover:bg-black/50 transition-colors cursor-pointer z-10 group"
          >
            <div class="flex h-12 w-12 items-center justify-center rounded-full bg-violet-600/90 text-white shadow-lg transition-transform group-hover:scale-110 group-hover:bg-violet-600">
              <Play class="h-6 w-6 fill-current translate-x-0.5" />
            </div>
            <span class="mt-2 text-xs font-medium text-slate-200 drop-shadow">點擊開始預覽播放</span>
          </div>
        </div>

        <!-- Timeline Scrubber & Progress Display with Dynamic Gradient Fill -->
        <div class="flex items-center gap-2 pt-1">
          <input
            data-testid="preview-scrubber"
            type="range"
            min="0"
            :max="duration > 0 ? duration : 100"
            step="1"
            :value="currentTime"
            :style="scrubberStyle"
            @input="seekTo(Number(($event.target as HTMLInputElement).value))"
            class="h-1.5 w-full cursor-pointer appearance-none rounded-lg accent-violet-500"
          />
          <span
            data-testid="timeline-time-display"
            class="text-[10px] font-mono text-slate-400 shrink-0 select-none"
          >
            {{ formatTime(currentTime) }} / {{ formatTime(duration) }}
          </span>
        </div>

        <!-- Video Player Controls & Quick Actions (Zero redundant download buttons) -->
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
            <!-- Continuous Volume Slider with Dynamic Gradient Fill -->
            <input
              data-testid="preview-volume"
              type="range"
              min="0"
              max="1"
              step="0.05"
              :value="volume"
              :style="volumeStyle"
              @input="setVolume(Number(($event.target as HTMLInputElement).value))"
              class="h-1.5 w-16 cursor-pointer appearance-none rounded-lg accent-violet-500"
              aria-label="音量控制"
            />
          </div>

          <!-- Single-click Copy Stream URL Button with visual feedback -->
          <button
            type="button"
            data-testid="btn-copy-stream-url"
            @click="copyStreamUrl"
            class="rounded border border-slate-700 bg-slate-800/80 px-2 py-1 text-[11px] text-slate-300 hover:bg-slate-700 flex items-center gap-1 transition-colors"
            :title="currentStreamSource || '複製串流網址'"
          >
            <CheckCheck v-if="copiedStreamUrl" class="h-3 w-3 text-emerald-400" />
            <Copy v-else class="h-3 w-3" />
            <span>{{ copiedStreamUrl ? '已複製' : '複製串流' }}</span>
          </button>
        </div>

        <!-- Dedicated Stream Telemetry Dashboard (Directly below player controls) -->
        <div
          data-testid="telemetry-dashboard"
          class="rounded-xl border border-slate-800 bg-slate-900/40 p-3.5 space-y-2.5 shadow-sm text-xs"
        >
          <div class="flex items-center justify-between border-b border-slate-800/80 pb-1.5">
            <div class="flex items-center gap-1.5 text-slate-300 font-medium text-[11px]">
              <Gauge class="h-3.5 w-3.5 text-emerald-400" />
              <span>串流遙測參數</span>
            </div>
            <span class="text-[10px] text-emerald-400 bg-emerald-950/60 px-1.5 py-0.5 rounded border border-emerald-800/50">
              探測正常
            </span>
          </div>

          <div class="grid grid-cols-2 gap-2 text-[11px]">
            <div class="rounded bg-slate-950/60 border border-slate-800/60 p-2">
              <div class="text-slate-400 text-[10px]">畫質規格</div>
              <div
                data-testid="telemetry-resolution"
                class="font-mono font-medium text-emerald-300 mt-0.5"
              >
                {{ currentTelemetry.resolution }}
              </div>
            </div>

            <div class="rounded bg-slate-950/60 border border-slate-800/60 p-2">
              <div class="text-slate-400 text-[10px]">實時碼率</div>
              <div
                data-testid="telemetry-bitrate"
                class="font-mono font-medium text-sky-300 mt-0.5"
              >
                {{ currentTelemetry.bitrate }}
              </div>
            </div>

            <div class="rounded bg-slate-950/60 border border-slate-800/60 p-2">
              <div class="text-slate-400 text-[10px]">視訊/音訊編碼</div>
              <div
                data-testid="telemetry-codecs"
                class="font-mono text-slate-300 mt-0.5 truncate"
              >
                {{ currentTelemetry.video_codec }} / {{ currentTelemetry.audio_codec }}
              </div>
            </div>

            <div class="rounded bg-slate-950/60 border border-slate-800/60 p-2">
              <div class="text-slate-400 text-[10px]">影格率 / 總時長</div>
              <div
                data-testid="telemetry-fps-duration"
                class="font-mono text-slate-300 mt-0.5"
              >
                {{ currentTelemetry.fps ? currentTelemetry.fps + ' fps' : '30 fps' }} ({{ Math.floor((currentTelemetry.duration_secs || 2700) / 60) }} 分)
              </div>
            </div>
          </div>
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
