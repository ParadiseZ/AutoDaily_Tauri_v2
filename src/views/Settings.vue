<template>
  <div class="p-6">
    <h1 class="text-2xl font-bold mb-6">设置</h1>
    
    <div class="columns-1 md:columns-2 gap-6 space-y-6">
      <!-- Basic Settings Block -->
      <div class="card bg-base-100 shadow-xl border border-base-300 break-inside-avoid">
        <div class="card-body p-4">
          <h2 class="card-title text-lg mb-4">基础设置</h2>
          
          <div class="grid grid-cols-1 gap-4">
<!--          <div class="flex justify-between items-center">
              <span class="font-medium">Language</span>
              <select class="select select-bordered select-sm w-40">
                <option>English</option>
                <option>Chinese</option>
              </select>
            </div>-->
            
            <div class="flex justify-between items-center">
              <span class="font-medium">启动模式</span>
              <select class="select select-bordered select-sm w-40">
                <option>正常</option>
                <option>最小化</option>
                <option>托盘</option>
              </select>
            </div>

            <div class="flex justify-between items-center">
               <span class="font-medium">开机自启</span>
               <input type="checkbox" class="toggle toggle-primary toggle-sm" />
            </div>
            <div class="flex justify-between items-center">
               <span class="font-medium">保持置顶</span>
               <input type="checkbox" class="toggle toggle-primary toggle-sm" />
            </div>

            <div class="flex justify-between items-center">
              <span class="font-medium">主题设置</span>
              <select class="select select-bordered select-sm w-40" v-model="currentAppTheme">
                <option v-for="theme in themes.slice(0, 2)" @click="setTheme(theme,appThemeKey)" :value="theme">
                  {{ theme === 'dark' ? '深色' : '浅色' }}
                </option>
              </select>
            </div>

            <div class="flex justify-between items-center">
              <span class="font-medium">启动页面</span>
              <select class="select select-bordered select-sm w-40" v-model="currentRouter">
                <option v-for="route in routesDisplay" @click="setToStore(defaultRouterKey,route)" :value="route">
                  {{ route.label}}
                </option>
              </select>
            </div>
          </div>
        </div>
      </div>

      <!-- Performance Block -->
      <div class="card bg-base-100 shadow-xl border border-base-300 break-inside-avoid">
        <div class="card-body p-4">
           <h2 class="card-title text-lg mb-4">性能设置</h2>
           <div class="flex justify-between items-center mb-2">
              <span class="font-medium">并行任务数</span>
              <input type="number" class="input input-bordered input-sm w-20" value="4" />
           </div>
           <div class="flex justify-between items-center">
              <span class="font-medium">GPU推理</span>
              <input type="checkbox" class="toggle toggle-secondary toggle-sm" checked />
           </div>
        </div>
      </div>

       <!-- About Block -->
      <div class="card bg-base-100 shadow-xl border border-base-300 break-inside-avoid">
        <div class="card-body p-4">
           <h2 class="card-title text-lg mb-4">关于</h2>
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
import { onMounted } from 'vue';
import { useThemeManager } from './script-editor/composables/index.js';
import {appThemeKey, defaultRouterKey,setToStore} from '../store/store.js'
import {THEMES} from "./script-editor/config.js";
import { currentRouter,routesDisplay } from '../router/index.js'

const themes = THEMES;
// 基础设置
const {
  currentAppTheme,
  setTheme,
  initTheme } = useThemeManager()

// 生命周期
onMounted(() => {
  initTheme(appThemeKey)
});
</script>
