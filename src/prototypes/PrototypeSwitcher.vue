<script setup lang="ts">
import { onMounted, onUnmounted, computed } from 'vue';
import { ChevronLeft, ChevronRight, Layers } from 'lucide-vue-next';

interface VariantOption {
  key: string;
  name: string;
  desc: string;
}

const props = defineProps<{
  variants: VariantOption[];
  current: string;
}>();

const emit = defineEmits<{
  (e: 'change', key: string): void;
}>();

const currentIndex = computed(() => {
  const idx = props.variants.findIndex(v => v.key === props.current);
  return idx >= 0 ? idx : 0;
});

const currentVariant = computed(() => {
  return props.variants[currentIndex.value] || props.variants[0];
});

const selectVariant = (key: string) => {
  emit('change', key);
  try {
    const url = new URL(window.location.href);
    url.searchParams.set('variant', key);
    window.history.replaceState({}, '', url.toString());
  } catch {
    // Ignore in non-browser envs
  }
};

const goPrev = () => {
  const nextIdx = (currentIndex.value - 1 + props.variants.length) % props.variants.length;
  selectVariant(props.variants[nextIdx].key);
};

const goNext = () => {
  const nextIdx = (currentIndex.value + 1) % props.variants.length;
  selectVariant(props.variants[nextIdx].key);
};

const handleKeyDown = (e: KeyboardEvent) => {
  const target = e.target as HTMLElement | null;
  if (
    target &&
    (target.tagName === 'INPUT' ||
      target.tagName === 'TEXTAREA' ||
      target.isContentEditable)
  ) {
    return;
  }

  if (e.key === 'ArrowLeft') {
    e.preventDefault();
    goPrev();
  } else if (e.key === 'ArrowRight') {
    e.preventDefault();
    goNext();
  }
};

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});
</script>

<template>
  <aside
    aria-label="原型版本切換器"
    class="fixed bottom-5 left-1/2 -translate-x-1/2 z-50 flex items-center gap-1.5 rounded-full border border-indigo-400/40 bg-slate-900/95 px-3 py-1.5 text-xs text-slate-100 shadow-2xl backdrop-blur-md select-none ring-1 ring-white/10"
  >
    <div class="flex items-center gap-1.5 pr-2 border-r border-slate-700/80 text-indigo-400 font-semibold">
      <Layers class="h-3.5 w-3.5 animate-pulse" />
      <span class="text-[11px] uppercase tracking-wider text-slate-300">原型方案</span>
    </div>

    <button
      type="button"
      @click="goPrev"
      aria-label="上一個版本 (←)"
      title="上一個版本 (快捷鍵：方向鍵左 ←)"
      class="flex h-7 w-7 items-center justify-center rounded-full text-slate-300 hover:bg-slate-800 hover:text-white transition-colors"
    >
      <ChevronLeft class="h-4 w-4" />
    </button>

    <div class="flex items-center gap-1 px-1.5 min-w-[190px] justify-center text-center">
      <span class="font-mono font-bold text-indigo-400 bg-indigo-950/80 px-1.5 py-0.5 rounded border border-indigo-500/30">
        方案 {{ currentVariant.key }}
      </span>
      <span class="font-medium text-slate-200 text-xs truncate max-w-[140px]">
        {{ currentVariant.name }}
      </span>
    </div>

    <button
      type="button"
      @click="goNext"
      aria-label="下一個版本 (→)"
      title="下一個版本 (快捷鍵：方向鍵右 →)"
      class="flex h-7 w-7 items-center justify-center rounded-full text-slate-300 hover:bg-slate-800 hover:text-white transition-colors"
    >
      <ChevronRight class="h-4 w-4" />
    </button>
  </aside>
</template>
