<template>
  <div class="p-6">
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold">设备列表</h1>
      <button class="btn btn-primary btn-sm" @click="openModal(null)"><Plus class="w-4 h-4 mr-1" /> 添加设备</button>
    </div>

    <div class="overflow-x-auto bg-base-100 rounded-lg shadow">
      <table class="table w-full">
        <thead>
          <tr>
            <th>名称</th>
            <th>ip/名称</th>
            <th>CPU核心</th>
            <th>日志级别</th>
            <th>启用</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="device in devices" :key="device.id">
            <td>
              <div class="font-bold">{{ device.data.deviceName }}</div>
              <div class="text-xs opacity-50">{{ device.id }}</div>
            </td>
            <td>
              <span v-if="device.data.adbConnect" class="badge badge-ghost badge-sm"
                >{{ getConnectDisplay(device.data.adbConnect) }}</span
              >
            </td>
            <td>{{ device.data.cores?.join(',') || 'None' }}</td>
            <td>{{ device.data.logLevel }}</td>
            <td>
              <input
                type="checkbox"
                class="toggle toggle-sm toggle-success"
                :checked="device.data.enable"
                @click="toggleEnable(device)"
              />
            </td>
            <td>
              <button class="btn btn-ghost btn-xs" @click="openModal(device)">编辑</button>
              <button class="btn btn-ghost btn-xs text-error" @click="deleteDevice(device.id, device.data.deviceName)">
                删除
              </button>
            </td>
          </tr>
          <tr v-if="devices.length === 0">
            <td colspan="6" class="text-center py-4 opacity-50">嗯...你可以添加一个设备</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Modal -->
    <dialog id="device_modal" class="modal">
      <div class="modal-box w-11/12 max-w-2xl">
        <h3 class="font-bold text-lg mb-4">{{ isEditing ? '编辑' : '添加' }}</h3>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div class="form-control">
            <label class="label"><span class="label-text">名称</span></label>
            <input
              type="text"
              v-model="form.data.deviceName"
              class="input input-bordered w-full"
              placeholder="MuMu12"
            />
          </div>

          <div class="form-control">
            <label class="label"><span class="label-text">日志级别</span></label>
            <select v-model="form.data.logLevel" class="select select-bordered w-full">
              <option value="Off">Off</option>
              <option value="Error">Error</option>
              <option value="Warn">Warn</option>
              <option value="Info">Info</option>
              <option value="Debug">Debug</option>
            </select>
          </div>

          <div class="form-control">
            <label class="label cursor-pointer justify-start gap-4">
              <span class="label-text">日志写入文件</span>
              <input type="checkbox" v-model="form.data.logToFile" class="checkbox checkbox-sm" />
            </label>
          </div>

          <div class="form-control md:col-span-2">
            <label class="label"><span class="label-text">CPU核心 (多选)</span></label>
            <div class="flex flex-wrap gap-2 p-2 bg-base-200 rounded-lg">
              <label
                v-for="i in cpuCount"
                :key="i - 1"
                class="label cursor-pointer flex gap-1 bg-base-100 px-2 py-1 rounded border border-base-300 hover:bg-base-300 transition-colors"
              >
                <input type="checkbox" :value="i - 1" v-model="form.data.cores" class="checkbox checkbox-xs" />
                <span class="label-text text-xs">Core {{ i - 1 }}</span>
              </label>
            </div>
          </div>

          <div class="form-control">
            <label class="label"><span class="label-text">截图方式</span></label>
            <select v-model="capMethodType" class="select select-bordered w-full">
              <option value="window">窗口</option>
              <option value="adb">ADB</option>
            </select>
          </div>

          <div class="form-control" v-if="capMethodType === 'window'">
            <label class="label"><span class="label-text">窗口名称</span></label>
            <input
              type="text"
              v-model="capMethodValue"
              class="input input-bordered w-full"
              placeholder="MuMu安卓设备"
            />
          </div>
          <div class="form-control" v-if="capMethodType === 'adb'">
            <label class="label"><span class="label-text">ADB设备名称</span></label>
            <input
              type="text"
              v-model="capMethodValue"
              class="input input-bordered w-full"
              placeholder="emulator-5554"
            />
          </div>

          <div class="divider md:col-span-2 font-bold text-sm">连接设置</div>

          <div class="form-control">
            <label class="label"><span class="label-text">连接方式</span></label>
            <select v-model="connectMethod" class="select select-bordered w-full">
              <option value="directTcp">TCP 直连</option>
              <option value="serverConnectByIp">ADB 服务 (IP)</option>
              <option value="serverConnectByName">ADB 服务 (设备名)</option>
            </select>
          </div>

          <div class="form-control" v-if="connectMethod === 'directTcp'">
            <label class="label"><span class="label-text">设备地址 (IP:端口)</span></label>
            <input type="text" v-model="connectAddr" class="input input-bordered w-full" placeholder="127.0.0.1:5555" />
          </div>

          <template v-if="connectMethod === 'serverConnectByIp'">
            <div class="form-control">
              <label class="label"><span class="label-text">设备地址 (IP:端口)</span></label>
              <input type="text" v-model="connectAddr" class="input input-bordered w-full" placeholder="127.0.0.1:5555" />
            </div>
          </template>

          <template v-if="connectMethod === 'serverConnectByName'">
            <div class="form-control">
              <label class="label"><span class="label-text">设备名称</span></label>
              <input type="text" v-model="connectDeviceName" class="input input-bordered w-full" placeholder="emulator-5554" />
            </div>
          </template>

          <div class="divider md:col-span-2 font-bold text-sm">控制</div>

          <div class="form-control md:col-span-2">
            <label class="label cursor-pointer justify-start gap-4">
              <span class="label-text">启用</span>
              <input type="checkbox" v-model="form.data.enable" class="checkbox" />
            </label>
          </div>
          <div class="form-control md:col-span-2">
            <label class="label cursor-pointer justify-start gap-4">
              <span class="label-text">自动启动</span>
              <input type="checkbox" v-model="form.data.autoStart" class="checkbox" />
            </label>
            <span class="label-text-alt opacity-50 ml-1">启用设备时自动启动并连接目标设备，然后调度脚本队列</span>
          </div>
        </div>

        <div class="modal-action">
          <form method="dialog">
            <button class="btn">取消</button>
            <button class="btn btn-primary ml-2" @click.prevent="saveDevice">保存</button>
          </form>
        </div>
      </div>
    </dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue';
import { confirm, message } from '@tauri-apps/plugin-dialog';
import { Plus } from 'lucide-vue-next';
import type { DeviceTable, DeviceConfig, CapMethod, ImageCompression, LogLevel, ADBConnectConfig } from '@/types/bindings';

import { useDevices } from '@/assets/js/useDevices';

const {
  getAllDevices,
  saveDevice: apiSaveDevice,
  deleteDevice: apiDeleteDevice,
  getUuidV7,
  getCpuCount,
} = useDevices();

const devices = ref<DeviceTable[]>([]);
const isEditing = ref(false);
const capMethodType = ref<'window' | 'adb'>('window');
const capMethodValue = ref('');
const cpuCount = ref<number>(0);

// ADB 连接方式
const connectMethod = ref<'directTcp' | 'serverConnectByIp' | 'serverConnectByName'>('directTcp');
const connectAddr = ref('');         // IP:Port 格式
const connectDeviceName = ref('');   // 设备名称（serverConnectByName 用）

const defaultFormData = (): DeviceConfig => ({
  deviceName: '',
  cores: [],
  logLevel: 'Off' as LogLevel,
  logToFile: true,
  capMethod: 'adb',
  imageCompression: 'WindowOriginal' as ImageCompression,
  enable: true,
  exePath: null,
  exeArgs: null,
  adbConnect: null,
  autoStart: false,
});

const form = reactive<{ id: string | null; data: DeviceConfig }>({
  id: '',
  data: defaultFormData(),
});

/** 从 ADBConnectConfig 提取显示用文本 */
const getConnectDisplay = (cfg: ADBConnectConfig): string => {
  if ('directTcp' in cfg) return cfg.directTcp || '未设置';
  if ('serverConnectByIp' in cfg) return cfg.serverConnectByIp.clientConnect || '未设置';
  if ('serverConnectByName' in cfg) return cfg.serverConnectByName.deviceName || '未设置';
  return '未知';
};

/** 从 ADBConnectConfig 解析到表单字段 */
const parseAdbConnect = (cfg: ADBConnectConfig | null) => {
  if (!cfg) {
    connectMethod.value = 'directTcp';
    connectAddr.value = '';
    connectDeviceName.value = '';
    return;
  }
  if ('directTcp' in cfg) {
    connectMethod.value = 'directTcp';
    connectAddr.value = cfg.directTcp || '';
  } else if ('serverConnectByIp' in cfg) {
    connectMethod.value = 'serverConnectByIp';
    connectAddr.value = cfg.serverConnectByIp.clientConnect || '';
  } else if ('serverConnectByName' in cfg) {
    connectMethod.value = 'serverConnectByName';
    connectDeviceName.value = cfg.serverConnectByName.deviceName || '';
  }
};

/** 从表单字段构建 ADBConnectConfig（adb_path 不存储在此，运行时注入） */
const buildAdbConnect = (): ADBConnectConfig | null => {
  if (connectMethod.value === 'directTcp') {
    return connectAddr.value ? { directTcp: connectAddr.value } : null;
  }
  if (connectMethod.value === 'serverConnectByIp') {
    return connectAddr.value
      ? { serverConnectByIp: { adbConfig: { adbPath: null, serverConnect: null }, clientConnect: connectAddr.value } }
      : null;
  }
  if (connectMethod.value === 'serverConnectByName') {
    return connectDeviceName.value
      ? { serverConnectByName: { adbConfig: { adbPath: null, serverConnect: null }, deviceName: connectDeviceName.value } }
      : null;
  }
  return null;
};

const loadDevices = async () => {
  try {
    devices.value = await getAllDevices();
  } catch (e) {
    await message('加载设备失败: ' + e, { title: '错误', kind: 'error' });
  }
};

const openModal = (device: DeviceTable | null) => {
  if (device) {
    isEditing.value = true;
    const cloned = JSON.parse(JSON.stringify(device));
    form.id = cloned.id;
    form.data = cloned.data;

    // Parse capMethod
    if (form.data.capMethod) {
      if (typeof form.data.capMethod === 'object' && 'window' in form.data.capMethod) {
        capMethodType.value = 'window';
        capMethodValue.value = form.data.capMethod.window;
      } else if (form.data.capMethod === 'adb') {
        capMethodType.value = 'adb';
        capMethodValue.value = '';
      }
    } else {
      capMethodType.value = 'window';
      capMethodValue.value = '';
    }

    // Parse adbConnect
    parseAdbConnect(form.data.adbConnect);
  } else {
    isEditing.value = false;
    form.id = null;
    form.data = defaultFormData();
    capMethodType.value = 'window';
    capMethodValue.value = '';
    parseAdbConnect(null);
  }
  (document.getElementById('device_modal') as HTMLDialogElement).showModal();
};

const saveDevice = async () => {
  try {
    if (capMethodType.value === 'window') {
      form.data.capMethod = { window: capMethodValue.value };
    } else {
      form.data.capMethod = 'adb';
    }

    // 构建 adbConnect
    form.data.adbConnect = buildAdbConnect();

    if (!form.id) {
      form.id = (await getUuidV7()) as string;
    }
    await apiSaveDevice(form as DeviceTable);
    (document.getElementById('device_modal') as HTMLDialogElement).close();
    await loadDevices();
  } catch (e) {
    await message('保存失败: ' + e, { title: '错误', kind: 'error' });
  }
};

const deleteDevice = async (id: string, name: string) => {
  if (!(await confirm('确定要删除【' + name + '】吗？', { title: '删除设备', kind: 'warning' }))) return;
  try {
    await apiDeleteDevice(id);
    await loadDevices();
  } catch (e) {
    await message('删除失败: ' + e, { title: '错误', kind: 'error' });
  }
};

const toggleEnable = async (device: DeviceTable) => {
  device.data.enable = !device.data.enable;
  try {
    await apiSaveDevice(device);
  } catch (e) {
    await message('保存失败: ' + e, { title: '错误', kind: 'error' });
    device.data.enable = !device.data.enable; // revert
  }
};

onMounted(async () => {
  await loadDevices();
  try {
    cpuCount.value = await getCpuCount();
  } catch (e) {
    await message('获取CPU核心数失败: ' + e, { title: '错误', kind: 'error' });
  }
});
</script>
