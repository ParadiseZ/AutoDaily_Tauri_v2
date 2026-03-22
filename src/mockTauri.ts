if (typeof window !== 'undefined' && !(window as any).__TAURI_INTERNALS__) {
    console.warn('Tauri environment not detected. Mocking Tauri APIs for local browser development...');
    (window as any).__TAURI_INTERNALS__ = {
        invoke: async (cmd: string, args: any) => {
            console.log(`[Tauri Mock] invoke: ${cmd}`, args);
            // Mock responses for common Tauri commands
            if (cmd === 'plugin:store|load') return {};
            if (cmd === 'plugin:store|get') return null;
            if (cmd === 'plugin:store|set') return null;
            if (cmd === 'plugin:store|save') return null;
            if (cmd === 'backend_get_profile') return { success: false, data: null };
            return null;
        },
        transformCallback: function (callback: any) {
            return Math.random();
        },
        unregisterCallback: function () { }
    };
}
