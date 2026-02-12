import {Store} from "@tauri-apps/plugin-store";

const store = await Store.load('autodaily.config.json');
export const defaultRouterKey = 'routerStart';
export const editorThemeKey = 'editorTheme';
export const appThemeKey = 'appTheme';

export const deviceKey = 'editorDevice';

export async function getFromStore<T>(key: string): Promise<T | null | undefined> {
    return await store.get<T>(key);
}

export async function setToStore(key: string, value: any) {
    await store.set(key, value);
    await store.save();
}