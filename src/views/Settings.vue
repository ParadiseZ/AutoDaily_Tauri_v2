<template>
  <div class="p-6">
    <h1 class="text-2xl font-bold mb-6">System Settings</h1>
    
    <div class="card bg-base-100 shadow-xl border border-base-300 max-w-2xl">
      <div class="card-body">
        <h2 class="card-title mb-4">Appearance</h2>
        
        <div class="form-control">
          <label class="label">
            <span class="label-text font-bold">Theme</span>
          </label>
          <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
            <button 
              v-for="theme in themes" 
              :key="theme"
              class="btn btn-outline btn-block justify-start capitalize"
              :class="{ 'btn-active': currentTheme === theme }"
              @click="setTheme(theme)"
            >
              <div class="flex gap-2 items-center w-full">
                <div class="w-4 h-4 rounded-full bg-primary" :data-theme="theme"></div>
                <div class="w-4 h-4 rounded-full bg-secondary" :data-theme="theme"></div>
                <span class="flex-1 text-left">{{ theme }}</span>
                <Check v-if="currentTheme === theme" class="w-4 h-4" />
              </div>
            </button>
          </div>
        </div>

        <div class="divider"></div>

        <h2 class="card-title mb-4">General</h2>
        <div class="form-control w-full">
          <label class="label cursor-pointer justify-start gap-4">
            <span class="label-text">Start on Boot</span> 
            <input type="checkbox" class="toggle toggle-primary" checked />
          </label>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { Check } from 'lucide-vue-next';

const themes = [
  'light',
  'dark',
  'cupcake',
  'bumblebee',
  'emerald',
  'corporate',
  'synthwave',
  'retro',
  'cyberpunk',
  'valentine',
  'halloween',
  'garden',
  'forest',
  'aqua',
  'lofi',
  'pastel',
  'fantasy',
  'wireframe',
  'black',
  'luxury',
  'dracula',
  'cmyk',
  'autumn',
  'business',
  'acid',
  'lemonade',
  'night',
  'coffee',
  'winter',
  'dim',
  'nord',
  'sunset',
];

const currentTheme = ref('dark');

const setTheme = (theme) => {
  currentTheme.value = theme;
  document.documentElement.setAttribute('data-theme', theme);
  localStorage.setItem('theme', theme);
};

onMounted(() => {
  const savedTheme = localStorage.getItem('theme') || 'dark';
  setTheme(savedTheme);
});
</script>
