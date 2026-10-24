<script setup lang="ts">
import { Calendar, Tag, ChevronRight } from 'lucide-vue-next';
import type { SearchResult } from '../types';

defineProps<{
  results: SearchResult[];
  searching: boolean;
  searched: boolean;
  keyword: string;
}>();

const emit = defineEmits<{
  (e: 'select', result: SearchResult): void;
}>();
</script>

<template>
  <div>
    <!-- Loading Skeleton State -->
    <div v-if="searching" data-testid="search-skeleton" class="mt-2 space-y-1.5">
      <div
        v-for="i in 3"
        :key="i"
        class="flex animate-pulse items-center justify-between rounded-lg border border-slate-800 bg-slate-800/30 p-3"
      >
        <div class="flex items-center gap-3">
          <div class="h-4 w-32 rounded bg-slate-700/60"></div>
          <div class="h-4 w-12 rounded bg-slate-800"></div>
          <div class="h-4 w-12 rounded bg-slate-800"></div>
        </div>
        <div class="h-4 w-8 rounded bg-slate-800"></div>
      </div>
    </div>

    <!-- Search Results List -->
    <div v-else-if="results.length > 0" class="mt-2 space-y-1.5 max-h-72 overflow-y-auto pr-1">
      <div
        v-for="(item, idx) in results"
        :key="idx"
        data-testid="search-result-item"
        @click="emit('select', item)"
        class="group flex cursor-pointer items-center justify-between rounded-lg border border-slate-800 bg-slate-800/40 p-3 transition-all hover:border-indigo-500/50 hover:bg-slate-800/80"
      >
        <div class="flex items-center gap-3">
          <span class="font-medium text-sm text-slate-100 group-hover:text-indigo-300 transition-colors">
            {{ item.title }}
          </span>
          <div class="flex items-center gap-1.5">
            <span
              v-if="item.year"
              class="inline-flex items-center gap-0.5 rounded px-1.5 py-0.5 text-[11px] font-medium bg-slate-700/60 text-slate-300"
            >
              <Calendar class="h-2.5 w-2.5 opacity-70" />
              {{ item.year }}
            </span>
            <span
              v-if="item.category"
              class="inline-flex items-center gap-0.5 rounded px-1.5 py-0.5 text-[11px] font-medium bg-indigo-950/50 text-indigo-300 border border-indigo-800/40"
            >
              <Tag class="h-2.5 w-2.5 opacity-70" />
              {{ item.category }}
            </span>
            <span
              class="inline-flex items-center rounded px-1.5 py-0.5 text-[11px] font-medium bg-slate-800 text-slate-400 border border-slate-700/50"
            >
              {{ item.source_name }}
            </span>
          </div>
        </div>

        <div class="flex items-center gap-1 text-xs text-slate-400 group-hover:text-indigo-400 transition-colors">
          <span>載入</span>
          <ChevronRight class="h-3.5 w-3.5 transition-transform group-hover:translate-x-0.5" />
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div
      v-else-if="searched && !searching && results.length === 0"
      data-testid="search-empty-state"
      class="mt-2 rounded-lg border border-slate-800/80 bg-slate-900/30 p-6 text-center text-xs text-slate-400"
    >
      <p class="font-medium text-slate-300">查無符合「{{ keyword }}」的影集</p>
      <p class="mt-1 text-slate-500">請嘗試不同的關鍵字，或切換至「直接輸入網址」手動載入。</p>
    </div>
  </div>
</template>
