<template>
  <div class="flex h-full min-h-0 flex-col gap-4">
    <AppPageHeader
      title="本地列表"
    />

    <div class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
    <div class="grid min-h-full gap-4 xl:grid-cols-[300px_minmax(0,1fr)_390px]">
      <ScriptListSidebar
        v-model:search-query="searchQuery"
        :scripts="filteredScripts"
        :selected-script-id="scriptStore.selectedScriptId"
        @select="scriptStore.selectScript"
        @create="openCreateDialog"
      />

      <ScriptDetailPanel
        :script="selectedScript"
        :upload-activities="selectedUploadActivities"
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
import AppPageHeader from '@/components/shared/AppPageHeader.vue';
import { requestAppConfirm } from '@/services/appDialogService';
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
import type { ScriptTableRecord, ScriptUploadActivity } from '@/types/app/domain';
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
const uploadActivitiesByScriptId = ref<Record<string, ScriptUploadActivity[]>>({});
const pendingUploadScriptId = ref<string | null>(null);
const pendingUploadRetrying = ref(false);

const isAuthFailure = (message?: string | null) =>
  Boolean(message && (message.includes('401') || message.includes('未登录') || message.includes('认证失败')));

const normalizeResultMessage = (message: string | null | undefined, fallback: string) => {
  const trimmed = message?.trim();
  return trimmed ? trimmed : fallback;
};

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

const selectedUploadActivities = computed(() =>
  selectedScript.value ? uploadActivitiesByScriptId.value[selectedScript.value.id] ?? [] : [],
);

const pushUploadActivity = (
  scriptId: string,
  activity: Omit<ScriptUploadActivity, 'id' | 'scriptId' | 'at'> & { at?: string },
) => {
  const nextActivity: ScriptUploadActivity = {
    id: `${scriptId}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    scriptId,
    at: activity.at ?? new Date().toISOString(),
    ...activity,
  };
  const current = uploadActivitiesByScriptId.value[scriptId] ?? [];
  const previous = current[0];
  const merged =
    previous &&
    previous.status === 'waitingAuth' &&
    nextActivity.status === 'waitingAuth' &&
    previous.message === nextActivity.message
      ? [{ ...previous, at: nextActivity.at, autoRetry: nextActivity.autoRetry }]
      : [nextActivity, ...current];

  uploadActivitiesByScriptId.value = {
    ...uploadActivitiesByScriptId.value,
    [scriptId]: merged.slice(0, 6),
  };
};

const queueUploadAfterLogin = (scriptId: string, message: string) => {
  pendingUploadScriptId.value = scriptId;
  const script = scriptStore.scripts.find((item) => item.id === scriptId) ?? selectedScript.value;
  pushUploadActivity(scriptId, {
    status: 'waitingAuth',
    message,
    cloudId: script?.data.cloudId ?? null,
    username: userStore.userProfile?.username ?? userStore.authSession?.username ?? null,
    autoRetry: true,
  });
  userStore.openAuthModal();
  showToast(message, 'warning');
};

const ensureUploadAuth = async (scriptId: string) => {
  if (!userStore.authSession) {
    queueUploadAfterLogin(scriptId, '上传前请先登录，登录后会自动继续');
    return false;
  }

  const profile = userStore.userProfile ?? await userStore.checkProfile();
  if (!profile) {
    queueUploadAfterLogin(scriptId, '登录信息已失效，请重新登录后继续上传');
    return false;
  }

  return true;
};

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

const performUpload = async (scriptId: string) => {
  const script = scriptStore.scripts.find((item) => item.id === scriptId) ?? selectedScript.value;
  try {
    const result = await scriptStore.uploadScript(scriptId);
    if (!result.success) {
      const message = normalizeResultMessage(result.message, '上传失败');
      if (isAuthFailure(message)) {
        queueUploadAfterLogin(scriptId, '登录已失效，请重新登录后继续上传');
        return;
      }
      throw new Error(message);
    }
    const message = normalizeResultMessage(result.message, '脚本已上传');
    pushUploadActivity(scriptId, {
      status: 'success',
      message,
      cloudId: script?.data.cloudId ?? scriptId,
      username: userStore.userProfile?.username ?? userStore.authSession?.username ?? null,
      autoRetry: false,
    });
    if (pendingUploadScriptId.value === scriptId) {
      pendingUploadScriptId.value = null;
    }
    showToast(message, 'success');
    await scriptStore.loadScripts();
    void userStore.checkProfile();
  } catch (error) {
    const message = normalizeResultMessage(error instanceof Error ? error.message : null, '上传失败');
    if (isAuthFailure(message)) {
      queueUploadAfterLogin(scriptId, '登录已失效，请重新登录后继续上传');
      return;
    }
    pushUploadActivity(scriptId, {
      status: 'error',
      message,
      cloudId: script?.data.cloudId ?? null,
      username: userStore.userProfile?.username ?? userStore.authSession?.username ?? null,
      autoRetry: false,
    });
    showToast(message, 'error');
  }
};

const handleUpload = async (scriptId: string) => {
  if (!(await ensureUploadAuth(scriptId))) {
    return;
  }
  await performUpload(scriptId);
};

const handleClone = async (scriptId: string) => {
  try {
    const approved = await requestAppConfirm({
      title: '克隆脚本',
      message: '克隆后会生成一个新的本地脚本副本，是否继续？',
      confirmText: '克隆',
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
  const approved = await requestAppConfirm({
    title: '删除脚本',
    message: '删除后脚本将从本地库中移除，这个操作无法撤销。',
    confirmText: '删除',
    tone: 'danger',
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

watch(
  () => [pendingUploadScriptId.value, userStore.authSession?.accessToken ?? null, userStore.userProfile?.id ?? null] as const,
  async ([scriptId, accessToken, profileId]) => {
    if (!scriptId || !accessToken || pendingUploadRetrying.value) {
      return;
    }

    pendingUploadRetrying.value = true;
    try {
      const profile = profileId ? userStore.userProfile : await userStore.checkProfile();
      if (!profile) {
        return;
      }

      if (pendingUploadScriptId.value !== scriptId) {
        return;
      }

      pendingUploadScriptId.value = null;
      await performUpload(scriptId);
    } finally {
      pendingUploadRetrying.value = false;
    }
  },
);
</script>
