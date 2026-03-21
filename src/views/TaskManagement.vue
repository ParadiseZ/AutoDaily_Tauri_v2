<template>
  <div class="w-full max-w-5xl mx-auto flex flex-col gap-6 py-4">
    <div class="px-2">
      <h1 class="text-2xl font-semibold tracking-tight text-base-content">任务管理</h1>
      <p class="text-sm text-base-content/60 mt-1">全局调度设备的运行脚本与任务队列。</p>
    </div>
    
    <!-- Global Control Bar -->
    <div class="bg-base-100 border border-base-content/5 rounded-2xl p-4 flex items-center justify-between shadow-sm">
      <div class="flex items-center gap-3">
        <div class="w-2.5 h-2.5 rounded-full bg-emerald-500 animate-pulse"></div>
        <span class="text-sm font-medium">3 台设备在线，2 台运行中</span>
      </div>
      <div class="flex gap-2">
        <button class="btn btn-sm btn-ghost text-base-content/70 hover:text-base-content bg-base-200/50 hover:bg-base-200 font-medium border-none shadow-none rounded-lg">全部暂停</button>
        <button class="btn btn-sm bg-base-content text-base-100 hover:bg-base-content/90 font-medium border-none shadow-none rounded-lg">全部启动</button>
      </div>
    </div>
    
    <!-- Device List Container -->
    <div class="flex flex-col gap-3 mt-2">
      <h2 class="text-xs font-semibold uppercase tracking-wider text-base-content/50 ml-2 mb-1">设备节点</h2>
      
      <!-- Device Item Placeholder -->
      <div class="bg-base-100 border border-base-content/5 rounded-2xl p-4 flex items-center justify-between shadow-sm hover:shadow-md transition-all duration-200 group cursor-default">
        <div class="flex items-center gap-4">
          <div class="w-10 h-10 rounded-xl bg-base-200/50 flex items-center justify-center text-base-content/60 group-hover:bg-primary/10 group-hover:text-primary transition-colors">
            <Smartphone class="w-5 h-5" />
          </div>
          <div class="flex flex-col justify-center">
            <h3 class="text-[14px] font-semibold text-base-content/90 tracking-wide">Nexus 7 · 192.168.1.101</h3>
            <div class="text-[12px] text-base-content/50 mt-0.5 flex items-center gap-2">
              <span class="text-emerald-500 font-medium flex items-center gap-1">
                <div class="w-1.5 h-1.5 rounded-full bg-emerald-500"></div>运行中
              </span>
              <span class="text-base-content/20">|</span>
              <span class="truncate max-w-[200px]">正在执行：日常任务一条龙</span>
            </div>
          </div>
        </div>
        <div class="flex items-center gap-2 opacity-60 group-hover:opacity-100 transition-opacity">
          <button class="btn btn-sm btn-square btn-ghost text-base-content/60 hover:text-base-content"><Pause class="w-4 h-4" /></button>
          <button class="btn btn-sm btn-square btn-ghost text-base-content/60 hover:text-base-content"><Settings class="w-4 h-4" /></button>
        </div>
      </div>
      
      <!-- Device Item Placeholder (Offline) -->
      <div class="bg-base-100 border border-base-content/5 rounded-2xl p-4 flex items-center justify-between shadow-sm hover:shadow-md transition-all duration-200 group cursor-default">
        <div class="flex items-center gap-4">
          <div class="w-10 h-10 rounded-xl bg-base-200/50 flex items-center justify-center text-base-content/40">
            <Smartphone class="w-5 h-5" />
          </div>
          <div class="flex flex-col justify-center">
            <h3 class="text-[14px] font-semibold text-base-content/60 tracking-wide">Pixel 5 · Offline</h3>
            <div class="text-[12px] text-base-content/40 mt-0.5 flex items-center gap-2">
              <span class="font-medium flex items-center gap-1 text-base-content/50">
                <div class="w-1.5 h-1.5 rounded-full bg-base-content/30"></div>未连接
              </span>
              <span class="text-base-content/20">|</span>
              <span>队列为空</span>
            </div>
          </div>
        </div>
        <div class="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
          <button class="btn btn-sm btn-square btn-ghost text-base-content/60 hover:text-base-content"><Settings class="w-4 h-4" /></button>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { Smartphone, Pause, Settings } from 'lucide-vue-next';
</script>
