<template>
  <div class="flex min-h-screen flex-col gap-6 px-6 py-6 lg:px-10">
    <div class="flex items-center justify-between gap-4">
      <div>
        <p class="text-xs uppercase tracking-[0.18em] text-[var(--app-text-faint)]">Editor</p>
        <h1 class="text-3xl font-semibold text-[var(--app-text-strong)]">脚本编辑工作区</h1>
      </div>
      <button class="app-button app-button-ghost" type="button" @click="router.push('/scripts')">
        返回脚本列表
      </button>
    </div>

    <div class="grid flex-1 gap-4 xl:grid-cols-[360px_minmax(0,1fr)]">
      <SurfacePanel class="space-y-4">
        <div>
          <p class="text-sm font-semibold text-[var(--app-text-strong)]">当前脚本</p>
          <p class="text-sm text-[var(--app-text-soft)]">
            {{ script?.data.name || '未选择脚本' }}
          </p>
        </div>
        <div class="space-y-3 text-sm text-[var(--app-text-soft)]">
          <p>编辑器页会继续沿用新的组件拆分规则，在这里承接图编辑器、属性面板和任务结构。</p>
          <p>当前这次重写先把外层工作区和脚本入口接起来，避免再跳回设置页。</p>
        </div>
      </SurfacePanel>

      <SurfacePanel class="flex min-h-[520px] items-center justify-center">
        <div class="max-w-xl space-y-3 text-center">
          <h2 class="text-2xl font-semibold text-[var(--app-text-strong)]">编辑器工作区正在迁移</h2>
          <p class="text-sm leading-6 text-[var(--app-text-soft)]">
            路由、脚本上下文和页面层级已经独立出来，下一步可以在这个工作区继续拆解流程编辑、策略编辑和属性面板，而不是把它们塞回主设置页。
          </p>
        </div>
      </SurfacePanel>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import { useScriptStore } from '@/store/script';

const route = useRoute();
const router = useRouter();
const scriptStore = useScriptStore();

const script = computed(() => {
  const scriptId = typeof route.query.scriptId === 'string' ? route.query.scriptId : '';
  return scriptStore.scripts.find((item) => item.id === scriptId) ?? null;
});

onMounted(async () => {
  if (!scriptStore.scripts.length) {
    await scriptStore.loadScripts();
  }
});
</script>
