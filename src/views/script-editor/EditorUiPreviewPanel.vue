<template>
  <div
    class="rounded-[22px] border border-[var(--app-border)] bg-[linear-gradient(160deg,rgba(255,255,255,0.92),rgba(243,247,255,0.9))] px-5 py-5 shadow-[var(--app-shadow-soft)]"
  >
    <div class="space-y-3">
      <div v-if="uiSchema.layout === 'horizontal'" class="editor-ui-preview-flow">
        <label class="editor-ui-toggle-chip">
          <input
            type="checkbox"
            :checked="taskEnabledPreview"
            data-testid="editor-ui-preview-task-enabled"
            @change="taskEnabledPreview = ($event.target as HTMLInputElement).checked"
          />
        </label>
        <span class="editor-ui-task-name">{{ taskName }}</span>
        <div
          v-for="(field, index) in uiSchema.fields"
          :key="field.id"
          class="editor-ui-preview-item"
          :class="{ 'editor-ui-preview-item-active': selectedUiFieldId === field.id }"
          @click="$emit('select-ui-field', field.id)"
        >
          <template v-if="field.control === 'checkbox'">
            <label v-if="field.checkboxStyle === 'switch'" class="editor-ui-switch" :class="{ 'editor-ui-switch-disabled': !isInteractive(field) }" @click.stop>
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
              @focus="$emit('select-ui-field', field.id)"
              @input="updatePreviewText(field, ($event.target as HTMLInputElement).value)"
            />
          </template>

          <template v-else-if="field.control === 'select'">
            <div class="editor-ui-inline-select-shell" @click.stop>
              <AppSelect
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
                  @focus="$emit('select-ui-field', field.id)"
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
              @focus="$emit('select-ui-field', field.id)"
              @input="updatePreviewText(field, ($event.target as HTMLInputElement).value)"
            />
            <span v-else class="editor-ui-static-text">{{ resolveTextPreviewValue(field) }}</span>
          </template>
        </div>
      </div>

      <template v-else>
        <div class="flex flex-wrap items-center gap-3">
          <label class="editor-ui-toggle-chip">
            <input
              type="checkbox"
              :checked="taskEnabledPreview"
              data-testid="editor-ui-preview-task-enabled"
              @change="taskEnabledPreview = ($event.target as HTMLInputElement).checked"
            />
          </label>
          <span class="editor-ui-task-name">{{ taskName }}</span>
          <button
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            @click="uiPreviewExpanded = !uiPreviewExpanded"
          >
            {{ uiPreviewExpanded ? '收起' : '展开' }}
          </button>
        </div>

        <div v-if="uiPreviewExpanded" class="grid gap-3">
          <div
            v-for="(field, index) in uiSchema.fields"
            :key="field.id"
            class="editor-ui-preview-item"
            :class="{ 'editor-ui-preview-item-active': selectedUiFieldId === field.id, 'editor-ui-preview-item-vertical': true }"
            @click="$emit('select-ui-field', field.id)"
          >
            <template v-if="field.control === 'checkbox'">
              <label v-if="field.checkboxStyle === 'switch'" class="editor-ui-switch" :class="{ 'editor-ui-switch-disabled': !isInteractive(field) }" @click.stop>
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
                @focus="$emit('select-ui-field', field.id)"
                @input="updatePreviewText(field, ($event.target as HTMLInputElement).value)"
              />
            </template>

            <template v-else-if="field.control === 'select'">
              <div class="editor-ui-inline-select-shell" @click.stop>
                <AppSelect
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
                    @focus="$emit('select-ui-field', field.id)"
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
                @focus="$emit('select-ui-field', field.id)"
                @input="updatePreviewText(field, ($event.target as HTMLInputElement).value)"
              />
              <span v-else class="editor-ui-static-text">{{ resolveTextPreviewValue(field) }}</span>
            </template>
          </div>
        </div>
      </template>
    </div>

    <EmptyState
      v-if="!uiSchema.fields.length"
      title="还没有可预览的字段"
      description="在中间插入 checkbox、radio、number 等模板后，这里会按排布方向实时预览。"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import type { EditorUiSchema, EditorUiField } from '@/views/script-editor/editorSchema';
import type { EditorInputEntry } from '@/views/script-editor/editorVariables';
import {
  parseFieldOptions,
  resolvePreviewValue,
  findBoundInputEntry,
} from '@/views/script-editor/editorUiPreview';

defineOptions({ name: 'EditorUiPreviewPanel' });

const props = defineProps<{
  taskName: string;
  uiSchema: EditorUiSchema;
  selectedUiFieldId: string | null;
  inputEntries: EditorInputEntry[];
}>();

const emit = defineEmits<{
  'select-ui-field': [fieldId: string];
  'update-input': [entryId: string, field: 'stringValue' | 'booleanValue', value: string | boolean];
}>();

const uiPreviewExpanded = ref(true);
const taskEnabledPreview = ref(true);
const localPreviewValues = ref<Record<string, string | boolean>>({});

const findPreviewEntry = (field: EditorUiField) => findBoundInputEntry(field, props.inputEntries);

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

const isInteractive = (field: EditorUiField) => (field.control === 'text' ? field.editable : true);

const updatePreviewText = (field: EditorUiField, value: string) => {
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
</script>

<style scoped>
.editor-ui-preview-flow {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.85rem;
}

.editor-ui-toggle-chip {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 42px;
  height: 42px;
  border-radius: 14px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.88);
}

.editor-ui-task-name {
  color: var(--app-text-strong);
  font-size: 1.08rem;
  font-weight: 700;
}

.editor-ui-preview-item {
  display: inline-flex;
  align-items: center;
  gap: 0.65rem;
  min-height: 42px;
  border-radius: 16px;
  border: 1px solid transparent;
  background: transparent;
  padding: 0.18rem 0.3rem;
  text-align: left;
  transition: border-color 0.16s ease, background 0.16s ease;
  cursor: pointer;
}

.editor-ui-preview-item:hover {
  border-color: rgba(70, 110, 255, 0.16);
  background: rgba(255, 255, 255, 0.4);
}

.editor-ui-preview-item-active {
  border-color: var(--app-state-active-border);
  background: var(--app-state-active-bg);
}

.editor-ui-preview-item-vertical {
  justify-content: flex-start;
  width: 100%;
}

.editor-ui-inline-control {
  min-width: 72px;
  border-radius: 12px;
  border: 1px solid var(--app-border);
  background: white;
  padding: 0.45rem 0.75rem;
  text-align: left;
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

.editor-ui-inline-select-shell :deep(.app-select-trigger) {
  min-width: 132px;
  height: 42px;
  border-radius: 12px;
  padding-inline: 0.75rem;
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
  font-size: 1rem;
  font-weight: 600;
  line-height: 1.2;
}
</style>
