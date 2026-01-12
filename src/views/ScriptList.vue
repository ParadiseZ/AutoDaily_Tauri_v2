<script setup>
import { ref, computed, onMounted, reactive } from 'vue';
import { useScripts } from '../assets/js/useScripts';
import {
  Layers,
  Search,
  Trash2,
  Edit,
  ChevronRight,
  Info,
  Settings,
  Cpu,
  Download,
  User,
  Calendar,
  MoreHorizontal,
  Package,
  Eye,
  Activity,
  Box,
  CheckCircle2,
  Clock,
  Plus,
  AlertTriangle,
  FileJson,
  X,
} from 'lucide-vue-next';
import ScriptConfigModal from './script-list/components/ScriptConfigModal.vue';

const { scripts, selectedScript, selectedTemplate, getAllScripts, saveScript, deleteScript, selectScript } =
  useScripts();

onMounted(() => {
  getAllScripts();
});

// New script modal state
const isNewModalOpen = ref(false);

const openNewModal = () => {
  isNewModalOpen.value = true;
};

const handleCreateScript = async (scriptData) => {
  try {
    await saveScript(scriptData);
    isNewModalOpen.value = false;
  } catch (e) {
    // The component handles validation, but backend errors bubble up
    alert('创建失败: ' + e);
    console.error(e);
  }
};

// Edit script modal state
const isEditModalOpen = ref(false);
const editingScriptData = ref(null);

const openEditModal = (script) => {
  editingScriptData.value = script;
  isEditModalOpen.value = true;
  expandedActionsId.value = null; // Close the action panel
};

const handleUpdateScript = async (scriptData) => {
  try {
    await saveScript(scriptData);
    isEditModalOpen.value = false;
    editingScriptData.value = null;
    // If we were editing the currently selected script, update it
    if (selectedScript.value?.id === scriptData.id) {
      // Find the updated script in the list and re-select it
      const updated = scripts.value.find((s) => s.id === scriptData.id);
      if (updated) {
        selectedScript.value = updated;
      }
    }
  } catch (e) {
    alert('更新失败: ' + e);
    console.error(e);
  }
};

const searchQuery = ref('');

const filteredScripts = computed(() => {
  if (!searchQuery.value) return scripts.value;
  return scripts.value.filter(
    (s) =>
      s.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      s.description?.toLowerCase().includes(searchQuery.value.toLowerCase())
  );
});

const handleSelect = (script) => {
  selectScript(script);
};

const formatTime = (time) => {
  if (!time) return '无';
  return time.split('T')[0];
};

const globalDelay = ref(500);
const randomRange = ref(5);

// 搜索与过滤逻辑已在上面定义，此处无需重复
</script>

<template>
  <div class="h-full flex bg-base-300 overflow-hidden font-sans">
    <!-- 第一栏：脚本列表 -->
    <div class="w-80 flex-none border-r border-base-content/10 bg-base-100 flex flex-col">
      <div class="p-4 border-b border-base-content/5">
        <h2 class="text-xl font-bold flex items-center gap-2 mb-4">
          <Package class="w-5 h-5 text-primary" />
          本地列表
        </h2>
        <label class="input input-bordered flex items-center gap-2 h-10 bg-base-200 border-none shadow-inner">
          <Search class="w-4 h-4 opacity-70" />
          <input type="text" v-model="searchQuery" class="grow text-sm" placeholder="搜索..." />
        </label>
      </div>

      <div class="grow overflow-y-auto custom-scrollbar p-2 space-y-2">
        <div
          v-for="script in filteredScripts"
          :key="script.name"
          @click="handleSelect(script)"
          class="group relative overflow-hidden rounded-xl border transition-all cursor-pointer"
          :class="[
            selectedScript?.name === script.name
              ? 'bg-primary/10 border-primary shadow-sm'
              : 'bg-base-200 border-transparent hover:bg-base-300 hover:border-base-content/10',
          ]"
        >
          <div class="p-3 flex items-start gap-3">
            <div class="w-10 h-10 rounded-lg bg-base-100 flex items-center justify-center shadow-sm">
              <Activity v-if="script.scriptType === 'Official'" class="w-5 h-5 text-primary" />
              <Box v-else class="w-5 h-5 text-secondary" />
            </div>

            <div class="grow min-w-0">
              <div class="flex items-center justify-between gap-1">
                <p
                  class="font-semibold truncate text-sm"
                  :class="selectedScript?.name === script.name ? 'text-primary' : ''"
                >
                  {{ script.name }}
                </p>
                <span class="text-[10px] opacity-50">{{ script.verName }}</span>
              </div>
              <p class="text-xs opacity-60 line-clamp-1 mt-0.5">{{ script.description }}</p>
            </div>

            <!-- 下拉操作菜单 (使用 details 原生处理点击外部关闭) -->
            <details class="dropdown dropdown-left flex-none self-center ml-1">
              <summary
                @click.stop
                class="btn btn-ghost btn-xs btn-circle hover:bg-base-content/10 cursor-pointer list-none"
              >
                <MoreHorizontal class="w-4 h-4" />
              </summary>
              <ul
                class="dropdown-content menu bg-base-100 rounded-xl w-44 p-1.5 shadow-xl border border-base-content/10 z-50"
              >
                <!-- 编辑信息 -->
                <li v-if="script.scriptType === 'custom'">
                  <a
                    @click="openEditModal(script)"
                    class="flex items-center gap-3 text-sm hover:bg-info/10 hover:text-info cursor-pointer"
                  >
                    <Edit class="w-4 h-4" />
                    <span>编辑信息</span>
                  </a>
                </li>
                <!-- 编辑逻辑 -->
                <li v-if="script.scriptType === 'custom'">
                  <a
                    @click="console.log('编辑逻辑', script.name)"
                    class="flex items-center gap-3 text-sm hover:bg-secondary/10 hover:text-secondary cursor-pointer"
                  >
                    <Settings class="w-4 h-4" />
                    <span>编辑逻辑</span>
                  </a>
                </li>

                <li class="my-1"><hr class="border-base-content/10" /></li>

                <!-- 上传到云端 -->
                <li v-if="script.scriptType === 'custom'">
                  <a
                    @click="console.log('上传', script.name)"
                    class="flex items-center gap-3 text-sm hover:bg-accent/10 hover:text-accent cursor-pointer"
                  >
                    <Download class="w-4 h-4 rotate-180" />
                    <span>上传到云端</span>
                  </a>
                </li>
                <!-- 检查更新 -->
                <li>
                  <a
                    @click="console.log('检查更新', script.name)"
                    class="flex items-center gap-3 text-sm hover:bg-primary/10 hover:text-primary cursor-pointer"
                  >
                    <Clock class="w-4 h-4" />
                    <span>检查更新</span>
                  </a>
                </li>

                <li class="my-1"><hr class="border-base-content/10" /></li>

                <!-- 删除 -->
                <li>
                  <a
                    @click="deleteScript(script)"
                    class="flex items-center gap-3 text-sm text-error hover:bg-error/10 cursor-pointer"
                  >
                    <Trash2 class="w-4 h-4" />
                    <span>删除</span>
                  </a>
                </li>
              </ul>
            </details>
          </div>
        </div>

        <div v-if="filteredScripts.length === 0" class="text-center py-10 opacity-30 flex flex-col items-center">
          <Search class="w-8 h-8 mb-2" />
          <p class="text-sm">未找到相关内容</p>
        </div>
      </div>

      <div class="p-4 border-t border-base-content/5">
        <button @click="openNewModal" class="btn btn-primary btn-block gap-2 shadow-lg shadow-primary/20">
          <Plus class="w-4 h-4" /> 新建
        </button>
      </div>
    </div>

    <!-- 第二栏：脚本详情 -->
    <div class="w-80 flex-none border-r border-base-content/10 bg-base-200/50 flex flex-col">
      <div v-if="selectedScript" class="flex flex-col h-full animate-in fade-in slide-in-from-left-4 duration-300">
        <div class="p-4 border-b border-base-content/5 bg-base-100">
          <h2 class="text-xl font-bold flex items-center gap-2 mb-1">
            <Info class="w-5 h-5 text-secondary" />
            详情
          </h2>
          <div
            class="badge badge-sm"
            :class="selectedScript.scriptType === 'Official' ? 'badge-primary' : 'badge-secondary'"
          >
            {{ selectedScript.scriptType === 'Official' ? '官方认证' : '本地脚本' }}
          </div>
        </div>

        <div class="grow overflow-y-auto custom-scrollbar p-4 space-y-6">
          <!-- 核心信息 -->
          <section>
            <!-- <h3 class="text-xs font-bold uppercase tracking-wider opacity-40 mb-3">基本信息</h3> -->
            <div class="space-y-3">
              <div class="flex flex-col">
                <span class="text-[10px] opacity-50">名称</span>
                <span class="text-sm font-medium">{{ selectedScript.name }}</span>
              </div>
              <div class="flex flex-col">
                <span class="text-[10px] opacity-50">版本号</span>
                <span class="text-sm font-medium">{{ selectedScript.verName }} ({{ selectedScript.verNum }})</span>
              </div>
              <div class="flex flex-col">
                <span class="text-[10px] opacity-50">脚本描述</span>
                <p class="text-sm opacity-80 leading-relaxed">{{ selectedScript.description || '暂无描述' }}</p>
              </div>
            </div>
          </section>

          <!-- 技术参数 -->
          <section class="bg-base-100/50 p-3 rounded-xl border border-base-content/5">
            <h3 class="text-xs font-bold uppercase tracking-wider opacity-40 mb-3 flex items-center gap-1">
              <Cpu class="w-3 h-3" /> 模型配置
            </h3>
            <div class="grid grid-cols-1 gap-3">
              <div class="flex items-center justify-between">
                <span class="text-xs opacity-60">应用包名</span>
                <span class="text-xs font-mono bg-base-300 px-1.5 py-0.5 rounded">{{
                  selectedScript.pkgName || '未指定'
                }}</span>
              </div>
              <div class="divider m-0 opacity-10"></div>
              <div class="flex items-center justify-between">
                <span class="text-xs opacity-60">图像识别</span>
                <span class="text-xs font-medium">{{ selectedScript.imgDetModel || '无' }}</span>
              </div>
              <div class="flex items-center justify-between">
                <span class="text-xs opacity-60">文本检测</span>
                <span class="text-xs font-medium">{{ selectedScript.txtDetModel || 'PaddleOCR_v5 (官中)' }}</span>
              </div>
              <div class="flex items-center justify-between">
                <span class="text-xs opacity-60">文本识别</span>
                <span class="text-xs font-medium">{{ selectedScript.txtRecModel || 'PaddleOCR_v5 (官中)' }}</span>
              </div>
            </div>
          </section>

          <!-- 统计与元数据 -->
          <section class="space-y-4 pt-2">
            <div class="flex items-center gap-4 text-xs">
              <div class="flex items-center gap-1.5 opacity-70">
                <User class="w-3.5 h-3.5" />
                <span>{{ selectedScript.userName }}</span>
              </div>
              <div class="flex items-center gap-1.5 opacity-70">
                <Download class="w-3.5 h-3.5" />
                <span>{{ selectedScript.downloadCount }} 次下载</span>
              </div>
            </div>

            <div class="p-3 bg-base-100 rounded-lg space-y-2 border border-base-content/5">
              <div class="flex justify-between text-[11px]">
                <span class="opacity-50 flex items-center gap-1"><Calendar class="w-3 h-3" /> 创建时间</span>
                <span class="opacity-80">{{ formatTime(selectedScript.createTime) }}</span>
              </div>
              <div class="flex justify-between text-[11px]">
                <span class="opacity-50 flex items-center gap-1"><Clock class="w-3 h-3" /> 最后更新</span>
                <span class="opacity-80">{{ formatTime(selectedScript.updateTime) }}</span>
              </div>
            </div>
          </section>
        </div>
      </div>

      <!-- 未选择状态 -->
      <div v-else class="flex flex-col items-center justify-center h-full opacity-20 p-8 text-center">
        <Info class="w-16 h-16 mb-4" />
        <p class="text-lg font-medium">详细信息</p>
        <p class="text-sm">查看详情、管理任务及模版设置</p>
      </div>
    </div>

    <!-- 第三栏：任务设置 -->
    <div class="grow bg-base-100 flex flex-col">
      <div v-if="selectedScript" class="flex flex-col h-full animate-in fade-in slide-in-from-bottom-2 duration-400">
        <!-- 任务设置页头：包含模版选择 -->
        <div class="p-4 border-b border-base-content/5 flex items-center justify-between bg-base-100">
          <div class="flex items-center gap-4 grow">
            <div class="flex items-center gap-2">
              <Layers class="w-4 h-4 text-accent" />
              <span class="text-xs font-bold uppercase tracking-wider opacity-60 text-nowrap">配置模板</span>
            </div>
            <select
              v-model="selectedTemplate"
              class="select select-bordered select-sm bg-base-200 border-none focus-visible:outline-none w-fit max-w-xs"
            >
              <option v-for="tpl in selectedScript.templates" :key="tpl.id" :value="tpl.id">
                {{ tpl.name }}
              </option>
              <option disabled v-if="selectedScript.templates?.length">──────</option>
              <option value="add_new">+ 新增模板</option>
            </select>
          </div>
        </div>

        <div class="grow overflow-hidden flex flex-col bg-base-100">
          <!-- 基础设置 -->
          <div class="flex-none p-4 pb-2 space-y-3 border-b border-base-content/5">
            <div class="flex items-center gap-2">
              <h2 class="text-[11px] font-bold uppercase tracking-widest opacity-50">基础设置</h2>
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div class="flex items-center justify-between p-2 rounded-lg bg-base-200/50 border border-base-content/5">
                <span class="text-xs font-medium opacity-80 text-nowrap">操作后延迟</span>
                <div class="flex items-center gap-1">
                  <input
                    type="number"
                    v-model="globalDelay"
                    class="w-12 bg-transparent text-xs font-bold text-right outline-none text-primary"
                  />
                  <span class="text-[10px] opacity-40">ms</span>
                </div>
              </div>

              <div class="flex items-center justify-between p-2 rounded-lg bg-base-200/50 border border-base-content/5">
                <span class="text-xs font-medium opacity-80 text-nowrap">随机坐标范围</span>
                <div class="flex items-center gap-1">
                  <input
                    type="number"
                    v-model="randomRange"
                    class="w-12 bg-transparent text-xs font-bold text-right outline-none text-primary"
                  />
                  <span class="text-[10px] opacity-40">px</span>
                </div>
              </div>
            </div>
          </div>
          <div class="grow p-4 flex flex-col min-h-0">
            <div class="flex items-center gap-2 mb-4">
              <h2 class="text-[11px] font-bold uppercase tracking-widest opacity-50">任务设置</h2>
            </div>

            <div class="grow overflow-y-auto custom-scrollbar space-y-1">
              <div
                v-for="task in selectedScript.tasks"
                :key="task.id"
                class="group flex items-center gap-3 p-2 rounded-lg hover:bg-base-200 transition-colors"
                :style="{ marginLeft: `${task.indent * 24}px` }"
              >
                <div class="flex-none flex items-center">
                  <input type="checkbox" v-model="task.enabled" class="checkbox checkbox-sm checkbox-primary" />
                </div>

                <div class="grow flex items-center gap-3">
                  <span class="text-sm" :class="task.enabled ? 'font-medium opacity-90' : 'opacity-40 italic'">
                    {{ task.name }}
                  </span>
                  <div v-if="task.indent > 0" class="h-px w-4 bg-base-content/10"></div>
                </div>

                <!-- <div class="flex-none opacity-0 group-hover:opacity-100 transition-opacity flex items-center gap-2">
                            <div class="flex items-center gap-1 text-[10px] bg-base-300 px-2 py-0.5 rounded text-base-content/60">
                                <Clock class="w-3 h-3" /> {{ task.delay }}ms
                            </div>
                            <button class="btn btn-square btn-ghost btn-xs">
                                <Settings class="w-3.5 h-3.5" />
                            </button>
                        </div> -->
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 未选择状态 -->
      <div v-else class="flex flex-col items-center justify-center h-full opacity-20 p-8 text-center">
        <Settings class="w-16 h-16 mb-4" />
        <p class="text-lg font-medium">任务设置</p>
        <p class="text-sm">选中脚本后可在此配置自动化流程</p>
      </div>
    </div>

    <!-- 新建脚本 Modal -->
    <ScriptConfigModal :is-open="isNewModalOpen" @close="isNewModalOpen = false" @save="handleCreateScript" />

    <!-- 编辑脚本信息 Modal -->
    <ScriptConfigModal
      :is-open="isEditModalOpen"
      :editing-script="editingScriptData"
      @close="
        isEditModalOpen = false;
        editingScriptData = null;
      "
      @save="handleUpdateScript"
    />
  </div>
</template>

<style scoped>
@import '../assets/css/script-list.css';
</style>
