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
import type { SeriesSession } from '../composables/useSeriesSession';

const props = withDefaults(
  defineProps<{
    episodes?: EpisodeInfo[];
    selectedEpisodes?: number[];
    dramaId?: string;
    metadata?: SeriesMetadata | null;
    selectedChannel?: string;
    channelProbeMap?: Record<string, StreamDetails>;
    session?: SeriesSession;
  }>(),
  {
    episodes: () => [],
    selectedEpisodes: () => [],
    dramaId: '',
    metadata: null,
    selectedChannel: '',
    channelProbeMap: () => ({}),
    session: undefined,
  }
);

const emit = defineEmits<{
  (e: 'update:selectedEpisodes', val: number[]): void;
  (e: 'change-preview', epNum: number): void;
  (e: 'probe-complete', payload: { channel: string; details: StreamDetails }): void;
}>();

const videoRef = ref<HTMLVideoElement | null>(null);
const rangeStart = ref(1);
const rangeEnd = ref(10);
const copiedStreamUrl = ref(false);

const effectiveEpisodes = computed<EpisodeInfo[]>(() => {
  if (props.session?.metadata?.value?.episodes) {
    return props.session.metadata.value.episodes;
  }
  return props.episodes || [];
});

const effectiveSelectedEpisodes = computed<number[]>(() => {
  if (props.session) {
    return props.session.selectedEpisodes.value;
  }
  return props.selectedEpisodes || [];
});

const effectiveMetadata = computed<SeriesMetadata | null>(() => {
  if (props.session?.metadata?.value) {
    return props.session.metadata.value;
  }
  return props.metadata ?? null;
});

const effectiveSelectedChannel = computed<string>(() => {
  if (props.session) {
    return props.session.selectedChannel.value;
  }
  return props.selectedChannel || '';
});

const currentDramaId = computed(() => {
  return props.dramaId || effectiveMetadata.value?.drama_id || '';
});

const currentChannelName = computed(() => {
  const ch = effectiveSelectedChannel.value;
  if (!ch) return '預設頻道';
  const found = effectiveMetadata.value?.channels?.find(c => c.id === ch);
  return found ? found.name : ch;
});

// Missing episodes detection for Semi-Auto Fallback & Channel Lock
const missingEpisodes = computed<number[]>(() => {
  if (props.session?.missingEpisodesForLockedChannel?.value) {
    return props.session.missingEpisodesForLockedChannel.value;
  }
  const ch = effectiveSelectedChannel.value;
  if (!ch || !effectiveMetadata.value) return [];
  return effectiveMetadata.value.episodes
    .filter((ep) => {
      if (!ep.available_channels || ep.available_channels.length === 0) return false;
      return !ep.available_channels.includes(ch);
    })
    .map((ep) => ep.ep_number);
});

const isEpisodeMissing = (epNum: number): boolean => {
  return missingEpisodes.value.includes(epNum);
};

// Fallback preview composable when session is not provided directly
const fallbackPreview = useEpisodePreview({ autoPlay: false });

const activePreviewEp = computed<number>(() => {
  return props.session ? props.session.activePreviewEp.value : fallbackPreview.activePreviewEp.value;
});

const streamDetails = computed<StreamDetails | null>(() => {
  return props.session ? props.session.streamDetails.value : fallbackPreview.streamDetails.value;
});

const currentStreamSource = computed<string>(() => {
  return props.session ? props.session.currentStreamSource.value : fallbackPreview.currentStreamSource.value;
});

const loadingStream = computed<boolean>(() => {
  return props.session ? props.session.loadingStream.value : fallbackPreview.loading.value;
});

const streamError = computed<string>(() => {
  return props.session ? props.session.streamError.value : fallbackPreview.error.value;
});

const isPlaying = computed<boolean>(() => {
  return props.session ? props.session.isPlaying.value : fallbackPreview.isPlaying.value;
});

const isMuted = computed<boolean>(() => {
  return props.session ? props.session.isMuted.value : fallbackPreview.isMuted.value;
});

const currentTime = computed<number>(() => {
  return props.session ? props.session.currentTime.value : fallbackPreview.currentTime.value;
});

const duration = computed<number>(() => {
  return props.session ? props.session.duration.value : fallbackPreview.duration.value;
});

const volume = computed<number>(() => {
  return props.session ? props.session.volume.value : fallbackPreview.volume.value;
});

const bindVideoElement = (el: HTMLVideoElement | null) => {
  if (props.session) {
    props.session.bindVideoElement(el);
  } else {
    fallbackPreview.bindVideoElement(el);
  }
};

const togglePlay = () => {
  if (props.session) {
    props.session.togglePlay();
  } else {
    fallbackPreview.togglePlay();
  }
};

const toggleMute = () => {
  if (props.session) {
    props.session.toggleMute();
  } else {
    fallbackPreview.toggleMute();
  }
};

const seekTo = (seconds: number) => {
  if (props.session) {
    props.session.seekTo(seconds);
  } else {
    fallbackPreview.seekTo(seconds);
  }
};

const setVolume = (val: number) => {
  if (props.session) {
    props.session.setVolume(val);
  } else {
    fallbackPreview.setVolume(val);
  }
};

const cleanup = () => {
  if (props.session) {
    props.session.cleanup();
  } else {
    fallbackPreview.cleanup();
  }
};

// Emit probe-complete when streamDetails updates for the active channel (legacy/fallback only)
watch(streamDetails, (newDetails) => {
  if (!props.session && newDetails && effectiveSelectedChannel.value) {
    emit('probe-complete', { channel: effectiveSelectedChannel.value, details: newDetails });
  }
});

const createProgressGradient = (percent: number) => {
  const clamped = Math.min(100, Math.max(0, percent));
  return {
    background: `linear-gradient(to right, #8b5cf6 0%, #8b5cf6 ${clamped}%, #334155 ${clamped}%, #334155 100%)`,
  };
};

const scrubberStyle = computed(() => {
  const percent = duration.value > 0 ? (currentTime.value / duration.value) * 100 : 0;
  return createProgressGradient(percent);
});

const volumeStyle = computed(() => {
  return createProgressGradient(volume.value * 100);
});

const rawProbeData = computed<StreamDetails | null>(() => {
  // If session is provided, bind to session.currentChannelProbe and session.channelProbeMap
  if (props.session) {
    if (props.session.currentChannelProbe?.value) {
      return props.session.currentChannelProbe.value;
    }
    const ch = props.session.selectedChannel?.value;
    if (ch && props.session.channelProbeMap?.value?.[ch]) {
      return props.session.channelProbeMap.value[ch];
    }
    if (props.session.streamDetails?.value) {
      return props.session.streamDetails.value;
    }
    return null;
  }
  // Prefer channel probe map if available for current channel
  const ch = effectiveSelectedChannel.value;
  if (ch && props.channelProbeMap?.[ch]) {
    return props.channelProbeMap[ch];
  }
  // Fall back to active preview streamDetails
  if (streamDetails.value) {
    return streamDetails.value;
  }
  return null;
});

const isStreamProbed = computed(() => {
  const p = rawProbeData.value;
  if (!p) return false;
  const hasValidRes = Boolean(
    p.resolution &&
    !p.resolution.includes('需 ffprobe') &&
    p.resolution !== '未知' &&
    p.resolution.trim() !== ''
  );
  const hasValidBitrate = Boolean(
    p.bitrate &&
    !p.bitrate.includes('需 ffprobe') &&
    p.bitrate !== '未知' &&
    p.bitrate.trim() !== ''
  );
  return hasValidRes || hasValidBitrate;
});

const telemetryDisplay = computed(() => {
  const p = rawProbeData.value;
  if (!isStreamProbed.value || !p) {
    return {
      resolution: '--',
      bitrate: '--',
      codecs: '--',
      fpsDuration: '--',
    };
  }

  const clean = (val: string | null | undefined) => {
    if (!val || val.includes('需 ffprobe') || val === '未知') return '';
    return val.trim();
  };

  const res = clean(p.resolution) || '--';
  const bitrate = clean(p.bitrate) || '--';

  const vCodec = clean(p.video_codec);
  const aCodec = clean(p.audio_codec);
  let codecs = '--';
  if (vCodec && aCodec) {
    codecs = `${vCodec} / ${aCodec}`;
  } else if (vCodec) {
    codecs = vCodec;
  } else if (aCodec) {
    codecs = aCodec;
  }

  const fpsText = p.fps ? `${p.fps} fps` : '';
  const dur = p.duration_secs ?? (duration.value > 0 ? duration.value : null);
  const durText = dur ? `${Math.floor(dur / 60)} 分` : '';
  let fpsDuration = '--';
  if (fpsText && durText) {
    fpsDuration = `${fpsText} (${durText})`;
  } else if (fpsText) {
    fpsDuration = fpsText;
  } else if (durText) {
    fpsDuration = `(${durText})`;
  }

  return {
    resolution: res,
    bitrate,
    codecs,
    fpsDuration,
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
      copiedStreamUrl.value = true;
      setTimeout(() => {
        copiedStreamUrl.value = false;
      }, 2000);
    } catch {
      // Ignore clipboard errors
    }
  }
};

// Bind video ref
onMounted(() => {
  if (videoRef.value) {
    bindVideoElement(videoRef.value);
  }
  // Initialize preview with the first episode
  if (effectiveEpisodes.value.length > 0) {
    const firstEp = effectiveEpisodes.value[0];
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

// Trigger preview for a specific episode directly without middleman
const switchPreview = (epNum: number) => {
  const ep = effectiveEpisodes.value.find(e => e.ep_number === epNum);
  if (!ep) return;

  if (props.session) {
    props.session.loadPreview(epNum, effectiveSelectedChannel.value);
  } else {
    emit('change-preview', epNum);
    fallbackPreview.loadPreview(
      currentDramaId.value,
      ep.ep_id,
      epNum,
      effectiveSelectedChannel.value,
      effectiveMetadata.value?.source_id
    );
  }
};

// Reload preview if user switches Channel in ChannelSelector
watch(
  effectiveSelectedChannel,
  (newChannel, oldChannel) => {
    if (newChannel && newChannel !== oldChannel && activePreviewEp.value) {
      const ep = effectiveEpisodes.value.find(e => e.ep_number === activePreviewEp.value);
      if (ep) {
        if (props.session) {
          props.session.loadPreview(activePreviewEp.value, newChannel);
        } else {
          fallbackPreview.loadPreview(
            currentDramaId.value,
            ep.ep_id,
            activePreviewEp.value,
            newChannel,
            effectiveMetadata.value?.source_id
          );
        }
      }
    }
  }
);

// Watch episodes list changes
watch(
  effectiveEpisodes,
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
  if (props.session) {
    props.session.toggleEpisode(epNum);
    return;
  }
  const current = [...effectiveSelectedEpisodes.value];
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
  if (props.session) {
    props.session.selectAllEpisodes();
    return;
  }
  emit('update:selectedEpisodes', effectiveEpisodes.value.map(e => e.ep_number));
};

const clearAll = () => {
  if (props.session) {
    props.session.clearSelectedEpisodes();
    return;
  }
  emit('update:selectedEpisodes', []);
};

const selectFirstN = (n: number) => {
  if (props.session) {
    props.session.selectFirstNEpisodes(n);
    return;
  }
  emit('update:selectedEpisodes', effectiveEpisodes.value.slice(0, n).map(e => e.ep_number));
};

const applyRange = () => {
  const start = Math.min(rangeStart.value, rangeEnd.value);
  const end = Math.max(rangeStart.value, rangeEnd.value);
  if (props.session) {
    props.session.selectEpisodeRange(start, end);
    return;
  }
  const inRange = effectiveEpisodes.value
    .filter(e => e.ep_number >= start && e.ep_number <= end)
    .map(e => e.ep_number);
  emit('update:selectedEpisodes', inRange);
};
</script>

<template>
  <div v-if="effectiveEpisodes.length > 0" class="rounded-xl border border-slate-800 bg-slate-900/40 p-4 space-y-4">
    <!-- 2-Column Responsive Workspace Layout: 7:5 Cinema Master Grid -->
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-5 items-start">
      <!-- Left Column: Episode Picker Matrix Grid (7 Columns Deck) -->
      <div data-testid="episode-deck" class="lg:col-span-7 space-y-3">
        <!-- Action Bar / Batch Selection Toolbar (Inside 7-Column Episode Deck) -->
        <div class="flex flex-wrap items-center justify-between gap-3 border-b border-slate-800/80 pb-3">
          <div class="flex items-center gap-2">
            <span class="text-xs font-semibold uppercase tracking-wider text-slate-400">
              集數選擇 (已選 {{ effectiveSelectedEpisodes.length }} / {{ effectiveEpisodes.length }} 集)
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

        <!-- Missing Episodes Warning Banner (Semi-Auto Fallback / Channel Lock) -->
        <div
          v-if="missingEpisodes.length > 0"
          data-testid="missing-episodes-banner"
          class="flex items-center gap-2 rounded-lg border border-amber-500/30 bg-amber-950/20 px-3 py-2 text-xs text-amber-300"
        >
          <AlertCircle class="h-4 w-4 shrink-0 text-amber-400" />
          <span>
            鎖定頻道「{{ currentChannelName }}」缺少第 {{ missingEpisodes.join('、') }} 集，請進行單集頻道配置。
          </span>
        </div>

        <div class="flex items-center justify-between text-xs text-slate-400 pb-1">
          <span>點選預覽可即刻檢視畫面，點選集數卡片或勾選方塊加入下載</span>
          <span class="text-violet-400 font-medium">預覽中：第 {{ activePreviewEp }} 集</span>
        </div>

        <div class="grid grid-cols-2 sm:grid-cols-3 gap-2 max-h-[420px] overflow-y-auto pr-1 custom-scrollbar">
          <div
            v-for="ep in effectiveEpisodes"
            :key="ep.ep_number"
            data-testid="episode-item"
            :data-episode-number="ep.ep_number"
            :data-missing="isEpisodeMissing(ep.ep_number)"
            @click="toggleEpisode(ep.ep_number)"
            :class="[
              'group flex items-center justify-between rounded-lg border p-2 text-xs select-none transition-all cursor-pointer',
              isEpisodeMissing(ep.ep_number)
                ? 'border-amber-900/60 bg-amber-950/20 text-amber-200/90 hover:border-amber-700'
                : activePreviewEp === ep.ep_number
                  ? 'border-violet-500 bg-violet-950/40 text-violet-100 ring-2 ring-violet-500/50 shadow-md shadow-violet-950/50'
                  : effectiveSelectedEpisodes.includes(ep.ep_number)
                    ? 'border-indigo-500/60 bg-indigo-950/20 text-slate-200 hover:border-indigo-400'
                    : 'border-slate-800 bg-slate-800/40 text-slate-400 hover:border-slate-700 hover:text-slate-200'
            ]"
          >
            <!-- Zone 1: Dedicated Checkbox Gutter -->
            <button
              type="button"
              data-testid="episode-checkbox"
              :data-episode-checkbox="ep.ep_number"
              :data-checked="effectiveSelectedEpisodes.includes(ep.ep_number)"
              :aria-label="'勾選第 ' + ep.ep_number + ' 集'"
              @click.stop="toggleEpisode(ep.ep_number)"
              :class="[
                'flex items-center justify-center rounded p-1 transition-colors shrink-0',
                effectiveSelectedEpisodes.includes(ep.ep_number)
                  ? 'text-indigo-400 hover:text-indigo-300'
                  : 'text-slate-500 hover:text-slate-400'
              ]"
            >
              <CheckSquare v-if="effectiveSelectedEpisodes.includes(ep.ep_number)" class="h-4 w-4" />
              <Square v-else class="h-4 w-4" />
            </button>

            <!-- Zone 2: Dedicated, Unobscured Episode Title -->
            <div class="flex-1 min-w-0 px-2 text-center flex items-center justify-center gap-1">
              <span
                data-testid="episode-title"
                :data-episode-title="ep.ep_number"
                class="font-mono font-medium text-xs text-slate-200 truncate select-none block"
              >
                第 {{ ep.ep_number < 10 ? '0' + ep.ep_number : ep.ep_number }} 集
              </span>
              <span
                v-if="isEpisodeMissing(ep.ep_number)"
                data-testid="missing-episode-badge"
                class="rounded bg-amber-900/70 border border-amber-700/50 px-1 py-0.2 text-[10px] text-amber-300 font-sans"
              >
                缺集
              </span>
            </div>

            <!-- Zone 3: Dedicated Preview Trigger Button -->
            <button
              type="button"
              data-testid="btn-preview-episode"
              :data-episode-preview="ep.ep_number"
              :data-active="activePreviewEp === ep.ep_number"
              :disabled="isEpisodeMissing(ep.ep_number)"
              :title="isEpisodeMissing(ep.ep_number) ? '當前鎖定頻道未提供此集數' : '預覽播放'"
              @click.stop="!isEpisodeMissing(ep.ep_number) && switchPreview(ep.ep_number)"
              :class="[
                'flex items-center gap-1 px-2 py-1 rounded text-[11px] font-medium transition-colors shrink-0',
                isEpisodeMissing(ep.ep_number)
                  ? 'opacity-40 cursor-not-allowed bg-slate-800 text-slate-500 border border-slate-800'
                  : activePreviewEp === ep.ep_number
                    ? 'bg-violet-600 text-white shadow-xs'
                    : 'bg-slate-800 text-slate-300 hover:bg-slate-700 hover:text-white border border-slate-700/60'
              ]"
            >
              <span class="sr-only" data-testid="btn-preview-ep"></span>
              <Play class="h-3 w-3 fill-current" :class="{ 'animate-pulse': isPlaying && activePreviewEp === ep.ep_number }" />
              <span>{{ activePreviewEp === ep.ep_number ? '播放中' : '預覽' }}</span>
            </button>
          </div>
        </div>
      </div>

      <!-- Right Column: Episode Preview Dock (5 Columns Deck) -->
      <div
        data-testid="preview-dock"
        class="lg:col-span-5 flex flex-col rounded-xl border border-slate-800 bg-slate-950/60 p-3.5 space-y-3"
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
            :poster="effectiveMetadata?.cover_url ?? undefined"
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
        <div data-testid="timeline-scrubber" class="flex items-center gap-2 pt-1">
          <input
            data-testid="preview-scrubber"
            data-timeline-scrubber
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
            <!-- Continuous Volume Slider with Dynamic Gradient Fill -->
            <div data-testid="volume-slider" class="flex items-center">
              <input
                data-testid="preview-volume"
                data-volume-slider
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
          </div>

          <!-- Single-click Copy Stream URL Button with visual feedback -->
          <button
            type="button"
            data-testid="btn-copy-stream-url"
            :disabled="!currentStreamSource"
            @click="copyStreamUrl"
            class="rounded border border-slate-700 bg-slate-800/80 px-2 py-1 text-[11px] text-slate-300 hover:bg-slate-700 flex items-center gap-1 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
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
            <span
              data-testid="telemetry-status-badge"
              :data-probed="isStreamProbed"
              :class="[
                'text-[10px] px-1.5 py-0.5 rounded border',
                isStreamProbed
                  ? 'text-emerald-400 bg-emerald-950/60 border-emerald-800/50'
                  : 'text-slate-400 bg-slate-950/60 border-slate-800/50'
              ]"
            >
              {{ isStreamProbed ? '探測正常' : '未探測' }}
            </span>
          </div>

          <div class="grid grid-cols-2 gap-2 text-[11px]">
            <div class="rounded bg-slate-950/60 border border-slate-800/60 p-2">
              <div class="text-slate-400 text-[10px]">畫質規格</div>
              <div
                data-testid="telemetry-resolution"
                class="font-mono font-medium text-emerald-300 mt-0.5"
              >
                {{ telemetryDisplay.resolution }}
              </div>
            </div>

            <div class="rounded bg-slate-950/60 border border-slate-800/60 p-2">
              <div class="text-slate-400 text-[10px]">即時碼率</div>
              <div
                data-testid="telemetry-bitrate"
                class="font-mono font-medium text-sky-300 mt-0.5"
              >
                {{ telemetryDisplay.bitrate }}
              </div>
            </div>

            <div class="rounded bg-slate-950/60 border border-slate-800/60 p-2">
              <div class="text-slate-400 text-[10px]">視訊/音訊編碼</div>
              <div
                data-testid="telemetry-codecs"
                class="font-mono text-slate-300 mt-0.5 truncate"
              >
                {{ telemetryDisplay.codecs }}
              </div>
            </div>

            <div class="rounded bg-slate-950/60 border border-slate-800/60 p-2">
              <div class="text-slate-400 text-[10px]">影格率 / 總時長</div>
              <div
                data-testid="telemetry-fps-duration"
                class="font-mono text-slate-300 mt-0.5"
              >
                {{ telemetryDisplay.fpsDuration }}
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
