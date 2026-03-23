<template>
  <div ref="root" class="relative">
    <button
      class="app-select app-select-trigger"
      :class="{ 'opacity-60': disabled }"
      type="button"
      :disabled="disabled"
      @click="toggleOpen"
    >
      <span class="truncate">{{ selectedOption?.label || placeholder }}</span>
      <ChevronDown class="h-4 w-4 shrink-0 text-[var(--app-text-faint)]" />
    </button>

    <transition name="select-fade">
      <div v-if="isOpen" class="app-select-menu" :class="align === 'right' ? 'right-0' : 'left-0'">
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
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
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

const handleOutsideClick = (event: MouseEvent) => {
  if (!root.value) {
    return;
  }

  if (!root.value.contains(event.target as Node)) {
    isOpen.value = false;
  }
};

onMounted(() => {
  document.addEventListener('mousedown', handleOutsideClick);
});

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleOutsideClick);
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
