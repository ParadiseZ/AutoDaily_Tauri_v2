<template>
  <div class="flex h-full min-h-0 flex-col gap-4">
    <AppPageHeader
      title="设备列表"
    >
      <template #actions>
        <button class="app-button app-button-primary shadow-lg shadow-(--app-accent-soft)" type="button" @click="openEditor(null)">
          <AppIcon name="plus" :size="18" />
          添加设备
        </button>
      </template>
    </AppPageHeader>

    <div class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
    <SurfacePanel v-if="deviceStore.devices.length" class="overflow-hidden p-0">
      <table class="app-table">
        <thead>
          <tr>
            <th>设备</th>
            <th>平台</th>
            <th>连接方式</th>
            <th>截图方式</th>
            <th>CPU</th>
            <th>状态</th>
            <th class="text-right">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="device in deviceStore.devices" :key="device.id">
            <td>
              <div class="space-y-1">
                <p class="font-medium text-(--app-text-strong)">{{ device.data.deviceName }}</p>
                <p class="text-xs text-(--app-text-faint)">{{ device.id }}</p>
              </div>
            </td>
            <td>{{ formatPlatformLabel(device.data.platform) }}</td>
            <td>{{ formatConnectLabel(device.data.adbConnect) }}</td>
            <td>{{ formatCaptureMethod(device.data.capMethod) }}</td>
            <td>{{ device.data.cores.length ? device.data.cores.join(', ') : '未绑定' }}</td>
            <td>
              <div class="flex flex-wrap items-center gap-3">
                <StatusBadge :label="device.data.enable ? '已启用' : '已停用'" :tone="device.data.enable ? 'success' : 'neutral'" />
                <div class="flex items-center gap-1.5 rounded-full bg-(--app-panel-muted) px-2.5 py-1">
                  <AppIcon type="custom" :name="deviceStore.isDeviceOnline(device.id) ? 'status-online' : 'status-offline'" :size="16" :color="deviceStore.isDeviceOnline(device.id) ? 'var(--app-vibrant-emerald)' : 'var(--app-text-faint)'" />
                  <span class="text-xs font-semibold tracking-wide" :class="deviceStore.isDeviceOnline(device.id) ? 'text-(--app-vibrant-emerald)' : 'text-(--app-text-faint)'">
                    {{ deviceStore.isDeviceOnline(device.id) ? '在线' : '离线' }}
                  </span>
                </div>
              </div>
            </td>
            <td>
              <div class="flex justify-end gap-2">
                <button class="app-button app-button-ghost h-9 px-3 group text-sm" type="button" @click="openEditor(device.id)">
                  <AppIcon name="edit-3" :size="16" class="text-(--app-text-soft) transition-colors group-hover:text-(--app-accent)" />
                  编辑
                </button>
                <button class="app-button app-button-danger h-9 px-3 group text-sm" type="button" @click="removeDevice(device.id)">
                  <AppIcon name="trash-2" :size="16" class="opacity-70 transition-opacity group-hover:opacity-100" />
                  删除
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </SurfacePanel>

    <EmptyState
      v-else
      title="设备列表还是空的"
      description="先创建一台设备，配置连接方式、截图方案和自动启动策略"
      icon="smartphone"
    />
    </div>

    <DeviceEditorDialog
      :open="editorOpen"
      :device="currentDevice"
      :cpu-count="deviceStore.cpuCount"
      @close="editorOpen = false"
      @save="saveDevice"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import AppPageHeader from '@/components/shared/AppPageHeader.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import StatusBadge from '@/components/shared/StatusBadge.vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import DeviceEditorDialog from '@/views/device-list/DeviceEditorDialog.vue';
import { useDeviceStore } from '@/store/device';
import { useSettingsStore } from '@/store/settings';
import { requestAppConfirm } from '@/services/appDialogService';
import { taskService } from '@/services/taskService';
import { showToast } from '@/utils/toast';
import { formatCaptureMethod, formatConnectLabel, formatPlatformLabel } from '@/utils/presenters';
import type { DeviceFormState } from '@/types/app/domain';
import type { ADBConnectConfig } from '@/types/bindings/ADBConnectConfig';
import type { DeviceTable } from '@/types/bindings/DeviceTable';

const deviceStore = useDeviceStore();
const settingsStore = useSettingsStore();
const editorOpen = ref(false);
const editingDeviceId = ref<string | null>(null);

const currentDevice = computed(
  () => deviceStore.devices.find((device) => device.id === editingDeviceId.value) ?? null,
);

const buildAdbConnect = (form: DeviceFormState): ADBConnectConfig | null => {
  const serverConfig = {
    adbPath: settingsStore.preferences.adbPath || null,
    serverConnect: `${settingsStore.preferences.adbServerHost}:${settingsStore.preferences.adbServerPort}`,
  };

  if (form.connectMethod === 'directTcp') {
    return {
      directTcp: form.connectAddress || null,
    };
  }

  if (form.connectMethod === 'serverConnectByIp') {
    return {
      serverConnectByIp: {
        adbConfig: serverConfig,
        clientConnect: form.connectAddress || null,
      },
    };
  }

  return {
    serverConnectByName: {
      adbConfig: serverConfig,
      deviceName: form.connectDeviceName || null,
    },
  };
};

const buildDeviceTable = async (form: DeviceFormState): Promise<DeviceTable> => ({
  id: form.id ?? (await taskService.requestUuid()),
  data: {
    deviceName: form.deviceName,
    platform: form.platform,
    exePath: form.exePath || null,
    exeArgs: form.exeArgs || null,
    cores: form.cores,
    logLevel: form.logLevel,
    logToFile: form.logToFile,
    adbConnect: buildAdbConnect(form),
    capMethod: form.capMethodType === 'adb' ? 'adb' : { window: form.capMethodValue || form.deviceName },
    imageCompression: form.capMethodType === 'adb' ? 'AdbOriginal' : 'WindowOriginal',
    enable: form.enable,
    autoStart: form.autoStart,
    executionPolicy: {
      actionWaitMs: Math.max(0, Number(form.actionWaitMs) || 0),
      progressTimeoutEnabled: form.progressTimeoutEnabled,
      progressTimeoutMs: Math.max(1000, Number(form.progressTimeoutMs) || 30000),
      timeoutAction: form.timeoutAction,
      timeoutNotifyChannels: [...form.timeoutNotifyChannels],
    },
  },
});

const openEditor = (deviceId: string | null) => {
  editingDeviceId.value = deviceId;
  editorOpen.value = true;
};

const saveDevice = async (form: DeviceFormState) => {
  try {
    const device = await buildDeviceTable(form);
    await deviceStore.saveDevice(device);
    editorOpen.value = false;
    showToast('设备已保存', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '设备保存失败', 'error');
  }
};

const removeDevice = async (deviceId: string) => {
  const approved = await requestAppConfirm({
    title: '删除设备',
    message: '删除后不会保留当前设备的本地配置，是否继续？',
    confirmText: '删除',
    tone: 'danger',
  });

  if (!approved) {
    return;
  }

  try {
    await deviceStore.deleteDevice(deviceId);
    showToast('设备已删除', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '删除失败', 'error');
  }
};

onMounted(async () => {
  await Promise.all([deviceStore.refreshAll(), settingsStore.loadPreferences()]);
});
</script>
