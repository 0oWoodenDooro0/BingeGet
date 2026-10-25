<script setup lang="ts">
import { computed } from 'vue';
import { Radio, Sparkles } from 'lucide-vue-next';
import type { ChannelOption } from '../types';
import {
  sortChannelsByQuality,
  getChannelQualityBadge,
  type StreamQualityInfo,
} from '../utils/channelQuality';

const props = defineProps<{
  channels: ChannelOption[];
  selectedChannel: string;
  probeMap?: Record<string, StreamQualityInfo>;
}>();

const emit = defineEmits<{
  (e: 'update:selectedChannel', val: string): void;
}>();

const sortedChannels = computed(() => {
  return sortChannelsByQuality(props.channels, props.probeMap);
});
</script>

<template>
  <div v-if="channels.length > 0" class="rounded-xl border border-slate-800 bg-slate-900/40 p-4 space-y-3">
    <!-- Header -->
    <div class="flex items-center justify-between border-b border-slate-800/80 pb-2">
      <div class="flex items-center gap-2">
        <Radio class="h-4 w-4 text-indigo-400" />
        <span class="text-xs font-semibold uppercase tracking-wider text-slate-400">
          播放線路 (依畫質高至低排序)
        </span>
      </div>
      <div class="flex items-center gap-2">
        <span class="text-[11px] text-emerald-400 flex items-center gap-1">
          <Sparkles class="h-3 w-3" /> 高畫質優先
        </span>
        <span class="text-xs text-slate-400">
          共 {{ sortedChannels.length }} 個線路
        </span>
      </div>
    </div>

    <!-- Quality Card Grid View -->
    <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
      <div
        v-for="c in sortedChannels"
        :key="c.id"
        data-testid="channel-item"
        :data-channel-id="c.id"
        @click="emit('update:selectedChannel', c.id)"
        :class="[
          'flex flex-col rounded-lg border p-2.5 text-left transition-all cursor-pointer',
          selectedChannel === c.id
            ? 'border-indigo-500 bg-indigo-600/20 text-white shadow-sm shadow-indigo-500/10'
            : 'border-slate-800 bg-slate-800/30 text-slate-400 hover:border-slate-700 hover:text-slate-200'
        ]"
      >
        <div class="flex items-center justify-between">
          <span class="font-medium text-xs truncate">{{ c.name }}</span>
          <span
            v-if="getChannelQualityBadge(c.id, probeMap?.[c.id]).tier === 'optimal'"
            class="text-[10px] text-emerald-400 bg-emerald-950/60 px-1 rounded border border-emerald-800/50 shrink-0 ml-1"
          >
            高畫質
          </span>
        </div>
        <div class="mt-1 flex items-center gap-1.5 text-[11px] text-slate-400">
          <span
            data-testid="channel-quality-badge"
            class="font-mono text-slate-300"
          >
            {{ getChannelQualityBadge(c.id, probeMap?.[c.id]).resolution }}
          </span>
          <span v-if="getChannelQualityBadge(c.id, probeMap?.[c.id]).bitrate">·</span>
          <span
            v-if="getChannelQualityBadge(c.id, probeMap?.[c.id]).bitrate"
            class="font-mono text-slate-400"
          >
            {{ getChannelQualityBadge(c.id, probeMap?.[c.id]).bitrate }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
