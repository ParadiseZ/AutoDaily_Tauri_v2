<template>
  <div class="flex h-full min-h-0 flex-col gap-4">
    <AppPageHeader
      title="本地列表"
    >
      <template #center>
      <ScriptTransferHistoryPanel
        title="上传记录"
        empty-title="还没有上传记录"
        empty-description="执行上传后，这里会显示模型传输进度、结果和错误信息。"
        :open="uploadHistoryOpen"
        :records="uploadTransferRecords"
        :get-progress-event="scriptTransferStore.getLatestProgressEvent"
        @toggle="uploadHistoryOpen = !uploadHistoryOpen"
        @pause-record="(recordId) => void handlePauseTransferRecord(recordId)"
        @resume-record="(recordId) => void handleResumeTransferRecord(recordId)"
        @delete-record="(recordId) => void handleDeleteTransferRecord(recordId)"
      />
      </template>
    </AppPageHeader>

    
    <div class="grid min-h-full gap-4 xl:grid-cols-[300px_minmax(0,1fr)_420px]">
        <ScriptListSidebar
          v-model:search-query="searchQuery"
          :scripts="filteredScripts"
          :selected-script-id="scriptStore.selectedScriptId"
          @select="scriptStore.selectScript"
          @create="openCreateDialog"
        />
        <ScriptDetailPanel
          :current-user-id="userStore.userProfile?.id ?? null"
          :current-username="userStore.userProfile?.username ?? userStore.authSession?.username ?? null"
          :script="selectedScript"
          :upload-pending="isSelectedScriptUploading"
          :upload-pending-label="selectedUploadPendingLabel"
          @open-editor="openEditor"
          @edit-info="openEditDialog"
          @upload="handleUpload"
          @clone="handleClone"
          @clear-logs="handleClearLogs"
          @delete="handleDelete"
        />

        <ScriptLogPanel
          :script="selectedScript"
          :logs="selectedScriptChangeLogs"
          :loading="changeLogsLoading"
          :load-failed="changeLogsLoadFailed"
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
import ScriptTransferHistoryPanel from '@/components/script-transfer/ScriptTransferHistoryPanel.vue';
import AppPageHeader from '@/components/shared/AppPageHeader.vue';
import { requestAppConfirm } from '@/services/appDialogService';
import { createScriptName, scriptService } from '@/services/scriptService';
import { taskService } from '@/services/taskService';
import ScriptDetailPanel from '@/views/script-list/ScriptDetailPanel.vue';
import ScriptInfoDialog from '@/views/script-list/ScriptInfoDialog.vue';
import ScriptLogPanel from '@/views/script-list/ScriptLogPanel.vue';
import ScriptListSidebar from '@/views/script-list/ScriptListSidebar.vue';
import { useDeviceStore } from '@/store/device';
import { useScriptStore } from '@/store/script';
import { useScriptTransferStore } from '@/store/scriptTransfer';
import { useTaskStore } from '@/store/task';
import { useUserStore } from '@/store/user';
import type { ScriptChangeLogRecord, ScriptTableRecord } from '@/types/app/domain';
import { createServerResponseError, isAuthFailure } from '@/utils/api';
import { formatScriptInfoValidationMessage, validateScriptInfo } from '@/utils/scriptInfoValidation';
import { showToast } from '@/utils/toast';

const router = useRouter();
const deviceStore = useDeviceStore();
const scriptStore = useScriptStore();
const scriptTransferStore = useScriptTransferStore();
const taskStore = useTaskStore();
const userStore = useUserStore();
const searchQuery = ref('');
const scriptInfoDialogOpen = ref(false);
const dialogMode = ref<'create' | 'edit'>('create');
const dialogScript = ref<ScriptTableRecord | null>(null);
const pendingUploadScriptId = ref<string | null>(null);
const pendingUploadRetrying = ref(false);
const uploadHistoryOpen = ref(false);
const uploadPendingScriptId = ref<string | null>(null);
const uploadPendingLabel = ref('上传中...');
const selectedScriptChangeLogs = ref<ScriptChangeLogRecord[]>([]);
const changeLogsLoading = ref(false);
const changeLogsLoadFailed = ref(false);

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

const uploadTransferRecords = computed(() => scriptTransferStore.getRecordsByDirection('upload'));
const isSelectedScriptUploading = computed(
  () => Boolean(selectedScript.value && uploadPendingScriptId.value === selectedScript.value.id),
);
const selectedUploadPendingLabel = computed(() =>
  isSelectedScriptUploading.value ? uploadPendingLabel.value : '上传',
);
const isTransferDeletedMessage = (message: string) => message.includes('传输已删除');

const isPublishedScript = (script: ScriptTableRecord | null | undefined) => script?.data.scriptType === 'published';
const canCloneScript = (
  script: ScriptTableRecord | null | undefined,
  currentUserId: string | null,
  currentUsername: string | null,
) =>
  Boolean(
    script &&
      (script.data.allowClone ||
        script.data.userId === currentUserId ||
        script.data.userName === currentUsername),
  );

const queueUploadAfterLogin = (scriptId: string, message: string) => {
  pendingUploadScriptId.value = scriptId;
  userStore.openAuthModal();
  showToast(message, 'warning');
};

const setUploadPendingState = (scriptId: string, label: string) => {
  uploadPendingScriptId.value = scriptId;
  uploadPendingLabel.value = label;
};

const clearUploadPendingState = (scriptId?: string | null) => {
  if (!scriptId || uploadPendingScriptId.value === scriptId) {
    uploadPendingScriptId.value = null;
    uploadPendingLabel.value = '上传中...';
  }
};

const ensureUploadAuth = async (scriptId: string) => {
  setUploadPendingState(scriptId, '正在检查登录状态...');
  if (!userStore.authSession) {
    clearUploadPendingState(scriptId);
    queueUploadAfterLogin(scriptId, '上传前请先登录，登录后会自动继续');
    return false;
  }

  const profile = await userStore.ensureProfileForAction('上传脚本');
  if (!profile) {
    clearUploadPendingState(scriptId);
    if (!userStore.authSession) {
      queueUploadAfterLogin(scriptId, '登录信息已失效，请重新登录后继续上传');
    }
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
  const script = scriptStore.scripts.find((item) => item.id === scriptId) ?? null;
  if (isPublishedScript(script)) {
    showToast('云端下载脚本请先克隆为本地脚本后再编辑', 'warning');
    return;
  }
  scriptStore.selectScript(scriptId);
  dialogMode.value = 'edit';
  dialogScript.value = script;
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

  const profile = userStore.userProfile ?? (await userStore.ensureProfileForAction('保存脚本信息'));
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

  throw new Error('当前无法确认登录用户，请稍后重试或重新登录');
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

const ensureScriptInfoReadyForUpload = async (scriptId: string) => {
  const script = scriptStore.scripts.find((item) => item.id === scriptId) ?? null;
  const issues = validateScriptInfo(script);
  if (!issues.length) {
    return true;
  }

  const approved = await requestAppConfirm({
    title: '脚本信息未填写完整',
    message: formatScriptInfoValidationMessage(issues, '上传前请先补齐以下脚本信息：'),
    confirmText: '去填写',
    cancelText: '暂不上传',
    tone: 'warning',
  });

  if (approved && script) {
    openEditDialog(script.id);
  }

  return false;
};

const openEditor = (scriptId: string) => {
  const script = scriptStore.scripts.find((item) => item.id === scriptId) ?? null;
  if (isPublishedScript(script)) {
    showToast('云端下载脚本请先克隆为本地脚本后再编辑', 'warning');
    return;
  }
  router.push({ path: '/editor', query: { scriptId } });
};

const performUpload = async (scriptId: string) => {
  try {
    setUploadPendingState(scriptId, '上传中...');
    const result = await scriptStore.uploadScript(scriptId);
    if (!result.success) {
      const message = createServerResponseError('backend_upload_script', result).message;
      if (isAuthFailure(message)) {
        clearUploadPendingState(scriptId);
        queueUploadAfterLogin(scriptId, '登录已失效，请重新登录后继续上传');
        return;
      }
      throw new Error(message);
    }
    const message = result.message?.trim() || '脚本已上传';
    if (pendingUploadScriptId.value === scriptId) {
      pendingUploadScriptId.value = null;
    }
    showToast(message, 'success');
    await scriptStore.loadScripts();
    void userStore.checkProfile();
  } catch (error) {
    const message = error instanceof Error ? error.message : '上传失败，请稍后重试。';
    if (isTransferDeletedMessage(message)) {
      return;
    }
    if (isAuthFailure(message)) {
      clearUploadPendingState(scriptId);
      queueUploadAfterLogin(scriptId, '登录已失效，请重新登录后继续上传');
      return;
    }
    showToast(message, 'error');
  } finally {
    clearUploadPendingState(scriptId);
  }
};

const handleUpload = async (scriptId: string) => {
  if (uploadPendingScriptId.value === scriptId) {
    return;
  }

  const script = scriptStore.scripts.find((item) => item.id === scriptId) ?? null;
  if (isPublishedScript(script)) {
    showToast('云端下载脚本不可直接上传，请先克隆为本地脚本', 'warning');
    return;
  }
  if (!(await ensureScriptInfoReadyForUpload(scriptId))) {
    return;
  }
  if (!(await ensureUploadAuth(scriptId))) {
    return;
  }
  setUploadPendingState(scriptId, '正在检查云端版本...');
  if (!(await ensureUploadVersionConfirmed(scriptId))) {
    clearUploadPendingState(scriptId);
    return;
  }
  await performUpload(scriptId);
};

const handleClone = async (scriptId: string) => {
  try {
    const script = scriptStore.scripts.find((item) => item.id === scriptId) ?? null;
    const currentUserId = userStore.userProfile?.id ?? null;
    const currentUsername = userStore.userProfile?.username ?? userStore.authSession?.username ?? null;
    if (!canCloneScript(script, currentUserId, currentUsername)) {
      showToast('作者未开放克隆权限', 'warning');
      return;
    }
    const approved = await requestAppConfirm({
      title: '克隆脚本',
      message: '克隆后会生成一个新的本地脚本副本，是否继续？',
      confirmText: '克隆',
    });
    if (!approved) {
      return;
    }

    const result = await scriptStore.cloneScript(scriptId, false);
    if (!result.success) {
      throw new Error(result.message || '克隆失败');
    }
    showToast(result.message || '脚本已克隆', 'success');
    await scriptStore.loadScripts();
  } catch (error) {
    showToast(error instanceof Error ? error.message : '克隆失败', 'error');
  }
};

const ensureUploadVersionConfirmed = async (scriptId: string) => {
  try {
    const preflight = await scriptService.preflightUploadLocalScript(scriptId);
    if (preflight.status === 'cloudMissing') {
      return true;
    }

    if (preflight.status === 'downgradeBlocked') {
      showToast(preflight.message, 'warning');
      return false;
    }

    const title = preflight.status === 'upgradeAvailable' ? '上传新版本' : '覆盖相同版本';
    const confirmText = preflight.status === 'upgradeAvailable' ? '上传更新' : '覆盖上传';
    return requestAppConfirm({
      title,
      message: preflight.message,
      confirmText,
      cancelText: '取消',
      tone: 'warning',
    });
  } catch (error) {
    showToast(error instanceof Error ? error.message : '获取云端版本信息失败', 'error');
    return false;
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

const handleDeleteTransferRecord = async (recordId: string) => {
  try {
    await scriptTransferStore.deleteRecord(recordId);
    showToast('传输记录已删除', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '删除记录失败', 'error');
  }
};

const handlePauseTransferRecord = async (recordId: string) => {
  try {
    await scriptTransferStore.pauseRecord(recordId);
  } catch (error) {
    showToast(error instanceof Error ? error.message : '暂停传输失败', 'error');
  }
};

const handleResumeTransferRecord = async (recordId: string) => {
  try {
    await scriptTransferStore.resumeRecord(recordId);
  } catch (error) {
    showToast(error instanceof Error ? error.message : '继续传输失败', 'error');
  }
};

onMounted(async () => {
  await Promise.all([scriptStore.loadScripts(), deviceStore.loadDevices()]);
  await taskStore.hydrateForDevices(deviceStore.devices.map((device) => device.id));
  await scriptTransferStore.loadRecords({
    direction: 'upload',
    limit: 80,
  });
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
  () => uploadTransferRecords.value.some((record) => record.status === 'running' || record.status === 'paused'),
  (hasActiveTransfer) => {
    if (hasActiveTransfer) {
      uploadHistoryOpen.value = true;
    }
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
      setUploadPendingState(scriptId, '上传中...');
      await performUpload(scriptId);
    } finally {
      pendingUploadRetrying.value = false;
    }
  },
);
</script>


