import { createApp } from "vue";
import App from "./App.vue";
import ElementPlus from 'element-plus';
import { createPinia } from 'pinia';
import router from './router';
import './assets/styles/main.scss';
import zhCn from 'element-plus/dist/locale/zh-cn.mjs';

const app = createApp(App);

// 使用Element Plus组件库，配置中文
app.use(ElementPlus, {
  locale: zhCn,
});

// 使用Pinia状态管理
app.use(createPinia());

// 使用Vue Router
app.use(router);

app.mount("#app");
