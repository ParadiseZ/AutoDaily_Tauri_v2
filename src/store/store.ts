import {Store} from "@tauri-apps/plugin-store";

const store = await Store.load('autodaily.config.json');
export const defaultRouterKey = 'routerStart';
export const editorThemeKey = 'editorTheme';
export const appThemeKey = 'appTheme';
export const systemPreferencesKey = 'systemPreferences';

export const deviceKey = 'editorDevice';
export const adbServerConfigKey = 'adbServerConfig';
export async function getFromStore<T>(key: string): Promise<T | null | undefined> {
    return await store.get<T>(key);
}

export async function setToStore<T>(key: string, value: T) {
    await store.set(key, value);
    await store.save();
}
