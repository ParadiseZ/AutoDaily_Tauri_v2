import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

export const useWindowsStore = defineStore('windows', {
  state: () => ({
    startMode: 'normal',
    closeExit: false,
    alwaysOnTop: false,
    idleAction: 'none',
    autoStart: false,
    remSizePosition: false,
    shortcut: {
      toggleWindow: 'Ctrl+H',
      toggleAllScripts: 'Alt+R',
      capture: 'Alt+A'
    }
  }),
  actions: {
    async loadConfig() {
      try {
        const configJson = await invoke('get_system_settings_cmd');
        const config = JSON.parse(configJson);
        config.shortcut.toggleWindow = config.shortcut.toggleWindow.replace('CommandOrControl', 'Ctrl');
        config.shortcut.toggleAllScripts = config.shortcut.toggleAllScripts.replace('CommandOrControl', 'Ctrl');
        config.shortcut.capture = config.shortcut.capture.replace('CommandOrControl', 'Ctrl');
        Object.assign(this.$state, config);
      } catch (error) {
        console.error('Failed to load window config:', error);
      }
    },
    async saveConfig() {
      try {
        const config = JSON.stringify(this.$state).replaceAll('Ctrl', 'CommandOrControl');
        const configJson = JSON.parse(config);
        await invoke('set_system_settings_cmd', { systemSettings: configJson });
      } catch (error) {
        console.error('Failed to save window config:', error);
      }
    }
  }
});
