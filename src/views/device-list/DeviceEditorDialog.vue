<template>
  <AppDialog
    :open="open"
    :title="device ? '编辑设备' : '添加设备'"
    width-class="max-w-4xl max-h-[calc(100vh-3rem)] flex flex-col"
    @close="$emit('close')"
  >
    <form class="flex min-h-0 flex-1 flex-col gap-5 overflow-hidden overflow-y-auto" @submit.prevent="$emit('save', form)">
      <fieldset :disabled="busy" class="flex min-h-0 flex-1 flex-col gap-5 overflow-hidden">
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

        <div class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
          <div class="space-y-5 pb-1">
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
                <label v-if="form.capMethodType === 'window'" class="grid gap-2">
                  <span class="text-sm text-(--app-text-soft)">窗口名 / 标识</span>
                  <input v-model.trim="form.capMethodValue" class="app-input" placeholder="窗口标题" />
                </label>
              </div>

              <div v-if="form.capMethodType === 'window'" class="grid gap-4 md:grid-cols-2">
                <label class="grid gap-2">
                  <span class="text-sm text-(--app-text-soft)">截图接口</span>
                  <AppSelect v-model="form.windowCaptureInterface" :options="windowCaptureInterfaceOptions" />
                </label>
                <label class="grid gap-2">
                  <span class="text-sm text-(--app-text-soft)">帧超时时间（秒）</span>
                  <input
                    v-model.number="form.frameTimeoutSecs"
                    class="app-input"
                    type="number"
                    min="1"
                    step="1"
                    placeholder="10"
                  />
                </label>
              </div>

              <div v-if="form.capMethodType === 'window'">
                <label class="grid gap-2">
                  <span class="text-sm text-(--app-text-soft)">窗口内容偏移（左,上,右,下）</span>
                  <input
                    v-model.trim="form.windowOffsets"
                    class="app-input"
                    placeholder="1,40,1,1"
                    pattern="\s*\d+\s*,\s*\d+\s*,\s*\d+\s*,\s*\d+\s*"
                    title="请按左,上,右,下填写四个非负整数，例如 1,40,1,1"
                  />
                  <span class="text-xs text-(--app-text-muted)">四个非负整数；左、上同时决定裁剪起点。</span>
                </label>
              </div>

              <div class="grid gap-4 md:grid-cols-2">
                <label class="grid gap-2">
                  <span class="text-sm text-(--app-text-soft)">连接通道</span>
                  <AppSelect v-model="form.transportKind" :options="transportOptions" />
                </label>
                <label v-if="form.transportKind === 'emulatorTcp'" class="grid gap-2">
                  <span class="text-sm text-(--app-text-soft)">模拟器连接方式</span>
                  <AppSelect v-model="form.emulatorConnectMode" :options="emulatorConnectModeOptions" />
                </label>
                <label v-else class="grid gap-2">
                  <span class="text-sm text-(--app-text-soft)">{{ transportFieldLabel }}</span>
                  <input
                    v-model.trim="form.connectIdentifier"
                    class="app-input"
                    :placeholder="transportFieldPlaceholder"
                  />
                </label>
              </div>

              <label v-if="form.transportKind === 'emulatorTcp'" class="grid gap-2">
                <span class="text-sm text-(--app-text-soft)">{{ transportFieldLabel }}</span>
                <input
                  v-if="usesAddressInput"
                  v-model.trim="form.connectAddress"
                  class="app-input"
                  :placeholder="transportFieldPlaceholder"
                />
                <input
                  v-else
                  v-model.trim="form.connectIdentifier"
                  class="app-input"
                  :placeholder="transportFieldPlaceholder"
                />
              </label>

              <div v-if="needsAdbServerConfig" class="grid gap-4 md:grid-cols-2">
                <label class="grid gap-2">
                  <span class="text-sm text-(--app-text-soft)">ADB 程序路径</span>
                  <div class="path-input-row">
                    <input v-model.trim="form.adbPath" class="app-input" placeholder="adb.exe 路径" />
                    <button class="app-button app-button-ghost path-picker-button" type="button" @click="pickAdbPath">
                      <AppIcon name="folder-open" :size="16" />
                    </button>
                  </div>
                </label>
                <label class="grid gap-2">
                  <span class="text-sm text-(--app-text-soft)">ADB Server</span>
                  <input v-model.trim="form.adbServerConnect" class="app-input" placeholder="127.0.0.1:5037" />
                </label>
              </div>

              <div v-if="form.transportKind === 'emulatorTcp'" class="grid gap-4 md:grid-cols-2">
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

              <div v-if="form.transportKind === 'emulatorTcp'" class="grid gap-2 md:max-w-sm">
                <label class="grid gap-2">
                  <span class="text-sm text-(--app-text-soft)">启动探测延迟（秒）</span>
                  <input
                    v-model.number="form.startupDelaySecs"
                    class="app-input"
                    type="number"
                    min="0"
                    step="1"
                    placeholder="15"
                  />
                </label>
                <p class="text-xs text-(--app-text-faint)">仅模拟器连接生效。启动模拟器后会先等待这段时间，再开始 shell 探测。</p>
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
                  <span class="text-sm text-(--app-text-strong)">自动运行队列</span>
                  <input v-model="form.autoStart" type="checkbox" class="toggle toggle-sm" />
                </label>
              </div>

              <label class="flex items-center justify-between rounded-[20px] border border-(--app-border) px-4 py-3">
                <div class="space-y-1">
                  <span class="text-sm text-(--app-text-strong)">日志写入文件</span>
                  <p class="text-xs text-(--app-text-faint)">关闭后仅保留前端实时日志，不再写入设备日志文件。</p>
                </div>
                <input v-model="form.logToFile" type="checkbox" class="toggle toggle-sm" />
              </label>
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
          </div>
        </div>

        <p v-if="busy" class="text-sm text-(--app-text-soft)">正在处理设备子进程，请稍候。</p>

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
      </fieldset>
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
  busy?: boolean;
}>();

defineEmits<{
  close: [];
  save: [form: DeviceFormState];
}>();

const createEmptyForm = (): DeviceFormState => ({
  id: null,
  deviceName: '',
  platform: 'android',
  transportKind: 'emulatorTcp',
  emulatorConnectMode: 'tcpAddress',
  startupDelaySecs: 15,
  exePath: '',
  exeArgs: '',
  cores: [],
  logLevel: 'Off',
  logToFile: true,
  capMethodType: 'window',
  capMethodValue: '',
  windowCaptureInterface: 'dxgi',
  frameTimeoutSecs: 10,
  windowOffsets: '1,40,1,1',
  connectAddress: '',
  connectIdentifier: '',
  adbPath: '',
  adbServerConnect: '127.0.0.1:5037',
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
const usesAddressInput = computed(() => form.transportKind === 'emulatorTcp' && form.emulatorConnectMode === 'tcpAddress');
const needsAdbServerConfig = computed(
  () => form.transportKind !== 'emulatorTcp' || form.emulatorConnectMode === 'identifier',
);
const transportFieldLabel = computed(() => {
  if (usesAddressInput.value) {
    return 'TCP 地址';
  }
  return '设备标识';
});
const transportFieldPlaceholder = computed(() => {
  if (usesAddressInput.value) {
    return '127.0.0.1:5555';
  }
  return '例如 emulator-5554 / 设备序列号';
});

const supportsWindowCapture = computed(() => form.transportKind === 'emulatorTcp');

const logLevelOptions = [
  { label: 'Off', value: 'Off' },
  { label: 'Error', value: 'Error' },
  { label: 'Warn', value: 'Warn' },
  { label: 'Info', value: 'Info' },
  { label: 'Debug', value: 'Debug' },
];

const captureOptions = computed(() => {
  if (supportsWindowCapture.value) {
    return [
      { label: '窗口截取', value: 'window' },
      { label: 'ADB 截图', value: 'adb' },
    ];
  }
  return [{ label: 'ADB 截图', value: 'adb', description: '当前通道不是模拟器，窗口截图不可用。' }];
});

const windowCaptureInterfaceOptions = [
  { label: 'DXGI', value: 'dxgi', description: '默认方案，等待桌面新帧后裁剪窗口区域。' },
  { label: 'DwmGetDxSharedSurface', value: 'dwmGetDxSharedSurface', description: '直接读取目标窗口的 DWM 共享纹理。' },
  { label: 'WGC', value: 'wgc', description: '通过 Windows Graphics Capture 直接采集目标窗口。' },
  { label: 'GDI', value: 'gdi', description: '兼容方案，直接做窗口截图。' },
];

const transportOptions = [
  { label: '模拟器', value: 'emulatorTcp' },
  { label: 'ADB USB', value: 'adbUsb' },
  { label: 'ADB 无线', value: 'adbWireless' },
];

const emulatorConnectModeOptions = [
  { label: 'TCP 地址', value: 'tcpAddress', description: '直接连接固定的模拟器 ADB 地址。' },
  { label: '设备标识', value: 'identifier', description: '通过 ADB Server 按设备标识连接模拟器。' },
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
  form.transportKind = device.data.transportKind;
  form.emulatorConnectMode = device.data.emulatorConnectMode ?? 'tcpAddress';
  form.startupDelaySecs = Number(device.data.startupDelaySecs ?? 15);
  form.connectAddress = device.data.connectAddress ?? '';
  form.connectIdentifier = device.data.connectIdentifier ?? '';
  form.adbPath = device.data.adbPath ?? '';
  form.adbServerConnect = device.data.adbServerConnect ?? '127.0.0.1:5037';
  form.exePath = device.data.exePath ?? '';
  form.exeArgs = device.data.exeArgs ?? '';
  form.cores = [...device.data.cores];
  form.logLevel = device.data.logLevel ?? 'Off';
  form.logToFile = device.data.logToFile ?? true;
  form.enable = device.data.enable;
  form.autoStart = device.data.autoStart;
  form.actionWaitMs = Number(device.data.executionPolicy?.actionWaitMs ?? 500);
  form.progressTimeoutEnabled = Boolean(device.data.executionPolicy?.progressTimeoutEnabled ?? false);
  form.progressTimeoutMs = Number(device.data.executionPolicy?.progressTimeoutMs ?? 30000);
  form.timeoutAction = normalizeTimeoutAction(device.data.executionPolicy?.timeoutAction);
  form.timeoutNotifyChannels = [...(device.data.executionPolicy?.timeoutNotifyChannels ?? [])];

  if (device.data.capMethod.type === 'adb') {
    form.capMethodType = 'adb';
  } else {
    form.capMethodType = 'window';
    form.capMethodValue = device.data.capMethod.title;
    form.windowCaptureInterface = device.data.capMethod.interface ?? 'dxgi';
    form.frameTimeoutSecs = Number(device.data.capMethod.frameTimeoutSecs ?? 10);
    form.windowOffsets = [
      device.data.capMethod.offsetLeftPx ?? 1,
      device.data.capMethod.offsetTopPx ?? 40,
      device.data.capMethod.offsetRightPx ?? 1,
      device.data.capMethod.offsetBottomPx ?? 1,
    ].join(',');
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

const pickAdbPath = async () => {
  const value = await dialogOpen({
    multiple: false,
    directory: false,
    filters: [{ name: 'ADB Executable', extensions: ['exe'] }],
  });
  if (typeof value === 'string' && value) {
    form.adbPath = value;
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

watch(
  () => [form.transportKind, form.capMethodType] as const,
  ([transportKind, capMethodType]) => {
    if (transportKind !== 'emulatorTcp' && capMethodType === 'window') {
      form.capMethodType = 'adb';
      form.capMethodValue = '';
    }
    if (capMethodType === 'adb') {
      form.windowCaptureInterface = 'dxgi';
      form.frameTimeoutSecs = 10;
      form.windowOffsets = '1,40,1,1';
    }
  },
);

</script>

<style scoped>
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
