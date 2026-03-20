<template>
  <div class="p-6 relative min-h-full">
    <!-- Header -->
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold">任务管理</h1>
      <div class="flex gap-2">
        <button class="btn btn-sm btn-primary" @click="startAll">全部开始</button>
        <button class="btn btn-sm btn-warning" @click="pauseAll">全部暂停</button>
        <button class="btn btn-sm btn-error text-white" @click="shutdownAll">全部停止</button>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="flex justify-center py-20">
      <span class="loading loading-spinner loading-lg text-primary"></span>
    </div>

    <!-- Empty State -->
    <div v-else-if="deviceList.length === 0" class="flex flex-col items-center justify-center py-20 opacity-40">
      <MonitorSmartphone class="w-16 h-16 mb-4" />
      <p class="text-lg font-medium">暂无设备</p>
      <p class="text-sm">请先在"设备列表"页面添加设备</p>
    </div>

    <!-- Device Grid -->
    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6 pb-20 h-[calc(100vh-9rem)]">
      <div
        v-for="device in deviceList"
        :key="device.id"
        class="card bg-base-100 shadow-xl border border-base-300 transition-all duration-300 h-full"
        :class="{ 'opacity-60 grayscale': !device.data?.enable }"
      >
        <div class="card-body p-4 h-full flex flex-col">
          <!-- Card Header: Title & Toggle -->
          <div class="flex justify-between items-center mb-2">
            <div class="flex items-center gap-2">
              <div :class="`w-3 h-3 rounded-full ${device.data?.enable ? 'bg-success' : 'bg-base-300'}`"></div>
              <h2 class="card-title text-base">{{ device.data?.deviceName || '未命名设备' }}</h2>
            </div>
          </div>

          <!-- Control Buttons -->
          <div class="flex gap-2 mb-4">
            <button class="btn btn-sm btn-primary flex-1" @click="startDevice(device.id)" title="启动">
              <Play class="w-4 h-4" />
            </button>
            <button class="btn btn-sm btn-warning flex-1" @click="pauseDevice(device.id)" title="暂停">
              <Pause class="w-4 h-4" />
            </button>
            <button class="btn btn-sm btn-error flex-1" @click="shutdownDevice(device.id)" title="停止">
              <Square class="w-4 h-4" />
            </button>
          </div>

          <!-- Task Queue -->
          <div class="bg-base-200 rounded-lg p-2 flex-1 flex flex-col">
            <div class="flex justify-between items-center mb-2">
              <span class="text-xs font-bold opacity-70">脚本队列</span>
              <div class="flex items-center gap-1">
                <span class="badge badge-sm badge-neutral">{{ (deviceAssignments[device.id] || []).length }}</span>
                <button
                  class="btn btn-xs btn-circle btn-primary"
                  @click="openAddScriptModal(device.id)"
                  title="添加脚本"
                >
                  <Plus class="w-3 h-3" />
                </button>
              </div>
            </div>

            <div class="space-y-2 flex-1 overflow-y-auto pr-1 custom-scrollbar">
              <div
                v-for="(assignment, idx) in (deviceAssignments[device.id] || [])"
                :key="assignment.id"
                class="bg-base-100 p-2 rounded text-xs flex items-center gap-2 group"
              >
                <span class="opacity-30 font-mono w-4 text-right shrink-0">{{ idx + 1 }}</span>
                <span class="flex-1 truncate font-medium">{{ getScriptName(assignment.scriptId) }}</span>
                <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button
                    class="btn btn-xs btn-circle btn-ghost text-error"
                    @click="removeAssignment(device.id, assignment.id)"
                    title="移除"
                  >
                    <X class="w-3 h-3" />
                  </button>
                </div>
              </div>

              <!-- 空闲显示 -->
              <div
                v-if="(deviceAssignments[device.id] || []).length === 0"
                class="text-center text-xs opacity-50 py-4"
              >
                空闲 — 点击 + 添加脚本
              </div>
            </div>
          </div>

          <!-- Expandable Schedule Records -->
          <div class="mt-2">
            <button
              class="btn btn-xs btn-ghost w-full gap-1 opacity-60 hover:opacity-100"
              @click="toggleScheduleRecords(device.id)"
            >
              <ChevronDown
                class="w-3 h-3 transition-transform"
                :class="{ 'rotate-180': expandedSchedule === device.id }"
              />
              <span>运行记录</span>
            </button>

            <div v-if="expandedSchedule === device.id" class="mt-2 space-y-1 max-h-48 overflow-y-auto custom-scrollbar">
              <div v-if="loadingSchedules" class="flex justify-center py-4">
                <span class="loading loading-spinner loading-sm"></span>
              </div>
              <template v-else>
                <div
                  v-for="record in deviceSchedules"
                  :key="record.id"
                  class="bg-base-200/50 p-2 rounded text-[10px] flex items-center gap-2"
                >
                  <div
                    class="w-2 h-2 rounded-full shrink-0"
                    :class="{
                      'bg-success': record.status === 'success',
                      'bg-error': record.status === 'failed',
                      'bg-warning': record.status === 'skipped',
                    }"
                  ></div>
                  <span class="flex-1 truncate">{{ getScriptName(record.scriptId) }}</span>
                  <span class="opacity-50">{{ formatTime(record.startedAt) }}</span>
                </div>
                <div v-if="deviceSchedules.length === 0" class="text-center text-xs opacity-50 py-2">暂无记录</div>
                <button
                  v-if="deviceSchedules.length > 0"
                  class="btn btn-xs btn-ghost btn-error w-full mt-1"
                  @click="clearDeviceSchedules(device.id)"
                >
                  清除全部记录
                </button>
              </template>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Add Script Modal -->
    <dialog class="modal" :class="{ 'modal-open': showAddScriptModal }">
      <div class="modal-box">
        <h3 class="font-bold text-lg mb-4">添加脚本到设备</h3>
        <div class="space-y-2 max-h-64 overflow-y-auto custom-scrollbar">
          <div
            v-for="script in scriptList"
            :key="script.id"
            class="flex items-center gap-3 p-3 rounded-lg cursor-pointer hover:bg-primary/10 transition-colors border border-base-300"
            @click="addScriptToDevice(script.id)"
          >
            <div class="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center">
              <Package class="w-4 h-4 text-primary" />
            </div>
            <div class="flex-1 min-w-0">
              <p class="font-medium text-sm truncate">{{ script.data.name }}</p>
              <p class="text-xs opacity-50 truncate">{{ script.data.description || '暂无描述' }}</p>
            </div>
          </div>
          <div v-if="scriptList.length === 0" class="text-center py-8 opacity-50 text-sm">暂无可用脚本</div>
        </div>
        <div class="modal-action">
          <button class="btn" @click="showAddScriptModal = false">取消</button>
        </div>
      </div>
      <form method="dialog" class="modal-backdrop"><button @click="showAddScriptModal = false">close</button></form>
    </dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue';
import {
  Play,
  Square,
  Pause,
  Plus,
  X,
  ChevronDown,
  MonitorSmartphone,
  Package,
} from 'lucide-vue-next';
import { invoke } from '@tauri-apps/api/core';
import { useDevices } from '@/assets/js/useDevices';
import { useAssignments } from '@/assets/js/useAssignments';
import type { DeviceTable, ScriptTable } from '@/types/bindings';
import type { DeviceScriptAssignment, DeviceScriptSchedule } from '@/assets/js/useAssignments';

const { getAllDevices, getUuidV7 } = useDevices();
const {
  getByDevice,
  save: saveAssignment,
  remove: removeAssignmentApi,
  clearSchedules: clearSchedulesApi,
  getSchedulesByDevice,
} = useAssignments();

const loading = ref(true);
const deviceList = ref<DeviceTable[]>([]);
const scriptList = ref<ScriptTable[]>([]);
const deviceAssignments = reactive<Record<string, DeviceScriptAssignment[]>>({});

// Add Script Modal
const showAddScriptModal = ref(false);
const addScriptTargetDeviceId = ref('');

// Schedule Records
const expandedSchedule = ref<string | null>(null);
const deviceSchedules = ref<DeviceScriptSchedule[]>([]);
const loadingSchedules = ref(false);

// Script name cache
const scriptNameMap = ref<Record<string, string>>({});

const getScriptName = (scriptId: string): string => {
  return scriptNameMap.value[scriptId] || '未知脚本';
};

const formatTime = (time: string): string => {
  try {
    const d = new Date(time);
    if (isNaN(d.getTime())) return time;
    const MM = String(d.getMonth() + 1).padStart(2, '0');
    const dd = String(d.getDate()).padStart(2, '0');
    const HH = String(d.getHours()).padStart(2, '0');
    const mm = String(d.getMinutes()).padStart(2, '0');
    return `${MM}-${dd} ${HH}:${mm}`;
  } catch {
    return time;
  }
};

// ============================================
// 数据加载
// ============================================

const loadData = async () => {
  loading.value = true;
  try {
    // Load devices
    deviceList.value = await getAllDevices();

    // Load scripts for the name map
    scriptList.value = await invoke<ScriptTable[]>('get_all_scripts_cmd');
    for (const s of scriptList.value) {
      scriptNameMap.value[s.id] = s.data.name;
    }

    // Load assignments for each device
    for (const device of deviceList.value) {
      try {
        const assignments = await getByDevice(device.id);
        deviceAssignments[device.id] = assignments;
      } catch {
        deviceAssignments[device.id] = [];
      }
    }
  } catch (e) {
    console.error('加载数据失败:', e);
  } finally {
    loading.value = false;
  }
};

// ============================================
// 脚本分配
// ============================================

const openAddScriptModal = (deviceId: string) => {
  addScriptTargetDeviceId.value = deviceId;
  showAddScriptModal.value = true;
};

const addScriptToDevice = async (scriptId: string) => {
  const deviceId = addScriptTargetDeviceId.value;
  const currentAssignments = deviceAssignments[deviceId] || [];
  const id = await getUuidV7();

  const newAssignment: DeviceScriptAssignment = {
    id,
    deviceId,
    scriptId,
    timeTemplateId: null,
    accountData: {},
    index: currentAssignments.length,
  };

  try {
    await saveAssignment(newAssignment);
    currentAssignments.push(newAssignment);
    deviceAssignments[deviceId] = [...currentAssignments];
    showAddScriptModal.value = false;
  } catch (e) {
    console.error('添加脚本失败:', e);
  }
};

const removeAssignment = async (deviceId: string, assignmentId: string) => {
  try {
    await removeAssignmentApi(assignmentId);
    const list = deviceAssignments[deviceId] || [];
    deviceAssignments[deviceId] = list.filter((a) => a.id !== assignmentId);
  } catch (e) {
    console.error('移除脚本失败:', e);
  }
};

// ============================================
// Schedule Records
// ============================================

const toggleScheduleRecords = async (deviceId: string) => {
  if (expandedSchedule.value === deviceId) {
    expandedSchedule.value = null;
    return;
  }

  expandedSchedule.value = deviceId;
  loadingSchedules.value = true;
  try {
    deviceSchedules.value = await getSchedulesByDevice(deviceId);
  } catch {
    deviceSchedules.value = [];
  } finally {
    loadingSchedules.value = false;
  }
};

const clearDeviceSchedules = async (deviceId: string) => {
  try {
    await clearSchedulesApi(deviceId);
    deviceSchedules.value = [];
  } catch (e) {
    console.error('清除记录失败:', e);
  }
};

// ============================================
// 设备控制
// ============================================

const startDevice = async (deviceId: string) => {
  try {
    await invoke('cmd_device_start', { deviceId });
  } catch (e) {
    console.error('启动失败:', e);
  }
};

const pauseDevice = async (deviceId: string) => {
  try {
    await invoke('cmd_device_pause', { deviceId });
  } catch (e) {
    console.error('暂停失败:', e);
  }
};

const shutdownDevice = async (deviceId: string) => {
  try {
    await invoke('cmd_device_shutdown', { deviceId });
  } catch (e) {
    console.error('停止失败:', e);
  }
};

const startAll = async () => {
  for (const device of deviceList.value) {
    if (device.data?.enable) {
      await startDevice(device.id);
    }
  }
};

const pauseAll = async () => {
  for (const device of deviceList.value) {
    await pauseDevice(device.id);
  }
};

const shutdownAll = async () => {
  for (const device of deviceList.value) {
    await shutdownDevice(device.id);
  }
};

// ============================================
// 初始化
// ============================================

onMounted(async () => {
  await loadData();
});
</script>
