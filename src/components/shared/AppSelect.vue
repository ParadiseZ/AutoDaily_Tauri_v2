<template>
  <div ref="root" class="relative">
    <button
      ref="trigger"
      class="app-select app-select-trigger"
      :class="{ 'opacity-60': disabled }"
      type="button"
      :disabled="disabled"
      @click="toggleOpen"
    >
      <span class="truncate">{{ selectedOption?.label || placeholder }}</span>
      <ChevronDown class="h-4 w-4 shrink-0 text-[var(--app-text-faint)]" />
    </button>

    <Teleport to="body">
      <transition name="select-fade">
        <div v-if="isOpen" ref="menu" class="app-select-menu app-select-menu-floating" :style="menuStyle">
          <button
            v-for="option in options"
            :key="String(option.value)"
            class="app-select-option"
            :class="{ 'app-select-option-active': isSelected(option.value), 'opacity-50': option.disabled }"
            type="button"
            :disabled="option.disabled"
            @click="selectOption(option.value)"
          >
            <span class="font-medium">{{ option.label }}</span>
            <span v-if="option.description" class="text-xs text-[var(--app-text-faint)]">{{ option.description }}</span>
          </button>
        </div>
      </transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { ChevronDown } from 'lucide-vue-next';

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
  }>(),
  {
    placeholder: '请选择',
    disabled: false,
    align: 'left',
  },
);

const emit = defineEmits<{
  'update:modelValue': [value: SelectValue];
}>();

const isOpen = ref(false);
const root = ref<HTMLElement | null>(null);
const trigger = ref<HTMLElement | null>(null);
const menu = ref<HTMLElement | null>(null);
const menuStyle = ref<Record<string, string>>({});

const selectedOption = computed(() =>
  props.options.find((option) => String(option.value) === String(props.modelValue)),
);

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

  menuStyle.value = {
    top: `${Math.max(top, viewportPadding)}px`,
    left: `${left}px`,
    width: `${rect.width}px`,
    maxHeight: `${Math.max(window.innerHeight - viewportPadding * 2, 180)}px`,
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
  updateMenuPosition();
});
</script>

<style scoped>
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
