import {Store} from "@tauri-apps/plugin-store";

const store = await Store.load('autodaily.config.json');
export const defaultRouterKey = 'routerStart';
export const editorThemeKey = 'editorTheme';
export const appThemeKey = 'appTheme';
export const systemPreferencesKey = 'systemPreferences';
export const visionLabPreferencesKey = 'visionLabPreferences';
export const visionLabLaunchPresetKey = 'visionLabLaunchPreset';
export const visionLabActiveTabKey = 'visionLabActiveTab';

export const deviceKey = 'editorDevice';
export async function getFromStore<T>(key: string): Promise<T | null | undefined> {
    return await store.get<T>(key);
}

export async function setToStore<T>(key: string, value: T) {
    await store.set(key, value);
    await store.save();
}
