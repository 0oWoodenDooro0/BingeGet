<script setup lang="ts">
import { Lock, AlertCircle } from 'lucide-vue-next';
import type { ChannelOption } from '../types';

defineProps<{
  channels: ChannelOption[];
  selectedChannel: string;
}>();

const emit = defineEmits<{
  (e: 'update:selectedChannel', val: string): void;
}>();
</script>

<template>
  <div v-if="channels.length > 0" class="rounded-xl border border-indigo-900/40 bg-indigo-950/20 p-4 space-y-2">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <div class="flex h-6 w-6 items-center justify-center rounded-md bg-indigo-500/20 text-indigo-400">
          <Lock class="h-3.5 w-3.5" />
        </div>
        <span class="text-xs font-semibold uppercase tracking-wider text-indigo-300">
          播放渠道標籤鎖定 (Channel Lock)
        </span>
      </div>
      <span class="text-[11px] text-slate-400">
        偵測到 {{ channels.length }} 個片源渠道
      </span>
    </div>

    <div class="flex flex-col sm:flex-row gap-3 pt-1">
      <div class="flex-1">
        <select
          :value="selectedChannel"
          @change="emit('update:selectedChannel', ($event.target as HTMLSelectElement).value)"
          class="w-full rounded-lg border border-indigo-500/30 bg-slate-900 px-3 py-2 text-sm text-slate-100 font-medium focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500"
        >
          <option v-for="c in channels" :key="c.id" :value="c.id" class="bg-slate-900 text-slate-100">
            {{ c.name }}
          </option>
        </select>
      </div>
    </div>

    <div class="flex items-center gap-1.5 text-xs text-indigo-300/80 pt-1">
      <AlertCircle class="h-3.5 w-3.5 shrink-0 text-indigo-400" />
      <span>系統已鎖定此渠道，避免不同集數因片源順序浮動而混入不同壓制組，確保全集字體與畫質一致。</span>
    </div>
  </div>
</template>
