import {Store} from "@tauri-apps/plugin-store";

const store = await Store.load('autodaily.config.json');
export const defaultRouterKey = 'routerStart';
export const editorThemeKey = 'editorTheme';
export const appThemeKey = 'appTheme';
export async function getFromStore(key) {
    return await store.get(key);
}

export async function setToStore(key, value) {
    await store.set(key, value);
}