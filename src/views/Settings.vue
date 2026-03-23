<template>
  <div class="space-y-6">
    <AppPageHeader
      eyebrow="Preferences"
      title="系统设置"
      description="账户、桌面行为、ADB 与日志配置保持统一来源，本地持久化和运行时命令分开处理。"
    />

    <div class="grid gap-4 xl:grid-cols-[1.05fr_0.95fr]">
      <div class="space-y-4">
        <SettingsSection title="账户信息" description="云端能力集中在这里管理，避免分散到脚本页和市场页。">
          <div v-if="!userStore.isLoggedIn" class="flex items-center justify-between gap-4 rounded-[20px] border border-[var(--app-border)] px-4 py-4">
            <div>
              <p class="text-sm font-medium text-[var(--app-text-strong)]">当前未登录</p>
              <p class="text-sm text-[var(--app-text-soft)]">登录后可同步脚本、访问脚本市场和管理用户名。</p>
            </div>
            <button class="app-button app-button-primary" type="button" @click="userStore.openAuthModal()">登录</button>
          </div>

          <template v-else>
            <div class="grid gap-3 md:grid-cols-2">
              <div class="rounded-[20px] border border-[var(--app-border)] px-4 py-4">
                <p class="text-xs uppercase tracking-[0.16em] text-[var(--app-text-faint)]">用户名</p>
                <div class="mt-2 flex items-center gap-3">
                  <input v-model.trim="usernameDraft" class="app-input" />
                  <button class="app-button app-button-ghost h-11 px-4" type="button" @click="saveUsername">
                    保存
                  </button>
                </div>
              </div>
              <div class="rounded-[20px] border border-[var(--app-border)] px-4 py-4">
                <p class="text-xs uppercase tracking-[0.16em] text-[var(--app-text-faint)]">邮箱</p>
                <p class="mt-3 text-sm font-medium text-[var(--app-text-strong)]">{{ userStore.userProfile?.email }}</p>
              </div>
            </div>

            <div class="grid gap-3 md:grid-cols-3">
              <div class="app-stat">
                <p class="app-stat-label">开发者状态</p>
                <p class="app-stat-value text-base">{{ userStore.isDeveloper ? '有效' : '普通用户' }}</p>
              </div>
              <div class="app-stat">
                <p class="app-stat-label">最近上传</p>
                <p class="app-stat-value text-base">{{ formatDate(userStore.userProfile?.lastScriptUploadTime) }}</p>
              </div>
              <div class="app-stat">
                <p class="app-stat-label">用户名修改</p>
                <p class="app-stat-value text-base">{{ formatDate(userStore.userProfile?.lastUsernameChangeTime) }}</p>
              </div>
            </div>

            <div class="flex justify-end">
              <button class="app-button app-button-danger" type="button" @click="userStore.logout()">退出登录</button>
            </div>
          </template>
        </SettingsSection>

        <SettingsSection title="界面与启动" description="这些偏工作流的偏好写入本地 Store，并且主题会立即反馈到桌面界面。">
          <div class="grid gap-4 md:grid-cols-2">
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">主题</span>
              <AppSelect v-model="settingsStore.preferences.appTheme" :options="themeOptions" @update:model-value="handleThemeChange" />
            </label>
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">默认页面</span>
              <AppSelect v-model="settingsStore.preferences.defaultRoute" :options="defaultRouteOptions" @update:model-value="handleRouteChange" />
            </label>
          </div>

          <div class="grid gap-3 md:grid-cols-2">
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">启动模式</span>
              <AppSelect v-model="settingsStore.preferences.startMode" :options="startModeOptions" @update:model-value="saveSystemPreferences" />
            </label>
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">空闲处理</span>
              <AppSelect v-model="settingsStore.preferences.idleAction" :options="idleActionOptions" @update:model-value="saveSystemPreferences" />
            </label>
          </div>

          <div class="grid gap-3 md:grid-cols-3">
            <label class="flex items-center justify-between rounded-[20px] border border-[var(--app-border)] px-4 py-3">
              <span class="text-sm text-[var(--app-text-strong)]">开机自启</span>
              <input v-model="settingsStore.preferences.autoStart" type="checkbox" class="toggle toggle-sm" @change="saveSystemPreferences" />
            </label>
            <label class="flex items-center justify-between rounded-[20px] border border-[var(--app-border)] px-4 py-3">
              <span class="text-sm text-[var(--app-text-strong)]">窗口置顶</span>
              <input v-model="settingsStore.preferences.alwaysOnTop" type="checkbox" class="toggle toggle-sm" @change="saveSystemPreferences" />
            </label>
            <label class="flex items-center justify-between rounded-[20px] border border-[var(--app-border)] px-4 py-3">
              <span class="text-sm text-[var(--app-text-strong)]">关闭即退出</span>
              <input v-model="settingsStore.preferences.closeExit" type="checkbox" class="toggle toggle-sm" @change="saveSystemPreferences" />
            </label>
          </div>
        </SettingsSection>

        <SettingsSection title="ADB 与环境" description="没有现成后端命令的字段保存在本地 Store，给设备编辑器和运行环境统一复用。">
          <div class="grid gap-4 md:grid-cols-[1fr_auto]">
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">ADB 路径</span>
              <input v-model="settingsStore.preferences.adbPath" class="app-input" placeholder="选择 adb.exe 路径" />
            </label>
            <button class="app-button app-button-ghost self-end" type="button" @click="pickAdbPath">选择路径</button>
          </div>
          <div class="grid gap-4 md:grid-cols-2">
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">ADB 服务 Host</span>
              <input v-model.trim="settingsStore.preferences.adbServerHost" class="app-input" placeholder="127.0.0.1" />
            </label>
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">ADB 服务 Port</span>
              <input v-model.number="settingsStore.preferences.adbServerPort" class="app-input" type="number" min="1" max="65535" />
            </label>
          </div>
          <div class="flex justify-end">
            <button class="app-button app-button-primary" type="button" @click="saveEnvironmentPreferences">保存环境配置</button>
          </div>
        </SettingsSection>
      </div>

      <div class="space-y-4">
        <SettingsSection title="日志设置" description="主进程日志由 Tauri 命令即时修改，日志路径和保留天数都在这里维护。">
          <div class="grid gap-4 md:grid-cols-2">
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">主进程日志级别</span>
              <AppSelect v-model="settingsStore.logConfig.logLevel" :options="logLevelOptions" />
            </label>
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">保留天数</span>
              <input v-model.number="settingsStore.logConfig.retentionDays" class="app-input" type="number" min="1" max="365" />
            </label>
          </div>
          <div class="grid gap-4 md:grid-cols-[1fr_auto]">
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">日志目录</span>
              <input v-model="settingsStore.logConfig.logDir" class="app-input" />
            </label>
            <button class="app-button app-button-ghost self-end" type="button" @click="pickLogDir">选择目录</button>
          </div>
          <div class="flex flex-wrap justify-end gap-3">
            <button class="app-button app-button-warning" type="button" @click="cleanLogs">立即清理</button>
            <button class="app-button app-button-primary" type="button" @click="saveLogSettings">保存日志配置</button>
          </div>
        </SettingsSection>

        <SettingsSection title="关于与更新" description="保持一个轻量的版本信息区，不把宣传内容塞进专业工具型桌面应用。">
          <div class="grid gap-3 md:grid-cols-2">
            <div class="rounded-[20px] border border-[var(--app-border)] p-4">
              <p class="text-xs uppercase tracking-[0.16em] text-[var(--app-text-faint)]">版本</p>
              <p class="mt-2 text-lg font-semibold text-[var(--app-text-strong)]">AutoDaily 0.1.0</p>
            </div>
            <div class="rounded-[20px] border border-[var(--app-border)] p-4">
              <p class="text-xs uppercase tracking-[0.16em] text-[var(--app-text-faint)]">最近更新</p>
              <p class="mt-2 text-sm text-[var(--app-text-strong)]">
                {{ settingsStore.updateInfo ? `${settingsStore.updateInfo.version} · ${formatDate(settingsStore.updateInfo.pubDate)}` : '尚未检查' }}
              </p>
            </div>
          </div>
          <div class="rounded-[20px] border border-[var(--app-border)] p-4 text-sm text-[var(--app-text-soft)]">
            {{ settingsStore.updateInfo?.notes || '当前还没有拉取更新说明。' }}
          </div>
          <div class="flex justify-end">
            <button class="app-button app-button-primary" type="button" @click="checkUpdate">检查更新</button>
          </div>
        </SettingsSection>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import AppSelect from '@/components/shared/AppSelect.vue';
import AppPageHeader from '@/components/shared/AppPageHeader.vue';
import SettingsSection from '@/views/settings/SettingsSection.vue';
import { useSettingsStore } from '@/store/settings';
import { useUserStore } from '@/store/user';
import { useThemeManager } from '@/composables/useThemeManager';
import { appThemeKey } from '@/store/store';
import { showToast } from '@/utils/toast';
import { formatDate } from '@/utils/presenters';

const settingsStore = useSettingsStore();
const userStore = useUserStore();
const { setTheme } = useThemeManager();
const usernameDraft = ref('');
const themeOptions = [
  { label: '跟随系统', value: 'system' },
  { label: '浅色', value: 'light' },
  { label: '深色', value: 'dark' },
];
const defaultRouteOptions = [
  { label: '任务管理', value: '/tasks' },
  { label: '设备列表', value: '/devices' },
  { label: '本地脚本', value: '/scripts' },
  { label: '脚本市场', value: '/market' },
  { label: '运行日志', value: '/logs' },
  { label: '系统设置', value: '/settings' },
];
const startModeOptions = [
  { label: '正常显示', value: 'normal' },
  { label: '最小化启动', value: 'minimized' },
  { label: '启动到托盘', value: 'tray' },
];
const idleActionOptions = [
  { label: '无动作', value: 'none' },
  { label: '关机', value: 'shutdown' },
  { label: '睡眠', value: 'sleep' },
  { label: '休眠', value: 'hibernate' },
];
const logLevelOptions = [
  { label: 'Debug', value: 'Debug' },
  { label: 'Info', value: 'Info' },
  { label: 'Warn', value: 'Warn' },
  { label: 'Error', value: 'Error' },
  { label: 'Off', value: 'Off' },
];

const handleThemeChange = async () => {
  await settingsStore.updatePreferences({ appTheme: settingsStore.preferences.appTheme });
  await setTheme(appThemeKey, settingsStore.preferences.appTheme);
};

const handleRouteChange = async () => {
  await settingsStore.updatePreferences({ defaultRoute: settingsStore.preferences.defaultRoute });
  showToast('默认页面已保存', 'success');
};

const saveSystemPreferences = async () => {
  try {
    await settingsStore.applySystemPreferences({
      startMode: settingsStore.preferences.startMode,
      closeExit: settingsStore.preferences.closeExit,
      alwaysOnTop: settingsStore.preferences.alwaysOnTop,
      idleAction: settingsStore.preferences.idleAction,
      autoStart: settingsStore.preferences.autoStart,
    });
    showToast('桌面行为已更新', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '更新失败', 'error');
  }
};

const saveEnvironmentPreferences = async () => {
  await settingsStore.updatePreferences({
    adbPath: settingsStore.preferences.adbPath,
    adbServerHost: settingsStore.preferences.adbServerHost,
    adbServerPort: settingsStore.preferences.adbServerPort,
  });
  showToast('环境配置已保存到本地', 'success');
};

const saveLogSettings = async () => {
  try {
    await settingsStore.updateLogSettings({
      logLevel: settingsStore.logConfig.logLevel,
      logDir: settingsStore.logConfig.logDir,
      retentionDays: settingsStore.logConfig.retentionDays,
    });
    showToast('日志配置已保存', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '日志配置保存失败', 'error');
  }
};

const cleanLogs = async () => {
  try {
    await settingsStore.cleanLogsNow();
    showToast('日志清理完成', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '日志清理失败', 'error');
  }
};

const pickAdbPath = async () => {
  const value = await open({ multiple: false, directory: false });
  if (typeof value === 'string') {
    settingsStore.preferences.adbPath = value;
  }
};

const pickLogDir = async () => {
  const value = await open({ directory: true, multiple: false });
  if (typeof value === 'string') {
    settingsStore.logConfig.logDir = value;
  }
};

const saveUsername = async () => {
  if (!usernameDraft.value || usernameDraft.value === userStore.userProfile?.username) {
    return;
  }

  try {
    await userStore.updateUsername(usernameDraft.value);
  } catch {}
};

const checkUpdate = async () => {
  try {
    const result = await settingsStore.refreshUpdateInfo();
    showToast(result ? `发现版本 ${result.version}` : '当前已是最新版本', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '检查失败', 'error');
  }
};

watch(
  () => userStore.userProfile?.username,
  (value) => {
    usernameDraft.value = value || '';
  },
  { immediate: true },
);

onMounted(async () => {
  await Promise.all([settingsStore.loadPreferences(), userStore.checkProfile()]);
});
</script>
