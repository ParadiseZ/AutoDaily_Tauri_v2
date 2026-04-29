<template>
  <AppDialog
    :open="appConfirmState.open"
    :title="appConfirmState.title"
    width-class="max-w-lg"
    @close="resolveAppConfirm(false)"
  >
    <div class="space-y-5">
      <p class="whitespace-pre-wrap break-words text-sm leading-6 text-(--app-text-soft)">
        {{ appConfirmState.message }}
      </p>
      <div class="flex flex-wrap justify-end gap-2">
        <button class="app-button app-button-ghost" type="button" @click="resolveAppConfirm(false)">
          {{ appConfirmState.cancelText }}
        </button>
        <button class="app-button" :class="confirmButtonClass" type="button" @click="resolveAppConfirm(true)">
          {{ appConfirmState.confirmText }}
        </button>
      </div>
    </div>
  </AppDialog>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import AppDialog from '@/components/shared/AppDialog.vue';
import { appConfirmState, resolveAppConfirm } from '@/services/appDialogService';

const confirmButtonClass = computed(() => {
  if (appConfirmState.tone === 'danger') return 'app-button-danger';
  if (appConfirmState.tone === 'warning') return 'app-button-warning';
  return 'app-button-primary';
});
</script>
