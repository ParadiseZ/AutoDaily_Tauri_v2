<template>
  <div class="p-6 relative min-h-full">
    <!-- Header -->
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold">Device Management</h1>
      <div class="flex gap-2">
        <!-- Filter/Sort controls could go here -->
      </div>
    </div>

    <!-- Device Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6 pb-20">
      <div v-for="device in devices" :key="device.id" class="card bg-base-100 shadow-xl border border-base-300">
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
            <button class="btn btn-sm btn-error flex-1" v-if="device.running">
              <Square class="w-4 h-4" /> Stop
            </button>
            <button class="btn btn-sm btn-primary flex-1" v-else>
              <Play class="w-4 h-4" /> Start
            </button>
            <button class="btn btn-sm btn-ghost border border-base-300 flex-1">
              Details
            </button>
          </div>

          <!-- Device Info -->
          <div class="text-xs opacity-70 mb-4 space-y-1">
            <p>IP: {{ device.ip }}</p>
            <p>Status: {{ device.status }}</p>
          </div>

          <!-- Task Queue -->
          <div class="bg-base-200 rounded-lg p-2">
            <div class="flex justify-between items-center mb-2">
              <span class="text-xs font-bold opacity-70">Task Queue</span>
              <span class="badge badge-sm badge-neutral">{{ device.queue.length }}</span>
            </div>
            <div class="space-y-2">
              <div v-if="device.currentTask" class="alert alert-success py-2 px-3 text-xs flex justify-start gap-2 rounded-md">
                <Play class="w-3 h-3" />
                <div>
                  <div class="font-bold">{{ device.currentTask.name }}</div>
                  <div class="opacity-70">Running: {{ device.currentTask.action }}</div>
                </div>
              </div>
              <div v-for="task in device.queue" :key="task.id" class="bg-base-100 p-2 rounded text-xs flex items-center gap-2 opacity-80">
                <Hourglass class="w-3 h-3" />
                {{ task.name }}
              </div>
              <div v-if="!device.currentTask && device.queue.length === 0" class="text-center text-xs opacity-50 py-2">
                Idle
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
import { Play, Square, Hourglass } from 'lucide-vue-next';

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
      { id: 2, name: 'Mail Collect' }
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
