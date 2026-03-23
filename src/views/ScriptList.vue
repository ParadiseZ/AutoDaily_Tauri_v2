<template>
  <div class="space-y-6">
    <AppPageHeader
      eyebrow="Library"
      title="本地脚本"
      description="列表、详情和下一步动作分离布局，浏览脚本时不用再被大量卡片和冗余按钮打断。"
    />

    <div class="grid gap-4 xl:grid-cols-[300px_minmax(0,1fr)_340px]">
      <ScriptListSidebar
        v-model:search-query="searchQuery"
        :scripts="filteredScripts"
        :selected-script-id="scriptStore.selectedScriptId"
        @select="scriptStore.selectScript"
        @create="handleCreateScript"
      />

      <ScriptDetailPanel
        :script="selectedScript"
        @open-editor="openEditor"
        @upload="handleUpload"
        @clone="handleClone"
        @clear-logs="handleClearLogs"
        @delete="handleDelete"
      />

      <ScriptTaskInspector
        :script="selectedScript"
        :tasks="selectedScript ? scriptStore.tasksByScriptId[selectedScript.id] ?? [] : []"
        :loading="selectedScript ? Boolean(scriptStore.taskLoading[selectedScript.id]) : false"
        :devices="deviceStore.devices"
        :assignments-by-device="taskStore.assignmentsByDevice"
        :time-templates="taskStore.timeTemplates"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { confirm } from '@tauri-apps/plugin-dialog';
import AppPageHeader from '@/components/shared/AppPageHeader.vue';
import ScriptDetailPanel from '@/views/script-list/ScriptDetailPanel.vue';
import ScriptListSidebar from '@/views/script-list/ScriptListSidebar.vue';
import ScriptTaskInspector from '@/views/script-list/ScriptTaskInspector.vue';
import { useDeviceStore } from '@/store/device';
import { useScriptStore } from '@/store/script';
import { useTaskStore } from '@/store/task';
import { useUserStore } from '@/store/user';
import { showToast } from '@/utils/toast';

const router = useRouter();
const deviceStore = useDeviceStore();
const scriptStore = useScriptStore();
const taskStore = useTaskStore();
const userStore = useUserStore();
const searchQuery = ref('');

const filteredScripts = computed(() => {
  const keyword = searchQuery.value.trim().toLowerCase();
  if (!keyword) {
    return scriptStore.sortedScripts;
  }

  return scriptStore.sortedScripts.filter((script) => {
    return (
      script.data.name.toLowerCase().includes(keyword) ||
      (script.data.description || '').toLowerCase().includes(keyword)
    );
  });
});

const selectedScript = computed(() => {
  const current = scriptStore.selectedScript;
  if (current) {
    return current;
  }
  return filteredScripts.value[0] ?? null;
});

const handleCreateScript = async () => {
  try {
    const script = await scriptStore.createScript(`未命名脚本 ${scriptStore.scripts.length + 1}`, userStore.userProfile);
    showToast('已创建本地脚本草稿', 'success');
    openEditor(script.id);
  } catch (error) {
    console.log(error);
    showToast(error instanceof Error ? error.message : '创建失败', 'error');
  }
};

const openEditor = (scriptId: string) => {
  router.push({ path: '/editor', query: { scriptId } });
};

const handleUpload = async (scriptId: string) => {
  if (!userStore.isLoggedIn) {
    userStore.openAuthModal();
    return;
  }

  try {
    const result = await scriptStore.uploadScript(scriptId);
    if (!result.success) {
      throw new Error(result.message || '上传失败');
    }
    showToast(result.message || '脚本已上传', 'success');
    await scriptStore.loadScripts();
  } catch (error) {
    showToast(error instanceof Error ? error.message : '上传失败', 'error');
  }
};

const handleClone = async (scriptId: string) => {
  try {
    const approved = await confirm('克隆后会生成一个新的本地脚本副本，是否继续？', {
      title: '克隆脚本',
      kind: 'info',
    });
    if (!approved) {
      return;
    }

    const result = await scriptStore.cloneScript(scriptId, userStore.userProfile?.id || null, false);
    if (!result.success) {
      throw new Error(result.message || '克隆失败');
    }
    showToast(result.message || '脚本已克隆', 'success');
    await scriptStore.loadScripts();
  } catch (error) {
    showToast(error instanceof Error ? error.message : '克隆失败', 'error');
  }
};

const handleClearLogs = async (scriptId: string) => {
  try {
    await taskStore.clearSchedulesByScript(scriptId);
    showToast('脚本运行记录已清理', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '清理失败', 'error');
  }
};

const handleDelete = async (scriptId: string) => {
  const approved = await confirm('删除后脚本将从本地库中移除，这个操作无法撤销。', {
    title: '删除脚本',
    kind: 'warning',
  });
  if (!approved) {
    return;
  }

  try {
    await scriptStore.deleteScript(scriptId);
    showToast('脚本已删除', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '删除失败', 'error');
  }
};

onMounted(async () => {
  await Promise.all([scriptStore.loadScripts(), deviceStore.loadDevices()]);
  await taskStore.hydrateForDevices(deviceStore.devices.map((device) => device.id));
});

watch(
  () => selectedScript.value?.id,
  async (scriptId) => {
    if (!scriptId) {
      return;
    }
    await scriptStore.loadScriptTasks(scriptId);
  },
  { immediate: true },
);

watch(
  () => filteredScripts.value.map((script) => script.id).join(','),
  () => {
    if (!filteredScripts.value.length) {
      return;
    }

    const currentId = scriptStore.selectedScriptId;
    const exists = filteredScripts.value.some((script) => script.id === currentId);
    if (!exists) {
      scriptStore.selectScript(filteredScripts.value[0].id);
    }
  },
);
</script>
