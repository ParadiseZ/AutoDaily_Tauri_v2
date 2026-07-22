<template>
  <div class="flex h-full min-h-0 flex-col gap-4">
    <AppPageHeader title="关于 AutoDaily" />

    <div class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
      <div class="grid gap-4 xl:grid-cols-[1.1fr_0.9fr]">
        <SurfacePanel class="space-y-5">
          <div class="flex items-center gap-4">
            <div class="flex h-16 w-16 items-center justify-center rounded-[22px] bg-(--app-accent-soft) text-(--app-accent)">
              <Command class="h-8 w-8" />
            </div>
            <div>
              <p class="text-2xl font-semibold text-(--app-text-strong)">AutoDaily</p>
              <p class="text-sm text-(--app-text-soft)">桌面端自动化控制台</p>
            </div>
          </div>

          <div class="space-y-3 text-sm leading-7 text-(--app-text-soft)">
            <p>面向本地设备调度、脚本资产维护、运行日志追踪和视觉自动化调试。</p>
            <p>当前版本侧重桌面端工作流，脚本、设备和环境配置都保留在本机可控范围内。</p>
          </div>

          <div class="grid gap-3 md:grid-cols-3">
            <div class="rounded-[20px] border border-(--app-border) bg-(--app-panel-muted)/65 px-4 py-3">
              <p class="text-xs text-(--app-text-faint)">设备</p>
              <p class="mt-1 text-base font-semibold text-(--app-text-strong)">多实例</p>
            </div>
            <div class="rounded-[20px] border border-(--app-border) bg-(--app-panel-muted)/65 px-4 py-3">
              <p class="text-xs text-(--app-text-faint)">脚本</p>
              <p class="mt-1 text-base font-semibold text-(--app-text-strong)">本地优先</p>
            </div>
            <div class="rounded-[20px] border border-(--app-border) bg-(--app-panel-muted)/65 px-4 py-3">
              <p class="text-xs text-(--app-text-faint)">运行</p>
              <p class="mt-1 text-base font-semibold text-(--app-text-strong)">可追踪</p>
            </div>
          </div>
        </SurfacePanel>

        <SurfacePanel class="space-y-4">
          <div class="app-stat">
            <p class="app-stat-label">版本</p>
            <p class="app-stat-value">0.1.0</p>
          </div>
          <div class="app-stat">
            <p class="app-stat-label">技术栈</p>
            <p class="app-stat-value text-base">Vue 3 · TypeScript · Tauri</p>
          </div>
          <div class="app-stat">
            <p class="app-stat-label">定位</p>
            <p class="app-stat-value text-base">多设备自动化工作台</p>
          </div>
        </SurfacePanel>
      </div>

      <SurfacePanel class="mt-4 space-y-4">
        <div class="flex flex-wrap items-start justify-between gap-4">
          <div>
            <p class="text-sm font-semibold text-(--app-text-strong)">关于与更新</p>
          </div>
          <button class="app-button app-button-primary shadow-lg shadow-(--app-accent-soft)" type="button" :disabled="appUpdateState.phase === 'checking'" @click="checkUpdate">
            <AppIcon name="refresh-cw" :size="16" />
            {{ appUpdateState.phase === 'checking' ? '检查中' : '检查更新' }}
          </button>
        </div>

        <div class="grid gap-3 md:grid-cols-2">
          <div class="rounded-[20px] border border-(--app-border) p-4">
            <p class="text-xs uppercase tracking-[0.16em] text-(--app-text-faint)">当前版本</p>
            <p class="mt-2 text-lg font-semibold text-(--app-text-strong)">AutoDaily 0.1.0</p>
          </div>
          <div class="rounded-[20px] border border-(--app-border) p-4">
            <p class="text-xs uppercase tracking-[0.16em] text-(--app-text-faint)">更新状态</p>
            <p class="mt-2 text-sm text-(--app-text-strong)">{{ updateStatusLabel }}</p>
          </div>
        </div>

        <div class="rounded-[20px] border border-(--app-border) p-4">
          <MarkdownView :content="latestReleaseNotes" :empty-text="latestReleaseNotesLoading ? '正在加载当前版本日志...' : '当前还没有拉取更新说明。'" />
        </div>

        <div class="flex flex-wrap justify-end gap-3">
          <button class="app-button app-button-primary" type="button" @click="openProductFeedback">
            <AppIcon name="message-circle-warning" :size="16" />
            反馈问题
          </button>
          <button class="app-button app-button-ghost" type="button" @click="openCurrentReleaseNotes">
            <AppIcon name="file-clock" :size="16" />
            当前版本日志
          </button>
          <button class="app-button app-button-ghost" type="button" @click="openFullChangelog">
            <AppIcon name="scroll-text" :size="16" />
            完整更新日志
          </button>
        </div>
      </SurfacePanel>
    </div>

    <MarkdownDocumentDialog
      :open="documentDialogOpen"
      :title="documentDialogTitle"
      :description="documentDialogDescription"
      :content="documentDialogContent"
      :loading="documentDialogLoading"
      :error="documentDialogError"
      @close="documentDialogOpen = false"
    />
    <SupportSubmissionDialog
      :open="feedbackDialogOpen"
      mode="product-feedback"
      @close="feedbackDialogOpen = false"
      @submitted="handleFeedbackSubmitted"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { Command } from '@lucide/vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppPageHeader from '@/components/shared/AppPageHeader.vue';
import MarkdownDocumentDialog from '@/components/shared/MarkdownDocumentDialog.vue';
import MarkdownView from '@/components/shared/MarkdownView.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import SupportSubmissionDialog from '@/components/support/SupportSubmissionDialog.vue';
import type { SupportSubmissionResult } from '@/services/supportService';
import { useUserStore } from '@/store/user';
import {
  appUpdateState,
  checkForAppUpdate,
} from '@/services/appUpdateService';
import {
  APP_FULL_CHANGELOG_URL,
  APP_LATEST_RELEASE_NOTES_URL,
  fetchMarkdownDocument,
} from '@/services/changelogService';
import { formatDate } from '@/utils/presenters';
import { showToast } from '@/utils/toast';

const latestReleaseNotes = ref('');
const latestReleaseNotesLoading = ref(false);
const documentDialogOpen = ref(false);
const documentDialogTitle = ref('');
const documentDialogDescription = ref('');
const documentDialogContent = ref('');
const documentDialogLoading = ref(false);
const documentDialogError = ref('');
const feedbackDialogOpen = ref(false);
const userStore = useUserStore();

const updateStatusLabel = computed(() => {
  if (appUpdateState.phase === 'checking') {
    return '正在检查更新...';
  }
  if (appUpdateState.phase === 'available') {
    return appUpdateState.date
      ? `${appUpdateState.version} · ${formatDate(appUpdateState.date)}`
      : `发现新版本 ${appUpdateState.version}`;
  }
  if (appUpdateState.phase === 'error') {
    console.log("updateStatusLabel:"+appUpdateState.error)
    return '检查更新失败，请稍后重试。';
  }
  if (appUpdateState.hasChecked) {
    return '已检查，当前已是最新版本';
  }
  return '尚未检查';
});

async function loadLatestReleaseNotes(showError = false) {
  latestReleaseNotesLoading.value = true;
  try {
    latestReleaseNotes.value = await fetchMarkdownDocument(APP_LATEST_RELEASE_NOTES_URL);
  } catch (error) {
    latestReleaseNotes.value = '';
    if (showError) {
      showToast(error instanceof Error ? error.message : '加载当前版本日志失败', 'error');
    }
  } finally {
    latestReleaseNotesLoading.value = false;
  }
}

async function openDocument(title: string, descriptionText: string, url: string) {
  documentDialogTitle.value = title;
  documentDialogDescription.value = descriptionText;
  documentDialogContent.value = '';
  documentDialogError.value = '';
  documentDialogLoading.value = true;
  documentDialogOpen.value = true;
  try {
    documentDialogContent.value = await fetchMarkdownDocument(url);
  } catch (error) {
    documentDialogError.value = error instanceof Error ? error.message : '加载更新日志失败';
    showToast(documentDialogError.value, 'error');
  } finally {
    documentDialogLoading.value = false;
  }
}

const openCurrentReleaseNotes = () =>
  openDocument('当前版本日志', '显示最新版本的发布说明。', APP_LATEST_RELEASE_NOTES_URL);

const openFullChangelog = () =>
  openDocument('完整更新日志', '显示应用完整版本历史。', APP_FULL_CHANGELOG_URL);

async function checkUpdate() {
  //const result = await checkForAppUpdate();
  await checkForAppUpdate();
  await loadLatestReleaseNotes(true);
  if (appUpdateState.phase === 'error') {
    console.log("checkUpdate: "+appUpdateState.error);
    //showToast('检查失败，请稍候重试。', 'error');
    return;
  }
  //showToast(result ? `发现版本 ${result.version}` : '当前已是最新版本', 'success');
}

async function openProductFeedback() {
  const profile = await userStore.ensureProfileForAction('反馈 AutoDaily 问题');
  if (!profile) {
    if (!userStore.authSession) userStore.openAuthModal();
    showToast('请先登录后再提交反馈', 'warning');
    return;
  }
  feedbackDialogOpen.value = true;
}

function handleFeedbackSubmitted(result: SupportSubmissionResult) {
  feedbackDialogOpen.value = false;
  const attachmentHint = result.failedScreenshots ? `，其中 ${result.failedScreenshots} 张截图上传失败` : '';
  showToast(`反馈已提交，编号 ${result.id}${attachmentHint}`, result.failedScreenshots ? 'warning' : 'success', 5000);
}

onMounted(() => {
  void loadLatestReleaseNotes(false);
});
</script>
