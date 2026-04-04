<script setup lang="ts">
import { computed } from 'vue';
import * as icons from 'lucide-vue-next';
import { SvgLogo, SvgNode, SvgStatus } from '@/components/shared/svg';

const props = defineProps({
  name: {
    type: String,
    required: true,
  },
  size: {
    type: [Number, String],
    default: 20,
  },
  strokeWidth: {
    type: [Number, String],
    default: 2,
  },
  color: {
    type: String,
    default: 'currentColor',
  },
  type: {
    type: String,
    default: 'lucide', // 'lucide' | 'custom'
    validator: (value: string) => ['lucide', 'custom'].includes(value),
  },
});

// Resolve the Lucide icon component if type is 'lucide'
const resolvedLucideIcon = computed(() => {
  if (props.type !== 'lucide') return null;
  const formattedName = props.name
    .split('-')
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join('');
  // @ts-ignore
  return icons[formattedName] || null;
});

// Resolve the Custom SVG component if type is 'custom'
const resolvedCustomIcon = computed(() => {
  if (props.type !== 'custom') return null;
  if (props.name === 'logo') return SvgLogo;
  if (props.name.startsWith('node-')) return SvgNode;
  if (props.name.startsWith('status-')) return SvgStatus;
  return null;
});
</script>

<template>
  <component
    v-if="type === 'lucide' && resolvedLucideIcon"
    :is="resolvedLucideIcon"
    :size="size"
    :stroke-width="strokeWidth"
    :color="color"
    class="app-icon transition-colors duration-200"
  />
  <component
    v-else-if="type === 'custom' && resolvedCustomIcon"
    :is="resolvedCustomIcon"
    :name="name"
    :size="size"
    :color="color"
    class="app-icon custom-svg-icon transition-colors duration-200"
  />
  <!-- Fallback empty icon if not found -->
  <span
    v-else
    class="inline-flex items-center justify-center opacity-50"
    :style="{ width: `${size}px`, height: `${size}px` }"
  >
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      :stroke-width="strokeWidth"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <circle cx="12" cy="12" r="10"></circle>
      <line x1="12" y1="8" x2="12" y2="12"></line>
      <line x1="12" y1="16" x2="12.01" y2="16"></line>
    </svg>
  </span>
</template>

<style scoped>
.app-icon {
  display: inline-flex;
  flex-shrink: 0;
}
</style>
