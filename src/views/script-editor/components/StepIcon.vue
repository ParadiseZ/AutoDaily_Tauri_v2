<template>
  <IconRenderer :icon="iconName" :class="className" />
</template>

<script setup lang="ts">
import { computed } from 'vue';
import IconRenderer from '../IconRenderer.vue';

const props = defineProps<{
  type?: string;
  category?: string;
  className?: string;
}>();

const iconName = computed(() => {
  const typeMap: Record<string, string> = {
    // Interaction
    ClickAction: 'cursor',
    SwipePoint: 'move',
    SwipePercent: 'move',
    WaitMs: 'clock',

    // Vision
    VisionSearch: 'zap',
    Ocr: 'type',
    FindObject: 'layers', // Box mapping to layers or default
    DetRec: 'target',
    TakeScreenshot: 'camera',

    // Control
    If: 'branch',
    While: 'repeat',
    Sequence: 'layers', // LayoutGrid mapping to layers
    ForEachActivity: 'repeat',
    Continue: 'play',
    Break: 'square',
    WaitUntil: 'clock',

    // Logic
    SetVar: 'variable',
    GetVar: 'terminal', // Assuming terminal exists or default
    FilterHits: 'filter',
    IncIndex: 'arrow-right',
    ResetIndex: 'repeat',

    // State
    SetState: 'settings',
    GetState: 'info',
    StopPolicy: 'eye-off',
    FinishTask: 'task',
  };

  const lowerType = props.type?.toLowerCase() || '';
  const legacyMap: Record<string, string> = {
    click: 'cursor',
    swipe: 'move',
    wait: 'clock',
  };

  return (props.type ? typeMap[props.type] : null) || legacyMap[lowerType] || 'box';
});
</script>
