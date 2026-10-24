<script setup lang="ts">
import { Play, RotateCcw, XCircle, CheckCircle, Clock, Loader2 } from 'lucide-vue-next';
import type { QueueItem } from '../types';

defineProps<{
  queue: QueueItem[];
}>();

const emit = defineEmits<{
  (e: 'retry', episode: number): void;
  (e: 'cancel', episode: number): void;
}>();

const getPercent = (item: QueueItem): number => {
  if (item.state.type === 'Completed') return 100;
  if (item.state.type === 'Progress') return item.state.data.percent;
  if (item.state.type === 'Muxing') return 99;
  return 0;
};
</script>

<template>
  <div v-if="queue.length > 0" class="rounded-xl border border-slate-800 bg-slate-900/40 p-4 space-y-3">
    <div class="flex items-center justify-between border-b border-slate-800/80 pb-2">
      <h2 class="text-xs font-semibold uppercase tracking-wider text-slate-400 flex items-center gap-1.5">
        <Play class="h-4 w-4 text-indigo-400" />
        下載隊列 ({{ queue.length }})
      </h2>
    </div>

    <div class="space-y-2 max-h-72 overflow-y-auto pr-1">
      <div
        v-for="item in queue"
        :key="item.id"
        class="rounded-lg border border-slate-800/80 bg-slate-950/40 p-3 space-y-2"
      >
        <div class="flex items-center justify-between text-xs">
          <div class="flex items-center gap-2">
            <span class="font-mono font-medium text-slate-200">
              {{ item.spec.custom_title }} - S{{ item.spec.season < 10 ? '0' + item.spec.season : item.spec.season }}E{{ item.spec.episode < 10 ? '0' + item.spec.episode : item.spec.episode }}.mp4
            </span>

            <span
              v-if="item.state.type === 'Downloading'"
              class="inline-flex items-center gap-1 text-[11px] font-medium text-amber-400 bg-amber-400/10 px-1.5 py-0.5 rounded"
            >
              <Loader2 class="h-3 w-3 animate-spin" />
              下載中
            </span>
            <span
              v-else-if="item.state.type === 'Progress'"
              class="inline-flex items-center gap-1 text-[11px] font-medium text-indigo-400 bg-indigo-500/10 px-1.5 py-0.5 rounded"
            >
              <Loader2 class="h-3 w-3 animate-spin" />
              {{ item.state.data.speed }} (剩餘 {{ item.state.data.eta }})
            </span>
            <span
              v-else-if="item.state.type === 'Muxing'"
              class="inline-flex items-center gap-1 text-[11px] font-medium text-purple-400 bg-purple-400/10 px-1.5 py-0.5 rounded"
            >
              <Loader2 class="h-3 w-3 animate-spin" />
              封裝中
            </span>
            <span
              v-else-if="item.state.type === 'Completed'"
              class="inline-flex items-center gap-1 text-[11px] font-medium text-emerald-400 bg-emerald-400/10 px-1.5 py-0.5 rounded"
            >
              <CheckCircle class="h-3 w-3" />
              已完成
            </span>
            <span
              v-else-if="item.state.type === 'Queued'"
              class="inline-flex items-center gap-1 text-[11px] font-medium text-slate-400 bg-slate-800 px-1.5 py-0.5 rounded"
            >
              <Clock class="h-3 w-3" />
              排隊中
            </span>
            <span
              v-else-if="item.state.type === 'Failed'"
              class="inline-flex items-center gap-1 text-[11px] font-medium text-rose-400 bg-rose-400/10 px-1.5 py-0.5 rounded"
            >
              <XCircle class="h-3 w-3" />
              失敗: {{ item.state.error }}
            </span>
          </div>

          <div class="flex items-center gap-2">
            <span class="font-mono text-slate-400">
              {{ getPercent(item).toFixed(1) }}%
            </span>

            <button
              v-if="item.state.type === 'Failed'"
              @click="emit('retry', item.spec.episode)"
              type="button"
              class="text-indigo-400 hover:text-indigo-300 p-1"
              title="重試"
            >
              <RotateCcw class="h-3.5 w-3.5" />
            </button>
            <button
              v-if="item.state.type !== 'Completed'"
              @click="emit('cancel', item.spec.episode)"
              type="button"
              class="text-slate-500 hover:text-rose-400 p-1"
              title="取消"
            >
              <XCircle class="h-3.5 w-3.5" />
            </button>
          </div>
        </div>

        <!-- Progress Bar -->
        <div class="h-1.5 w-full overflow-hidden rounded-full bg-slate-800">
          <div
            class="h-full rounded-full transition-all duration-300"
            :class="[
              item.state.type === 'Completed' ? 'bg-emerald-500' :
              item.state.type === 'Failed' ? 'bg-rose-500' :
              'bg-gradient-to-r from-indigo-500 to-violet-500'
            ]"
            :style="{ width: `${getPercent(item)}%` }"
          />
        </div>
      </div>
    </div>
  </div>
</template>
