<template>
  <div class="flex h-screen w-full bg-base-100 overflow-hidden font-sans text-base-content" data-tauri-drag-region>
    <!-- Sidebar -->
    <aside class="w-[220px] shrink-0 flex flex-col justify-between bg-base-200/40 backdrop-blur-2xl border-r border-base-content/5 relative z-10 transition-colors duration-300">
      
      <!-- Top Section -->
      <div>
        <!-- App Drag Handle / Logo Area -->
        <div class="h-[44px] w-full flex items-center px-5 mt-2" data-tauri-drag-region>
          <div class="flex items-center gap-2 pointer-events-none text-base-content/80">
            <Command class="w-5 h-5" />
            <span class="font-semibold text-[14px] tracking-wide">AutoDaily</span>
          </div>
        </div>

        <!-- Navigation Menu -->
        <nav class="flex flex-col gap-1 px-3 mt-4">
          <RouterLink
            v-for="route in mainRoutes"
            :key="route.path"
            :to="route.path"
            class="flex items-center gap-3 px-3 py-1.5 rounded-lg text-[13px] font-medium transition-colors duration-150 cursor-pointer"
            :class="[
              isActive(route.path) 
                ? 'bg-base-content/10 text-base-content shadow-sm' 
                : 'text-base-content/60 hover:bg-base-content/5 hover:text-base-content/80'
            ]"
          >
            <component :is="route.icon" class="w-4 h-4 opacity-80" />
            <span>{{ route.label }}</span>
          </RouterLink>
        </nav>
      </div>

      <!-- Bottom Spacer & User/Settings -->
      <div class="px-3 pb-4">
        <!-- Navigation Menu (Bottom) -->
        <nav class="flex flex-col gap-1 mb-2">
          <RouterLink
            v-for="route in bottomRoutes"
            :key="route.path"
            :to="route.path"
            class="flex items-center gap-3 px-3 py-1.5 rounded-lg text-[13px] font-medium transition-colors duration-150 cursor-pointer"
            :class="[
              isActive(route.path) 
                ? 'bg-base-content/10 text-base-content shadow-sm' 
                : 'text-base-content/60 hover:bg-base-content/5 hover:text-base-content/80'
            ]"
          >
            <component :is="route.icon" class="w-4 h-4 opacity-80" />
            <span>{{ route.label }}</span>
          </RouterLink>
        </nav>

        <!-- Optional Divider -->
        <div class="w-full h-px bg-base-content/5 my-2"></div>

        <!-- User Profile Area -->
        <div class="mt-2 px-2 flex items-center gap-3 cursor-pointer hover:bg-base-content/5 p-1.5 rounded-lg transition-colors" @click="handleUserClick">
          <div class="w-7 h-7 rounded-full bg-gradient-to-br from-base-content/80 to-base-content/40 flex items-center justify-center text-base-100 text-xs font-bold shadow-sm">
            {{ userInitial }}
          </div>
          <div class="flex flex-col">
            <span class="text-[12px] font-medium leading-tight text-base-content/80">{{ userName }}</span>
            <span class="text-[10px] text-base-content/40 leading-tight">{{ userStatus }}</span>
          </div>
        </div>
      </div>
    </aside>

    <!-- Main Content Area -->
    <main class="flex-1 flex flex-col min-w-0 bg-base-100 relative z-0">
      <!-- Top Titlebar / Toolbar (Invisible Drag Region) -->
      <header class="h-[44px] w-full flex items-center justify-between px-6 border-b border-base-content/5 shrink-0 bg-base-100" data-tauri-drag-region>
        <div class="text-[15px] font-semibold text-base-content/90 pointer-events-none">
          {{ currentRouteName }}
        </div>
        <!-- Right side tools can go here (e.g. search, global actions) -->
        <div class="flex items-center gap-3">
          <!-- Frame controls can be added here if not using OS default -->
        </div>
      </header>

      <!-- Scrollable Content -->
      <div class="flex-1 overflow-y-auto custom-scrollbar p-0">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
// NOTE: Extension omitted to let Vite resolve .ts or .js automatically
import { routesMenu } from '../router/index';
import { useUserStore } from '../store/user';
import { Command } from 'lucide-vue-next';

const route = useRoute();
const userStore = useUserStore();

// Split routes into main app logic and bottom actions (Settings/About)
const mainRoutes = computed(() => {
  return routesMenu.filter(r => !['/settings', '/about'].includes(r.path));
});

const bottomRoutes = computed(() => {
  return routesMenu.filter(r => ['/settings', '/about'].includes(r.path));
});

const isActive = (path: string) => {
  // exact match for / to prevent everything matching, otherwise startsWith
  if (path === '/') return route.path === '/';
  if (path === '/tasks') return route.path.startsWith('/tasks');
  return route.path.startsWith(path);
};

const currentRouteName = computed(() => {
  const current = routesMenu.find(r => route.path.startsWith(r.path));
  return current ? current.label : '';
});

const userInitial = computed(() => {
  return userStore.userProfile?.username?.charAt(0).toUpperCase() || 'G';
});

const userName = computed(() => {
  return userStore.userProfile?.username || 'Guest';
});

const userStatus = computed(() => {
  return userStore.isLoggedIn ? (userStore.userProfile?.sponsorUntil ? 'Pro' : 'Free') : 'Not Logged In';
});

const handleUserClick = () => {
  if (!userStore.isLoggedIn) {
    userStore.openAuthModal();
  } else {
    // Optionally route to settings/profile
  }
};
</script>

<style scoped>
/* Page transition matching Apple style fade */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease-out;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
