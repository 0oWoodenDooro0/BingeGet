<script setup lang="ts">
import { ref } from 'vue';
import { CheckSquare, Square, Check } from 'lucide-vue-next';
import type { EpisodeInfo } from '../types';

const props = defineProps<{
  episodes: EpisodeInfo[];
  selectedEpisodes: number[];
}>();

const emit = defineEmits<{
  (e: 'update:selectedEpisodes', val: number[]): void;
}>();

const rangeStart = ref(1);
const rangeEnd = ref(10);

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
  <div v-if="episodes.length > 0" class="rounded-xl border border-slate-800 bg-slate-900/40 p-4 space-y-3">
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
          class="rounded-md border border-slate-700 bg-slate-800/80 px-2.5 py-1 text-slate-300 hover:bg-slate-700 flex items-center gap-1"
        >
          <CheckSquare class="h-3 w-3" /> 全選
        </button>
        <button
          type="button"
          @click="clearAll"
          class="rounded-md border border-slate-700 bg-slate-800/80 px-2.5 py-1 text-slate-300 hover:bg-slate-700 flex items-center gap-1"
        >
          <Square class="h-3 w-3" /> 清空
        </button>
        <button
          type="button"
          @click="selectFirstN(10)"
          class="rounded-md border border-slate-700 bg-slate-800/80 px-2.5 py-1 text-slate-300 hover:bg-slate-700"
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
            class="rounded border border-indigo-500/30 bg-indigo-500/20 px-2 py-0.5 text-indigo-300 hover:bg-indigo-500/30"
          >
            套用範圍
          </button>
        </div>
      </div>
    </div>

    <!-- Episode Grid -->
    <div class="grid grid-cols-4 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-12 gap-2 max-h-60 overflow-y-auto pr-1">
      <button
        v-for="ep in episodes"
        :key="ep.ep_number"
        type="button"
        @click="toggleEpisode(ep.ep_number)"
        :class="[
          'relative flex flex-col items-center justify-center rounded-lg border p-2 text-xs font-medium transition-all',
          selectedEpisodes.includes(ep.ep_number)
            ? 'border-indigo-500 bg-indigo-600/20 text-white shadow-sm shadow-indigo-500/10'
            : 'border-slate-800 bg-slate-800/40 text-slate-400 hover:border-slate-700 hover:text-slate-200'
        ]"
      >
        <span>第 {{ ep.ep_number < 10 ? '0' + ep.ep_number : ep.ep_number }} 集</span>
        <div
          v-if="selectedEpisodes.includes(ep.ep_number)"
          class="absolute top-1 right-1 flex h-3.5 w-3.5 items-center justify-center rounded-full bg-indigo-500 text-white"
        >
          <Check class="h-2 w-2 stroke-[3]" />
        </div>
      </button>
    </div>
  </div>
</template>
