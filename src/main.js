import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
//import { createPinia } from 'pinia';
//import ElementPlus from 'element-plus';
//import zhCn from 'element-plus/es/locale/lang/zh-cn';
//import 'element-plus/dist/index.css'
import "./style.css";
// import './assets/styles/main.scss'; // Commenting out if it conflicts or ensuring it exists

const app = createApp(App);

// Use Element Plus (Legacy support)
//app.use(ElementPlus, {
//locale: zhCn,
//});

// Use Pinia
//app.use(createPinia());

// Use Router
app.use(router);

app.mount("#app");
