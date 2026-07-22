<template>
  <AppDialog
    :open="open"
    :title="dialogTitle"
    width-class="max-w-2xl max-h-[calc(100vh-3rem)]"
    @close="requestClose"
  >
    <form class="flex min-h-0 flex-1 flex-col" @submit.prevent="submit">
      <div class="min-h-0 flex-1 space-y-4 overflow-y-auto pr-1 custom-scrollbar">
        <div v-if="script" class="rounded-[18px] border border-(--app-border) bg-(--app-panel-muted)/65 p-4">
          <p class="mt-1 font-semibold text-(--app-text-strong)">{{ script.name }}</p>
          <p class="mt-1 text-xs text-(--app-text-soft)">作者：{{ script.authorName || '未知作者' }}</p>
        </div>

        <label class="support-field">
          <span class="support-label">{{ mode === 'report' ? '举报原因' : '反馈类型' }}</span>
          <AppSelect v-model="form.category" :options="categoryOptions" :disabled="submitting" />
        </label>

        <label v-if="mode !== 'report'" class="support-field">
          <span class="support-label">标题</span>
          <input v-model.trim="form.title" class="app-input" maxlength="120" placeholder="用一句话描述问题" :disabled="submitting" />
        </label>

        <label class="support-field">
          <span class="support-label">具体说明</span>
          <textarea
            v-model.trim="form.description"
            class="app-input min-h-28 resize-y"
            :maxlength="mode === 'report' ? 1000 : 2000"
            :placeholder="descriptionPlaceholder"
            :disabled="submitting"
          />
          <span class="text-right text-xs text-(--app-text-faint)">{{ form.description.length }} / {{ mode === 'report' ? 1000 : 2000 }}</span>
        </label>

        <template v-if="mode === 'product-feedback'">
          <label class="support-field"><span class="support-label">复现步骤（可选）</span><textarea v-model.trim="form.reproductionSteps" class="app-input min-h-20 resize-y" maxlength="2000" placeholder="按顺序写下出现问题前做了什么" :disabled="submitting" /></label>
          <div class="grid gap-4 md:grid-cols-2">
            <label class="support-field"><span class="support-label">期望结果（可选）</span><textarea v-model.trim="form.expectedBehavior" class="app-input min-h-20 resize-y" maxlength="1000" :disabled="submitting" /></label>
            <label class="support-field"><span class="support-label">实际结果（可选）</span><textarea v-model.trim="form.actualBehavior" class="app-input min-h-20 resize-y" maxlength="1000" :disabled="submitting" /></label>
          </div>
        </template>

        <div class="support-field">
          <div class="flex flex-wrap items-center justify-between gap-3">
            <div><p class="support-label">截图（可选）</p><p class="mt-1 text-xs text-(--app-text-faint)">支持JPEG/PNG，最多 5 张</p></div>
            <button class="app-button app-button-ghost" type="button" :disabled="submitting || screenshots.length >= 5" @click="chooseScreenshots"><AppIcon name="image-plus" :size="15" />选择截图</button>
          </div>
          <div v-if="screenshots.length" class="space-y-2">
            <div v-for="path in screenshots" :key="path" class="flex items-center justify-between gap-3 rounded-xl bg-(--app-panel-muted) px-3 py-2 text-xs">
              <span class="min-w-0 truncate">{{ fileName(path) }}</span>
              <button class="app-icon-button h-7 w-7" type="button" title="移除截图" :disabled="submitting" @click="removeScreenshot(path)"><AppIcon name="x" :size="14" /></button>
            </div>
          </div>
        </div>

      </div>

      <div class="mt-5 border-t border-(--app-border) pt-4">
        <p v-if="errorMessage" role="alert" aria-live="assertive" class="mb-3 rounded-xl border border-red-300/60 bg-red-50 px-4 py-3 text-sm text-red-800">{{ errorMessage }}</p>
        <p v-if="privacyHint" class="mb-2 text-xs text-(--app-text-faint)">{{ privacyHint }}</p>
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div aria-live="polite" class="flex min-h-5 items-center gap-2 text-xs" :class="submissionStatusTone">
            <AppIcon v-if="submitting" name="loader-circle" :size="14" class="app-loading-spinner" />
            <AppIcon v-else-if="canSubmit" name="circle-check" :size="14" />
            <span>{{ submissionStatus }}</span>
          </div>
          <div class="flex gap-2"><button class="app-button app-button-ghost" type="button" :disabled="submitting" @click="requestClose">取消</button><button class="app-button app-button-primary min-w-30" type="submit" :disabled="submitting"><AppIcon v-if="submitting" name="loader-circle" :size="15" class="app-loading-spinner" />{{ submitting ? submittingLabel : submitLabel }}</button></div>
        </div>
      </div>
    </form>
  </AppDialog>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue';
import { open as openFileDialog } from '@tauri-apps/plugin-dialog';
import AppDialog from '@/components/shared/AppDialog.vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import { supportService, type SupportDialogMode, type SupportScriptContext, type SupportSubmissionResult } from '@/services/supportService';
import { showToast } from '@/utils/toast';

const props = defineProps<{ open: boolean; mode: SupportDialogMode; script?: SupportScriptContext | null }>();
const emit = defineEmits<{ close: []; submitted: [result: SupportSubmissionResult] }>();
const submitting = ref(false);
const errorMessage = ref('');
const screenshots = ref<string[]>([]);
const form = reactive({ category: '', title: '', description: '', reproductionSteps: '', expectedBehavior: '', actualBehavior: '' });

const reportOptions = [
  { label: '恶意或危险行为', value: 'malicious_code' }, { label: '诈骗或诱导付费', value: 'fraud' },
  { label: '侵权或盗用', value: 'copyright' }, { label: '违法或不当内容', value: 'inappropriate_content' },
  { label: '描述严重不符', value: 'misleading' }, { label: '隐私问题', value: 'privacy' },
  { label: '完全无法使用', value: 'broken' }, { label: '垃圾或重复内容', value: 'spam' }, { label: '其它', value: 'other' },
];
const feedbackOptions = [
  { label: '程序错误', value: 'bug' }, { label: '功能建议', value: 'feature_request' },
  { label: '使用体验', value: 'usability' }, { label: '性能问题', value: 'performance' },
  { label: '兼容性问题', value: 'compatibility' }, { label: '其它', value: 'other' },
];
const categoryOptions = computed(() => props.mode === 'report' ? reportOptions : feedbackOptions);
const dialogTitle = computed(() => props.mode === 'report' ? '举报脚本' : props.mode === 'script-feedback' ? '反馈脚本问题' : '反馈 AutoDaily 问题');
const descriptionPlaceholder = computed(() => props.mode === 'report' ? '请说明你发现的问题和判断依据（至少 10 字）' : '请说明问题发生时的情况（至少 10 字）');
const privacyHint = computed(() => props.mode === 'report' ? '' : '提交前请检查截图中是否包含账号等隐私信息。');
const submitLabel = computed(() => props.mode === 'report' ? '提交举报' : '提交反馈');
const validationMessage = computed(() => {
  if (props.mode !== 'product-feedback' && !props.script?.cloudId) return '缺少云端脚本 ID，无法提交。请刷新脚本列表后重试。';
  if (!form.category) return props.mode === 'report' ? '请选择举报原因。' : '请选择反馈类型。';
  if (props.mode !== 'report' && form.title.length < 4) return `标题至少需要 4 个字，还差 ${4 - form.title.length} 个。`;
  const minimum = props.mode === 'report' ? 10 : 10;
  if (form.description.length < minimum) return `具体说明至少需要 ${minimum} 个字，还差 ${minimum - form.description.length} 个。`;
  return '';
});
const canSubmit = computed(() => !validationMessage.value);
const submittingLabel = computed(() => props.mode === 'report' ? '正在提交举报…' : '正在提交反馈…');
const submissionStatus = computed(() => submitting.value ? `${submittingLabel.value}请勿重复操作。` : canSubmit.value ? '内容已填写完整，可以提交。' : validationMessage.value);
const submissionStatusTone = computed(() => submitting.value ? 'text-(--app-accent)' : canSubmit.value ? 'text-emerald-700' : 'text-(--app-text-faint)');

function reset() { Object.assign(form, { category: categoryOptions.value[0]?.value || '', title: '', description: '', reproductionSteps: '', expectedBehavior: '', actualBehavior: '' }); screenshots.value = []; errorMessage.value = ''; }
function requestClose() { if (!submitting.value) emit('close'); }
function fileName(path: string) { return path.split(/[\\/]/).pop() || path; }
function removeScreenshot(path: string) { screenshots.value = screenshots.value.filter((item) => item !== path); }
async function chooseScreenshots() { const selected = await openFileDialog({ multiple: true, filters: [{ name: '截图', extensions: ['png', 'jpg', 'jpeg'] }] }); const paths = Array.isArray(selected) ? selected : selected ? [selected] : []; screenshots.value = [...new Set([...screenshots.value, ...paths])].slice(0, 5); }
async function submit() {
  if (submitting.value) return;
  if (!canSubmit.value) { errorMessage.value = validationMessage.value; return; }
  submitting.value = true; errorMessage.value = '';
  try {
    const result = props.mode === 'report'
      ? await supportService.reportScript(props.script!.cloudId, { category: form.category, description: form.description, screenshotPaths: screenshots.value })
      : await supportService.createFeedback({ targetType: props.mode === 'script-feedback' ? 'script' : 'product', scriptId: props.mode === 'script-feedback' ? props.script!.cloudId : null, category: form.category, title: form.title, description: form.description, reproductionSteps: form.reproductionSteps || null, expectedBehavior: form.expectedBehavior || null, actualBehavior: form.actualBehavior || null, runtimeType: props.script?.runtimeType || null, screenshotPaths: screenshots.value });
    emit('submitted', result);
  } catch (error) {
    const rawMessage = error instanceof Error ? error.message : typeof error === 'string' ? error : '';
    const message = rawMessage === 'Unauthorized' ? '登录状态已失效，请重新登录后再提交。' : rawMessage || '提交失败，请稍后重试。';
    errorMessage.value = message;
    showToast(message, 'error', 5000);
  }
  finally { submitting.value = false; }
}
watch(() => [props.open, props.mode, props.script?.cloudId], ([open]) => { if (open) reset(); }, { immediate: true });
watch(() => [form.category, form.title, form.description], () => { if (!submitting.value) errorMessage.value = ''; });
</script>

<style scoped>
.support-field { display: grid; gap: 0.55rem; }
.support-label { font-size: 0.875rem; font-weight: 600; color: var(--app-text-strong); }
</style>
