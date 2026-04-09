<template>
  <div class="space-y-6">
    <AppPageHeader
      title="本地列表"
    />

    <div class="grid gap-4 xl:grid-cols-[300px_minmax(0,1fr)_340px]">
      <ScriptListSidebar
        v-model:search-query="searchQuery"
        :scripts="filteredScripts"
        :selected-script-id="scriptStore.selectedScriptId"
        @select="scriptStore.selectScript"
        @create="openCreateDialog"
      />

      <ScriptDetailPanel
        :script="selectedScript"
        @open-editor="openEditor"
        @edit-info="openEditDialog"
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

    <ScriptInfoDialog
      :mode="dialogMode"
      :open="scriptInfoDialogOpen"
      :script="dialogScript"
      :task-options="[]"
      @close="closeInfoDialog"
      @save="handleSaveScriptInfo"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { confirm } from '@tauri-apps/plugin-dialog';
import AppPageHeader from '@/components/shared/AppPageHeader.vue';
import { createScriptName } from '@/services/scriptService';
import { taskService } from '@/services/taskService';
import ScriptDetailPanel from '@/views/script-list/ScriptDetailPanel.vue';
import ScriptInfoDialog from '@/views/script-list/ScriptInfoDialog.vue';
import ScriptListSidebar from '@/views/script-list/ScriptListSidebar.vue';
import ScriptTaskInspector from '@/views/script-list/ScriptTaskInspector.vue';
import { useDeviceStore } from '@/store/device';
import { useScriptStore } from '@/store/script';
import { useTaskStore } from '@/store/task';
import { useUserStore } from '@/store/user';
import type { ScriptTableRecord } from '@/types/app/domain';
import { showToast } from '@/utils/toast';

const router = useRouter();
const deviceStore = useDeviceStore();
const scriptStore = useScriptStore();
const taskStore = useTaskStore();
const userStore = useUserStore();
const searchQuery = ref('');
const scriptInfoDialogOpen = ref(false);
const dialogMode = ref<'create' | 'edit'>('create');
const dialogScript = ref<ScriptTableRecord | null>(null);

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

const openCreateDialog = async () => {
  try {
    dialogMode.value = 'create';
    dialogScript.value = await scriptStore.prepareScript(
      {
        userId: userStore.userProfile?.id,
        userName: userStore.authSession?.username || userStore.userProfile?.username || 'Guest',
      },
      createScriptName(scriptStore.scripts.length + 1),
    );
    scriptInfoDialogOpen.value = true;

    if (userStore.authSession && !userStore.userProfile?.id) {
      void userStore.checkProfile().then((profile) => {
        if (!profile || !dialogScript.value || dialogMode.value !== 'create') {
          return;
        }

        dialogScript.value = {
          ...dialogScript.value,
          data: {
            ...dialogScript.value.data,
            userId: profile.id,
            userName: profile.username,
          },
        };
      });
    }
  } catch (error) {
    showToast(error instanceof Error ? error.message : '初始化脚本失败', 'error');
  }
};

const openEditDialog = (scriptId: string) => {
  scriptStore.selectScript(scriptId);
  dialogMode.value = 'edit';
  dialogScript.value = scriptStore.scripts.find((script) => script.id === scriptId) ?? null;
  scriptInfoDialogOpen.value = true;
};

const closeInfoDialog = () => {
  scriptInfoDialogOpen.value = false;
  dialogScript.value = null;
};

const fallbackGuestScript = async (script: ScriptTableRecord): Promise<ScriptTableRecord> => ({
  ...script,
  data: {
    ...script.data,
    userId: script.data.userId?.trim() || (await taskService.requestUuid()),
    userName: 'Guest',
  },
});

const ensureScriptAuthorForSave = async (script: ScriptTableRecord): Promise<ScriptTableRecord> => {
  if (!userStore.authSession) {
    return fallbackGuestScript(script);
  }

  const profile = userStore.userProfile ?? (await userStore.checkProfile());
  if (profile) {
    return {
      ...script,
      data: {
        ...script.data,
        userId: profile.id,
        userName: profile.username,
      },
    };
  }

  return fallbackGuestScript(script);
};

const handleSaveScriptInfo = async (script: ScriptTableRecord) => {
  try {
    const scriptToSave = await ensureScriptAuthorForSave(script);
    await scriptStore.saveScript(scriptToSave);
    showToast(dialogMode.value === 'edit' ? '脚本信息已更新' : '已创建本地脚本', 'success');
    closeInfoDialog();
  } catch (error) {
    console.error(error);
    showToast(error instanceof Error ? error.message : '保存失败', 'error');
  }
};

const openEditor = (scriptId: string) => {
  router.push({ path: '/editor', query: { scriptId } });
};

const handleUpload = async (scriptId: string) => {
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
