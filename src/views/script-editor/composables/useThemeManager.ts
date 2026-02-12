import { ref } from 'vue';
import { getFromStore, setToStore, editorThemeKey, appThemeKey } from '../../../store/store';
import { DEFAULT_EDITOR_THEME, DEFAULT_APP_THEME } from '../config';

const currentEditorTheme = ref(DEFAULT_EDITOR_THEME);
const currentAppTheme = ref(DEFAULT_APP_THEME);

export function useThemeManager() {
    function toggleTheme(key: string) {
        if (key === editorThemeKey) {
            currentEditorTheme.value = currentEditorTheme.value === 'light' ? 'dark' : 'light';
            setAndSaveTheme(currentEditorTheme.value, editorThemeKey);
        }
    }

    function setAndSaveTheme(theme: string, key: string) {
        document.documentElement.setAttribute('data-theme', theme);
        void setToStore(key, theme);
    }

    function setTheme(theme: string, key: string) {
        if (key === editorThemeKey) {
            currentEditorTheme.value = theme;
        } else if (key === appThemeKey) {
            currentAppTheme.value = theme;
        }
        setAndSaveTheme(theme, key);
    }

    async function initTheme(key: string) {
        try {
            const savedTheme = await getFromStore(key);
            if (!savedTheme) return;
            if (key === editorThemeKey && savedTheme !== DEFAULT_EDITOR_THEME) {
                currentEditorTheme.value = savedTheme;
            } else if (key === appThemeKey && savedTheme !== DEFAULT_APP_THEME) {
                currentAppTheme.value = savedTheme;
            }
            setAndSaveTheme(savedTheme, key);
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

export default useThemeManager;
