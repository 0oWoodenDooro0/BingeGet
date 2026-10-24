<script setup lang="ts">
import { Film } from 'lucide-vue-next';
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
        <p class="text-xs text-slate-400">影集 m3u8 高速下載工具</p>
      </div>
    </div>
  </header>
</template>
