/**
 * Theme Manager Composable
 * 
 * 管理脚本编辑器的主题
 * - 主题切换
 * - 主题持久化
 */

import { ref } from 'vue';
import { getFromStore, setToStore, editorThemeKey, appThemeKey } from '../../../store/store.js';
import { DEFAULT_EDITOR_THEME, DEFAULT_APP_THEME } from '../config.js';

/**
 * Theme Manager Composable
 * 
 * @returns {Object} 主题相关的状态和方法
 */
const currentEditorTheme = ref(DEFAULT_EDITOR_THEME);
const currentAppTheme = ref(DEFAULT_APP_THEME);

/**
 * Theme Manager Composable
 * 
 * @returns {Object} 主题相关的状态和方法
 */
export function useThemeManager() {
    // 当前主题

    /**
     * 切换主题 (dark <-> light)
     */
    function toggleTheme(key) {
        if (key === editorThemeKey) {
            currentEditorTheme.value = currentEditorTheme.value === 'light' ? 'dark' : 'light';
            setAndSaveTheme(currentEditorTheme.value, editorThemeKey)
        }
    }

    function setAndSaveTheme(theme, key) {
        document.documentElement.setAttribute('data-theme', theme);
        void setToStore(key, theme);
    }

    /**
     * 设置主题
     * @param {string} theme
     * @param {string} key 区分是编辑器还是应用
     */
    function setTheme(theme, key) {
        if (key === editorThemeKey) {
            currentEditorTheme.value = theme;
        } else if (key === appThemeKey) {
            currentAppTheme.value = theme;
        }
        setAndSaveTheme(theme, key);
    }

    /**
     * 初始化主题 (从存储中加载)
     * @param {string} key 区分是编辑器还是应用
     */
    async function initTheme(key) {
        try {
            const savedTheme = await getFromStore(key);
            if (!savedTheme) return;
            if (key === editorThemeKey && savedTheme !== DEFAULT_EDITOR_THEME) {
                currentEditorTheme.value = savedTheme;
            } else if (key === appThemeKey && savedTheme !== DEFAULT_APP_THEME) {
                currentAppTheme.value = savedTheme;
            }
            setAndSaveTheme(savedTheme, key)
        } catch (error) {
            console.warn('[useThemeManager] Failed to load theme:', error);
        }
    }

    return {
        // 状态
        currentEditorTheme,
        currentAppTheme,

        // 方法
        toggleTheme,
        setTheme,
        initTheme,
    };
}
