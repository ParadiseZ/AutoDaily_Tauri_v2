<template>
  <div class="p-6">
    <h1 class="text-2xl font-bold mb-6">System Settings</h1>
    
    <div class="columns-1 md:columns-2 gap-6 space-y-6">
      <!-- Basic Settings Block -->
      <div class="card bg-base-100 shadow-xl border border-base-300 break-inside-avoid">
        <div class="card-body p-4">
          <h2 class="card-title text-lg mb-4">Basic Settings</h2>
          
          <div class="grid grid-cols-1 gap-4">
            <div class="flex justify-between items-center">
              <span class="font-medium">Language</span>
              <select class="select select-bordered select-sm w-40">
                <option>English</option>
                <option>Chinese</option>
              </select>
            </div>
            
            <div class="flex justify-between items-center">
              <span class="font-medium">Start Mode</span>
              <select class="select select-bordered select-sm w-40">
                <option>Normal</option>
                <option>Minimized</option>
                <option>Tray</option>
              </select>
            </div>

            <div class="flex justify-between items-center">
               <span class="font-medium">Start on Boot</span>
               <input type="checkbox" class="toggle toggle-primary toggle-sm" />
            </div>
             <div class="flex justify-between items-center">
               <span class="font-medium">Always on Top</span>
               <input type="checkbox" class="toggle toggle-primary toggle-sm" />
            </div>
          </div>
        </div>
      </div>

      <!-- Appearance Block -->
      <div class="card bg-base-100 shadow-xl border border-base-300 break-inside-avoid">
        <div class="card-body p-4">
          <h2 class="card-title text-lg mb-4">Appearance</h2>
          <div class="form-control">
            <label class="label">
              <span class="label-text font-bold">Theme</span>
            </label>
            <div class="grid grid-cols-2 gap-2">
               <button v-for="theme in visibleThemes" :key="theme" 
                 class="btn btn-xs btn-outline justify-start capitalize" 
                 :class="{ 'btn-active': currentTheme === theme }" 
                 @click="setTheme(theme)">
                 <div class="w-3 h-3 rounded-full bg-primary mr-2" :data-theme="theme"></div>
                 {{ theme }}
               </button>
            </div>
            <div class="text-xs text-center mt-3 opacity-50 cursor-pointer hover:opacity-100" @click="showAllThemes = !showAllThemes">
                {{ showAllThemes ? 'Show Less' : 'Show All (' + themes.length + ')' }}
            </div>
          </div>
        </div>
      </div>

      <!-- Performance Block -->
      <div class="card bg-base-100 shadow-xl border border-base-300 break-inside-avoid">
        <div class="card-body p-4">
           <h2 class="card-title text-lg mb-4">Performance</h2>
           <div class="flex justify-between items-center mb-2">
              <span class="font-medium">Max Concurrent Tasks</span>
              <input type="number" class="input input-bordered input-sm w-20" value="4" />
           </div>
           <div class="flex justify-between items-center">
              <span class="font-medium">GPU Acceleration</span>
              <input type="checkbox" class="toggle toggle-secondary toggle-sm" checked />
           </div>
        </div>
      </div>

       <!-- About Block -->
      <div class="card bg-base-100 shadow-xl border border-base-300 break-inside-avoid">
        <div class="card-body p-4">
           <h2 class="card-title text-lg mb-4">About</h2>
           <div class="text-sm opacity-70">
              <p>Version: 2.0.0 Alpha</p>
              <p>Build: 20251205</p>
              <p class="mt-2">AutoDaily is an automation tool designed for efficiency.</p>
           </div>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, computed } from 'vue';
import { Check } from 'lucide-vue-next';
import {
  THEMES,
  DEFAULT_VISIBLE_THEMES_COUNT,
  DEFAULT_THEME,
} from './script-editor/config.js';

// 使用统一配置
const themes = THEMES;

const currentTheme = ref(DEFAULT_THEME);
const showAllThemes = ref(false);

const visibleThemes = computed(() => {
    return showAllThemes.value ? themes : themes.slice(0, DEFAULT_VISIBLE_THEMES_COUNT);
});

const setTheme = (theme) => {
  currentTheme.value = theme;
  document.documentElement.setAttribute('data-theme', theme);
  localStorage.setItem('theme', theme);
};

onMounted(() => {
  const savedTheme = localStorage.getItem('theme') || DEFAULT_THEME;
  setTheme(savedTheme);
});
</script>
