<template>
  <div class="p-6">
    <h1 class="text-2xl font-bold mb-6">设置</h1>

    <div class="columns-1 md:columns-2 gap-6 space-y-6">
      <!-- Basic Settings Block -->
      <div class="card bg-base-100 shadow-xl border border-base-300 break-inside-avoid">
        <div class="card-body p-4">
          <h2 class="card-title text-lg mb-4">基础设置</h2>

          <div class="grid grid-cols-1 gap-4">
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
              <select
                class="select select-bordered select-sm w-40"
                v-model="currentAppTheme"
                @change="setTheme(currentAppTheme, appThemeKey)"
              >
                <option v-for="theme in themes.slice(0, 2)" :key="theme" :value="theme">
                  {{ theme === 'dark' ? '深色' : '浅色' }}
                </option>
              </select>
            </div>

            <div class="flex justify-between items-center">
              <span class="font-medium">启动页面</span>
              <select
                class="select select-bordered select-sm w-40"
                v-model="currentRouter"
                @change="handleRouterChange"
              >
                <option v-for="route in routesDisplay" :key="route.path" :value="route">
                  {{ route.label }}
                </option>
              </select>
            </div>
          </div>
        </div>
      </div>

      <!-- ADB Settings Block -->
      <div class="card bg-base-100 shadow-xl border border-base-300 break-inside-avoid">
        <div class="card-body p-4">
          <h2 class="card-title text-lg mb-4">ADB 设置</h2>
          <div class="grid grid-cols-1 gap-4">
            <div class="flex justify-between items-center gap-2">
              <span class="font-medium shrink-0">ADB 路径</span>
              <div class="flex gap-1 flex-1 justify-end">
                <input
                  type="text"
                  v-model="adbPath"
                  class="input input-bordered input-sm w-48 read-only:bg-base-300"
                  placeholder="adb.exe 路径"
                  readonly
                />
                <button class="btn btn-sm btn-outline" @click="selectAdbPath">···</button>
              </div>
            </div>

            <div class="flex justify-between items-center gap-2">
              <span class="font-medium shrink-0">ADB 服务地址</span>
              <div class="flex gap-1 items-center">
                <input
                  type="text"
                  v-model="adbServerHost"
                  class="input input-bordered input-sm w-28"
                  placeholder="127.0.0.1"
                />
                <span class="text-sm">:</span>
                <input
                  type="number"
                  v-model.number="adbServerPort"
                  class="input input-bordered input-sm w-20"
                  placeholder="5037"
                />
                <button class="btn btn-sm btn-primary" @click="handleSaveAdbConfig">保存</button>
              </div>
            </div>
          </div>
        </div>
      </div>
      <!-- Log Settings Block -->
      <div class="card bg-base-100 shadow-xl border border-base-300 break-inside-avoid">
        <div class="card-body p-4">
          <h2 class="card-title text-lg mb-4">日志设置</h2>
          <div class="grid grid-cols-1 gap-4">
            <div class="flex justify-between items-center">
              <span class="font-medium">主进程日志级别</span>
              <select class="select select-bordered select-sm w-40" v-model="logLevel" @change="handleLogLevelChange">
                <option value="Debug">Debug</option>
                <option value="Info">Info</option>
                <option value="Warn">Warn</option>
                <option value="Error">Error</option>
                <option value="Off">Off</option>
              </select>
            </div>

            <div class="flex justify-between items-center gap-2">
              <span class="font-medium shrink-0">日志目录</span>
              <div class="flex gap-1 flex-1 justify-end">
                <input
                  type="text"
                  v-model="logDir"
                  class="input input-bordered input-sm w-48 read-only:bg-base-300"
                  placeholder="logs"
                  readonly
                />
                <button class="btn btn-sm btn-outline" @click="selectLogDir">···</button>
              </div>
            </div>

            <div class="flex justify-between items-center">
              <span class="font-medium">日志保留天数</span>
              <div class="flex gap-1 items-center">
                <input
                  type="number"
                  v-model.number="retentionDays"
                  class="input input-bordered input-sm w-20"
                  min="0"
                  max="365"
                />
                <button class="btn btn-sm btn-primary" @click="handleRetentionDaysChange">保存</button>
              </div>
            </div>

            <div class="flex justify-end">
              <button class="btn btn-sm btn-warning" @click="handleCleanLogs">手动清理日志</button>
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
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useThemeManager } from './script-editor/composables/index.js';
import { appThemeKey, defaultRouterKey, adbServerConfigKey, setToStore, getFromStore } from '../store/store.js';
import { THEMES } from './script-editor/config.js';
import { currentRouter, routesDisplay } from '../router/index.js';
import { showToast } from '../utils/toast.js';

const themes = THEMES;
// 基础设置
const { currentAppTheme, setTheme } = useThemeManager();

// ADB 设置
const adbPath = ref('');
const adbServerHost = ref('127.0.0.1');
const adbServerPort = ref(5037);

// 日志设置
const logLevel = ref('Info');
const logDir = ref('logs');
const retentionDays = ref(7);

// 启动页面设置
const handleRouterChange = async () => {
  await setToStore(defaultRouterKey, currentRouter.value);
};

// 加载日志配置
const loadLogConfig = async () => {
  try {
    const config = await invoke('get_log_config_cmd');
    logDir.value = config.logDir || 'logs';
    logLevel.value = config.logLevel || 'Info';
    retentionDays.value = config.retentionDays || 7;
  } catch (e) {
    console.error('加载日志配置失败:', e);
    showToast('加载日志配置失败', 'error');
  }
};

// 日志级别变更
const handleLogLevelChange = async () => {
  try {
    await invoke('update_log_level_cmd', { logLevel: logLevel.value });
  } catch (e) {
    console.error('更新日志级别失败:', e);
    showToast('更新日志级别失败', 'error');
  }
};

// 选择日志目录
const selectLogDir = async () => {
  try {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      logDir.value = selected;
      handleLogDirChange();
    }
  } catch (e) {
    console.error('选择目录失败:', e);
    showToast('选择目录失败', 'error');
  }
};

// 日志目录变更
const handleLogDirChange = async () => {
  try {
    await invoke('update_log_dir_cmd', { logDir: logDir.value });
    showToast('日志目录已变更，重启生效', 'info');
  } catch (e) {
    console.error('更新日志目录失败:', e);
    showToast('更新日志目录失败', 'error');
  }
};

// 保留天数变更
const handleRetentionDaysChange = async () => {
  try {
    await invoke('update_retention_days_cmd', { days: retentionDays.value });
    showToast('保存成功', 'success');
  } catch (e) {
    console.error('更新保留天数失败:', e);
    showToast('更新保留天数失败', 'error');
  }
};

// 手动清理日志
const handleCleanLogs = async () => {
  try {
    await invoke('clean_logs_now_cmd');
    showToast('清理日志成功', 'success');
  } catch (e) {
    console.error('清理日志失败:', e);
    showToast('清理日志失败', 'error');
  }
};

onMounted(async () => {
  loadLogConfig();
  loadAdbConfig();
});

// ADB 配置加载
const loadAdbConfig = async () => {
  try {
    const config = await getFromStore(adbServerConfigKey);
    if (config) {
      adbPath.value = config.adbPath || '';
      adbServerHost.value = config.serverHost || '127.0.0.1';
      adbServerPort.value = config.serverPort || 5037;
    }
  } catch (e) {
    console.error('加载ADB配置失败:', e);
  }
};

// 选择 ADB 路径
const selectAdbPath = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'ADB', extensions: ['exe'] }],
    });
    if (selected) {
      adbPath.value = selected;
      await handleSaveAdbConfig();
    }
  } catch (e) {
    console.error('选择ADB路径失败:', e);
    showToast('选择ADB路径失败', 'error');
  }
};

// 保存 ADB 配置
const handleSaveAdbConfig = async () => {
  try {
    const config = {
      adbPath: adbPath.value,
      serverHost: adbServerHost.value,
      serverPort: adbServerPort.value,
    };
    await setToStore(adbServerConfigKey, config);
    // TODO: 广播到所有运行中子进程 (Phase 2 后续实现)
    // await invoke('cmd_broadcast_adb_path', { adbPath: adbPath.value });
    showToast('ADB配置已保存', 'success');
  } catch (e) {
    console.error('保存ADB配置失败:', e);
    showToast('保存ADB配置失败', 'error');
  }
};
