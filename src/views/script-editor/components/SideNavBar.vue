<template>
  <div class="w-16 flex flex-col items-center py-4 bg-base-300 border-r border-base-300 gap-2 shadow-inner z-20">
    <div v-for="item in navItems" :key="item.id" class="tooltip tooltip-right" :data-tip="item.label">
      <button
        class="group w-12 h-12 rounded-xl flex items-center justify-center transition-all duration-200 relative"
        :class="[
          modelValue === item.id
            ? 'bg-primary text-primary-content shadow-lg scale-105'
            : 'text-base-content/60 hover:bg-base-100 hover:text-primary',
        ]"
        @click="$emit('update:modelValue', item.id)"
      >
        <component :is="item.icon" class="w-6 h-6" />

        <!-- Active indicator -->
        <span v-if="modelValue === item.id" class="absolute left-0 w-1 h-6 bg-primary-content rounded-r-full"></span>
      </button>
    </div>

    <div class="mt-auto opacity-20 hover:opacity-100 transition-opacity">
      <div class="w-8 h-px bg-base-content/20 mb-4 mx-auto"></div>
      <button class="btn btn-ghost btn-circle btn-sm">
        <Settings class="w-5 h-5" />
      </button>
    </div>
  </div>
</template>

<script setup>
import { SquareStack, Library, LayoutGrid, ListTodo, Settings } from 'lucide-vue-next';

const props = defineProps({
  modelValue: {
    type: String,
    required: true,
  },
});

defineEmits(['update:modelValue']);

const navItems = [
  { id: 'task', label: '任务', icon: SquareStack },
  { id: 'policy_set', label: '策略集', icon: Library },
  { id: 'policy_group', label: '策略组', icon: LayoutGrid },
  { id: 'policy', label: '策略', icon: ListTodo },
];
</script>
