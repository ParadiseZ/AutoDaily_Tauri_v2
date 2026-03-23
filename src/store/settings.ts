import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { getFromStore, setToStore, appThemeKey, defaultRouterKey, systemPreferencesKey } from '@/store/store';
import { settingsService } from '@/services/settingsService';
import type {
    AppTheme,
    DefaultRoute,
    LogConfig,
    SystemConfigPayload,
    SystemPreferences,
    UpdateInfo,
} from '@/types/app/domain';
import {
    DEFAULT_LOG_CONFIG,
    DEFAULT_SYSTEM_PREFERENCES,
} from '@/types/app/domain';

const toRouteValue = (value: DefaultRoute | { path?: DefaultRoute } | null | undefined): DefaultRoute => {
    if (!value) {
        return DEFAULT_SYSTEM_PREFERENCES.defaultRoute;
    }

    if (typeof value === 'string') {
        return value;
    }

    return value.path ?? DEFAULT_SYSTEM_PREFERENCES.defaultRoute;
};

export const useSettingsStore = defineStore('settings', () => {
    const preferences = ref<SystemPreferences>(DEFAULT_SYSTEM_PREFERENCES);
    const logConfig = ref<LogConfig>(DEFAULT_LOG_CONFIG);
    const loading = ref(false);
    const updateInfo = ref<UpdateInfo | null>(null);

    const interfacePreferences = computed(() => ({
        appTheme: preferences.value.appTheme,
        defaultRoute: preferences.value.defaultRoute,
    }));

    const loadPreferences = async () => {
        loading.value = true;
        try {
            const [savedPreferences, savedTheme, savedRoute] = await Promise.all([
                getFromStore<SystemPreferences>(systemPreferencesKey),
                getFromStore<AppTheme>(appThemeKey),
                getFromStore<DefaultRoute | { path?: DefaultRoute }>(defaultRouterKey),
            ]);

            preferences.value = {
                ...DEFAULT_SYSTEM_PREFERENCES,
                ...(savedPreferences ?? {}),
                appTheme: savedTheme ?? savedPreferences?.appTheme ?? DEFAULT_SYSTEM_PREFERENCES.appTheme,
                defaultRoute: toRouteValue(savedRoute ?? savedPreferences?.defaultRoute),
            };

            try {
                logConfig.value = {
                    ...DEFAULT_LOG_CONFIG,
                    ...(await settingsService.getLogConfig()),
                };
            } catch {
                logConfig.value = DEFAULT_LOG_CONFIG;
            }
        } finally {
            loading.value = false;
        }
    };

    const persistPreferences = async () => {
        await Promise.all([
            setToStore(systemPreferencesKey, preferences.value),
            setToStore(appThemeKey, preferences.value.appTheme),
            setToStore(defaultRouterKey, preferences.value.defaultRoute),
        ]);
    };

    const updatePreferences = async (patch: Partial<SystemPreferences>) => {
        preferences.value = {
            ...preferences.value,
            ...patch,
        };
        await persistPreferences();
    };

    const applySystemPreferences = async (patch: Partial<SystemPreferences>) => {
        await updatePreferences(patch);
        const payload: SystemConfigPayload = {
            startMode: preferences.value.startMode,
            closeExit: preferences.value.closeExit,
            alwaysOnTop: preferences.value.alwaysOnTop,
            idleAction: preferences.value.idleAction,
            maxIdleRetryNum: preferences.value.maxIdleRetryNum,
            autoStart: preferences.value.autoStart,
            shortcut: preferences.value.shortcut,
        };
        await settingsService.applySystemConfig(payload);
    };

    const refreshUpdateInfo = async () => {
        updateInfo.value = await settingsService.checkUpdate();
        return updateInfo.value;
    };

    const updateLogSettings = async (patch: Partial<LogConfig>) => {
        if (patch.logLevel && patch.logLevel !== logConfig.value.logLevel) {
            await settingsService.updateLogLevel(patch.logLevel);
        }
        if (patch.logDir && patch.logDir !== logConfig.value.logDir) {
            await settingsService.updateLogDir(patch.logDir);
        }
        if (
            typeof patch.retentionDays === 'number' &&
            patch.retentionDays !== logConfig.value.retentionDays
        ) {
            await settingsService.updateRetentionDays(patch.retentionDays);
        }

        logConfig.value = {
            ...logConfig.value,
            ...patch,
        };
    };

    const cleanLogsNow = async () => {
        await settingsService.cleanLogs();
    };

    return {
        applySystemPreferences,
        cleanLogsNow,
        interfacePreferences,
        loadPreferences,
        loading,
        logConfig,
        preferences,
        refreshUpdateInfo,
        updateInfo,
        updateLogSettings,
        updatePreferences,
    };
});
