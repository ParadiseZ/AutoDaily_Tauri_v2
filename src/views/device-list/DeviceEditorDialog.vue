<template>
  <AppDialog
    :open="open"
    :title="device ? '编辑设备' : '添加设备'"
    width-class="max-w-4xl"
    @close="$emit('close')"
  >
    <form class="grid gap-5" @submit.prevent="$emit('save', form)">
      <div class="grid gap-4 md:grid-cols-2">
        <label class="grid gap-2">
          <span class="text-sm text-[var(--app-text-soft)]">名称</span>
          <input v-model.trim="form.deviceName" class="app-input" placeholder="MuMu 模拟器 12" />
        </label>
        <label class="grid gap-2">
          <span class="text-sm text-[var(--app-text-soft)]">日志级别</span>
          <AppSelect v-model="form.logLevel" :options="logLevelOptions" />
        </label>
      </div>

      <div class="grid gap-4 md:grid-cols-2">
        <label class="grid gap-2">
          <span class="text-sm text-[var(--app-text-soft)]">运行平台</span>
          <AppSelect v-model="form.platform" :options="platformOptions" />
        </label>
      </div>

      <div class="grid gap-4 md:grid-cols-2">
        <label class="grid gap-2">
          <span class="text-sm text-[var(--app-text-soft)]">截图方式</span>
          <AppSelect v-model="form.capMethodType" :options="captureOptions" />
        </label>
        <label class="grid gap-2">
          <span class="text-sm text-[var(--app-text-soft)]">窗口名 / 标识</span>
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
          <span class="text-sm text-[var(--app-text-soft)]">连接方式</span>
          <AppSelect v-model="form.connectMethod" :options="connectOptions" />
        </label>
        <label class="grid gap-2">
          <span class="text-sm text-[var(--app-text-soft)]">地址 / 设备名</span>
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
          <span class="text-sm text-[var(--app-text-soft)]">设备启动路径（可选）</span>
          <input v-model.trim="form.exePath" class="app-input" placeholder="模拟器启动路径" />
        </label>
        <label class="grid gap-2">
          <span class="text-sm text-[var(--app-text-soft)]">启动参数（可选）</span>
          <input v-model.trim="form.exeArgs" class="app-input" placeholder="例如 --instance 1" />
        </label>
      </div>

      <div class="grid gap-3">
        <div class="flex items-center justify-between">
          <span class="text-sm font-medium text-[var(--app-text-strong)]">CPU 核心绑定</span>
          <span class="text-xs text-[var(--app-text-faint)]">影响执行效率</span>
        </div>
        <div class="flex flex-wrap gap-10">
          <label v-for="index in cpuCount" :key="index-1" class="flex items-center gap-2">
            <input
                type="checkbox"
                :value="index"
                class="w-4 h-4"
                :class="{ 'app-button-primary': form.cores.includes(index - 1) }"
                @click="toggleCore(index - 1)"
            />
            <span class="text-sm">{{ index }}</span>
          </label>
        </div>
      </div>

      <div class="grid gap-4 rounded-[24px] border border-[var(--app-border)] bg-[var(--app-panel-muted)]/60 p-4">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-semibold text-[var(--app-text-strong)]">执行策略</p>
            <p class="text-xs text-[var(--app-text-faint)]">设备级运行策略，作用于当前设备会话，不属于脚本定义。</p>
          </div>
        </div>

        <div class="grid gap-4 md:grid-cols-2">
          <label class="grid gap-2">
            <span class="text-sm text-[var(--app-text-soft)]">动作后等待（毫秒）</span>
            <input v-model.number="form.actionWaitMs" class="app-input" type="number" min="0" step="100" />
          </label>
          <label class="grid gap-2">
            <span class="text-sm text-[var(--app-text-soft)]">超时行为</span>
            <AppSelect v-model="form.timeoutAction" :options="timeoutActionOptions" />
          </label>
        </div>

        <div class="grid gap-4 md:grid-cols-2">
          <label class="flex items-center justify-between rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel)] px-4 py-3">
            <span class="text-sm text-[var(--app-text-strong)]">启用无有效进展超时</span>
            <input v-model="form.progressTimeoutEnabled" type="checkbox" class="toggle toggle-sm" />
          </label>
          <label class="grid gap-2">
            <span class="text-sm text-[var(--app-text-soft)]">超时时间（毫秒）</span>
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
          <span class="text-sm text-[var(--app-text-soft)]">通知渠道</span>
          <div class="flex flex-wrap gap-3">
            <label class="flex items-center gap-2 rounded-full border border-[var(--app-border)] bg-[var(--app-panel)] px-3 py-2 text-sm text-[var(--app-text-strong)]">
              <input v-model="form.timeoutNotifyChannels" type="checkbox" value="systemNotification" class="h-4 w-4" />
              系统通知
            </label>
            <label class="flex items-center gap-2 rounded-full border border-[var(--app-border)] bg-[var(--app-panel)] px-3 py-2 text-sm text-[var(--app-text-strong)]">
              <input v-model="form.timeoutNotifyChannels" type="checkbox" value="email" class="h-4 w-4" />
              邮件
            </label>
          </div>
          <p class="text-xs text-[var(--app-text-faint)]">通知可多选；行为只有一个。真正的超时检测与执行动作仍在后续 runtime 阶段接入。</p>
        </div>
      </div>

      <div class="grid gap-3 md:grid-cols-2">
        <label class="flex items-center justify-between rounded-[20px] border border-[var(--app-border)] px-4 py-3">
          <span class="text-sm text-[var(--app-text-strong)]">启用设备</span>
          <input v-model="form.enable" type="checkbox" class="toggle toggle-sm" />
        </label>
        <label class="flex items-center justify-between rounded-[20px] border border-[var(--app-border)] px-4 py-3">
          <span class="text-sm text-[var(--app-text-strong)]">自动启动设备进程</span>
          <input v-model="form.autoStart" type="checkbox" class="toggle toggle-sm" />
        </label>
      </div>

      <div class="flex justify-end gap-3 pt-2">
        <button class="app-button app-button-ghost text-[var(--app-text-strong)] group" type="button" @click="$emit('close')">
          <AppIcon name="x" :size="16" class="opacity-70 transition-opacity group-hover:opacity-100" />
          取消
        </button>
        <button class="app-button app-button-primary shadow-lg shadow-[var(--app-accent-soft)]" type="submit">
          <AppIcon name="save" :size="16" />
          保存设备
        </button>
      </div>
    </form>
  </AppDialog>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue';
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

const platformOptions = [
  { label: 'Android', value: 'android' },
  { label: '桌面程序', value: 'desktop' },
];

const timeoutActionOptions = [
  { label: '只通知', value: 'notifyOnly', description: '只发通知，不改变执行流。' },
  { label: '暂停执行', value: 'pauseExecution', description: '进入暂停态，等待人工介入。' },
  { label: '停止执行', value: 'stopExecution', description: '结束当前执行。' },
  { label: '重启应用', value: 'restartApp', description: '先做通用重启，再由脚本自行恢复业务流。' },
  { label: '执行恢复任务', value: 'runRecoveryTask', description: '使用脚本预先配置的恢复任务。' },
  { label: '跳过当前任务', value: 'skipCurrentTask', description: '跳过当前任务，继续后续任务。' },
];

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
  form.timeoutAction = device.data.executionPolicy?.timeoutAction ?? 'stopExecution';
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

watch(
  () => [props.open, props.device] as const,
  ([open, device]) => {
    if (open) {
      syncForm(device);
    }
  },
  { immediate: true },
);
</script>
