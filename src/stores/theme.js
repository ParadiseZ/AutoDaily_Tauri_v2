import { defineStore } from 'pinia';
import { ref, watch } from 'vue';

export const useThemeStore = defineStore('theme', () => {
  // 从localStorage获取主题颜色，默认为Element Plus蓝
  const primaryColor = ref(localStorage.getItem('theme-primary-color') || '#409EFF');
  // 从localStorage获取暗色/亮色模式，默认跟随系统
  const isDark = ref(localStorage.getItem('theme-dark-mode') === 'true' || 
    (localStorage.getItem('theme-dark-mode') === null && 
     window.matchMedia('(prefers-color-scheme: dark)').matches));

  // 监听系统颜色模式变化
  window.matchMedia('(prefers-color-scheme: dark)')
    .addEventListener('change', event => {
      if (localStorage.getItem('theme-dark-mode') === null) {
        isDark.value = event.matches;
      }
    });
  
  // 改变主题色
  function changePrimaryColor(color) {
    primaryColor.value = color;
    localStorage.setItem('theme-primary-color', color);
    
    // 更新CSS变量
    document.documentElement.style.setProperty('--el-color-primary', color);
    
    // 计算并设置衍生颜色
    // Light 3 (60% white mix)
    const light3 = lightenColor(color, 0.6);
    document.documentElement.style.setProperty('--el-color-primary-light-3', light3);
    
    // Light 5 (80% white mix)
    const light5 = lightenColor(color, 0.8);
    document.documentElement.style.setProperty('--el-color-primary-light-5', light5);
    
    // Light 7 (90% white mix)
    const light7 = lightenColor(color, 0.9);
    document.documentElement.style.setProperty('--el-color-primary-light-7', light7);
    
    // Light 9 (95% white mix)
    const light9 = lightenColor(color, 0.95);
    document.documentElement.style.setProperty('--el-color-primary-light-9', light9);
    
    // Dark 2 (20% black mix)
    const dark2 = darkenColor(color, 0.2);
    document.documentElement.style.setProperty('--el-color-primary-dark-2', dark2);
  }
  
  // 切换暗色/亮色模式
  function toggleDarkMode() {
    isDark.value = !isDark.value;
    localStorage.setItem('theme-dark-mode', isDark.value);
  }
  
  // 辅助函数：颜色混合 - 变亮
  function lightenColor(hex, amount) {
    return blendColors(hex, '#ffffff', amount);
  }
  
  // 辅助函数：颜色混合 - 变暗
  function darkenColor(hex, amount) {
    return blendColors(hex, '#000000', amount);
  }
  
  // 辅助函数：颜色混合
  function blendColors(color1, color2, ratio) {
    // 将颜色转换为RGB
    const c1 = hexToRgb(color1);
    const c2 = hexToRgb(color2);
    
    // 混合颜色
    const r = Math.round(c1.r * (1 - ratio) + c2.r * ratio);
    const g = Math.round(c1.g * (1 - ratio) + c2.g * ratio);
    const b = Math.round(c1.b * (1 - ratio) + c2.b * ratio);
    
    // 转回Hex
    return rgbToHex(r, g, b);
  }
  
  // 辅助函数：Hex转RGB
  function hexToRgb(hex) {
    const shorthand = /^#?([a-f\d])([a-f\d])([a-f\d])$/i;
    hex = hex.replace(shorthand, (m, r, g, b) => r + r + g + g + b + b);
    
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ? {
      r: parseInt(result[1], 16),
      g: parseInt(result[2], 16),
      b: parseInt(result[3], 16)
    } : { r: 0, g: 0, b: 0 };
  }
  
  // 辅助函数：RGB转Hex
  function rgbToHex(r, g, b) {
    return `#${((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1)}`;
  }
  
  // 监听暗色模式变化，添加/移除类
  watch(isDark, (newVal) => {
    if (newVal) {
      document.documentElement.classList.add('dark-theme');
    } else {
      document.documentElement.classList.remove('dark-theme');
    }
  }, { immediate: true });
  
  // 初始化主题
  changePrimaryColor(primaryColor.value);
  
  return {
    primaryColor,
    isDark,
    changePrimaryColor,
    toggleDarkMode
  };
}); 