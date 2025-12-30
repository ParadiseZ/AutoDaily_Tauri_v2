<template>
  <div class="drawer lg:drawer-open">
    <input id="my-drawer-2" type="checkbox" class="drawer-toggle" />
    <div class="drawer-content flex flex-col bg-base-200 min-h-screen">
      <!-- Navbar -->
      <div class="navbar bg-base-100 shadow-sm lg:hidden">
        <div class="flex-none">
          <label for="my-drawer-2" class="btn btn-square btn-ghost drawer-button">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="inline-block w-6 h-6 stroke-current"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path></svg>
          </label>
        </div>
        <div class="flex-1">
          <a class="btn btn-ghost normal-case text-xl">AutoDaily</a>
        </div>
      </div>

      <!-- Page Content -->
      <main class="flex-1 overflow-y-auto p-4">
        <router-view />
      </main>
    </div> 
    <div class="drawer-side z-20">
      <label for="my-drawer-2" class="drawer-overlay"></label> 
      <aside class="bg-base-100 w-64 h-full flex flex-col border-r border-base-300">
        <!-- Logo Area -->
        <div class="p-4 flex items-center gap-2 border-b border-base-300">
            <div class="avatar placeholder">
              <div class="bg-neutral text-neutral-content rounded-full w-10">
                <span class="text-xl">AD</span>
              </div>
            </div>
            <span class="text-xl font-bold">AutoDaily</span>
        </div>

        <!-- Menu -->

        <ul class="menu p-4 w-full flex-1 gap-2">
          <li v-for="route in routesMenu">
            <router-link :to="route.path" active-class="active">
              <component :is="route.icon" class="w-5 h-5" />
              {{ route.label }}
            </router-link>
          </li>
        </ul>

        <!-- User Info (Bottom Left) -->
        <div class="p-4 border-t border-base-300 mt-auto">
          <div class="flex items-center gap-3">
            <div class="avatar online">
              <div class="w-10 rounded-full">
                <img src="https://daisyui.com/images/stock/photo-1534528741775-53994a69daeb.jpg" alt="User Avatar" />
              </div>
            </div>
            <div>
              <div class="font-bold">Admin User</div>
              <div class="text-xs opacity-50">v2.0.0 Alpha</div>
            </div>
          </div>
        </div>
      </aside>
    </div>
  </div>
</template>

<script setup>
import { routesMenu } from '../router/index.js'
</script>
