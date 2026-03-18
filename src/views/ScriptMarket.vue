<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@/utils/api';
import { useUserStore } from '@/store/user';
import { showToast } from '@/utils/toast';

const userStore = useUserStore();

// Search state
const keyword = ref('');
const author = ref('');
const runtimeType = ref(''); // empty for all, or 'AI', 'AIAndVision'
const currentPage = ref(1);
const pageSize = ref(10);
const totalRecords = ref(0);

// Models
const scripts = ref<any[]>([]);
const isLoading = ref(false);
const downloadingId = ref<string | null>(null);

const fetchScripts = async (page = 1) => {
    isLoading.value = true;
    currentPage.value = page;
    
    try {
        const req = {
            page: currentPage.value,
            size: pageSize.value,
            keyword: keyword.value ? keyword.value : undefined,
            author: author.value ? author.value : undefined,
            runtimeType: runtimeType.value ? runtimeType.value : undefined
        };
        
        const res = await invoke('backend_search_scripts', { req });
        if (res && res.success && res.data) {
            scripts.value = res.data.records || [];
            totalRecords.value = res.data.total || 0;
        } else {
            showToast(res?.message || '获取云端脚本失败', 'error');
        }
    } catch (e: any) {
        showToast(e.message || '搜索发生异常', 'error');
    } finally {
        isLoading.value = false;
    }
};

const handleDownload = async (script: any) => {
    if (!userStore.isLoggedIn) {
        showToast('请先登录后再下载', 'warning');
        userStore.openAuthModal();
        return;
    }

    // Lock UI for this specific script
    downloadingId.value = script.id;

    try {
        // 1. Download Script Metadata & JSON DB config
        const res = await invoke('backend_download_script', { 
            scriptId: script.id, 
            currentUserId: userStore.userProfile?.id 
        });

        if (res && res.success) {
            // New local UUID of the script
            const localScriptId = res.data; 

            // 2. Check and Download Models if needed
            if (script.imgDetModel) {
                showToast(`正在下载检测模型: ${script.imgDetModel}...`, 'info');
                // The models are saved to a directory logic we pass
                await invoke('backend_download_model', {
                    scriptId: script.id,
                    modelType: 'det',
                    saveDir: `./models/${localScriptId}` 
                });
            }
            if (script.txtDetModel) {
                showToast(`正在下载文字框模型...`, 'info');
                await invoke('backend_download_model', {
                    scriptId: script.id,
                    modelType: 'rec_det',
                    saveDir: `./models/${localScriptId}` 
                });
            }
            
            showToast('脚本下载完成，请到本地列表中查看', 'success');
        } else {
            showToast(res?.message || '下载脚本失败', 'error');
        }
    } catch (e: any) {
        showToast(e.message || '下载异常', 'error');
    } finally {
        downloadingId.value = null;
    }
};

onMounted(() => {
    fetchScripts(1);
});
</script>

<template>
    <div class="p-6 h-full flex flex-col gap-6">
        <!-- Header & Search Bar -->
        <div class="flex flex-col md:flex-row justify-between items-start md:items-end gap-4">
            <div>
                <h1 class="text-2xl font-bold flex items-center gap-2">
                    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-primary"><path d="M4 22h14a2 2 0 0 0 2-2V7l-5-5H6a2 2 0 0 0-2 2v4"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/><path d="M3 15h6"/><path d="M3 18h6"/><path d="M3 12h6"/></svg>
                    插件市场
                </h1>
                <p class="text-base-content/60 mt-1">发现并下载云端的优秀自动化脚本</p>
            </div>
            
            <div class="flex flex-wrap gap-2 items-center w-full md:w-auto">
                <select class="select select-bordered select-sm max-w-xs" v-model="runtimeType" @change="fetchScripts(1)">
                    <option value="">所有类型</option>
                    <option value="AIAndVision">视觉 & AI</option>
                    <option value="AI">纯 AI</option>
                </select>
                <input type="text" placeholder="作者名..." class="input input-bordered input-sm w-32" v-model="author" @keyup.enter="fetchScripts(1)" />
                <input type="text" placeholder="搜索脚本的关键词..." class="input input-bordered input-sm w-48 md:w-64" v-model="keyword" @keyup.enter="fetchScripts(1)" />
                <button class="btn btn-primary btn-sm" @click="fetchScripts(1)" :disabled="isLoading">
                    <span v-if="isLoading" class="loading loading-spinner loading-xs"></span>
                    搜索
                </button>
            </div>
        </div>

        <!-- Script Grid -->
        <div class="flex-1 overflow-y-auto">
            <div v-if="scripts.length === 0 && !isLoading" class="flex flex-col items-center justify-center h-64 text-base-content/50">
                <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="mb-4 opacity-50"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
                <p>未找到符合条件的脚本</p>
            </div>

            <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6 pb-4">
                <div v-for="script in scripts" :key="script.id" class="card bg-base-100 shadow-xl border border-base-200 transition-all hover:shadow-2xl hover:-translate-y-1">
                    <div class="card-body p-5 gap-3">
                        <div class="flex justify-between items-start">
                            <h2 class="card-title text-lg truncate pr-2" :title="script.name">
                                {{ script.name }}
                            </h2>
                            <div class="badge badge-sm uppercase shrink-0" :class="script.runtimeType === 'AIAndVision' ? 'badge-secondary' : 'badge-accent'">
                                {{ script.runtimeType === 'AIAndVision' ? '视觉' : '文本' }}
                            </div>
                        </div>
                        
                        <p class="text-sm text-base-content/70 line-clamp-2 min-h-[2.5rem]" :title="script.description">
                            {{ script.description || '暂无详细描述...' }}
                        </p>
                        
                        <div class="flex items-center gap-2 text-xs text-base-content/60 mt-2">
                            <span class="flex items-center gap-1">
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                                {{ script.userName || '匿名作者' }}
                            </span>
                            <span class="flex items-center gap-1 ml-auto">
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" x2="12" y1="15" y2="3"/></svg>
                                {{ script.downloadCount || 0 }} 次下载
                            </span>
                        </div>

                        <div class="card-actions justify-end mt-4 pt-4 border-t border-base-200">
                            <!-- Show Model Tags if any are attached -->
                            <div class="flex gap-1 mr-auto self-center">
                                <div v-if="script.imgDetModel" class="tooltip tooltip-right" data-tip="附带视觉检测模型">
                                    <div class="badge badge-outline badge-xs opacity-60 px-1 py-2">DET</div>
                                </div>
                                <div v-if="script.txtDetModel || script.txtRecModel" class="tooltip tooltip-right" data-tip="附带文字识别模型">
                                    <div class="badge badge-outline badge-xs opacity-60 px-1 py-2">OCR</div>
                                </div>
                            </div>

                            <button 
                                class="btn btn-primary btn-sm px-6" 
                                :disabled="downloadingId === script.id"
                                @click="handleDownload(script)"
                            >
                                <span v-if="downloadingId === script.id" class="loading loading-spinner loading-xs"></span>
                                {{ downloadingId === script.id ? '下载中...' : '下载到本地' }}
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Pagination -->
            <div v-if="totalRecords > 0" class="flex justify-center mt-6">
                <div class="join">
                    <button class="join-item btn btn-sm" :disabled="currentPage === 1" @click="fetchScripts(currentPage - 1)">«</button>
                    <button class="join-item btn btn-sm pointer-events-none">第 {{ currentPage }} 页 / 共 {{ Math.ceil(totalRecords / pageSize) }} 页</button>
                    <button class="join-item btn btn-sm" :disabled="currentPage >= Math.ceil(totalRecords / pageSize)" @click="fetchScripts(currentPage + 1)">»</button>
                </div>
            </div>
        </div>
    </div>
</template>