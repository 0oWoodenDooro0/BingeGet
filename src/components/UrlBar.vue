<script setup lang="ts">
import { ref } from 'vue';
import { Search, Loader2, Globe } from 'lucide-vue-next';

const props = defineProps<{
  loading: boolean;
}>();

const emit = defineEmits<{
  (e: 'parse', url: string): void;
}>();

const url = ref('https://chinaq.fun/tv-cn/202614235/');

const handleParse = () => {
  if (!url.value.trim() || props.loading) return;
  emit('parse', url.value.trim());
};
</script>

<template>
  <div class="rounded-xl border border-slate-800 bg-slate-900/40 p-4 shadow-sm backdrop-blur-sm">
    <label class="mb-2 block text-xs font-semibold uppercase tracking-wider text-slate-400">
      影集網址
    </label>
    <div class="flex gap-2">
      <div class="relative flex-1">
        <div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3 text-slate-500">
          <Globe class="h-4 w-4" />
        </div>
        <input
          v-model="url"
          type="text"
          placeholder="貼上影集網址，例如 https://chinaq.fun/tv-cn/202614235/"
          class="w-full rounded-lg border border-slate-700/80 bg-slate-800/80 py-2.5 pl-10 pr-4 text-sm text-slate-100 placeholder-slate-500 transition-all focus:border-indigo-500 focus:bg-slate-800 focus:outline-none focus:ring-1 focus:ring-indigo-500"
          @keyup.enter="handleParse"
        />
      </div>
      <button
        @click="handleParse"
        :disabled="loading || !url.trim()"
        class="inline-flex items-center justify-center gap-2 rounded-lg bg-indigo-600 px-6 py-2.5 text-sm font-medium text-white shadow-lg shadow-indigo-600/30 transition-all hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 disabled:cursor-not-allowed disabled:opacity-50"
      >
        <Loader2 v-if="loading" class="h-4 w-4 animate-spin" />
        <Search v-else class="h-4 w-4" />
        <span>{{ loading ? '解析中...' : '開始解析' }}</span>
      </button>
    </div>
  </div>
</template>
