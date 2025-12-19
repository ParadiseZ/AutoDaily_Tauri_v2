import { createRouter, createWebHistory } from 'vue-router';
import { getFromStore,defaultRouterKey } from '../store/store.js';
import Settings from '../views/Settings.vue';

const defaultPath = await getFromStore(defaultRouterKey) === '/'? '/tasks' : '/editor';
const routes = [
    {
        path: '/',
        redirect: defaultPath,
    },
    {
        path: '/tasks',
        name: 'TaskManagement',
        component: () => import('../views/TaskManagement.vue')
    },
    {
        path: '/devices',
        name: 'DeviceList',
        component: () => import('../views/DeviceList.vue')
    },
    {
        path: '/logs',
        name: 'Logs',
        component: () => import('../views/Logs.vue')
    },
    {
        path: '/settings',
        name: 'Settings',
        component: Settings
    },
    {
        path: '/editor',
        name: 'ScriptEditor',
        component: () => import('../views/ScriptEditor.vue')
    }
];

const router = createRouter({
    history: createWebHistory(),
    routes
});

export default router;
