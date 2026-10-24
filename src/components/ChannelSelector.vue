<script setup lang="ts">
import { Radio } from 'lucide-vue-next';
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
  <div v-if="channels.length > 0" class="rounded-xl border border-slate-800 bg-slate-900/40 p-4 space-y-3">
    <div class="flex items-center justify-between border-b border-slate-800/80 pb-2">
      <div class="flex items-center gap-2">
        <Radio class="h-4 w-4 text-indigo-400" />
        <span class="text-xs font-semibold uppercase tracking-wider text-slate-400">
          播放線路
        </span>
      </div>
      <span class="text-xs text-slate-400">
        共 {{ channels.length }} 個線路
      </span>
    </div>

    <div>
      <select
        :value="selectedChannel"
        @change="emit('update:selectedChannel', ($event.target as HTMLSelectElement).value)"
        class="w-full rounded-lg border border-slate-700 bg-slate-800/80 px-3 py-2 text-sm text-slate-100 font-medium focus:border-indigo-500 focus:outline-none"
      >
        <option
          v-for="c in channels"
          :key="c.id"
          :value="c.id"
          class="bg-slate-900 text-slate-100"
        >
          {{ c.name }}
        </option>
      </select>
    </div>
  </div>
</template>
