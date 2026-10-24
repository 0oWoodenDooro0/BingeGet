<script setup lang="ts">
import { Film, Sparkles } from 'lucide-vue-next';
</script>

<template>
  <header class="flex items-center justify-between border-b border-slate-800 bg-slate-900/60 px-6 py-4 backdrop-blur-md">
    <div class="flex items-center space-x-3">
      <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-tr from-indigo-600 to-violet-500 shadow-lg shadow-indigo-500/20">
        <Film class="h-5 w-5 text-white" />
      </div>
      <div>
        <div class="flex items-center space-x-2">
          <h1 class="text-lg font-bold tracking-tight text-white">BingeGet</h1>
          <span class="rounded-full bg-indigo-500/10 px-2 py-0.5 text-xs font-medium text-indigo-400 border border-indigo-500/20">v0.1.0</span>
        </div>
        <p class="text-xs text-slate-400">影集 m3u8 自動抓取與極速下載器 (Channel Lock 壓制源鎖定)</p>
      </div>
    </div>
    
    <div class="flex items-center space-x-3 text-xs text-slate-400">
      <span class="inline-flex items-center gap-1.5 rounded-md bg-slate-800/80 px-2.5 py-1 text-slate-300">
        <Sparkles class="h-3.5 w-3.5 text-amber-400" />
        n-m3u8dl-re + ffmpeg 驅動
      </span>
    </div>
  </header>
</template>
