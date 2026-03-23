import { ref } from 'vue';
import { getFromStore, setToStore } from '@/store/store';
import type { AppTheme } from '@/types/app/domain';

// Global state for current setting
const currentThemeSetting = ref<AppTheme>('system');
let mediaQueryListener: ((e: MediaQueryListEvent) => void) | null = null;
let mediaQuery: MediaQueryList | null = null;

export function useThemeManager() {
    const applyTheme = (theme: AppTheme) => {
        let actualTheme = theme;
        if (theme === 'system') {
            if (!mediaQuery) mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
            actualTheme = mediaQuery.matches ? 'dark' : 'light';
        }
        
        // DaisyUI logic
        document.documentElement.setAttribute('data-theme', actualTheme);
        
        // Tailwind logic (often class='dark' on HTML element)
        if (actualTheme === 'dark') {
            document.documentElement.classList.add('dark');
        } else {
            document.documentElement.classList.remove('dark');
        }
    };

    const handleSystemChange = (e: MediaQueryListEvent) => {
        if (currentThemeSetting.value === 'system') {
            applyTheme('system');
        }
    };

    const initTheme = async (storeKey: string) => {
        if (!mediaQuery) {
            mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
        }

        try {
            const savedTheme = await getFromStore<string>(storeKey);
            if (savedTheme && ['light', 'dark', 'system'].includes(savedTheme)) {
                currentThemeSetting.value = savedTheme as AppTheme;
            }
        } catch (e) {
            console.warn('Failed to load theme from store, defaulting to system:', e);
        }

        applyTheme(currentThemeSetting.value);

        // Setup system listener if not already initialized
        if (!mediaQueryListener) {
            mediaQueryListener = handleSystemChange;
            mediaQuery.addEventListener('change', mediaQueryListener as EventListener);
        }
    };

    const setTheme = async (storeKey: string, theme: AppTheme) => {
        currentThemeSetting.value = theme;
        applyTheme(theme);
        try {
            await setToStore(storeKey, theme);
        } catch (e) {
            console.error('Failed to save theme to store:', e);
        }
    };

    return {
        initTheme,
        setTheme,
        currentThemeSetting
    };
}
