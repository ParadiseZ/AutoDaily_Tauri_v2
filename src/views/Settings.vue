<template>
  <div class="w-full max-w-4xl mx-auto flex flex-col gap-8 pb-10 pt-4">
    <div class="px-2">
      <h1 class="text-2xl font-semibold tracking-tight text-base-content">系统设置</h1>
      <p class="text-sm text-base-content/50 mt-1">管理系统行为、外观与本地配置。</p>
    </div>
    
    <!-- Group: Conventional -->
    <div class="flex flex-col gap-2">
      <h2 class="text-[11px] font-semibold uppercase tracking-wider text-base-content/50 ml-4 mb-1">常规</h2>
      <div class="bg-base-100 border border-base-content/5 rounded-2xl overflow-hidden shadow-sm">
        
        <div class="p-4 flex items-center justify-between bg-base-100 hover:bg-base-200/80 transition-colors cursor-pointer" @click="toggleAutoStart">
          <div>
            <div class="text-[14px] font-medium text-base-content/90">开机自启</div>
            <div class="text-[12px] text-base-content/50 mt-0.5">登录操作系统后自动运行 AutoDaily</div>
          </div>
          <!-- DaisyUI switch styled to look more like iOS -->
          <input type="checkbox" v-model="autoStart" class="toggle toggle-md bg-base-300 border-base-300 checked:bg-primary checked:border-primary hover:opacity-90 transition-all" @click.stop />
        </div>
        
        <div class="w-auto h-px bg-base-content/5 ml-4"></div>
        
        <div class="p-4 flex items-center justify-between bg-base-100 hover:bg-base-200/80 transition-colors">
          <div>
            <div class="text-[14px] font-medium text-base-content/90">外观模式</div>
            <div class="text-[12px] text-base-content/50 mt-0.5">选择界面色彩主题</div>
          </div>
          <select v-model="themeSetting" @change="handleThemeChange" class="select select-sm select-bordered w-32 bg-base-200/50 text-[13px] rounded-lg focus:outline-none focus:ring-2 focus:ring-base-content/20 transition-all border-none">
            <option value="system">随系统</option>
            <option value="light">浅色</option>
            <option value="dark">深色</option>
          </select>
        </div>

      </div>
    </div>

    <!-- Group: ADB Configuration -->
    <div class="flex flex-col gap-2">
      <h2 class="text-[11px] font-semibold uppercase tracking-wider text-base-content/50 ml-4 mb-1">ADB 配置</h2>
      <div class="bg-base-100 border border-base-content/5 rounded-2xl overflow-hidden shadow-sm">
        <div class="p-4 flex flex-col gap-3 bg-base-100">
          <div class="flex items-center justify-between">
             <div class="text-[14px] font-medium text-base-content/90">ADB 可执行路径</div>
             <button class="btn btn-xs font-normal bg-base-200 hover:bg-base-300 border-none rounded-md text-base-content/80">选择路径</button>
          </div>
          <div class="flex gap-2 items-center">
            <input type="text" class="input input-sm w-full bg-base-200/30 text-base-content/70 rounded-md border border-base-content/10 focus:outline-none focus:border-base-content/30 text-[13px] font-mono" value="C:\Android\platform-tools\adb.exe" readonly />
          </div>
        </div>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useThemeManager } from '../composables/useThemeManager';
import { appThemeKey, getFromStore } from '../store/store';

const autoStart = ref(true);
const toggleAutoStart = () => {
    autoStart.value = !autoStart.value;
};

// Theme integration
const { setTheme } = useThemeManager();
const themeSetting = ref('system');

onMounted(async () => {
    const saved = await getFromStore<string>(appThemeKey);
    if (saved && ['light', 'dark', 'system'].includes(saved)) {
        themeSetting.value = saved;
    }
});

const handleThemeChange = async () => {
    await setTheme(appThemeKey, themeSetting.value as 'light' | 'dark' | 'system');
};
</script>
