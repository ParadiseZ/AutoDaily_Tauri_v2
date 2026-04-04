import { createRouter, createWebHistory } from 'vue-router';
import { ref } from "vue";
import { getFromStore, defaultRouterKey } from '@/store/store';

const storedRoute = await getFromStore<string | { path?: string }>(defaultRouterKey).catch(() => '/tasks');
const defaultPath = typeof storedRoute === 'string' ? storedRoute : storedRoute?.path || '/tasks';
const routes = [
    {
        path: '/',
        redirect: defaultPath,
    },
    {
        path: '/tasks',
        name: 'TaskManagement',
        label: "任务管理",
        icon: 'list-todo',
        component: () => import('../views/TaskManagement.vue')
    },
    {
        path: '/logs',
        name: 'Logs',
        label: "运行日志",
        icon: 'file-text',
        component: () => import('../views/Logs.vue')
    },
    {
        path: '/scripts',
        name: 'LocalScriptList',
        label: "本地列表",
        icon: 'file-code',
        component: () => import('../views/ScriptList.vue')
    },
    {
        path: '/market',
        name: 'ScriptMarket',
        label: "脚本市场",
        icon: 'shopping-bag',
        component: () => import('../views/ScriptMarket.vue')
    },
    {
        path: '/devices',
        name: 'DeviceList',
        label: "设备列表",
        icon: 'smartphone',
        component: () => import('../views/DeviceList.vue')
    },
    {
        path: '/settings',
        name: 'Settings',
        label: "系统设置",
        icon: 'settings',
        component: () => import('../views/Settings.vue')
    },
    {
        path: '/about',
        name: 'About',
        label: "关于项目",
        icon: 'info',
        component: () => import('../views/About.vue')
    },
    {
        path: '/editor',
        name: 'ScriptEditor',
        label: "脚本开发",
        icon: 'monitor',
        component: () => import('../views/ScriptEditor.vue')
    }
];

const router = createRouter({
    history: createWebHistory(),
    routes
});

export const routesMenu = routes.filter(r => r.label).map(r => ({ path: r.path, label: r.label, icon: r.icon }))
export const routesDisplay = routes.filter(r => r.label).map(r => ({ path: r.path, label: r.label }))

export const currentRouter = ref(routesDisplay.find(r => r.path === defaultPath) || routesDisplay[0]);

export default router;
