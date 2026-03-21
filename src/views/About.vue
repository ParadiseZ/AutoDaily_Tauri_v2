<template>
  <div class="w-full max-w-3xl mx-auto flex flex-col items-center justify-center gap-6 py-20">
    <div class="w-24 h-24 rounded-3xl bg-gradient-to-tr from-base-content/80 to-base-content/30 flex items-center justify-center shadow-lg">
      <Command class="w-12 h-12 text-base-100" />
    </div>
    
    <div class="text-center">
      <h1 class="text-3xl font-semibold tracking-tight text-base-content">AutoDaily</h1>
      <p class="text-[13px] font-mono text-base-content/50 mt-2">Vers. 0.1.0-alpha</p>
    </div>
    
    <div class="w-full max-w-md bg-base-100 border border-base-content/5 rounded-2xl p-6 mt-6 shadow-sm flex flex-col items-center">
       <p class="text-sm text-base-content/70 text-center leading-relaxed">
          AutoDaily 是一款专注于多设备管理的自适应视觉测试与控制台工具。<br/>
          旨在提供最高效的原生体验。
       </p>
       
       <button class="btn btn-sm mt-6 font-medium bg-base-content text-base-100 rounded-lg hover:bg-base-content/90 border-none tracking-wide px-6">
          检查更新
       </button>
    </div>
  </div>
</template>
<script setup lang="ts">
import { Command } from 'lucide-vue-next';
</script>
