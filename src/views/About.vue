<template>
  <div class="space-y-6">
    <AppPageHeader
      title="关于 AutoDaily"
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
          <p>面向本地设备调度、脚本资产维护、运行日志追踪和视觉自动化调试。</p>
          <p>当前版本侧重桌面端工作流，脚本、设备和环境配置都保留在本机可控范围内。</p>
        </div>

        <div class="grid gap-3 md:grid-cols-3">
          <div class="rounded-[20px] border border-[var(--app-border)] bg-[var(--app-panel-muted)]/65 px-4 py-3">
            <p class="text-xs text-[var(--app-text-faint)]">设备</p>
            <p class="mt-1 text-base font-semibold text-[var(--app-text-strong)]">多实例</p>
          </div>
          <div class="rounded-[20px] border border-[var(--app-border)] bg-[var(--app-panel-muted)]/65 px-4 py-3">
            <p class="text-xs text-[var(--app-text-faint)]">脚本</p>
            <p class="mt-1 text-base font-semibold text-[var(--app-text-strong)]">本地优先</p>
          </div>
          <div class="rounded-[20px] border border-[var(--app-border)] bg-[var(--app-panel-muted)]/65 px-4 py-3">
            <p class="text-xs text-[var(--app-text-faint)]">运行</p>
            <p class="mt-1 text-base font-semibold text-[var(--app-text-strong)]">可追踪</p>
          </div>
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
