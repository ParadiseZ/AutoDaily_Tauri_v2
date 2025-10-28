import { createRouter, createWebHashHistory } from 'vue-router';

const routes = [
  {
    path: '/',
    name: 'Dashboard',
    component: () => import('../views/Dashboard.vue'),
    meta: { title: '控制面板' }
  },
  {
    path: '/local-scripts',
    name: 'LocalScripts',
    component: () => import('../views/LocalScripts.vue'),
    meta: { title: '本地列表' }
  },
  {
    path: '/marketplace',
    name: 'Marketplace',
    component: () => import('../views/Marketplace.vue'),
    meta: { title: '搜索' }
  },
  {
    path: '/settings',
    name: 'settings',
    component: () => import('../views/Settings.vue'),
    meta: { title: '设置' },
    children: [
      {
        path: 'window',
        name: 'settings-window',
        component: () => import('../views/Settings/Window.vue'),
        meta: { title: '窗口设置' }
      },
      {
        path: 'settings-script-global-config',
        name: 'settings-scriptGlobalConfig',
        component: () => import('../views/Settings/ScriptGlobalConfig.vue'),
        meta: { title: '脚本全局设置' }
      },
      {
        path: 'devices',
        name: 'settings-devices',
        component: () => import('../views/Settings/Devices.vue'),
        meta: { title: '设备配置' }
      },
      {
        path: 'resources',
        name: 'settings-resources',
        component: () => import('../views/Settings/Resources.vue'),
        meta: { title: '资源配置' }
      },
      {
        path: 'theme',
        name: 'settings-theme',
        component: () => import('../views/Settings/Theme.vue'),
        meta: { title: '主题设置' }
      },
      {
        path: '',
        redirect: '/settings/window'
      }
    ]
  },
  {
    path: '/about',
    name: 'About',
    component: () => import('../views/About.vue'),
    meta: { title: '关于' }
  },
  {
    path: '/developer',
    name: 'Developer',
    component: () => import('../views/Developer.vue'),
    meta: { title: '开发者' },
    children: [
      {
        path: 'capture-test',
        name: 'CaptureTest',
        component: () => import('../views/Developer/CaptureTest.vue'),
        meta: { title: '截图测试' }
      },
      {
        path: 'ocr-test',
        name: 'OCRTest',
        component: () => import('../views/Developer/OCRTest.vue'),
        meta: { title: 'OCR测试' }
      },
      {
        path: 'yolo-test',
        name: 'YoloTest',
        component: () => import('../views/Developer/YoloTest.vue'),
        meta: { title: 'Yolo测试' }
      },
      {
        path: 'onnx-test',
        name: 'OnnxTest',
        component: () => import('../views/Developer/OnnxTest.vue'),
        meta: { title: 'Onnx测试' }
      },
      {
        path: 'performance-test',
        name: 'PerformanceTest',
        component: () => import('../views/Developer/PerformanceTest.vue'),
        meta: { title: '性能测试' }
      },
      {
        path: '',
        redirect: '/developer/capture-test'
      }
    ]
  }
];

const router = createRouter({
  history: createWebHashHistory(),
  routes
});

// 设置页面标题
router.beforeEach((to, from, next) => {
  if (to.meta.title) {
    document.title = `AutoDaily - ${to.meta.title}`;
  }
  next();
});

export default router; 