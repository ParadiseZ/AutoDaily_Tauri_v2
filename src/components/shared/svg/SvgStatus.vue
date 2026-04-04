<script setup lang="ts">
defineProps({
  size: {
    type: [Number, String],
    default: 20
  },
  color: {
    type: String,
    default: 'var(--app-vibrant-emerald)' // default success
  },
  name: {
    type: String, // e.g. 'status-online', 'status-offline', 'status-busy'
    default: 'status-online'
  }
})
</script>

<template>
  <svg :width="size" :height="size" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
    <template v-if="name === 'status-online' || name === 'status-active'">
      <!-- Glowing active dot -->
      <circle cx="12" cy="12" r="6" :fill="color" />
      <circle cx="12" cy="12" r="9" :stroke="color" stroke-width="2" stroke-opacity="0.4" class="origin-center animate-ping" />
      <circle cx="12" cy="12" r="11" :fill="color" fill-opacity="0.1" />
    </template>
    
    <template v-else-if="name === 'status-offline'">
      <!-- Offline sleep dot -->
      <circle cx="12" cy="12" r="5" stroke="var(--app-text-faint)" stroke-width="2" stroke-dasharray="2 2" />
      <path d="M9 15L15 9" stroke="var(--app-text-faint)" stroke-width="2" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'status-busy'">
      <!-- Busy processing dots -->
      <circle cx="7" cy="12" r="2.5" fill="var(--app-vibrant-amber)" class="animate-bounce" style="animation-delay: 0ms" />
      <circle cx="12" cy="12" r="2.5" fill="var(--app-vibrant-amber)" class="animate-bounce" style="animation-delay: 150ms" />
      <circle cx="17" cy="12" r="2.5" fill="var(--app-vibrant-amber)" class="animate-bounce" style="animation-delay: 300ms" />
      <rect x="2" y="5" width="20" height="14" rx="4" stroke="var(--app-vibrant-amber)" stroke-width="1.5" stroke-opacity="0.4" />
    </template>
  </svg>
</template>
