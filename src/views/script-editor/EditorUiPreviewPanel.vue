<template>
  <div
    class="editor-ui-preview-shell"
    :class="{
      'editor-ui-preview-shell-embedded': embedded,
      'editor-ui-preview-shell-active': active,
      'editor-ui-preview-shell-readonly': readonly,
    }"
  >
    <div class="editor-ui-preview-flow" :style="{ paddingLeft: `${Math.max(0, indentLevel ?? 0) * 1.05}rem` }">
      <span class="editor-ui-tone-bar" :class="toneBarClass" />

      <label v-if="showEnabledToggle" class="editor-ui-toggle-chip" @click.stop>
        <input
          type="checkbox"
          :checked="taskEnabledPreview"
          :disabled="readonly"
          data-testid="editor-ui-preview-task-enabled"
          @click.stop
          @change="handleTaskEnabledEvent"
        />
      </label>

      <span class="editor-ui-task-name" :class="taskToneClass">{{ taskName }}</span>

      <div class="editor-ui-preview-fields">
        <div
          v-for="(field, index) in uiSchema.fields"
          :key="field.id"
          class="editor-ui-preview-item"
          :class="{ 'editor-ui-preview-item-active': !readonly && selectedUiFieldId === field.id }"
          @click="handleSelectField(field.id)"
        >
          <template v-if="field.control === 'checkbox'">
            <label
              v-if="field.checkboxStyle === 'switch'"
              class="editor-ui-switch"
              :class="{ 'editor-ui-switch-disabled': !isInteractive(field) }"
              @click.stop
            >
              <input
                type="checkbox"
                class="sr-only"
                :checked="Boolean(resolveFieldPreviewValue(field))"
                :disabled="!isInteractive(field)"
                :data-testid="`editor-ui-preview-control-${index}`"
                @change="updatePreviewBoolean(field, ($event.target as HTMLInputElement).checked)"
              />
              <span class="editor-ui-switch-track">
                <span class="editor-ui-switch-thumb" />
              </span>
            </label>
            <input
              v-else
              type="checkbox"
              :checked="Boolean(resolveFieldPreviewValue(field))"
              :disabled="!isInteractive(field)"
              :data-testid="`editor-ui-preview-control-${index}`"
              @click.stop
              @change="updatePreviewBoolean(field, ($event.target as HTMLInputElement).checked)"
            />
          </template>

          <template v-else-if="field.control === 'number'">
            <input
              :value="resolveNumberPreviewValue(field)"
              class="editor-ui-inline-control editor-ui-inline-control-number"
              type="number"
              :disabled="!isInteractive(field)"
              :data-testid="`editor-ui-preview-control-${index}`"
              @click.stop
              @focus="handleSelectField(field.id)"
              @input="updatePreviewText(field, ($event.target as HTMLInputElement).value)"
            />
          </template>

          <template v-else-if="field.control === 'select'">
            <div class="editor-ui-inline-select-shell" @click.stop>
              <EditorSelectField
                :model-value="resolveSelectPreviewValue(field)"
                :options="getPreviewOptionsForField(field)"
                placeholder="请选择"
                :disabled="!isInteractive(field)"
                :test-id="`editor-ui-preview-control-${index}`"
                @update:model-value="updatePreviewText(field, String($event ?? ''))"
              />
            </div>
          </template>

          <template v-else-if="field.control === 'slider'">
            <div class="editor-ui-slider-shell" @click.stop>
              <input
                :value="resolveNumberPreviewValue(field)"
                class="editor-ui-slider"
                type="range"
                :min="getSliderMin(field)"
                :max="getSliderMax(field)"
                :step="getSliderStep(field)"
                :disabled="!isInteractive(field)"
                :data-testid="`editor-ui-preview-control-${index}`"
                @input="updateSliderValue(field, ($event.target as HTMLInputElement).value)"
              />
              <span class="editor-ui-slider-value">{{ resolveNumberPreviewValue(field) }}</span>
            </div>
          </template>

          <template v-else-if="field.control === 'radio'">
            <span class="editor-ui-inline-options">
              <label
                v-for="option in parseFieldOptions(field)"
                :key="option"
                class="editor-ui-inline-pill editor-ui-inline-radio"
                :class="{ 'editor-ui-inline-pill-active': resolveFieldPreviewValue(field) === option }"
                @click.stop
              >
                <input
                  type="radio"
                  class="sr-only"
                  :checked="resolveFieldPreviewValue(field) === option"
                  :disabled="!isInteractive(field)"
                  :data-testid="`editor-ui-preview-control-${index}`"
                  @focus="handleSelectField(field.id)"
                  @change="updatePreviewText(field, option)"
                />
                <span class="editor-ui-radio-dot" />
                {{ option }}
              </label>
            </span>
          </template>

          <template v-else>
            <input
              v-if="field.editable"
              :value="resolveTextPreviewValue(field)"
              class="editor-ui-inline-control editor-ui-inline-control-text"
              type="text"
              :disabled="!isInteractive(field)"
              :data-testid="`editor-ui-preview-control-${index}`"
              @click.stop
              @focus="handleSelectField(field.id)"
              @input="updatePreviewText(field, ($event.target as HTMLInputElement).value)"
            />
            <span v-else class="editor-ui-static-text">{{ resolveTextPreviewValue(field) }}</span>
          </template>
        </div>

      </div>

      <div class="editor-ui-task-cycle-shell" @click.stop>
        <template v-if="showTaskCycle">
          <EditorSelectField
            v-if="editableCycle"
            :model-value="selectedTaskCycleValue"
            :options="taskCycleOptions"
            placeholder="选择默认周期"
            test-id="editor-ui-preview-task-cycle"
            @update:model-value="$emit('update:default-task-cycle-value', String($event || 'everyRun'))"
          />
          <span v-else class="editor-ui-task-cycle">{{ taskCycleLabel }}</span>
          <input
            v-if="editableCycle && (defaultTaskCycleMode === 'weekDay' || defaultTaskCycleMode === 'monthDay')"
            :value="defaultTaskCycleDay"
            class="editor-ui-cycle-day-input"
            type="number"
            :min="1"
            :max="defaultTaskCycleMode === 'weekDay' ? 7 : 31"
            @input="$emit('update:default-task-cycle-day', Number(($event.target as HTMLInputElement).value || 1))"
          />
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { TaskCycle } from '@/types/bindings/TaskCycle';
import type { TaskTone } from '@/types/bindings/TaskTone';
import { formatTaskCycleLabel } from '@/utils/presenters';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import type { EditorUiSchema, EditorUiField } from '@/views/script-editor/editorSchema';
import { taskCycleOptions } from '@/views/script-editor/editorTaskMeta';
import type { EditorInputEntry } from '@/views/script-editor/editorVariables';
import { findBoundInputEntry, parseFieldOptions, resolvePreviewValue } from '@/views/script-editor/editorUiPreview';

defineOptions({ name: 'EditorUiPreviewPanel' });

const props = withDefaults(defineProps<{
  taskName: string;
  defaultTaskCycle: TaskCycle;
  defaultTaskCycleValue?: string;
  defaultTaskCycleMode?: 'named' | 'weekDay' | 'monthDay';
  defaultTaskCycleDay?: number;
  editableCycle?: boolean;
  showEnabledToggle: boolean;
  defaultEnabled: boolean;
  taskTone: TaskTone;
  requireBoundInput?: boolean;
  showTaskCycle?: boolean;
  embedded?: boolean;
  readonly?: boolean;
  active?: boolean;
  indentLevel?: number;
  uiSchema: EditorUiSchema;
  selectedUiFieldId: string | null;
  inputEntries: EditorInputEntry[];
}>(), {
  requireBoundInput: false,
  showTaskCycle: true,
});

const emit = defineEmits<{
  'select-ui-field': [fieldId: string];
  'update-input': [entryId: string, field: 'stringValue' | 'booleanValue', value: string | boolean];
  'update:default-enabled': [value: boolean];
  'update:default-task-cycle-value': [value: string];
  'update:default-task-cycle-day': [value: number];
}>();

const taskEnabledPreview = ref(props.defaultEnabled);
const localPreviewValues = ref<Record<string, string | boolean>>({});
const taskCycleLabel = computed(() => formatTaskCycleLabel(props.defaultTaskCycle));
const selectedTaskCycleValue = computed(() => props.defaultTaskCycleValue ?? 'everyRun');
const taskToneClass = computed(() => {
  if (props.taskTone === 'warning') return 'editor-ui-task-name-warning';
  if (props.taskTone === 'danger') return 'editor-ui-task-name-danger';
  return '';
});
const toneBarClass = computed(() => {
  if (props.taskTone === 'warning') return 'editor-ui-tone-bar-warning';
  if (props.taskTone === 'danger') return 'editor-ui-tone-bar-danger';
  return 'editor-ui-tone-bar-normal';
});

const findPreviewEntry = (field: EditorUiField) => findBoundInputEntry(field, props.inputEntries);

const handleSelectField = (fieldId: string) => {
  if (props.readonly) {
    return;
  }
  emit('select-ui-field', fieldId);
};

const resolveFieldPreviewValue = (field: EditorUiField) => {
  const entry = findPreviewEntry(field);
  if (entry && entry.namespace === 'input') {
    return resolvePreviewValue(field, props.inputEntries);
  }

  return localPreviewValues.value[field.id] ?? null;
};

const getPreviewOptionsForField = (field: EditorUiField) => {
  const options = parseFieldOptions(field).map((option) => ({
    label: option,
    value: option,
  }));
  if (options.length) {
    return options;
  }

  const preview = resolveSelectPreviewValue(field);
  return [{ label: preview || '请选择', value: preview || '请选择' }];
};

const resolveNumberPreviewValue = (field: EditorUiField) => {
  const value = resolveFieldPreviewValue(field);
  return value === null || value === undefined || value === '' ? '0' : String(value);
};

const resolveSliderMode = (field: EditorUiField) => {
  const entry = findPreviewEntry(field);
  if (entry?.type === 'float') {
    return 'float' as const;
  }
  if (entry?.type === 'int') {
    return 'int' as const;
  }

  if (!Number.isInteger(field.min) || !Number.isInteger(field.max) || !Number.isInteger(field.step)) {
    return 'float' as const;
  }

  return 'int' as const;
};

const getSliderMin = (field: EditorUiField) => (resolveSliderMode(field) === 'float' ? field.min : Math.round(field.min));
const getSliderMax = (field: EditorUiField) => (resolveSliderMode(field) === 'float' ? field.max : Math.round(field.max));
const getSliderStep = (field: EditorUiField) =>
  resolveSliderMode(field) === 'float' ? field.step || 0.01 : Math.max(1, Math.round(field.step || 1));

const normalizeSliderValue = (field: EditorUiField, rawValue: string) => {
  const parsed = Number(rawValue);
  if (!Number.isFinite(parsed)) {
    return resolveSliderMode(field) === 'float' ? String(getSliderMin(field)) : String(Math.round(getSliderMin(field)));
  }

  const clamped = Math.min(getSliderMax(field), Math.max(getSliderMin(field), parsed));
  return resolveSliderMode(field) === 'float' ? String(clamped) : String(Math.round(clamped));
};

const resolveSelectPreviewValue = (field: EditorUiField) => {
  const value = resolveFieldPreviewValue(field);
  const options = parseFieldOptions(field);
  if (value !== null && value !== undefined && String(value).trim()) {
    return String(value);
  }
  return options[0] ?? '请选择';
};

const resolveTextPreviewValue = (field: EditorUiField) => {
  const value = resolveFieldPreviewValue(field);
  if (value !== null && value !== undefined && String(value).trim()) {
    return String(value);
  }
  return field.placeholder || field.description || '';
};

const isInteractive = (field: EditorUiField) => {
  if (props.readonly) {
    return false;
  }

  if (props.requireBoundInput) {
    const entry = findPreviewEntry(field);
    if (!entry || entry.namespace !== 'input') {
      return false;
    }
  }

  return field.control === 'text' ? field.editable : true;
};

const updatePreviewText = (field: EditorUiField, value: string) => {
  if (props.readonly) {
    return;
  }

  const entry = findPreviewEntry(field);
  if (!entry || entry.namespace !== 'input') {
    localPreviewValues.value = {
      ...localPreviewValues.value,
      [field.id]: value,
    };
    return;
  }

  emit('select-ui-field', field.id);
  emit('update-input', entry.id, 'stringValue', value);
};

const updatePreviewBoolean = (field: EditorUiField, value: boolean) => {
  if (props.readonly) {
    return;
  }

  const entry = findPreviewEntry(field);
  if (!entry || entry.namespace !== 'input') {
    localPreviewValues.value = {
      ...localPreviewValues.value,
      [field.id]: value,
    };
    return;
  }

  emit('select-ui-field', field.id);
  emit('update-input', entry.id, 'booleanValue', value);
};

const updateSliderValue = (field: EditorUiField, value: string) => {
  const normalized = normalizeSliderValue(field, value);
  updatePreviewText(field, normalized);
};

const updateTaskEnabled = (value: boolean) => {
  if (props.readonly) {
    return;
  }
  taskEnabledPreview.value = value;
  emit('update:default-enabled', value);
};

const handleTaskEnabledEvent = (event: Event) => {
  updateTaskEnabled((event.target as HTMLInputElement).checked);
};

watch(
  () => props.defaultEnabled,
  (value) => {
    taskEnabledPreview.value = value;
  },
  { immediate: true },
);
</script>

<style scoped>
.editor-ui-preview-shell {
  border-radius: 16px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.72);
  padding: 0.82rem 1rem;
  transition: border-color 0.16s ease, background 0.16s ease, box-shadow 0.16s ease;
}

.editor-ui-preview-shell-active {
  border-color: color-mix(in srgb, var(--app-accent) 34%, var(--app-border));
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--app-accent) 18%, transparent);
}

.editor-ui-preview-shell-readonly:hover {
  background: rgba(255, 255, 255, 0.8);
}

.editor-ui-preview-shell-embedded {
  width: 100%;
}

.editor-ui-preview-flow {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 0.7rem;
}

.editor-ui-tone-bar {
  width: 0.28rem;
  flex: 0 0 0.28rem;
  align-self: stretch;
  border-radius: 999px;
}

.editor-ui-tone-bar-normal {
  background: rgba(148, 163, 184, 0.42);
}

.editor-ui-tone-bar-warning {
  background: rgba(245, 158, 11, 0.9);
}

.editor-ui-tone-bar-danger {
  background: rgba(239, 68, 68, 0.92);
}

.editor-ui-toggle-chip {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 42px;
  height: 42px;
  flex: 0 0 42px;
  border-radius: 14px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.88);
}

.editor-ui-task-name {
  flex: 0 0 auto;
  color: var(--app-text-strong);
  font-size: 1rem;
  font-weight: 700;
}

.editor-ui-task-name-warning {
  color: #a16207;
}

.editor-ui-task-name-danger {
  color: #b91c1c;
}

.editor-ui-preview-fields {
  display: flex;
  min-width: 0;
  flex: 1 1 auto;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.55rem;
}

.editor-ui-preview-item {
  display: inline-flex;
  align-items: center;
  gap: 0.55rem;
  min-height: 42px;
  min-width: 0;
  border-radius: 14px;
  border: 1px solid transparent;
  background: transparent;
  padding: 0.16rem 0.28rem;
  transition: border-color 0.16s ease, background 0.16s ease;
}

.editor-ui-preview-item-active {
  border-color: var(--app-state-active-border);
  background: var(--app-state-active-bg);
}

.editor-ui-inline-control {
  min-width: 72px;
  border-radius: 12px;
  border: 1px solid var(--app-border);
  background: white;
  padding: 0.45rem 0.75rem;
  color: var(--app-text-strong);
  appearance: none;
  outline: none;
}

.editor-ui-inline-control:disabled {
  cursor: not-allowed;
  background: rgba(255, 255, 255, 0.72);
  color: var(--app-text-soft);
}

.editor-ui-inline-control-number {
  width: 92px;
  min-width: 92px;
}

.editor-ui-inline-control-text {
  width: 132px;
  min-width: 132px;
}

.editor-ui-inline-select-shell {
  min-width: 132px;
}

.editor-ui-inline-options {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 0.45rem;
}

.editor-ui-inline-pill {
  border-radius: 999px;
  border: 1px solid var(--app-border);
  background: white;
  padding: 0.3rem 0.7rem;
  font-size: 0.75rem;
  color: var(--app-text-soft);
  cursor: pointer;
}

.editor-ui-inline-radio {
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
}

.editor-ui-radio-dot {
  width: 0.7rem;
  height: 0.7rem;
  border-radius: 999px;
  border: 1px solid var(--app-border-strong);
  background: white;
  box-shadow: inset 0 0 0 2px white;
}

.editor-ui-inline-pill-active {
  border-color: var(--app-state-active-border);
  background: var(--app-state-active-bg);
  color: var(--app-text-strong);
}

.editor-ui-inline-pill-active .editor-ui-radio-dot {
  border-color: var(--app-accent);
  background: var(--app-accent);
}

.editor-ui-switch {
  display: inline-flex;
  align-items: center;
}

.editor-ui-switch-track {
  position: relative;
  display: inline-flex;
  width: 42px;
  height: 24px;
  align-items: center;
  border-radius: 999px;
  background: rgba(148, 163, 184, 0.38);
  transition: background 0.16s ease;
}

.editor-ui-switch-thumb {
  width: 18px;
  height: 18px;
  margin-left: 3px;
  border-radius: 999px;
  background: white;
  box-shadow: 0 2px 6px rgba(15, 23, 42, 0.18);
  transition: transform 0.16s ease;
}

.editor-ui-switch input:checked + .editor-ui-switch-track {
  background: color-mix(in srgb, var(--app-accent) 72%, white);
}

.editor-ui-switch input:checked + .editor-ui-switch-track .editor-ui-switch-thumb {
  transform: translateX(18px);
}

.editor-ui-switch-disabled {
  opacity: 0.62;
}

.editor-ui-slider-shell {
  display: inline-flex;
  align-items: center;
  gap: 0.6rem;
  min-width: 200px;
}

.editor-ui-slider {
  width: 148px;
  accent-color: var(--app-accent);
}

.editor-ui-slider-value {
  min-width: 2.75rem;
  color: var(--app-text-strong);
  font-size: 0.85rem;
  font-weight: 600;
  text-align: center;
}

.editor-ui-static-text {
  color: var(--app-text-strong);
  font-size: 0.95rem;
  font-weight: 600;
  line-height: 1.2;
}

.editor-ui-task-cycle-shell {
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  gap: 0.55rem;
}

.editor-ui-task-cycle {
  display: inline-flex;
  align-items: center;
  border-radius: 999px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.7);
  padding: 0.22rem 0.55rem;
  font-size: 0.72rem;
  color: var(--app-text-faint);
}

.editor-ui-cycle-day-input {
  width: 74px;
  border-radius: 12px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.84);
  padding: 0.55rem 0.65rem;
  color: var(--app-text-strong);
}
</style>
