<script setup lang="ts">
import { Folder, Sliders, Layers } from 'lucide-vue-next';

defineProps<{
  customTitle: string;
  season: number;
  saveDir: string;
  threads: number;
  maxConcurrent: number;
  totalEpisodes: number;
}>();

const emit = defineEmits<{
  (e: 'update:customTitle', val: string): void;
  (e: 'update:season', val: number): void;
  (e: 'update:saveDir', val: string): void;
  (e: 'update:threads', val: number): void;
  (e: 'update:maxConcurrent', val: number): void;
}>();
</script>

<template>
  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    <!-- Naming Configuration -->
    <div class="rounded-xl border border-slate-800 bg-slate-900/40 p-4 space-y-3">
      <div class="flex items-center justify-between border-b border-slate-800/80 pb-2">
        <h2 class="text-xs font-semibold uppercase tracking-wider text-slate-400 flex items-center gap-1.5">
          <Layers class="h-4 w-4 text-indigo-400" />
          影集命名與媒體庫規範 (Jellyfin / Plex)
        </h2>
        <span v-if="totalEpisodes > 0" class="text-xs font-medium text-slate-400">
          共 {{ totalEpisodes }} 集
        </span>
      </div>

      <div class="grid grid-cols-3 gap-3">
        <div class="col-span-2">
          <label class="block text-xs text-slate-400 mb-1">自訂劇名 (必填)</label>
          <input
            :value="customTitle"
            @input="emit('update:customTitle', ($event.target as HTMLInputElement).value)"
            type="text"
            placeholder="例如：繁花"
            class="w-full rounded-lg border border-slate-700 bg-slate-800/80 px-3 py-2 text-sm text-slate-100 placeholder-slate-500 focus:border-indigo-500 focus:outline-none"
          />
        </div>

        <div>
          <label class="block text-xs text-slate-400 mb-1">季數</label>
          <div class="relative">
            <span class="absolute inset-y-0 left-0 flex items-center pl-3 text-xs text-slate-400">S</span>
            <input
              :value="season"
              @input="emit('update:season', parseInt(($event.target as HTMLInputElement).value) || 1)"
              type="number"
              min="1"
              max="99"
              class="w-full rounded-lg border border-slate-700 bg-slate-800/80 py-2 pl-7 pr-3 text-sm text-slate-100 focus:border-indigo-500 focus:outline-none"
            />
          </div>
        </div>
      </div>

      <div class="text-[11px] text-slate-500 font-mono bg-slate-950/40 px-2.5 py-1.5 rounded border border-slate-800/50">
        預計命名範例：<span class="text-indigo-400 font-medium">{{ customTitle || '劇名' }} - S{{ season < 10 ? '0' + season : season }}E01.mp4</span>
      </div>
    </div>

    <!-- Download Settings -->
    <div class="rounded-xl border border-slate-800 bg-slate-900/40 p-4 space-y-3">
      <div class="flex items-center justify-between border-b border-slate-800/80 pb-2">
        <h2 class="text-xs font-semibold uppercase tracking-wider text-slate-400 flex items-center gap-1.5">
          <Sliders class="h-4 w-4 text-indigo-400" />
          儲存目錄與併發效能
        </h2>
      </div>

      <div>
        <label class="block text-xs text-slate-400 mb-1">儲存目錄</label>
        <div class="flex gap-2">
          <div class="relative flex-1">
            <input
              :value="saveDir"
              @input="emit('update:saveDir', ($event.target as HTMLInputElement).value)"
              type="text"
              placeholder="/media/TV/ShowName"
              class="w-full rounded-lg border border-slate-700 bg-slate-800/80 px-3 py-2 text-sm text-slate-100 focus:border-indigo-500 focus:outline-none"
            />
          </div>
          <button
            type="button"
            class="rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-xs font-medium text-slate-300 hover:bg-slate-700 flex items-center gap-1"
          >
            <Folder class="h-3.5 w-3.5" />
            瀏覽
          </button>
        </div>
      </div>

      <div class="grid grid-cols-2 gap-3 pt-1">
        <div>
          <div class="flex justify-between text-xs text-slate-400 mb-1">
            <span>單集切片線程</span>
            <span class="text-indigo-400 font-medium">{{ threads }}</span>
          </div>
          <input
            :value="threads"
            @input="emit('update:threads', parseInt(($event.target as HTMLInputElement).value))"
            type="range"
            min="4"
            max="32"
            step="4"
            class="w-full accent-indigo-500"
          />
        </div>

        <div>
          <div class="flex justify-between text-xs text-slate-400 mb-1">
            <span>同時下載集數</span>
            <span class="text-indigo-400 font-medium">{{ maxConcurrent }} 集並發</span>
          </div>
          <select
            :value="maxConcurrent"
            @change="emit('update:maxConcurrent', parseInt(($event.target as HTMLSelectElement).value))"
            class="w-full rounded-lg border border-slate-700 bg-slate-800 px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none"
          >
            <option :value="1">1 集 (最安全防封)</option>
            <option :value="2">2 集 (推薦平衡)</option>
            <option :value="3">3 集 (極速下載)</option>
          </select>
        </div>
      </div>
    </div>
  </div>
</template>
