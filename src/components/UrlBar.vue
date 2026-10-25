<script setup lang="ts">
import { ref } from 'vue';
import { Search, Loader2, Globe, Tv, AlertCircle } from 'lucide-vue-next';
import SearchResultList from './SearchResultList.vue';
import type { SearchResult } from '../types';

const props = defineProps<{
  loading: boolean;
  onSearch?: (keyword: string) => Promise<SearchResult[]>;
}>();

const emit = defineEmits<{
  (e: 'parse', url: string): void;
  (e: 'select-result', result: SearchResult): void;
}>();

const mode = ref<'search' | 'url'>('search');

// URL mode state
const url = ref('');

// Search mode state
const keyword = ref('');
const searching = ref(false);
const searched = ref(false);
const searchError = ref('');
const results = ref<SearchResult[]>([]);

const isTauri = () => {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
};

const handleParse = () => {
  if (!url.value.trim() || props.loading) return;
  emit('parse', url.value.trim());
};

const handleSearch = async () => {
  const kw = keyword.value.trim();
  if (!kw || searching.value || props.loading) return;

  searching.value = true;
  searched.value = true;
  searchError.value = '';
  results.value = [];

  try {
    if (props.onSearch) {
      results.value = await props.onSearch(kw);
    } else if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      results.value = await invoke<SearchResult[]>('search_series', { keyword: kw });
    } else {
      // Mock fallback for browser / development
      results.value = [
        {
          title: `${kw} (示範影集)`,
          year: '2024',
          category: '韓劇',
          url: 'https://example.com/drama/sample/',
          source_id: 'sample',
          source_name: 'SampleSource',
        },
      ];
    }
  } catch (err: unknown) {
    searchError.value = err instanceof Error ? err.message : String(err);
  } finally {
    searching.value = false;
  }
};

const handleSelect = (result: SearchResult) => {
  url.value = result.url;
  emit('select-result', result);
};
</script>

<template>
  <div class="rounded-xl border border-slate-800 bg-slate-900/40 p-4 shadow-sm backdrop-blur-sm">
    <!-- Mode Tabs -->
    <div class="mb-3 flex items-center gap-2 border-b border-slate-800/80 pb-3">
      <button
        type="button"
        data-testid="tab-search"
        @click="mode = 'search'"
        :class="[
          'inline-flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-semibold transition-all',
          mode === 'search'
            ? 'bg-indigo-600 text-white shadow-sm shadow-indigo-600/30'
            : 'text-slate-400 hover:bg-slate-800 hover:text-slate-200'
        ]"
      >
        <Search class="h-3.5 w-3.5" />
        <span>影集搜尋</span>
      </button>

      <button
        type="button"
        data-testid="tab-url"
        @click="mode = 'url'"
        :class="[
          'inline-flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-semibold transition-all',
          mode === 'url'
            ? 'bg-indigo-600 text-white shadow-sm shadow-indigo-600/30'
            : 'text-slate-400 hover:bg-slate-800 hover:text-slate-200'
        ]"
      >
        <Globe class="h-3.5 w-3.5" />
        <span>直接輸入網址</span>
      </button>
    </div>

    <!-- Search Mode View -->
    <div v-if="mode === 'search'" class="space-y-3">
      <div class="flex gap-2">
        <div class="relative flex-1">
          <div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3 text-slate-500">
            <Tv class="h-4 w-4" />
          </div>
          <input
            v-model="keyword"
            data-testid="search-input"
            type="text"
            placeholder="輸入影集名稱或關鍵字"
            class="w-full rounded-lg border border-slate-700/80 bg-slate-800/80 py-2.5 pl-10 pr-4 text-sm text-slate-100 placeholder-slate-500 transition-all focus:border-indigo-500 focus:bg-slate-800 focus:outline-none focus:ring-1 focus:ring-indigo-500"
            @keyup.enter="handleSearch"
          />
        </div>
        <button
          data-testid="search-btn"
          type="button"
          @click="handleSearch"
          :disabled="searching || loading || !keyword.trim()"
          class="inline-flex items-center justify-center gap-2 rounded-lg bg-indigo-600 px-6 py-2.5 text-sm font-medium text-white shadow-lg shadow-indigo-600/30 transition-all hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 disabled:cursor-not-allowed disabled:opacity-50"
        >
          <Loader2 v-if="searching" class="h-4 w-4 animate-spin" />
          <Search v-else class="h-4 w-4" />
          <span>{{ searching ? '搜尋中...' : '搜尋' }}</span>
        </button>
      </div>

      <!-- Search Error Banner -->
      <div v-if="searchError" class="flex items-center gap-2 rounded-lg border border-rose-500/30 bg-rose-500/10 p-2.5 text-xs text-rose-300">
        <AlertCircle class="h-4 w-4 shrink-0 text-rose-400" />
        <span>{{ searchError }}</span>
      </div>

      <!-- Delegated Search Results & States -->
      <SearchResultList
        :results="results"
        :searching="searching"
        :searched="searched"
        :keyword="keyword"
        @select="handleSelect"
      />
    </div>

    <!-- Direct URL Mode View -->
    <div v-else class="space-y-2">
      <div class="flex gap-2">
        <div class="relative flex-1">
          <div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3 text-slate-500">
            <Globe class="h-4 w-4" />
          </div>
          <input
            v-model="url"
            data-testid="url-input"
            type="text"
            placeholder="影集網址"
            class="w-full rounded-lg border border-slate-700/80 bg-slate-800/80 py-2.5 pl-10 pr-4 text-sm text-slate-100 placeholder-slate-500 transition-all focus:border-indigo-500 focus:bg-slate-800 focus:outline-none focus:ring-1 focus:ring-indigo-500"
            @keyup.enter="handleParse"
          />
        </div>
        <button
          data-testid="submit-btn"
          type="button"
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
  </div>
</template>
