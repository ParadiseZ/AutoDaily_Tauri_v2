<template>
  <div class="editor-overview-panel custom-scrollbar">
    <slot />
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'EditorOverviewPanel' });
</script>

<style scoped>
@reference "../../style.css";

.editor-overview-panel {
  @apply min-h-0 flex-1 space-y-2 overflow-y-auto pr-1;
}
</style>
