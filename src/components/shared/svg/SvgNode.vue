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
  <svg :width="size" :height="size" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
    <!-- Outer glow loop -->
    <path d="M12 2C6.47715 2 2 6.47715 2 12C2 17.5228 6.47715 22 12 22C17.5228 22 22 17.5228 22 12C22 6.47715 17.5228 2 12 2Z" 
      :stroke="color" stroke-width="1.5" stroke-dasharray="4 4" class="origin-center animate-[spin_8s_linear_infinite]" />
    
    <!-- Central Node -->
    <rect x="8" y="8" width="8" height="8" rx="2" :stroke="color" stroke-width="2" :fill="color" fill-opacity="0.2" />
    
    <!-- Connections -->
    <path d="M12 4L12 8" :stroke="color" stroke-width="2" stroke-linecap="round" />
    <path d="M12 16L12 20" :stroke="color" stroke-width="2" stroke-linecap="round" />
    <path d="M4 12L8 12" :stroke="color" stroke-width="2" stroke-linecap="round" />
    <path d="M16 12L20 12" :stroke="color" stroke-width="2" stroke-linecap="round" />
  </svg>
</template>
