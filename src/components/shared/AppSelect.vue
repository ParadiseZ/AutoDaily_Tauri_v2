<template>
  <div ref="root" class="relative">
    <button
      ref="trigger"
      class="app-select app-select-trigger"
      :class="{ 'opacity-60': disabled }"
      type="button"
      :data-testid="testId"
      :disabled="disabled"
      @click="toggleOpen"
    >
      <span class="app-select-trigger-copy">
        <span class="truncate font-medium">{{ selectedOption?.label || placeholder }}</span>
        <span
          v-if="showDescription && selectedOption?.description"
          class="truncate text-xs text-(--app-text-faint)"
        >
          {{ selectedOption.description }}
        </span>
      </span>
      <AppIcon name="chevron-down" :size="16" class="shrink-0 text-(--app-text-faint)" />
    </button>

    <Teleport to="body">
      <transition name="select-fade">
        <div
          v-if="isOpen"
          ref="menu"
          class="app-select-menu app-select-menu-floating"
          :data-testid="testId ? `${testId}-menu` : undefined"
          :style="menuStyle"
        >
          <div v-if="searchable" class="app-select-search-wrap">
            <input
              ref="searchInput"
              v-model="searchQuery"
              class="app-select-search"
              type="search"
              :placeholder="searchPlaceholder"
              :data-testid="testId ? `${testId}-search` : undefined"
              @keydown.stop
            />
          </div>
          <button
            v-for="option in filteredOptions"
            :key="String(option.value)"
            class="app-select-option"
            :class="{ 'app-select-option-active': isSelected(option.value), 'opacity-50': option.disabled }"
            type="button"
            :data-testid="testId ? `${testId}-option-${String(option.value)}` : undefined"
            :disabled="option.disabled"
            @click="selectOption(option.value)"
          >
            <span class="font-medium">{{ option.label }}</span>
            <span v-if="props.showDescription && option.description" class="text-xs text-(--app-text-faint)">{{ option.description }}</span>
          </button>
          <div v-if="!filteredOptions.length" class="app-select-empty">没有匹配项</div>
        </div>
      </transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import AppIcon from '@/components/shared/AppIcon.vue';

type SelectValue = string | number | null;

interface SelectOption {
  label: string;
  value: SelectValue;
  description?: string;
  disabled?: boolean;
}

const props = withDefaults(
  defineProps<{
    modelValue: SelectValue;
    options: SelectOption[];
    placeholder?: string;
    disabled?: boolean;
    align?: 'left' | 'right';
    showDescription?: boolean;
    searchable?: boolean;
    searchPlaceholder?: string;
    maxMenuHeight?: number;
    testId?: string;
  }>(),
  {
    placeholder: '请选择',
    disabled: false,
    align: 'left',
    showDescription: false,
    searchable: false,
    searchPlaceholder: '搜索',
    maxMenuHeight: 360,
    testId: undefined,
  },
);

const emit = defineEmits<{
  'update:modelValue': [value: SelectValue];
}>();

const isOpen = ref(false);
const root = ref<HTMLElement | null>(null);
const trigger = ref<HTMLElement | null>(null);
const menu = ref<HTMLElement | null>(null);
const searchInput = ref<HTMLInputElement | null>(null);
const menuStyle = ref<Record<string, string>>({});
const searchQuery = ref('');

const selectedOption = computed(() =>
  props.options.find((option) => String(option.value) === String(props.modelValue)),
);

const filteredOptions = computed(() => {
  const keyword = searchQuery.value.trim().toLowerCase();
  if (!keyword) {
    return props.options;
  }

  return props.options.filter((option) =>
    `${option.label} ${option.description ?? ''}`.toLowerCase().includes(keyword),
  );
});

const isSelected = (value: SelectValue) => String(value) === String(props.modelValue);

const selectOption = (value: SelectValue) => {
  emit('update:modelValue', value);
  isOpen.value = false;
};

const toggleOpen = () => {
  if (props.disabled) {
    return;
  }
  isOpen.value = !isOpen.value;
};

const updateMenuPosition = () => {
  if (!trigger.value) {
    return;
  }

  const rect = trigger.value.getBoundingClientRect();
  const menuWidth = Math.max(rect.width, menu.value?.offsetWidth ?? rect.width);
  const viewportPadding = 12;
  const availableRight = window.innerWidth - viewportPadding;
  const nextLeft = props.align === 'right' ? rect.right - menuWidth : rect.left;
  const left = Math.min(Math.max(nextLeft, viewportPadding), availableRight - menuWidth);
  const menuHeight = menu.value?.offsetHeight ?? 0;
  const openAbove = rect.bottom + menuHeight + 8 > window.innerHeight - viewportPadding && rect.top > menuHeight;
  const top = openAbove ? rect.top - menuHeight - 8 : rect.bottom + 8;
  const maxHeight = Math.min(props.maxMenuHeight, Math.max(window.innerHeight - viewportPadding * 2, 180));

  menuStyle.value = {
    top: `${Math.max(top, viewportPadding)}px`,
    left: `${left}px`,
    minWidth: `${rect.width}px`,
    maxHeight: `${maxHeight}px`,
  };
};

const handleOutsideClick = (event: MouseEvent) => {
  if (!root.value) {
    return;
  }

  const target = event.target as Node;
  if (!root.value.contains(target) && !menu.value?.contains(target)) {
    isOpen.value = false;
  }
};

const handleViewportChange = () => {
  if (isOpen.value) {
    updateMenuPosition();
  }
};

onMounted(() => {
  document.addEventListener('mousedown', handleOutsideClick);
  window.addEventListener('resize', handleViewportChange);
  window.addEventListener('scroll', handleViewportChange, true);
});

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleOutsideClick);
  window.removeEventListener('resize', handleViewportChange);
  window.removeEventListener('scroll', handleViewportChange, true);
});

watch(isOpen, async (open) => {
  if (!open) {
    return;
  }

  await nextTick();
  searchQuery.value = '';
  updateMenuPosition();
  if (props.searchable) {
    await nextTick();
    searchInput.value?.focus();
  }
});
</script>

<style scoped>
.app-select-trigger-copy {
  display: flex;
  min-width: 0;
  flex: 1 1 auto;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.1rem;
}

.select-fade-enter-active,
.select-fade-leave-active {
  transition: opacity 0.14s ease, transform 0.14s ease;
}

.select-fade-enter-from,
.select-fade-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
