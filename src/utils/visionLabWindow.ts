import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { setToStore, visionLabLaunchPresetKey } from '@/store/store';
import type { VisionLabLaunchPreset } from '@/types/app/domain';

export const VISION_LAB_WINDOW_LABEL = 'vision-lab';

async function focusWindow(win: WebviewWindow) {
    await win.show();
    await win.unminimize();
    await win.setFocus();
}

export async function openVisionLabWindow(preset?: VisionLabLaunchPreset | null) {
    const existing = await WebviewWindow.getByLabel(VISION_LAB_WINDOW_LABEL);
    if (existing) {
        await focusWindow(existing);
        return;
    }

    if (preset) {
        await setToStore(visionLabLaunchPresetKey, preset);
    }

    const win = new WebviewWindow(VISION_LAB_WINDOW_LABEL, {
        url: '/vision-lab?standalone=1',
        title: '视觉测试',
        width: 1680,
        height: 1020,
        minWidth: 1240,
        minHeight: 780,
        center: true,
        focus: true,
        devtools: true,
    });

    await new Promise<void>((resolve, reject) => {
        void win.once('tauri://created', () => resolve());
        void win.once('tauri://error', (event) => reject(event.payload));
    });

    await focusWindow(win);
}
