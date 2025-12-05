<template>
  <div class="p-6 relative min-h-full">
    <!-- Header -->
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold">设备管理</h1>
      <div class="flex gap-2">
        <button class="btn btn-sm btn-primary">全部开始</button>
        <button class="btn btn-sm btn-warning">全部暂停</button>
        <button class="btn btn-sm btn-error text-white">全部取消</button>
      </div>
    </div>

    <!-- Device Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6 pb-20">
      <div v-for="device in devices" :key="device.id" 
           class="card bg-base-100 shadow-xl border border-base-300 transition-all duration-300"
           :class="{ 'opacity-60 grayscale': !device.enabled }">
        <div class="card-body p-4">
          <!-- Card Header: Title & Toggle -->
          <div class="flex justify-between items-center mb-2">
            <div class="flex items-center gap-2">
              <div :class="`w-3 h-3 rounded-full ${device.online ? 'bg-success' : 'bg-error'}`"></div>
              <h2 class="card-title text-base">{{ device.name }}</h2>
            </div>
            <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="device.enabled" />
          </div>

          <!-- Control Buttons (Top of Body) -->
          <div class="flex gap-2 mb-4">
            <button class="btn btn-sm btn-primary flex-1">
              <Play class="w-4 h-4" />
            </button>
            <button class="btn btn-sm btn-warning flex-1">
              <Pause class="w-4 h-4" />
            </button>
            <button class="btn btn-sm btn-outline flex-1">
              <Settings class="w-4 h-4" />
            </button>
          </div>

          <!-- Device Info -->
          <div class="flex justify-between items-end mb-4">
            <div class="text-xs opacity-70 space-y-1">
              <p>IP: {{ device.ip }}</p>
              <p>状态: {{ device.status }}</p>
            </div>
            <button class="btn btn-xs btn-outline btn-primary">
              <Plus class="w-3 h-3" />
            </button>
          </div>

          <!-- Task Queue -->
          <div class="bg-base-200 rounded-lg p-2">
            <div class="flex justify-between items-center mb-2">
              <span class="text-xs font-bold opacity-70">任务队列</span>
              <span class="badge badge-sm badge-neutral">{{ device.queue.length }}</span>
            </div>
            <div class="space-y-2 max-h-[40vh] overflow-y-auto pr-1 custom-scrollbar">
              <!-- 当前任务 -->
              <div v-if="device.currentTask" class="alert alert-success py-2 px-3 text-xs flex justify-start gap-2 rounded-md">
                <Play class="w-3 h-3" />
                <div class="flex-1">
                  <div class="font-bold">{{ device.currentTask.name }}</div>
                  <div class="opacity-70">运行中: {{ device.currentTask.action }}</div>
                </div>
                <div class="flex gap-1">
                   <button class="btn btn-xs btn-circle btn-ghost" title="暂停"><Pause class="w-3 h-3" /></button>
                   <button class="btn btn-xs btn-circle btn-ghost" title="取消"><X class="w-3 h-3" /></button>
                </div>
              </div>
              <!-- 任务队列 -->
              <div v-for="task in device.queue" :key="task.id" class="bg-base-100 p-2 rounded text-xs flex items-center gap-2 opacity-80 group">
                <Hourglass class="w-3 h-3" />
                <span class="flex-1">{{ task.name }}</span>
                <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                   <button class="btn btn-xs btn-circle btn-ghost" title="暂停"><Pause class="w-3 h-3" /></button>
                   <button class="btn btn-xs btn-circle btn-ghost" title="取消"><X class="w-3 h-3" /></button>
                </div>
              </div>
              <!-- 空闲显示 -->
              <div v-if="!device.currentTask && device.queue.length === 0" class="text-center text-xs opacity-50 py-2">
                空闲
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- FAB: Run All -->
    <div class="fixed bottom-8 right-8">
      <button class="btn btn-circle btn-primary btn-lg shadow-lg" @click="runAll">
        <Play class="w-8 h-8" />
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import { Play, Square, Hourglass, Pause, Plus, X, Settings } from 'lucide-vue-next';

const devices = ref([
  {
    id: 1,
    name: 'Pixel 6 Pro',
    ip: '192.168.1.101',
    status: 'Connected',
    online: true,
    enabled: false,
    running: false,
    currentTask: { name: 'Game A - Daily', action: 'Sign-in' },
    queue: [
      { id: 1, name: 'Dungeon Run' },
      { id: 2, name: 'Mail Collect' },
      { id: 3, name: 'Reward Claim' },
      { id: 4, name: 'Reward Claim' },
      { id: 5, name: 'Reward Claim' },
      { id: 6, name: 'Reward Claim' },
      { id: 7, name: 'Reward Claim' },
      { id: 8, name: 'Reward Claim' },
      { id: 9, name: 'Reward Claim' },
      { id: 10, name: 'Reward Claim' },
      { id: 11, name: 'Reward Claim' },



      { id: 12, name: 'Reward Claim' },
      { id: 13, name: 'Reward Claim' },
      { id: 14, name: 'Reward Claim' },
      { id: 15, name: 'Reward Claim' },
      { id: 16, name: 'Reward Claim' },
      { id: 17, name: 'Reward Claim' },
      { id: 18, name: 'Reward Claim' },
      { id: 19, name: 'Reward Claim' },
      { id: 20, name: 'Reward Claim' },
      { id: 21, name: 'Reward Claim' },
      { id: 22, name: 'Reward Claim' },
      { id: 23, name: 'Reward Claim' }
    ]
  },
  {
    id: 2,
    name: 'Emulator-5554',
    ip: 'localhost:5555',
    status: 'Connected',
    online: true,
    enabled: true,
    running: true,
    currentTask: null,
    queue: []
  },
  {
    id: 3,
    name: 'Xiaomi 13',
    ip: '192.168.1.105',
    status: 'Offline',
    online: false,
    enabled: false,
    running: false,
    currentTask: { name: 'Game B - Event', action: 'Boss Battle' },
    queue: [
      { id: 3, name: 'Reward Claim' }
    ]
  },
  {
    id: 4,
    name: 'Samsung S23',
    ip: '192.168.1.108',
    status: 'Connected',
    online: true,
    enabled: true,
    running: false,
    currentTask: null,
    queue: []
  }
]);

const runAll = () => {
  console.log('Running all enabled devices...');
};
</script>
