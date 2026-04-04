<script setup lang="ts">
defineProps({
  size: {
    type: [Number, String],
    default: 20
  },
  color: {
    type: String,
    default: 'currentColor'
  }
})
</script>

<template>
  <svg :width="size" :height="size" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
    <rect x="4" y="4" width="24" height="24" rx="6" :stroke="color" stroke-width="2.5" />
    <path d="M10 16L14 20L22 12" :stroke="color" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" />
    <!-- decorative elements for the logo -->
    <path d="M6 10H8" :stroke="color" stroke-width="2" stroke-linecap="round" />
    <path d="M6 14H8" :stroke="color" stroke-width="2" stroke-linecap="round" />
  </svg>
</template>
