import { createRouter, createWebHistory } from 'vue-router';
import { ref } from "vue";
import { Monitor, Smartphone, FileCode, ShoppingBag, SettingsIcon, Info, FileText, ListTodo } from 'lucide-vue-next';
import { getFromStore, defaultRouterKey } from '../store/store.js';

const defaultPath = await getFromStore(defaultRouterKey).then(r => r.path).catch(() => '/tasks');
const routes = [
    {
        path: '/',
        redirect: defaultPath,
        //label: "根路径" //不展示根目录
    },
    {
        path: '/tasks',
        name: 'TaskManagement',
        label: "任务管理",
        icon: ListTodo,
        component: () => import('../views/TaskManagement.vue')
    },
    {
        path: '/logs',
        name: 'Logs',
        label: "运行日志",
        icon: FileText,
        component: () => import('../views/Logs.vue')
    },
    {
        path: '/scripts',
        name: 'LocalScriptList',
        label: "本地列表",
        icon: FileCode,
        component: () => import('../views/ScriptList.vue')
    },
    {
        path: '/market',
        name: 'ScriptMarket',
        label: "脚本市场",
        icon: ShoppingBag,
        component: () => import('../views/ScriptMarket.vue')
    },
    {
        path: '/devices',
        name: 'DeviceList',
        label: "设备列表",
        icon: Smartphone,
        component: () => import('../views/DeviceList.vue')
    },
    {
        path: '/settings',
        name: 'Settings',
        label: "系统设置",
        icon: SettingsIcon,
        component: () => import('../views/Settings.vue')
    },
    {
        path: '/about',
        name: 'About',
        label: "关于项目",
        icon: Info,
        component: () => import('../views/About.vue')
    },
    {
        path: '/editor',
        name: 'ScriptEditor',
        label: "脚本开发",
        icon: Monitor,
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
