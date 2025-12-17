import { createRouter, createWebHistory } from 'vue-router';

import Settings from '../views/Settings.vue';

const routes = [
    {
        path: '/',
        redirect: '/tasks'
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
