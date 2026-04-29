import { reactive } from 'vue';

export type AppConfirmTone = 'primary' | 'warning' | 'danger';

export interface AppConfirmOptions {
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    tone?: AppConfirmTone;
}

export const appConfirmState = reactive({
    open: false,
    title: '',
    message: '',
    confirmText: '确定',
    cancelText: '取消',
    tone: 'primary' as AppConfirmTone,
});

let pendingResolve: ((approved: boolean) => void) | null = null;

export function requestAppConfirm(options: AppConfirmOptions): Promise<boolean> {
    if (pendingResolve) {
        pendingResolve(false);
    }

    appConfirmState.title = options.title;
    appConfirmState.message = options.message;
    appConfirmState.confirmText = options.confirmText ?? '确定';
    appConfirmState.cancelText = options.cancelText ?? '取消';
    appConfirmState.tone = options.tone ?? 'primary';
    appConfirmState.open = true;

    return new Promise<boolean>((resolve) => {
        pendingResolve = resolve;
    });
}

export function resolveAppConfirm(approved: boolean) {
    if (!appConfirmState.open && !pendingResolve) {
        return;
    }

    appConfirmState.open = false;
    const resolve = pendingResolve;
    pendingResolve = null;
    resolve?.(approved);
}
