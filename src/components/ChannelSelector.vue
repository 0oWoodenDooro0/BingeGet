<script setup lang="ts">
import { Radio } from 'lucide-vue-next';
import type { ChannelOption } from '../types';
import type { StreamQualityInfo } from '../utils/channelQuality';

defineProps<{
  channels: ChannelOption[];
  selectedChannel: string;
  probeMap?: Record<string, StreamQualityInfo>;
}>();

const emit = defineEmits<{
  (e: 'update:selectedChannel', val: string): void;
}>();
</script>

<template>
  <div v-if="channels.length > 0" class="rounded-xl border border-slate-800 bg-slate-900/40 p-4 space-y-3">
    <!-- Header -->
    <div class="flex items-center justify-between border-b border-slate-800/80 pb-2">
      <div class="flex items-center gap-2">
        <Radio class="h-4 w-4 text-indigo-400" />
        <span class="text-xs font-semibold uppercase tracking-wider text-slate-400">
          播放頻道
        </span>
      </div>
      <span data-testid="channel-summary-counter" class="text-xs text-slate-400">
        共 {{ channels.length }} 個可用頻道
      </span>
    </div>

    <!-- Channel Buttons Grid -->
    <div class="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-6 gap-2">
      <button
        type="button"
        v-for="c in channels"
        :key="c.id"
        data-testid="channel-item"
        :data-channel-id="c.id"
        @click="emit('update:selectedChannel', c.id)"
        :class="[
          'flex items-center justify-center rounded-lg border py-2.5 px-3 text-center text-xs font-medium transition-all cursor-pointer select-none',
          selectedChannel === c.id
            ? 'border-indigo-500 bg-indigo-600/20 text-white shadow-sm shadow-indigo-500/10 ring-1 ring-indigo-500/50'
            : 'border-slate-800 bg-slate-800/30 text-slate-400 hover:border-slate-700 hover:text-slate-200 hover:bg-slate-800/60'
        ]"
      >
        <span class="truncate">{{ c.name }}</span>
      </button>
    </div>
  </div>
</template>
