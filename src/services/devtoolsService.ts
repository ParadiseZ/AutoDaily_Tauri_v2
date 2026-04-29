import { invoke } from '@/utils/api';
import { showToast } from '@/utils/toast';

export async function openCurrentDevtools() {
    try {
        await invoke('open_current_devtools_cmd');
    } catch (error) {
        console.error('[devtools] open_current_devtools_cmd failed', error);
        showToast('打开开发者工具失败', 'error');
    }
}

export function reloadCurrentPage() {
    window.location.reload();
}
