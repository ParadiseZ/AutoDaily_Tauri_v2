/**
 * Theme Manager Composable
 * 
 * 管理脚本编辑器的主题
 * - 主题切换
 * - 主题持久化
 */

import { ref, onMounted } from 'vue';
import { getFromStore, setToStore, defaultEditorThemeKey } from '../../../store/store.js';
import { DEFAULT_THEME } from '../config.js';

/**
 * Theme Manager Composable
 * 
 * @returns {Object} 主题相关的状态和方法
 */
export function useThemeManager() {
    // 当前主题
    const currentTheme = ref(DEFAULT_THEME);

    /**
     * 切换主题 (dark <-> light)
     */
    function toggleTheme() {
        currentTheme.value = currentTheme.value === 'light' ? 'dark' : 'light';
        document.documentElement.setAttribute('data-theme', currentTheme.value);
        setToStore(defaultEditorThemeKey, currentTheme.value);
    }

    /**
     * 设置主题
     * @param {string} theme 
     */
    function setTheme(theme) {
        currentTheme.value = theme;
        document.documentElement.setAttribute('data-theme', theme);
        setToStore(defaultEditorThemeKey, theme);
    }

    /**
     * 初始化主题 (从存储中加载)
     */
    async function initTheme() {
        try {
            const savedTheme = await getFromStore(defaultEditorThemeKey);
            if (savedTheme && savedTheme !== DEFAULT_THEME) {
                currentTheme.value = savedTheme;
                document.documentElement.setAttribute('data-theme', savedTheme);
            }
        } catch (error) {
            console.warn('[useThemeManager] Failed to load theme:', error);
        }
    }

    return {
        // 状态
        currentTheme,

        // 方法
        toggleTheme,
        setTheme,
        initTheme,
    };
}

export default useThemeManager;
