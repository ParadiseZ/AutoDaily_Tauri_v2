<template>
  <div class="p-6">
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold">设备列表</h1>
      <button class="btn btn-primary btn-sm" @click="openModal()">
        <Plus class="w-4 h-4 mr-1" /> 添加设备
      </button>
    </div>

    <div class="overflow-x-auto bg-base-100 rounded-lg shadow">
      <table class="table w-full">
        <thead>
          <tr>
            <th>名称</th>
            <th>类型</th>
            <th>核心数</th>
            <th>日志级别</th>
            <th>状态</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="device in devices" :key="device.deviceId">
            <td>
              <div class="font-bold">{{ device.deviceName }}</div>
              <div class="text-xs opacity-50">{{ device.deviceId }}</div>
            </td>
            <td>
              <span v-if="device.adbInfo" class="badge badge-ghost badge-sm">Emulator</span>
              <span v-else class="badge badge-ghost badge-sm">Windows</span>
            </td>
            <td>{{ device.cores }}</td>
            <td>{{ device.logLevel }}</td>
            <td>
              <input type="checkbox" class="toggle toggle-sm toggle-success" :checked="device.enable" @click="toggleEnable(device)" />
            </td>
            <td>
              <button class="btn btn-ghost btn-xs" @click="openModal(device)">编辑</button>
              <button class="btn btn-ghost btn-xs text-error" @click="deleteDevice(device.deviceId)">删除</button>
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
            <label class="label"><span class="label-text">设备名称</span></label>
            <input type="text" v-model="form.deviceName" class="input input-bordered w-full" placeholder="MuMu12"/>
          </div>
          
          <div class="form-control">
            <label class="label"><span class="label-text">CPU核心</span></label>
            <input type="number" v-model.number="form.cores" class="input input-bordered w-full" placeholder="4"/>
          </div>

          <div class="form-control">
            <label class="label"><span class="label-text">日志级别</span></label>
            <select v-model="form.logLevel" class="select select-bordered w-full">
              <option value="Off">Off</option>
              <option value="Error">Error</option>
              <option value="Warn">Warn</option>
              <option value="Info">Info</option>
              <option value="Debug">Debug</option>
              <option value="Trace">Trace</option>
            </select>
          </div>

          <div class="form-control">
            <label class="label"><span class="label-text">截图方式</span></label>
            <select v-model="capMethodType" class="select select-bordered w-full">
              <option value="Window">Window</option>
              <option value="Adb">Adb</option>
            </select>
          </div>
          
           <div class="form-control" v-if="capMethodType === 'Window'">
            <label class="label"><span class="label-text">窗口名称</span></label>
            <input type="text" v-model="capMethodValue" class="input input-bordered w-full" placeholder="MuMu安卓设备"/>
          </div>
          <div class="form-control" v-if="capMethodType === 'Adb'">
            <label class="label"><span class="label-text">ADB设备名称</span></label>
            <input type="text" v-model="capMethodValue" class="input input-bordered w-full" placeholder="emulator-5554" />
          </div>

          <div class="divider md:col-span-2 font-bold text-sm">高级</div>

          <!-- <div class="form-control">
            <label class="label"><span class="label-text">图片压缩</span></label>
             <select v-model="form.imageCompression" class="select select-bordered w-full">
              <option value="WindowOriginal">Window Original</option>
              <option value="Jpg">JPG</option>
              <option value="Png">PNG</option>
            </select>
          </div> -->

          <div class="form-control">
             <label class="label"><span class="label-text">IP (Optional)</span></label>
             <input type="text" v-model="adbIp" class="input input-bordered w-full" placeholder="127.0.0.1" />
          </div>
           <div class="form-control">
             <label class="label"><span class="label-text">ADB端口 (Optional)</span></label>
             <input type="number" v-model.number="adbPort" class="input input-bordered w-full" placeholder="5555" />
          </div>

          <div class="form-control md:col-span-2">
            <label class="label cursor-pointer justify-start gap-4">
              <span class="label-text">启用</span>
              <input type="checkbox" v-model="form.enable" class="checkbox" />
            </label>
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

<script setup>
import { ref, onMounted, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Plus } from 'lucide-vue-next';

const devices = ref([]);
const isEditing = ref(false);
const capMethodType = ref('Window');
const capMethodValue = ref('');

const adbIp = ref('');
const adbPort = ref(null);

const form = reactive({
  deviceId: '',
  deviceName: '',
  cores: 4,
  logLevel: 'Off',
  capMethod: null,
  imageCompression: 'WindowOriginal',
  enable: true,
  exePath: null,
  exeArgs: null,
  adbInfo: null
});

const loadDevices = async () => {
  try {
    const res = await invoke('get_all_devices_cmd');
    console.log('Loaded devices:', res);
    devices.value = Object.values(res);
  } catch (e) {
    console.error('Failed to load devices:', e);
  }
};

const openModal = (device = null) => {
  if (device) {
    isEditing.value = true;
    Object.assign(form, JSON.parse(JSON.stringify(device)));
    
    // Parse capMethod
    if (device.capMethod) {
        if (device.capMethod.window) {
            capMethodType.value = 'Window';
            capMethodValue.value = device.capMethod.window;
        } else if (device.capMethod.adb) {
            capMethodType.value = 'Adb';
            capMethodValue.value = device.capMethod.adb;
        } else if (typeof device.capMethod === 'string') {
             // Handle if it comes as string
        } else {
             const key = Object.keys(device.capMethod)[0];
             capMethodType.value = key; 
             capMethodValue.value = device.capMethod[key];
        }
    }
    
    // Parse adbInfo
    if (device.adbInfo) {
        adbIp.value = device.adbInfo.ipAddr || device.adbInfo.ip_addr || ''; 
        adbPort.value = device.adbInfo.port;
    } else {
        adbIp.value = '';
        adbPort.value = null;
    }

  } else {
    isEditing.value = false;
    // Generate UUID v7 compatible ID (mocking it with v4 for now, ideally backend generates)
    // But since we are sending the whole config, we need an ID.
    // Let's use a placeholder or ask backend to generate. 
    // For now, random UUID.
    form.deviceId = crypto.randomUUID(); 
    form.deviceName = null;
    form.cores = null;
    form.logLevel = 'Off';
    form.enable = true;
    form.imageCompression = 'WindowOriginal';
    capMethodType.value = 'Window';
    capMethodValue.value = '';
    adbIp.value = '';
    adbPort.value = null;
  }
  document.getElementById('device_modal').showModal();
};

const saveDevice = async () => {
  try {
    const method = {};
    method[capMethodType.value] = capMethodValue.value;
    form.capMethod = method;

    if (adbIp.value && adbPort.value) {
        form.adbInfo = {
            ip_addr: adbIp.value,
            port: adbPort.value,
            states: 'Disconnect'
        };
    } else {
        form.adbInfo = null;
    }

    await invoke('save_device_cmd', { device: form });
    document.getElementById('device_modal').close();
    await loadDevices();
  } catch (e) {
    console.error('Failed to save device:', e);
    alert('Failed to save: ' + e);
  }
};

const deleteDevice = async (id) => {
  if (!confirm('Are you sure?')) return;
  try {
    await invoke('delete_device_cmd', { deviceId: id });
    await loadDevices();
  } catch (e) {
    console.error('Failed to delete device:', e);
  }
};

const toggleEnable = async (device) => {
    device.enable = !device.enable;
    try {
        await invoke('save_device_cmd', { device });
    } catch(e) {
        console.error(e);
        device.enable = !device.enable; // revert
    }
}

onMounted(() => {
  loadDevices();
});
</script>
