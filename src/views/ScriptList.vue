<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import type { Ref } from 'vue';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useScripts } from '../assets/js/useScripts';
import { confirm } from '@tauri-apps/plugin-dialog';
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
  ChevronDown,
} from 'lucide-vue-next';
import ScriptConfigModal from './script-list/components/ScriptConfigModal.vue';
import type { ScriptTable, ScriptInfo, DetectorType, RecognizerType, ScriptTaskTable } from '../types/bindings';

interface ExtendedScriptInfo extends ScriptInfo {
  tasks?: ScriptTaskTable[];
  templates?: string[];
}

interface ExtendedScriptTable extends Omit<ScriptTable, 'data'> {
  data: ExtendedScriptInfo;
}

const { scripts, selectedScript, selectedTemplate, getAllScripts, saveScript, deleteScript, selectScript } =
  useScripts() as unknown as {
    scripts: Ref<ExtendedScriptTable[]>;
    selectedScript: Ref<ExtendedScriptTable | null>;
    selectedTemplate: Ref<string | null>;
    getAllScripts: () => Promise<ExtendedScriptTable[]>;
    saveScript: (data: any) => Promise<void>;
    deleteScript: (script: ExtendedScriptTable) => Promise<void>;
    selectScript: (script: ExtendedScriptTable) => void;
  };

const isNewModalOpen = ref(false);
const isEditModalOpen = ref(false);
const editingScriptData = ref<ExtendedScriptTable | null>(null);
const searchQuery = ref('');
const openDropdownId = ref<string | null>(null);
const expandedModelInfo = ref<'imgDet' | 'txtDet' | 'txtRec' | null>(null);
const globalDelay = ref(500);
const randomRange = ref(5);

const openNewModal = () => {
  isNewModalOpen.value = true;
};

const handleCreateScript = async (scriptData: any) => {
  try {
    await saveScript(scriptData);
    isNewModalOpen.value = false;
  } catch (e) {
    alert('创建失败: ' + e);
    console.error(e);
  }
};

const openEditModal = (script: ExtendedScriptTable) => {
  editingScriptData.value = script;
  isEditModalOpen.value = true;
};

const handleUpdateScript = async (scriptData: ScriptInfo & { id: string }) => {
  try {
    await saveScript(scriptData);
    isEditModalOpen.value = false;
    editingScriptData.value = null;
    if (selectedScript.value?.id === scriptData.id) {
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

const filteredScripts = computed(() => {
  if (!searchQuery.value) return scripts.value;
  const q = searchQuery.value.toLowerCase();
  return scripts.value.filter(
    (s) => s.data.name.toLowerCase().includes(q) || s.data.description?.toLowerCase().includes(q)
  );
});

const handleSelect = (script: ExtendedScriptTable) => {
  selectScript(script);
  openDropdownId.value = null;
};

const toggleDropdown = (e: MouseEvent, scriptId: string) => {
  e.preventDefault();
  e.stopPropagation();
  openDropdownId.value = openDropdownId.value === scriptId ? null : scriptId;
};

const confirmDelete = async (script: ExtendedScriptTable) => {
  openDropdownId.value = null;
  const confirmed = await confirm(`确定要删除 "${script.data.name}" 吗？\n此操作不可撤销。`, {
    title: '删除确认',
    kind: 'warning',
  });
  if (confirmed) {
    await deleteScript(script);
  }
};

const toggleModelInfo = (modelType: 'imgDet' | 'txtDet' | 'txtRec') => {
  expandedModelInfo.value = expandedModelInfo.value === modelType ? null : modelType;
};

const getModelTypeName = (model: DetectorType | RecognizerType | null | undefined) => {
  if (!model) return null;
  if ('Yolo11' in model) return 'YOLO11';
  if ('PaddleDbNet' in model) return 'PaddleDbNet';
  if ('PaddleCrnn' in model) return 'PaddleCrnn';
  return '自定义';
};

const getModelDisplayParams = (model: DetectorType | RecognizerType | null | undefined) => {
  if (!model) return [];

  if ('Yolo11' in model) {
    const m = model.Yolo11;
    return [
      { label: '执行器', value: m.base_model?.execution_provider || 'CPU' },
      { label: '输入尺寸', value: `${m.base_model?.input_width || 640} × ${m.base_model?.input_height || 640}` },
      { label: '类别数', value: m.class_count || 80 },
      { label: '置信度阈值', value: m.confidence_thresh || 0.25 },
      { label: 'IOU阈值', value: m.iou_thresh || 0.45 },
    ];
  }

  if ('PaddleDbNet' in model) {
    const m = model.PaddleDbNet;
    return [
      { label: '执行器', value: m.base_model?.execution_provider || 'CPU' },
      { label: '输入尺寸', value: `${m.base_model?.input_width || 640} × ${m.base_model?.input_height || 640}` },
      { label: '二值化阈值', value: m.db_thresh || 0.3 },
      { label: '框阈值', value: m.db_box_thresh || 0.5 },
      { label: '扩充比例', value: m.unclip_ratio || 1.5 },
      { label: '膨胀', value: m.use_dilation ? '是' : '否' },
    ];
  }

  if ('PaddleCrnn' in model) {
    const m = model.PaddleCrnn;
    return [
      { label: '执行器', value: m.base_model?.execution_provider || 'CPU' },
      { label: '输入尺寸', value: `${m.base_model?.input_width || 320} × ${m.base_model?.input_height || 48}` },
    ];
  }
  return [];
};

const formatTime = (time: string | null | undefined) => {
  if (!time) return '无';
  return time.split('T')[0];
};

const openEditor = async (scriptId: string) => {
  const webview = new WebviewWindow('script-editor', {
    url: '/editor?id=' + scriptId,
    title: '逻辑编辑',
    width: 1400,
    height: 900,
    center: true,
    focus: true,
  });

  await webview.once('tauri://error', function (e) {
    console.error('打开编辑器失败', e);
  });
};

onMounted(async () => {
  await getAllScripts();
});
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

      <div class="grow overflow-y-auto overflow-x-visible custom-scrollbar p-2 space-y-2">
        <div
          v-for="script in filteredScripts"
          :key="script.id"
          @click="handleSelect(script)"
          class="group relative rounded-xl border transition-all cursor-pointer"
          :class="[
            selectedScript?.id === script.id
              ? 'bg-primary/10 border-primary shadow-sm'
              : 'bg-base-200 border-transparent hover:bg-base-300 hover:border-base-content/10',
          ]"
        >
          <div class="p-3 flex items-start gap-3">
            <div class="w-10 h-10 rounded-lg bg-base-100 flex items-center justify-center shadow-sm">
              <Download v-if="script.data.scriptType === 'published'" class="w-5 h-5 text-accent" />
              <Box v-else class="w-5 h-5 text-secondary" />
            </div>

            <div class="grow min-w-0">
              <div class="flex items-center justify-between gap-1">
                <p
                  class="font-semibold truncate text-sm"
                  :class="selectedScript?.id === script.id ? 'text-primary' : ''"
                >
                  {{ script.data.name }}
                </p>
                <span class="text-[10px] opacity-50">{{ script.data.verName }}</span>
              </div>
              <p class="text-xs opacity-60 line-clamp-1 mt-0.5">{{ script.data.description }}</p>
            </div>

            <!-- 下拉操作菜单 -->
            <details class="dropdown dropdown-left flex-none self-center ml-1" :open="openDropdownId === script.id">
              <summary
                @click="toggleDropdown($event, script.id)"
                class="btn btn-ghost btn-xs btn-circle hover:bg-base-content/10 cursor-pointer list-none"
              >
                <MoreHorizontal class="w-4 h-4" />
              </summary>
              <ul
                class="dropdown-content menu bg-base-100 rounded-xl w-44 p-1.5 shadow-xl border border-base-content/10 z-50"
              >
                <template v-if="script.data.scriptType === 'dev'">
                  <li>
                    <a
                      @click="
                        openEditModal(script);
                        openDropdownId = null;
                      "
                      class="flex items-center gap-3 text-sm hover:bg-info/10 hover:text-info cursor-pointer"
                    >
                      <Edit class="w-4 h-4" />
                      <span>编辑信息</span>
                    </a>
                  </li>
                  <li>
                    <a
                      @click="
                        openEditor(script.id);
                        openDropdownId = null;
                      "
                      class="flex items-center gap-3 text-sm hover:bg-secondary/10 hover:text-secondary cursor-pointer"
                    >
                      <Settings class="w-4 h-4" />
                      <span>编辑逻辑</span>
                    </a>
                  </li>
                  <li class="my-1"><hr class="border-base-content/10" /></li>
                  <li v-if="!script.data.cloudId">
                    <a
                      @click="openDropdownId = null"
                      class="flex items-center gap-3 text-sm hover:bg-accent/10 hover:text-accent cursor-pointer"
                    >
                      <Download class="w-4 h-4 rotate-180" />
                      <span>上传到云端</span>
                    </a>
                  </li>
                  <li v-else>
                    <a
                      @click="openDropdownId = null"
                      class="flex items-center gap-3 text-sm hover:bg-accent/10 hover:text-accent cursor-pointer"
                    >
                      <Download class="w-4 h-4 rotate-180" />
                      <span>更新云端版本</span>
                    </a>
                  </li>
                </template>
                <template v-else-if="script.data.scriptType === 'published'">
                  <li>
                    <a
                      @click="openDropdownId = null"
                      class="flex items-center gap-3 text-sm hover:bg-primary/10 hover:text-primary cursor-pointer"
                    >
                      <Clock class="w-4 h-4" />
                      <span>检查更新</span>
                    </a>
                  </li>
                </template>
                <li class="my-1"><hr class="border-base-content/10" /></li>
                <li>
                  <a
                    @click="confirmDelete(script)"
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
            :class="selectedScript.data.scriptType === 'dev' ? 'badge-secondary' : 'badge-accent'"
          >
            {{ selectedScript.data.scriptType === 'dev' ? '本地开发' : '云端下载' }}
          </div>
        </div>

        <div class="grow overflow-y-auto custom-scrollbar p-4 space-y-6">
          <section>
            <div class="space-y-3">
              <div class="flex flex-col">
                <span class="text-[10px] opacity-50">名称</span>
                <span class="text-sm font-medium">{{ selectedScript.data.name }}</span>
              </div>
              <div class="flex flex-col">
                <span class="text-[10px] opacity-50">版本号</span>
                <span class="text-sm font-medium"
                  >{{ selectedScript.data.verName }} ({{ selectedScript.data.verNum }})</span
                >
              </div>
              <div class="flex flex-col">
                <span class="text-[10px] opacity-50">脚本描述</span>
                <p class="text-sm opacity-80 leading-relaxed">{{ selectedScript.data.description || '暂无描述' }}</p>
              </div>
            </div>
          </section>

          <section class="bg-base-100/50 p-3 rounded-xl border border-base-content/5">
            <h3 class="text-xs font-bold uppercase tracking-wider opacity-40 mb-3 flex items-center gap-1">
              <Cpu class="w-3 h-3" /> 模型配置
            </h3>
            <div class="grid grid-cols-1 gap-2">
              <div class="flex items-center justify-between">
                <span class="text-xs opacity-60">应用包名</span>
                <span class="text-xs font-mono bg-base-300 px-1.5 py-0.5 rounded max-w-[140px] truncate">{{
                  selectedScript.data.pkgName || '未指定'
                }}</span>
              </div>
            </div>
            <div class="divider m-0 opacity-10"></div>

            <div class="space-y-4 pt-2">
              <!-- 图像识别 -->
              <div class="space-y-1">
                <div class="flex items-center justify-between">
                  <span class="text-xs opacity-60">图像识别</span>
                  <div class="flex items-center gap-1">
                    <span class="text-xs font-medium">{{
                      getModelTypeName(selectedScript.data.imgDetModel) || '无'
                    }}</span>
                    <button
                      v-if="selectedScript.data.imgDetModel"
                      @click="toggleModelInfo('imgDet')"
                      class="btn btn-ghost btn-xs btn-circle cursor-pointer"
                    >
                      <ChevronDown
                        class="w-3 h-3 transition-transform"
                        :class="expandedModelInfo === 'imgDet' ? 'rotate-180' : ''"
                      />
                    </button>
                  </div>
                </div>
                <div
                  v-if="expandedModelInfo === 'imgDet' && selectedScript.data.imgDetModel"
                  class="bg-base-200/50 rounded-lg p-2 space-y-1"
                >
                  <div
                    v-for="param in getModelDisplayParams(selectedScript.data.imgDetModel)"
                    :key="param.label"
                    class="flex justify-between text-[10px]"
                  >
                    <span class="opacity-50">{{ param.label }}</span>
                    <span class="font-medium">{{ param.value }}</span>
                  </div>
                </div>
              </div>

              <!-- 文本检测 -->
              <div class="space-y-1">
                <div class="flex items-center justify-between">
                  <span class="text-xs opacity-60">文本检测</span>
                  <div class="flex items-center gap-1">
                    <span class="text-xs font-medium">{{
                      getModelTypeName(selectedScript.data.txtDetModel) || '内置'
                    }}</span>
                    <button
                      v-if="selectedScript.data.txtDetModel"
                      @click="toggleModelInfo('txtDet')"
                      class="btn btn-ghost btn-xs btn-circle cursor-pointer"
                    >
                      <ChevronDown
                        class="w-3 h-3 transition-transform"
                        :class="expandedModelInfo === 'txtDet' ? 'rotate-180' : ''"
                      />
                    </button>
                  </div>
                </div>
                <div
                  v-if="expandedModelInfo === 'txtDet' && selectedScript.data.txtDetModel"
                  class="bg-base-200/50 rounded-lg p-2 space-y-1"
                >
                  <div
                    v-for="param in getModelDisplayParams(selectedScript.data.txtDetModel)"
                    :key="param.label"
                    class="flex justify-between text-[10px]"
                  >
                    <span class="opacity-50">{{ param.label }}</span>
                    <span class="font-medium">{{ param.value }}</span>
                  </div>
                </div>
              </div>

              <!-- 文本识别 -->
              <div class="space-y-1">
                <div class="flex items-center justify-between">
                  <span class="text-xs opacity-60">文本识别</span>
                  <div class="flex items-center gap-1">
                    <span class="text-xs font-medium">{{
                      getModelTypeName(selectedScript.data.txtRecModel) || '内置'
                    }}</span>
                    <button
                      v-if="selectedScript.data.txtRecModel"
                      @click="toggleModelInfo('txtRec')"
                      class="btn btn-ghost btn-xs btn-circle cursor-pointer"
                    >
                      <ChevronDown
                        class="w-3 h-3 transition-transform"
                        :class="expandedModelInfo === 'txtRec' ? 'rotate-180' : ''"
                      />
                    </button>
                  </div>
                </div>
                <div
                  v-if="expandedModelInfo === 'txtRec' && selectedScript.data.txtRecModel"
                  class="bg-base-200/50 rounded-lg p-2 space-y-1"
                >
                  <div
                    v-for="param in getModelDisplayParams(selectedScript.data.txtRecModel)"
                    :key="param.label"
                    class="flex justify-between text-[10px]"
                  >
                    <span class="opacity-50">{{ param.label }}</span>
                    <span class="font-medium">{{ param.value }}</span>
                  </div>
                </div>
              </div>
            </div>
          </section>

          <section class="bg-base-100/50 p-3 rounded-xl border border-base-content/5 space-y-3">
            <div class="flex items-center justify-between">
              <span class="text-xs opacity-60 flex items-center gap-1.5"><User class="w-3 h-3" /> 作者</span>
              <span class="text-xs font-medium">{{ selectedScript.data.userName || '未知' }}</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-xs opacity-60 flex items-center gap-1.5"><Activity class="w-3 h-3" /> 下载量</span>
              <span class="text-xs font-medium">{{ selectedScript.data.downloadCount || 0 }}</span>
            </div>
            <div class="divider m-0 opacity-10"></div>
            <div class="flex items-center justify-between">
              <span class="text-xs opacity-60 flex items-center gap-1.5"><Calendar class="w-3 h-3" /> 创建时间</span>
              <span class="text-[10px] font-medium opacity-80">{{ formatTime(selectedScript.data.createTime) }}</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-xs opacity-60 flex items-center gap-1.5"><Clock class="w-3 h-3" /> 最后更新</span>
              <span class="text-[10px] font-medium opacity-80">{{ formatTime(selectedScript.data.updateTime) }}</span>
            </div>
          </section>
        </div>
      </div>
      <div v-else class="flex flex-col items-center justify-center h-full opacity-20 p-8 text-center">
        <Info class="w-16 h-16 mb-4" />
        <p class="text-lg font-medium">详情</p>
        <p class="text-sm">选中脚本以查看详细配置</p>
      </div>
    </div>

    <!-- 第三栏：任务设置 -->
    <div class="grow bg-base-100 flex flex-col">
      <div v-if="selectedScript" class="flex flex-col h-full animate-in fade-in slide-in-from-bottom-2 duration-400">
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
              <template v-if="selectedScript.data.templates">
                <option v-for="tpl in selectedScript.data.templates" :key="tpl" :value="tpl">{{ tpl }}</option>
              </template>
              <option disabled v-if="selectedScript.data.templates?.length">──────</option>
              <option value="add_new">+ 新增模板</option>
            </select>
          </div>
        </div>

        <div class="grow overflow-hidden flex flex-col bg-base-100">
          <div class="flex-none p-4 pb-2 space-y-3 border-b border-base-content/5">
            <h2 class="text-[11px] font-bold uppercase tracking-widest opacity-50">基础设置</h2>
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
                <span class="text-xs font-medium opacity-80 text-nowrap">随机范围</span>
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
            <h2 class="text-[11px] font-bold uppercase tracking-widest opacity-50 mb-4">任务设置</h2>
            <div class="grow overflow-y-auto custom-scrollbar space-y-1">
              <div
                v-for="task in selectedScript.data.tasks"
                :key="task.id"
                class="group flex items-center gap-3 p-2 rounded-lg hover:bg-base-200 transition-colors"
              >
                <div class="flex-none flex items-center">
                  <input
                    type="checkbox"
                    :checked="!task.isHidden"
                    @change="(e) => (task.isHidden = !(e.target as HTMLInputElement).checked)"
                    class="checkbox checkbox-sm checkbox-primary"
                  />
                </div>
                <div class="grow flex items-center gap-3">
                  <span class="text-sm" :class="!task.isHidden ? 'font-medium opacity-90' : 'opacity-40 italic'">{{
                    task.name
                  }}</span>
                </div>
              </div>
              <div v-if="!selectedScript.data.tasks?.length" class="text-center py-10 opacity-30 italic text-sm">
                暂无预设任务
              </div>
            </div>
          </div>
        </div>
      </div>
      <div v-else class="flex flex-col items-center justify-center h-full opacity-20 p-8 text-center">
        <Settings class="w-16 h-16 mb-4" />
        <p class="text-lg font-medium">任务设置</p>
        <p class="text-sm">选中脚本以进行配置</p>
      </div>
    </div>

    <!-- Modals -->
    <ScriptConfigModal :is-open="isNewModalOpen" @close="isNewModalOpen = false" @save="handleCreateScript" />
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
