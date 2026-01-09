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
  X
} from 'lucide-vue-next';

const { 
  scripts, 
  selectedScript, 
  selectedTemplate, 
  getAllScripts,
  saveScript,
  deleteScript, 
  selectScript, 
  editScript 
} = useScripts();

onMounted(() => {
  getAllScripts();
});

const isNewModalOpen = ref(false);
const newScript = reactive({
  name: '',
  description: '',
  pkgName: '',
  imgDetModelType: 'None', 
  txtDetModelType: 'None',
  txtRecModelType: 'None',
  yoloParams: {
    inputWidth: 640,
    inputHeight: 640,
    classCount: 80,
    confidenceThresh: 0.25,
    iouThresh: 0.45,
    labelPath: ''
  },
  dbNetParams: {
    inputWidth: 640,
    inputHeight: 640,
    dbThresh: 0.3,
    dbBoxThresh: 0.6,
    unclipRatio: 1.5,
    useDilation: false
  }
});

const openNewModal = () => {
  isNewModalOpen.value = true;
};

const handleCreateScript = async () => {
  if (!newScript.name) return;

  const scriptData = {
    name: newScript.name,
    description: newScript.description || null,
    pkgName: newScript.pkgName || null,
    scriptType: 'Custom',
    verName: 'v1.0.0',
    verNum: 1,
    latestVer: 1,
    downloadCount: 0,
    isValid: true,
    createTime: new Date().toISOString(),
    updateTime: new Date().toISOString(),
    userName: 'Local User',
    tasks: [],
    templates: []
  };

  if (newScript.imgDetModelType === 'Yolo11') {
    scriptData.imgDetModel = {
      Yolo11: {
        baseModel: {
          inputWidth: parseInt(newScript.yoloParams.inputWidth),
          inputHeight: parseInt(newScript.yoloParams.inputHeight),
          modelPath: '',
          executionProvider: 'Cpu',
          intraThreadNum: 4,
          intraSpinning: true,
          interThreadNum: 4,
          interSpinning: true,
          modelType: 'Yolo11'
        },
        classCount: parseInt(newScript.yoloParams.classCount),
        classLabels: [],
        confidenceThresh: parseFloat(newScript.yoloParams.confidenceThresh),
        iouThresh: parseFloat(newScript.yoloParams.iouThresh),
        labelPath: newScript.yoloParams.labelPath,
        txtIdx: null
      }
    };
  }

  if (newScript.txtDetModelType === 'PaddleDbNet') {
    scriptData.txtDetModel = {
      PaddleDbNet: {
        baseModel: {
          inputWidth: parseInt(newScript.dbNetParams.inputWidth),
          inputHeight: parseInt(newScript.dbNetParams.inputHeight),
          modelPath: '',
          executionProvider: 'Cpu',
          intraThreadNum: 4,
          intraSpinning: true,
          interThreadNum: 4,
          interSpinning: true,
          modelType: 'PaddleDet5'
        },
        dbThresh: parseFloat(newScript.dbNetParams.dbThresh),
        dbBoxThresh: parseFloat(newScript.dbNetParams.dbBoxThresh),
        unclipRatio: parseFloat(newScript.dbNetParams.unclipRatio),
        useDilation: newScript.dbNetParams.useDilation
      }
    };
  }

  try {
    await saveScript(scriptData);
    isNewModalOpen.value = false;
    newScript.name = '';
    newScript.description = '';
  } catch (e) {
    alert('创建失败: ' + e);
  }
};

const searchQuery = ref('');
const expandedActionsId = ref(null);

const filteredScripts = computed(() => {
  if (!searchQuery.value) return scripts.value;
  return scripts.value.filter(s => 
    s.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
    s.description?.toLowerCase().includes(searchQuery.value.toLowerCase())
  );
});

const toggleActions = (e, scriptName) => {
  e.stopPropagation();
  expandedActionsId.value = expandedActionsId.value === scriptName ? null : scriptName;
};

const handleSelect = (script) => {
  selectScript(script);
  expandedActionsId.value = null; 
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
              : 'bg-base-200 border-transparent hover:bg-base-300 hover:border-base-content/10'
          ]"
        >
          <div class="p-3 flex items-start gap-3">
            <div class="w-10 h-10 rounded-lg bg-base-100 flex items-center justify-center shadow-sm">
              <Activity v-if="script.scriptType === 'Official'" class="w-5 h-5 text-primary" />
              <Box v-else class="w-5 h-5 text-secondary" />
            </div>
            
            <div class="grow min-w-0">
              <div class="flex items-center justify-between gap-1">
                <p class="font-semibold truncate text-sm" :class="selectedScript?.name === script.name ? 'text-primary' : ''">
                  {{ script.name }}
                </p>
                <span class="text-[10px] opacity-50">{{ script.verName }}</span>
              </div>
              <p class="text-xs opacity-60 line-clamp-1 mt-0.5">{{ script.description }}</p>
            </div>

            <!-- 扩展功能键 -->
            <div class="flex-none flex items-center self-center ml-1">
              <button 
                @click="toggleActions($event, script.name)"
                class="btn btn-ghost btn-xs btn-circle hover:bg-base-content/10"
              >
                <MoreHorizontal class="w-4 h-4" />
              </button>
            </div>
          </div>

          <!-- 展开的操作面板 -->
          <div 
            v-if="expandedActionsId === script.name"
            class="absolute inset-y-0 right-0 bg-base-100 border-l border-base-content/10 flex items-center px-2 gap-1 z-10 animate-in slide-in-from-right duration-200"
          >
            <button 
              v-if="script.scriptType === 'Custom'"
              @click.stop="editScript(script)"
              class="btn btn-square btn-ghost btn-sm text-info tooltip tooltip-left" 
              data-tip="编辑"
            >
              <Edit class="w-4 h-4" />
            </button>
            <button 
              @click.stop="deleteScript(script)"
              class="btn btn-square btn-ghost btn-sm text-error tooltip tooltip-left" 
              data-tip="删除"
            >
              <Trash2 class="w-4 h-4" />
            </button>
            <button @click.stop="expandedActionsId = null" class="btn btn-square btn-ghost btn-sm">
              <ChevronRight class="w-4 h-4 rotate-180" />
            </button>
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
          <div class="badge badge-sm" :class="selectedScript.scriptType === 'Official' ? 'badge-primary' : 'badge-secondary'">
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
                  <span class="text-xs font-mono bg-base-300 px-1.5 py-0.5 rounded">{{ selectedScript.pkgName || '未指定' }}</span>
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
                     <input type="number" v-model="globalDelay" class="w-12 bg-transparent text-xs font-bold text-right outline-none text-primary" />
                     <span class="text-[10px] opacity-40">ms</span>
                   </div>
                 </div>
                 
                 <div class="flex items-center justify-between p-2 rounded-lg bg-base-200/50 border border-base-content/5">
                   <span class="text-xs font-medium opacity-80 text-nowrap">随机坐标范围</span>
                   <div class="flex items-center gap-1">
                     <input type="number" v-model="randomRange" class="w-12 bg-transparent text-xs font-bold text-right outline-none text-primary" />
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
    <dialog class="modal" :class="{ 'modal-open': isNewModalOpen }">
      <div class="modal-box w-11/12 max-w-2xl bg-base-100 p-0 overflow-hidden border border-base-content/10">
        <div class="p-4 border-b border-base-content/5 flex items-center justify-between bg-base-200/50">
          <h3 class="font-bold flex items-center gap-2">
            <Plus class="w-5 h-5 text-primary" />
            新建
          </h3>
          <button @click="isNewModalOpen = false" class="btn btn-ghost btn-sm btn-square">
            <X class="w-4 h-4" />
          </button>
        </div>
        
        <div class="p-6 max-h-[70vh] overflow-y-auto custom-scrollbar space-y-6">
          <!-- 基础设置 -->
          <div class="grid grid-cols-2 gap-4">
            <div class="form-control col-span-1">
              <label class="label"><span class="label-text font-bold">名称</span></label>
              <input type="text" v-model="newScript.name" class="input input-bordered w-full" />
            </div>
            <div class="form-control">
              <label class="label"><span class="label-text font-bold">包名/主Activity</span></label>
              <input type="text" v-model="newScript.pkgName" placeholder="example.app/com.UnityPlayerActivity" class="input input-bordered w-full" />
            </div>
            <div class="form-control">
              <label class="label"><span class="label-text font-bold">描述</span></label>
              <input type="text" v-model="newScript.description" class="input input-bordered w-full" />
            </div>
          </div>

          <div class="divider opacity-50">模型配置</div>

          <!-- 模型选择 -->
          <div class="grid grid-cols-2 gap-6">
             <!-- 图像检测 -->
             <div class="space-y-4">
                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-bold flex items-center gap-2">
                      <Eye class="w-4 h-4 text-primary" /> 图像检测模型
                    </span>
                  </label>
                  <select v-model="newScript.imgDetModelType" class="select select-bordered w-full">
                    <option value="None">不设置 (禁用)</option>
                    <option value="Yolo11">YOLO11</option>
<!--                    <option value="PaddleDbNet">DBNet (文字定位专家)</option>-->
                  </select>
                </div>

                <!-- YOLO 参数 -->
                <div v-if="newScript.imgDetModelType === 'Yolo11'" class="p-4 bg-base-200 rounded-xl space-y-3 animate-in fade-in duration-300">
                   <div class="grid grid-cols-2 gap-2">
                      <div class="form-control">
                        <label class="label text-[10px] opacity-60"><span>输入宽度</span></label>
                        <input type="number" v-model="newScript.yoloParams.inputWidth" class="input input-sm input-bordered" />
                      </div>
                      <div class="form-control">
                        <label class="label text-[10px] opacity-60"><span>输入高度</span></label>
                        <input type="number" v-model="newScript.yoloParams.inputHeight" class="input input-sm input-bordered" />
                      </div>
                   </div>
                   <div class="form-control">
                     <label class="label text-[10px] opacity-60"><span>置信度：{{newScript.yoloParams.confidenceThresh}}</span></label>
                     <input type="range" min="0" max="1" step="0.05" v-model="newScript.yoloParams.confidenceThresh" class="range range-xs range-primary" />
                     <div class="flex justify-between text-[10px] mt-1 opacity-40"><span>0</span><span>{{newScript.yoloParams.confidenceThresh}}</span><span>1</span></div>
                   </div>
                  <div class="form-control">
                    <label class="label text-[10px] opacity-60"><span>iou阈值：{{newScript.yoloParams.iouThresh}}</span></label>
                    <input type="range" min="0" max="1" step="0.05" v-model="newScript.yoloParams.iouThresh" class="range range-xs range-primary" />
                    <div class="flex justify-between text-[10px] mt-1 opacity-40"><span>0</span><span>{{newScript.yoloParams.iouThresh}}</span><span>1</span></div>
                  </div>
                </div>
             </div>

             <!-- 文本检测 -->
             <div class="space-y-4">
                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-bold flex items-center gap-2">
                      <FileJson class="w-4 h-4 text-secondary" /> 文本检测模型
                    </span>
                  </label>
                  <select v-model="newScript.txtDetModelType" class="select select-bordered w-full">
                    <option value="None">不设置 (禁用OCR)</option>
                    <option value="PaddleDbNet">Paddle DBNet (推荐)</option>
                    <option value="Yolo11">YOLO11</option>
                  </select>
                </div>

                <!-- DBNet 参数 -->
                <div v-if="newScript.txtDetModelType === 'PaddleDbNet'" class="p-4 bg-base-200 rounded-xl space-y-3 animate-in fade-in duration-300">
                   <div class="form-control">
                     <label class="label text-[10px] opacity-60"><span>二值化阈值 (dbThresh)</span></label>
                     <input type="number" step="0.1" v-model="newScript.dbNetParams.dbThresh" class="input input-sm input-bordered" />
                   </div>
                   <div class="form-control">
                     <label class="label text-[10px] opacity-60"><span>框扩充比例 (unclip)</span></label>
                     <input type="number" step="0.1" v-model="newScript.dbNetParams.unclipRatio" class="input input-sm input-bordered" />
                   </div>
                   <div class="flex items-center gap-2">
                      <input type="checkbox" v-model="newScript.dbNetParams.useDilation" class="checkbox checkbox-xs" />
                      <span class="text-xs opacity-60">使用膨胀操作</span>
                   </div>
                </div>
             </div>
          </div>

          <!-- 警告提示 -->
          <div v-if="newScript.imgDetModelType === 'None' || newScript.txtDetModelType === 'None'" 
               class="alert alert-warning shadow-sm border-none bg-warning/10 text-warning-content py-3 rounded-xl">
            <AlertTriangle class="w-5 h-5" />
            <div class="text-xs">
              <h3 class="font-bold">注意事项</h3>
              <p>未配置检测模型将导致脚本编辑中的 OCR 和 图像识别 功能不可用。</p>
            </div>
          </div>
        </div>

        <div class="p-4 border-t border-base-content/5 bg-base-200/30 flex justify-end gap-3">
          <button @click="isNewModalOpen = false" class="btn btn-ghost">取消</button>
          <button @click="handleCreateScript" class="btn btn-primary px-8" :disabled="!newScript.name">创建</button>
        </div>
      </div>
      <form method="dialog" class="modal-backdrop" @click="isNewModalOpen = false">
        <button>close</button>
      </form>
    </dialog>

  </div>
</template>

<style scoped>
@import "../assets/css/script-list.css";
</style>