<template>
  <AppDialog
    :open="open"
    :title="device ? '编辑设备' : '添加设备'"
    width-class="max-w-4xl"
    @close="$emit('close')"
  >
    <form class="grid gap-5" @submit.prevent="$emit('save', form)">
      <div class="overflow-x-auto">
        <div class="editor-panel-tabs min-w-max">
          <button
            v-for="tab in tabs"
            :key="tab.value"
            type="button"
            class="editor-panel-tab"
            :class="{ 'editor-panel-tab-active': activeTab === tab.value }"
            @click="activeTab = tab.value"
          >
            {{ tab.label }}
          </button>
        </div>
      </div>

      <template v-if="activeTab === 'basic'">
        <div class="grid gap-4 md:grid-cols-2">
          <label class="grid gap-2">
            <span class="text-sm text-(--app-text-soft)">名称</span>
            <input v-model.trim="form.deviceName" class="app-input" placeholder="MuMu 模拟器 12" />
          </label>
          <label class="grid gap-2">
            <span class="text-sm text-(--app-text-soft)">日志级别</span>
            <AppSelect v-model="form.logLevel" :options="logLevelOptions" />
          </label>
        </div>

        <div class="grid gap-4 md:grid-cols-2">
          <label class="grid gap-2">
            <span class="text-sm text-(--app-text-soft)">截图方式</span>
            <AppSelect v-model="form.capMethodType" :options="captureOptions" />
          </label>
          <label class="grid gap-2">
            <span class="text-sm text-(--app-text-soft)">窗口名 / 标识</span>
            <input
              v-model.trim="form.capMethodValue"
              class="app-input"
              :placeholder="form.capMethodType === 'window' ? '窗口标题' : 'ADB 截图无需额外配置'"
              :disabled="form.capMethodType === 'adb'"
            />
          </label>
        </div>

        <div class="grid gap-4 md:grid-cols-2">
          <label class="grid gap-2">
            <span class="text-sm text-(--app-text-soft)">连接方式</span>
            <AppSelect v-model="form.connectMethod" :options="connectOptions" />
          </label>
          <label class="grid gap-2">
            <span class="text-sm text-(--app-text-soft)">地址 / 设备名</span>
            <input
              v-if="form.connectMethod !== 'serverConnectByName'"
              v-model.trim="form.connectAddress"
              class="app-input"
              placeholder="127.0.0.1:5555"
            />
            <input
              v-else
              v-model.trim="form.connectDeviceName"
              class="app-input"
              placeholder="emulator-5554"
            />
          </label>
        </div>

        <div class="grid gap-4 md:grid-cols-2">
          <label class="grid gap-2">
            <span class="text-sm text-(--app-text-soft)">设备启动路径（可选）</span>
            <div class="path-input-row">
              <input v-model.trim="form.exePath" class="app-input" placeholder="模拟器启动路径" />
              <button class="app-button app-button-ghost path-picker-button" type="button" @click="pickExePath">
                <AppIcon name="folder-open" :size="16" />
              </button>
            </div>
          </label>
          <label class="grid gap-2">
            <span class="text-sm text-(--app-text-soft)">启动参数（可选）</span>
            <input v-model.trim="form.exeArgs" class="app-input" placeholder="例如 --instance 1" />
          </label>
        </div>

        <div class="grid gap-3">
          <div class="flex items-center justify-between">
            <span class="text-sm font-medium text-(--app-text-strong)">CPU 核心绑定</span>
            <span class="text-xs text-(--app-text-faint)">显示的是物理核心索引，共 {{ cpuCount }} 个</span>
          </div>
          <div class="flex flex-wrap gap-2">
            <label
              v-for="core in cpuIndexes"
              :key="core"
              class="flex items-center gap-2 rounded-full border border-(--app-border) px-3 py-2 text-sm"
            >
              <input
                type="checkbox"
                class="h-4 w-4"
                :checked="form.cores.includes(core)"
                @change="toggleCore(core)"
              />
              <span>{{ core }}</span>
            </label>
          </div>
        </div>

        <div class="grid gap-3 md:grid-cols-2">
          <label class="flex items-center justify-between rounded-[20px] border border-(--app-border) px-4 py-3">
            <span class="text-sm text-(--app-text-strong)">启用设备</span>
            <input v-model="form.enable" type="checkbox" class="toggle toggle-sm" />
          </label>
          <label class="flex items-center justify-between rounded-[20px] border border-(--app-border) px-4 py-3">
            <span class="text-sm text-(--app-text-strong)">自动启动设备进程</span>
            <input v-model="form.autoStart" type="checkbox" class="toggle toggle-sm" />
          </label>
        </div>
      </template>

      <template v-else>
        <div class="grid gap-4 rounded-[24px] border border-(--app-border) bg-(--app-panel-muted)/60 p-4">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-semibold text-(--app-text-strong)">执行策略</p>
              <p class="text-xs text-(--app-text-faint)">设备级运行策略，作用于当前设备会话，不属于脚本定义。</p>
            </div>
          </div>

          <div class="grid gap-4 md:grid-cols-2">
            <label class="grid gap-2">
              <span class="text-sm text-(--app-text-soft)">动作后等待（毫秒）</span>
              <input v-model.number="form.actionWaitMs" class="app-input" type="number" min="0" step="100" />
            </label>
            <label class="grid gap-2">
              <span class="text-sm text-(--app-text-soft)">超时行为</span>
              <AppSelect v-model="form.timeoutAction" :options="timeoutActionOptions" />
            </label>
          </div>

          <div class="grid gap-4 md:grid-cols-2">
            <label class="flex items-center justify-between rounded-[18px] border border-(--app-border) bg-(--app-panel) px-4 py-3">
              <span class="text-sm text-(--app-text-strong)">启用无有效进展超时</span>
              <input v-model="form.progressTimeoutEnabled" type="checkbox" class="toggle toggle-sm" />
            </label>
            <label class="grid gap-2">
              <span class="text-sm text-(--app-text-soft)">超时时间（毫秒）</span>
              <input
                v-model.number="form.progressTimeoutMs"
                class="app-input"
                type="number"
                min="1000"
                step="1000"
                :disabled="!form.progressTimeoutEnabled"
              />
            </label>
          </div>

          <div class="grid gap-2">
            <span class="text-sm text-(--app-text-soft)">通知渠道</span>
            <div class="flex flex-wrap gap-3">
              <label class="flex items-center gap-2 rounded-full border border-(--app-border) bg-(--app-panel) px-3 py-2 text-sm text-(--app-text-strong)">
                <input v-model="form.timeoutNotifyChannels" type="checkbox" value="systemNotification" class="h-4 w-4" />
                系统通知
              </label>
              <label class="flex items-center gap-2 rounded-full border border-(--app-border) bg-(--app-panel) px-3 py-2 text-sm text-(--app-text-strong)">
                <input v-model="form.timeoutNotifyChannels" type="checkbox" value="email" class="h-4 w-4" />
                邮件
              </label>
            </div>
          </div>
        </div>
      </template>

      <div class="flex justify-end gap-3 pt-2">
        <button class="app-button app-button-ghost text-(--app-text-strong) group" type="button" @click="$emit('close')">
          <AppIcon name="x" :size="16" class="opacity-70 transition-opacity group-hover:opacity-100" />
          取消
        </button>
        <button class="app-button app-button-primary shadow-lg shadow-(--app-accent-soft)" type="submit">
          <AppIcon name="save" :size="16" />
          保存设备
        </button>
      </div>
    </form>
  </AppDialog>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue';
import { open as dialogOpen } from '@tauri-apps/plugin-dialog';
import AppSelect from '@/components/shared/AppSelect.vue';
import AppDialog from '@/components/shared/AppDialog.vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import type { DeviceFormState } from '@/types/app/domain';
import type { DeviceTable } from '@/types/bindings/DeviceTable';

const props = defineProps<{
  open: boolean;
  device: DeviceTable | null;
  cpuCount: number;
}>();

defineEmits<{
  close: [];
  save: [form: DeviceFormState];
}>();

const createEmptyForm = (): DeviceFormState => ({
  id: null,
  deviceName: '',
  platform: 'android',
  exePath: '',
  exeArgs: '',
  cores: [],
  logLevel: 'Info',
  logToFile: true,
  capMethodType: 'window',
  capMethodValue: '',
  connectMethod: 'directTcp',
  connectAddress: '',
  connectDeviceName: '',
  enable: true,
  autoStart: false,
  actionWaitMs: 500,
  progressTimeoutEnabled: false,
  progressTimeoutMs: 30000,
  timeoutAction: 'stopExecution',
  timeoutNotifyChannels: [],
});

const form = reactive<DeviceFormState>(createEmptyForm());
const activeTab = ref<'basic' | 'policy'>('basic');

const tabs = [
  { label: '其余信息', value: 'basic' as const },
  { label: '执行策略', value: 'policy' as const },
];

const cpuIndexes = computed(() => Array.from({ length: props.cpuCount }, (_, index) => index));

const logLevelOptions = [
  { label: 'Off', value: 'Off' },
  { label: 'Error', value: 'Error' },
  { label: 'Warn', value: 'Warn' },
  { label: 'Info', value: 'Info' },
  { label: 'Debug', value: 'Debug' },
];

const captureOptions = [
  { label: '窗口截取', value: 'window' },
  { label: 'ADB 截图', value: 'adb' },
];

const connectOptions = [
  { label: 'TCP 直连', value: 'directTcp' },
  { label: 'ADB 服务（按 IP）', value: 'serverConnectByIp' },
  { label: 'ADB 服务（按名称）', value: 'serverConnectByName' },
];

/*const platformOptions = [
  { label: 'Android', value: 'android' },
  { label: '桌面程序', value: 'desktop' },
];*/

const timeoutActionOptions = [
  { label: '停止执行', value: 'stopExecution', description: '结束当前执行。' },
  { label: '执行恢复任务', value: 'runRecoveryTask', description: '使用脚本预先配置的恢复任务。' },
  { label: '跳过当前任务', value: 'skipCurrentTask', description: '跳过当前任务，继续后续任务。' },
];

function normalizeTimeoutAction(value: string | null | undefined): DeviceFormState['timeoutAction'] {
  if (value === 'runRecoveryTask' || value === 'skipCurrentTask') {
    return value;
  }
  return 'stopExecution';
}

const syncForm = (device: DeviceTable | null) => {
  Object.assign(form, createEmptyForm());
  if (!device) {
    return;
  }

  form.id = device.id;
  form.deviceName = device.data.deviceName;
  form.platform = device.data.platform ?? 'android';
  form.exePath = device.data.exePath ?? '';
  form.exeArgs = device.data.exeArgs ?? '';
  form.cores = [...device.data.cores];
  form.logLevel = device.data.logLevel;
  form.logToFile = device.data.logToFile;
  form.enable = device.data.enable;
  form.autoStart = device.data.autoStart;
  form.actionWaitMs = Number(device.data.executionPolicy?.actionWaitMs ?? 500);
  form.progressTimeoutEnabled = Boolean(device.data.executionPolicy?.progressTimeoutEnabled ?? false);
  form.progressTimeoutMs = Number(device.data.executionPolicy?.progressTimeoutMs ?? 30000);
  form.timeoutAction = normalizeTimeoutAction(device.data.executionPolicy?.timeoutAction);
  form.timeoutNotifyChannels = [...(device.data.executionPolicy?.timeoutNotifyChannels ?? [])];

  if (typeof device.data.capMethod === 'string') {
    form.capMethodType = 'adb';
  } else {
    form.capMethodType = 'window';
    form.capMethodValue = device.data.capMethod.window;
  }

  const connect = device.data.adbConnect;
  if (!connect) {
    return;
  }

  if ('directTcp' in connect) {
    form.connectMethod = 'directTcp';
    form.connectAddress = connect.directTcp ?? '';
  } else if ('serverConnectByIp' in connect) {
    form.connectMethod = 'serverConnectByIp';
    form.connectAddress = connect.serverConnectByIp.clientConnect ?? '';
  } else if ('serverConnectByName' in connect) {
    form.connectMethod = 'serverConnectByName';
    form.connectDeviceName = connect.serverConnectByName.deviceName ?? '';
  }
};

const toggleCore = (core: number) => {
  const exists = form.cores.includes(core);
  form.cores = exists ? form.cores.filter((item) => item !== core) : [...form.cores, core].sort((a, b) => a - b);
};

const pickExePath = async () => {
  const value = await dialogOpen({
    multiple: false,
    directory: false,
    filters: [{ name: 'Executable Files', extensions: ['exe', 'bat', 'cmd', 'lnk'] }],
  });
  if (typeof value === 'string' && value) {
    form.exePath = value;
  }
};

watch(
  () => [props.open, props.device] as const,
  ([open, device]) => {
    if (open) {
      activeTab.value = 'basic';
      syncForm(device);
    }
  },
  { immediate: true },
);
</script>

<style scoped>
.editor-panel-tabs {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  border-bottom: 1px solid var(--app-border);
}

.editor-panel-tab {
  position: relative;
  margin-bottom: -1px;
  border-bottom: 2px solid transparent;
  padding: 0.75rem 0.35rem 0.85rem;
  color: var(--app-text-faint);
  font-size: 0.93rem;
  font-weight: 600;
  transition: color 0.16s ease, border-color 0.16s ease;
}

.editor-panel-tab:hover {
  color: var(--app-text-soft);
}

.editor-panel-tab-active {
  border-bottom-color: var(--app-accent);
  color: var(--app-text-strong);
}

.path-input-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 0.65rem;
  align-items: center;
}

.path-picker-button {
  min-width: 2.75rem;
  height: 2.75rem;
  padding: 0;
}
</style>
