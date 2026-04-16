<template>
  <div class="space-y-6">
    <AppPageHeader
      title="系统设置"
    />

    <div class="grid gap-4 xl:grid-cols-[1.05fr_0.95fr]">
      <div class="space-y-4">
        <SettingsSection icon="user" title="账户信息">
          <div v-if="!userStore.isLoggedIn" class="flex items-center justify-between gap-4 rounded-[20px] border border-[var(--app-border)] px-4 py-4">
            <div>
              <p class="text-sm font-medium text-[var(--app-text-strong)]">当前未登录</p>
              <p class="text-sm text-[var(--app-text-soft)]">登录后可同步脚本、访问脚本市场和管理用户名。</p>
            </div>
            <button class="app-button app-button-primary shadow-lg shadow-[var(--app-vibrant-blue)]/30" type="button" @click="userStore.openAuthModal()">
              <AppIcon name="log-in" :size="16" />
              登录
            </button>
          </div>

          <template v-else>
            <div class="grid gap-3 md:grid-cols-2">
              <div class="rounded-[20px] border border-[var(--app-border)] px-4 py-4">
                <p class="text-xs uppercase tracking-[0.16em] text-[var(--app-text-faint)]">用户名</p>
                <div class="mt-2 flex items-center gap-3">
                  <input v-model.trim="usernameDraft" class="app-input" />
                  <button class="app-button app-button-ghost h-11 px-4 group" type="button" @click="saveUsername">
                    <AppIcon name="save" :size="16" class="text-[var(--app-text-faint)] group-hover:text-[var(--app-accent)] transition-colors" />
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
              <button class="app-button app-button-danger px-4" type="button" @click="userStore.logout()">
                <AppIcon name="log-out" :size="16" />
                退出登录
              </button>
            </div>
          </template>
        </SettingsSection>

        <SettingsSection icon="monitor" title="界面与启动" description="这些偏工作流的偏好写入本地 Store，并且主题会立即反馈到桌面界面。">
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

        <SettingsSection icon="terminal-square" title="ADB 与环境" description="没有现成后端命令的字段保存在本地 Store，给设备编辑器和运行环境统一复用。">
          <div class="grid gap-4 md:grid-cols-[1fr_auto]">
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">ADB 路径</span>
              <input v-model="settingsStore.preferences.adbPath" class="app-input" placeholder="选择 adb.exe 路径" />
            </label>
            <button class="app-button app-button-ghost group self-end" type="button" @click="pickAdbPath">
              <AppIcon name="folder-open" :size="16" class="text-[var(--app-text-soft)] group-hover:text-[var(--app-accent)] transition-colors" />
              选择路径
            </button>
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
            <button class="app-button app-button-primary shadow-lg shadow-[var(--app-accent-soft)]" type="button" @click="saveEnvironmentPreferences">
              <AppIcon name="save" :size="16" />
              保存环境配置
            </button>
          </div>
        </SettingsSection>

        <SettingsSection icon="file-search" title="OCR 文字缓存" description="启用后按脚本名称读写 JSON 缓存文件，目录留空时回退到应用数据目录下的默认缓存文件夹。">
          <div class="grid gap-3 md:grid-cols-[minmax(0,0.75fr)_minmax(0,1.25fr)]">
            <label class="flex items-center justify-between rounded-[20px] border border-[var(--app-border)] px-4 py-3">
              <span class="text-sm text-[var(--app-text-strong)]">启用缓存文字检测结果</span>
              <input v-model="settingsStore.preferences.ocrTextCacheEnabled" type="checkbox" class="toggle toggle-sm" />
            </label>
            <div class="rounded-[20px] border border-[var(--app-border)] px-4 py-3 text-sm text-[var(--app-text-soft)]">
              仅在脚本运行时加载和写入缓存，缓存内容按脚本名称分文件保存。
            </div>
          </div>
          <div class="grid gap-4 md:grid-cols-[1fr_auto]">
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">缓存目录</span>
              <input
                v-model.trim="settingsStore.preferences.ocrTextCacheDir"
                class="app-input"
                placeholder="留空时使用应用默认缓存目录"
              />
            </label>
            <button class="app-button app-button-ghost group self-end" type="button" @click="pickOcrTextCacheDir">
              <AppIcon name="folder-open" :size="16" class="text-[var(--app-text-soft)] group-hover:text-[var(--app-accent)] transition-colors" />
              选择目录
            </button>
          </div>
          <div class="grid gap-4 md:grid-cols-2">
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">视觉签名网格(px)</span>
              <input
                v-model.number="settingsStore.preferences.visionSignatureGridSize"
                class="app-input"
                type="number"
                min="1"
                step="1"
              />
            </label>
            <div class="rounded-[20px] border border-[var(--app-border)] px-4 py-3 text-sm text-[var(--app-text-soft)]">
              稳定排序、相对位置判断和动作签名会按该像素挡位离散化；原始执行坐标仍保留精确值。
            </div>
          </div>
          <div class="flex justify-end">
            <button class="app-button app-button-primary shadow-lg shadow-[var(--app-accent-soft)]" type="button" @click="saveVisionCachePreferences">
              <AppIcon name="save" :size="16" />
              保存缓存设置
            </button>
          </div>
        </SettingsSection>
      </div>

      <div class="space-y-4">
        <SettingsSection icon="file-clock" title="日志设置" description="主进程日志由 Tauri 命令即时修改，日志路径和保留天数都在这里维护。">
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
            <button class="app-button app-button-ghost group self-end" type="button" @click="pickLogDir">
              <AppIcon name="folder-open" :size="16" class="text-[var(--app-text-soft)] group-hover:text-[var(--app-accent)] transition-colors" />
              选择目录
            </button>
          </div>
          <div class="flex flex-wrap justify-end gap-3">
            <button class="app-button app-button-warning shadow-md shadow-amber-500/10" type="button" @click="cleanLogs">
              <AppIcon name="trash-2" :size="16" />
              立即清理
            </button>
            <button class="app-button app-button-primary shadow-lg shadow-[var(--app-accent-soft)]" type="button" @click="saveLogSettings">
              <AppIcon name="save" :size="16" />
              保存日志配置
            </button>
          </div>
        </SettingsSection>

        <SettingsSection icon="mail" title="邮件通知" description="支持 163、QQ、Gmail、Outlook 预设，也支持自定义 SMTP。QQ、Gmail 等通常需要授权码或应用专用密码，而不是登录密码。">
          <div class="grid gap-3 md:grid-cols-2">
            <label class="flex items-center justify-between rounded-[20px] border border-[var(--app-border)] px-4 py-3">
              <span class="text-sm text-[var(--app-text-strong)]">桌面超时提醒</span>
              <input v-model="settingsStore.emailConfig.desktopNotice" type="checkbox" class="toggle toggle-sm" />
            </label>
            <label class="flex items-center justify-between rounded-[20px] border border-[var(--app-border)] px-4 py-3">
              <span class="text-sm text-[var(--app-text-strong)]">邮件超时提醒</span>
              <input v-model="settingsStore.emailConfig.emailNotification" type="checkbox" class="toggle toggle-sm" />
            </label>
          </div>

          <div class="grid gap-4 md:grid-cols-2">
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">SMTP 服务商</span>
              <AppSelect v-model="settingsStore.emailConfig.provider" :options="emailProviderOptions" />
            </label>
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">连接加密</span>
              <AppSelect
                v-model="settingsStore.emailConfig.security"
                :options="emailSecurityOptions"
                :disabled="settingsStore.emailConfig.provider !== 'custom'"
              />
            </label>
          </div>

          <div class="grid gap-4 md:grid-cols-[1fr_180px]">
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">SMTP 服务器</span>
              <input
                v-model.trim="settingsStore.emailConfig.smtpServer"
                class="app-input"
                :disabled="settingsStore.emailConfig.provider !== 'custom'"
                placeholder="smtp.example.com"
              />
            </label>
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">端口</span>
              <input
                v-model.number="settingsStore.emailConfig.smtpPort"
                class="app-input"
                type="number"
                min="1"
                max="65535"
                :disabled="settingsStore.emailConfig.provider !== 'custom'"
              />
            </label>
          </div>

          <div class="grid gap-4 md:grid-cols-2">
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">SMTP 用户名</span>
              <input v-model.trim="settingsStore.emailConfig.username" class="app-input" placeholder="通常为邮箱地址" />
            </label>
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">SMTP 密码 / 授权码</span>
              <input v-model="settingsStore.emailConfig.password" class="app-input" type="password" placeholder="建议使用服务商授权码" />
            </label>
          </div>

          <div class="grid gap-4 md:grid-cols-2">
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">发件人名称</span>
              <input v-model.trim="settingsStore.emailConfig.senderName" class="app-input" placeholder="AutoDaily" />
            </label>
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">发件人邮箱</span>
              <input v-model.trim="settingsStore.emailConfig.senderEmail" class="app-input" placeholder="留空时回退到 SMTP 用户名" />
            </label>
          </div>

          <div class="grid gap-4 md:grid-cols-[1fr_180px]">
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">收件人</span>
              <textarea
                v-model.trim="settingsStore.emailConfig.recipient"
                class="app-input min-h-[104px] resize-y py-3"
                placeholder="支持多个邮箱，使用逗号、分号或换行分隔"
              />
            </label>
            <label class="grid gap-2">
              <span class="text-sm text-[var(--app-text-soft)]">超时时间(秒)</span>
              <input
                v-model.number="settingsStore.emailConfig.timeoutSeconds"
                class="app-input"
                type="number"
                min="5"
                max="300"
              />
            </label>
          </div>

          <div class="rounded-[20px] border border-[var(--app-border)] px-4 py-3 text-sm text-[var(--app-text-soft)]">
            常见预设：163 / QQ 默认走 465 + SSL/TLS，Outlook 默认走 587 + STARTTLS。Gmail 建议开启两步验证后使用应用专用密码。
          </div>

          <div class="flex flex-wrap justify-end gap-3">
            <button class="app-button app-button-ghost" type="button" @click="sendTestEmail">
              <AppIcon name="send" :size="16" />
              测试发送
            </button>
            <button class="app-button app-button-primary shadow-lg shadow-[var(--app-accent-soft)]" type="button" @click="saveEmailSettings">
              <AppIcon name="save" :size="16" />
              保存邮件配置
            </button>
          </div>
        </SettingsSection>

        <SettingsSection icon="info" title="关于与更新" description="保持一个轻量的版本信息区，不把宣传内容塞进专业工具型桌面应用。">
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
            <button class="app-button app-button-primary shadow-lg shadow-[var(--app-accent-soft)]" type="button" @click="checkUpdate">
              <AppIcon name="refresh-cw" :size="16" />
              检查更新
            </button>
          </div>
        </SettingsSection>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import AppPageHeader from '@/components/shared/AppPageHeader.vue';
import SettingsSection from '@/views/settings/SettingsSection.vue';
import { useSettingsStore } from '@/store/settings';
import { useUserStore } from '@/store/user';
import { useThemeManager } from '@/composables/useThemeManager';
import { settingsService } from '@/services/settingsService';
import { appThemeKey } from '@/store/store';
import { showToast } from '@/utils/toast';
import { formatDate } from '@/utils/presenters';
import type { EmailConfig, EmailProviderPreset, EmailSecurity, VisionTextCacheConfig } from '@/types/app/domain';

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
const emailProviderOptions = [
  { label: '自定义', value: 'custom' },
  { label: '163 邮箱', value: '163' },
  { label: 'QQ 邮箱', value: 'qq' },
  { label: 'Gmail', value: 'gmail' },
  { label: 'Outlook', value: 'outlook' },
];
const emailSecurityOptions = [
  { label: 'SSL / TLS', value: 'tlsWrapper' },
  { label: 'STARTTLS', value: 'startTls' },
  { label: '明文', value: 'none' },
];
const emailProviderPresetMap: Record<Exclude<EmailProviderPreset, 'custom'>, { smtpServer: string; smtpPort: number; security: EmailSecurity }> = {
  '163': { smtpServer: 'smtp.163.com', smtpPort: 465, security: 'tlsWrapper' },
  qq: { smtpServer: 'smtp.qq.com', smtpPort: 465, security: 'tlsWrapper' },
  gmail: { smtpServer: 'smtp.gmail.com', smtpPort: 465, security: 'tlsWrapper' },
  outlook: { smtpServer: 'smtp-mail.outlook.com', smtpPort: 587, security: 'startTls' },
};

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

const saveVisionCachePreferences = async () => {
  const config: VisionTextCacheConfig = {
    enabled: settingsStore.preferences.ocrTextCacheEnabled,
    dir: settingsStore.preferences.ocrTextCacheDir,
    signatureGridSize: Math.max(1, Number(settingsStore.preferences.visionSignatureGridSize) || 8),
  };

  try {
    await settingsService.updateVisionTextCacheConfig(config);
    await settingsStore.updatePreferences({
      ocrTextCacheEnabled: config.enabled,
      ocrTextCacheDir: config.dir,
      visionSignatureGridSize: config.signatureGridSize,
    });
    showToast('OCR 缓存设置已保存', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : 'OCR 缓存设置保存失败', 'error');
  }
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

const syncEmailProviderPreset = () => {
  const provider = settingsStore.emailConfig.provider;
  if (provider === 'custom') {
    return;
  }

  const preset = emailProviderPresetMap[provider];
  settingsStore.emailConfig.smtpServer = preset.smtpServer;
  settingsStore.emailConfig.smtpPort = preset.smtpPort;
  settingsStore.emailConfig.security = preset.security;
};

const normalizedEmailConfig = (): EmailConfig => {
  syncEmailProviderPreset();
  return {
    ...settingsStore.emailConfig,
    senderName: settingsStore.emailConfig.senderName.trim(),
    senderEmail: settingsStore.emailConfig.senderEmail.trim(),
    username: settingsStore.emailConfig.username.trim(),
    recipient: settingsStore.emailConfig.recipient.trim(),
    timeoutSeconds: Math.max(5, Number(settingsStore.emailConfig.timeoutSeconds) || 60),
  };
};

const saveEmailSettings = async () => {
  try {
    const config = normalizedEmailConfig();
    await settingsStore.saveEmailSettings(config);
    showToast('邮件配置已保存', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '邮件配置保存失败', 'error');
  }
};

const sendTestEmail = async () => {
  try {
    await settingsStore.sendEmailTest(normalizedEmailConfig());
    showToast('测试邮件已发送，请检查收件箱', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '测试邮件发送失败', 'error');
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

const pickOcrTextCacheDir = async () => {
  const value = await open({ directory: true, multiple: false });
  if (typeof value === 'string') {
    settingsStore.preferences.ocrTextCacheDir = value;
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

watch(
  () => settingsStore.emailConfig.provider,
  () => {
    syncEmailProviderPreset();
  },
  { immediate: true },
);

onMounted(async () => {
  await Promise.all([settingsStore.loadPreferences(), userStore.checkProfile()]);
});
</script>
