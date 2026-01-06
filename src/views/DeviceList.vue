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
              <span v-if="device.data.adbInfo" class="badge badge-ghost badge-sm">{{device.data.adbInfo.ipAddr}}:{{device.data.adbInfo.port}}</span>
            </td>
            <td>{{ device.data.cores?.join(',') || 'None' }}</td>
            <td>{{ device.data.logLevel }}</td>
            <td>
              <input type="checkbox" class="toggle toggle-sm toggle-success" :checked="device.data.enable" @click="toggleEnable(device)" />
            </td>
            <td>
              <button class="btn btn-ghost btn-xs" @click="openModal(device)">编辑</button>
              <button class="btn btn-ghost btn-xs text-error" @click="deleteDevice(device.id, device.data.deviceName)">删除</button>
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
            <input type="text" v-model="form.data.deviceName" class="input input-bordered w-full" placeholder="MuMu12"/>
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
          
          <div class="form-control md:col-span-2">
            <label class="label"><span class="label-text">CPU核心 (多选)</span></label>
            <div class="flex flex-wrap gap-2 p-2 bg-base-200 rounded-lg">
              <label v-for="i in cpuCount" :key="i-1" class="label cursor-pointer flex gap-1 bg-base-100 px-2 py-1 rounded border border-base-300 hover:bg-base-300 transition-colors">
                <input type="checkbox" :value="i-1" v-model="form.data.cores" class="checkbox checkbox-xs" />
                <span class="label-text text-xs">Core {{ i-1 }}</span>
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
            <input type="text" v-model="capMethodValue" class="input input-bordered w-full" placeholder="MuMu安卓设备"/>
          </div>
          <div class="form-control" v-if="capMethodType === 'adb'">
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
             <label class="label"><span class="label-text">IP</span></label>
             <input type="text" v-model="adbIp" class="input input-bordered w-full" placeholder="127.0.0.1" />
          </div>
           <div class="form-control">
             <label class="label"><span class="label-text">端口</span></label>
             <input type="number" v-model.number="adbPort" class="input input-bordered w-full" placeholder="5555" />
          </div>

          <div class="form-control md:col-span-2">
            <label class="label cursor-pointer justify-start gap-4">
              <span class="label-text">启用</span>
              <input type="checkbox" v-model="form.data.enable" class="checkbox" />
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
import { confirm, message } from '@tauri-apps/plugin-dialog';
import { Plus } from 'lucide-vue-next';

import { useDevices } from '../assets/js/useDevices.js'

const { 
  getAllDevices, 
  saveDevice: apiSaveDevice, 
  deleteDevice: apiDeleteDevice, 
  getUuidV7, 
  getCpuCount 
} = useDevices();

const devices = ref([]);
const isEditing = ref(false);
const capMethodType = ref('window');
const capMethodValue = ref('');
const cpuCount = ref(0);

const adbIp = ref('');
const adbPort = ref(null);

const form = reactive({
  id:'',
  data:{
    deviceName: '',
    cores: [],
    logLevel: 'Off',
    capMethod: null,
    imageCompression: 'WindowOriginal',
    enable: true,
    exePath: null,
    exeArgs: null,
    adbInfo: null
  }
});

const loadDevices = async () => {
  try {
    devices.value = await getAllDevices();
  } catch (e) {
    await message('加载设备失败: ' + e, { title: '错误', type: 'error' });
  }
};

const openModal = (device = null) => {
  if (device) {
    isEditing.value = true;
    // Deep clone the device object
    const cloned = JSON.parse(JSON.stringify(device));
    form.id = cloned.id;
    form.data = cloned.data;
    
    // Parse capMethod
    if (form.data.capMethod) {
        if (form.data.capMethod.window) {
            capMethodType.value = 'window';
            capMethodValue.value = form.data.capMethod.window;
        } else if (form.data.capMethod.adb) {
            capMethodType.value = 'adb';
            capMethodValue.value = form.data.capMethod.adb;
        }
    }
    
    // Parse adbInfo
    if (form.data.adbInfo) {
        adbIp.value = form.data.adbInfo.ipAddr || '';
        adbPort.value = form.data.adbInfo.port;
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
    form.id = null;
    form.data = {
      deviceName: '',
      cores: [],
      logLevel: 'Off',
      capMethod: null,
      imageCompression: 'WindowOriginal',
      enable: true,
      exePath: null,
      exeArgs: null,
      adbInfo: null
    };
    capMethodType.value = 'window';
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
    form.data.capMethod = method;

    if (adbIp.value && adbPort.value) {
        form.data.adbInfo = {
            ipAddr: adbIp.value,
            port: adbPort.value,
            states: 'disconnect'
        };
    } else {
        form.data.adbInfo = null;
    }
    if(!form.id){
      form.id = await getUuidV7();
    }
    await apiSaveDevice(form);
    document.getElementById('device_modal').close();
    await loadDevices();
  } catch (e) {
    await message('保存失败: ' + e, { title: '错误', type: 'error' });
  }
};

const deleteDevice = async (id, name) => {
  if (!await confirm('确定要删除【'+name+'】吗？', {title: '删除设备', kind: 'warning'})) return;
  try {
    await apiDeleteDevice(id);
    await loadDevices();
  } catch (e) {
    await message('删除失败: ' + e, { title: '错误', type: 'error' });
  }
};

const toggleEnable = async (device) => {
    device.data.enable = !device.data.enable;
    try {
        await apiSaveDevice(device);
    } catch(e) {
      await message('保存失败: ' + e, { title: '错误', type: 'error' });
      device.data.enable = !device.data.enable; // revert
    }
}

onMounted(async () => {
  await loadDevices();
  try {
    cpuCount.value = await getCpuCount();
  } catch (e) {
    await message('获取CPU核心数失败: ' + e, { title: '错误', type: 'error' });
  }
});
</script>
