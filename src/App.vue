<script setup>
import { ref, onMounted } from "vue";
import TheHeader from './components/layout/TheHeader.vue';
import TheSidebar from './components/layout/TheSidebar.vue';
import { useNotification } from './composables/useNotification';
import { useShutdownHandler } from './utils/shutdownHandler';

const { checkNotificationPermission } = useNotification();
const { initShutdownListener } = useShutdownHandler();

onMounted(async () => {
  // 初始化通知权限
  await checkNotificationPermission();
  
  // 初始化关机事件监听
  await initShutdownListener();
});
</script>

<template>
  <div id="app">
    <TheHeader />
    <div class="app-container">
      <TheSidebar />
      <main class="main-content">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </main>
    </div>
  </div>
</template>

<style>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
