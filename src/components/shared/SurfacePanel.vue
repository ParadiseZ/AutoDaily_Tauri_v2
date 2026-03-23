<template>
  <component :is="tag" class="app-panel" :class="panelClasses">
    <slot />
  </component>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    tag?: string;
    tone?: 'default' | 'muted';
    padding?: 'sm' | 'md' | 'lg';
  }>(),
  {
    tag: 'section',
    tone: 'default',
    padding: 'md',
  },
);

const panelClasses = computed(() => [
  props.tone === 'muted' ? 'app-panel-muted' : '',
  props.padding === 'sm' ? 'p-4' : props.padding === 'lg' ? 'p-7' : 'p-5',
]);
</script>
