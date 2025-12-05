import { createRouter, createWebHistory } from 'vue-router';
import DeviceManagement from '../views/DeviceManagement.vue';
import Settings from '../views/Settings.vue';

const routes = [
    {
        path: '/',
        redirect: '/devices'
    },
    {
        path: '/devices',
        name: 'DeviceManagement',
        component: DeviceManagement
    },
    {
        path: '/settings',
        name: 'Settings',
        component: Settings
    }
];

const router = createRouter({
    history: createWebHistory(),
    routes
});

export default router;
