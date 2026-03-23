<template>
  <div class="space-y-6">
    <AppPageHeader
      eyebrow="About"
      title="关于 AutoDaily"
      description="它是一套偏专业工具型的桌面自动化工作台，目标不是展示，而是让多设备脚本运行更稳定、更清楚。"
    />

    <div class="grid gap-4 xl:grid-cols-[1.1fr_0.9fr]">
      <SurfacePanel class="space-y-5">
        <div class="flex items-center gap-4">
          <div class="flex h-16 w-16 items-center justify-center rounded-[22px] bg-[var(--app-accent-soft)] text-[var(--app-accent)]">
            <Command class="h-8 w-8" />
          </div>
          <div>
            <p class="text-2xl font-semibold text-[var(--app-text-strong)]">AutoDaily</p>
            <p class="text-sm text-[var(--app-text-soft)]">桌面端自动化控制台</p>
          </div>
        </div>

        <div class="space-y-3 text-sm leading-7 text-[var(--app-text-soft)]">
          <p>产品核心围绕设备调度、脚本资产、运行日志和系统环境四个工作面展开。</p>
          <p>这次前端重写的方向不是把它做成 SaaS 后台，而是让它更接近安静、克制、系统化的桌面专业工具。</p>
        </div>
      </SurfacePanel>

      <SurfacePanel class="space-y-4">
        <div class="app-stat">
          <p class="app-stat-label">版本</p>
          <p class="app-stat-value">0.1.0</p>
        </div>
        <div class="app-stat">
          <p class="app-stat-label">技术栈</p>
          <p class="app-stat-value text-base">Vue 3 · TypeScript · Tauri</p>
        </div>
        <div class="app-stat">
          <p class="app-stat-label">定位</p>
          <p class="app-stat-value text-base">多设备自动化工作台</p>
        </div>
      </SurfacePanel>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Command } from 'lucide-vue-next';
import AppPageHeader from '@/components/shared/AppPageHeader.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
</script>
