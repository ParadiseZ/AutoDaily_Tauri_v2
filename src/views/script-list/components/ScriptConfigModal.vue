<script setup>
import { reactive, watch, ref, computed } from 'vue';
import { X, Plus, Edit, Eye, FileJson, Type, AlertTriangle } from 'lucide-vue-next';
import { open } from '@tauri-apps/plugin-dialog';

const props = defineProps({
  isOpen: Boolean,
  editingScript: {
    type: Object,
    default: null,
  },
});

const emit = defineEmits(['close', 'save']);

const executionProviders = ['CPU', 'DirectML', 'Cuda'];

const ModelSource = {
  BuiltIn: 'BuiltIn',
  Custom: 'Custom',
};

const ModelAlgorithm = {
  None: 'None',
  Yolo11: 'Yolo11',
  PaddleDbNet: 'PaddleDet5',
  PaddleCrnn: 'PaddleCrnn5',
};

const yoloDefaultParams = {
  inputWidth: 640,
  inputHeight: 640,
  classCount: 80,
  confidenceThresh: 0.25,
  iouThresh: 0.45,
};
const dbNetDefaultParams = {
  inputWidth: 640,
  inputHeight: 640,
  dbThresh: 0.3,
  dbBoxThresh: 0.5,
  unclipRatio: 1.5,
  useDilation: false,
};

const crnnDefaultParams = {
  inputWidth: 320,
  inputHeight: 48,
};

// Computed to check if we're in edit mode
const isEditMode = computed(() => !!props.editingScript);

const activeTab = ref('basic'); // basic, det, rec

const formState = reactive({
  name: '',
  pkgName: '',
  description: '',

  // Image Detection
  imgDetType: 'None',
  imgDetSource: 'Custom',
  yoloParams: {
    // Base Model Params
    modelPath: '',
    executionProvider: executionProviders[0],
    inputWidth: yoloDefaultParams.inputWidth,
    inputHeight: yoloDefaultParams.inputHeight,

    // Yolo Specific
    classCount: yoloDefaultParams.classCount,
    confidenceThresh: yoloDefaultParams.confidenceThresh,
    iouThresh: yoloDefaultParams.iouThresh,
    labelPath: '',
  },

  // Text Detection
  txtDetType: 'None',
  txtDetSource: 'Custom',
  txtDetYoloParams: {
    // Base Model Params
    modelPath: '',
    executionProvider: executionProviders[0],
    inputWidth: yoloDefaultParams.inputWidth,
    inputHeight: yoloDefaultParams.inputHeight,

    // Yolo Specific
    classCount: yoloDefaultParams.classCount,
    confidenceThresh: yoloDefaultParams.confidenceThresh,
    iouThresh: yoloDefaultParams.iouThresh,
    labelPath: '',
    txtIdx: 0,
  },

  dbNetParams: {
    // Base Model Params
    modelPath: '',
    executionProvider: executionProviders[0],
    inputWidth: dbNetDefaultParams.inputWidth,
    inputHeight: dbNetDefaultParams.inputHeight,

    // DBNet Specific
    dbThresh: dbNetDefaultParams.dbThresh,
    dbBoxThresh: dbNetDefaultParams.dbBoxThresh,
    unclipRatio: dbNetDefaultParams.unclipRatio,
    useDilation: false,
  },

  // Text Recognition
  txtRecType: 'PaddleCrnn5',
  txtRecSource: 'BuiltIn', // Default to BuiltIn for CRNN as it's common
  crnnParams: {
    // Base Model Params
    modelPath: '',
    executionProvider: executionProviders[0],
    inputWidth: crnnDefaultParams.inputWidth,
    inputHeight: crnnDefaultParams.inputHeight,

    // CRNN Specific
    dictPath: '',
  },
});

const handleSave = () => {
  if (!formState.name) return;

  // Construct the data structure matching Rust structs
  // Note: ScriptInfo is camelCase, but internal structs are snake_case (standard Rust serde default)

  // When editing, preserve existing script data; when creating, use defaults
  const existingScript = props.editingScript;

  const scriptData = {
    // Preserve id if editing (needed for update)
    id: existingScript?.id || undefined,
    userId: existingScript?.userId || '019b82ca280377a09eeb95dbdca056cc',
    name: formState.name,
    description: formState.description || null,
    pkgName: formState.pkgName || null,
    scriptType: existingScript?.scriptType || 'dev',
    verName: existingScript?.verName || 'v1.0.0',
    verNum: existingScript?.verNum || 1,
    latestVer: existingScript?.latestVer || 1,
    downloadCount: existingScript?.downloadCount || 0,
    isValid: existingScript?.isValid ?? true,
    createTime: existingScript?.createTime || new Date().toISOString(),
    updateTime: new Date().toISOString(), // Always update this
    userName: existingScript?.userName || 'Local User',
    cloudId: existingScript?.cloudId || null, // 云端关联 ID
    // Preserve tasks and templates when editing
    tasks: existingScript?.tasks || [],
    templates: existingScript?.templates || [],
  };

  // Image Detection Model Construction yolo11
  if (formState.imgDetType === ModelAlgorithm.Yolo11) {
    scriptData.imgDetModel = {
      Yolo11: {
        base_model: {
          input_width: parseInt(formState.yoloParams.inputWidth),
          input_height: parseInt(formState.yoloParams.inputHeight),
          model_source: formState.imgDetSource,
          model_path: formState.imgDetSource === ModelSource.BuiltIn ? '' : formState.yoloParams.modelPath,
          execution_provider: formState.yoloParams.executionProvider,
          intra_thread_num: 4,
          intra_spinning: true,
          inter_thread_num: 1,
          inter_spinning: true,
          model_type: ModelAlgorithm.Yolo11, // Ensure enum match
        },
        class_count: parseInt(formState.yoloParams.classCount),
        class_labels: [],
        confidence_thresh: parseFloat(formState.yoloParams.confidenceThresh),
        iou_thresh: parseFloat(formState.yoloParams.iouThresh),
        label_path: formState.yoloParams.labelPath,
        txt_idx: 0,
      },
    };
  }

  // Text Detection Model Construction
  if (formState.txtDetType === ModelAlgorithm.Yolo11) {
    scriptData.txtDetModel = {
      Yolo11: {
        base_model: {
          input_width: parseInt(formState.txtDetYoloParams.inputWidth),
          input_height: parseInt(formState.txtDetYoloParams.inputHeight),
          model_source: formState.txtDetSource,
          model_path: formState.txtDetSource === ModelSource.BuiltIn ? '' : formState.txtDetYoloParams.modelPath,
          execution_provider: formState.txtDetYoloParams.executionProvider,
          intra_thread_num: 4,
          intra_spinning: true,
          inter_thread_num: 1,
          inter_spinning: true,
          model_type: ModelAlgorithm.Yolo11,
        },
        class_count: parseInt(formState.txtDetYoloParams.classCount),
        class_labels: [],
        confidence_thresh: parseFloat(formState.txtDetYoloParams.confidenceThresh),
        iou_thresh: parseFloat(formState.txtDetYoloParams.iouThresh),
        label_path: formState.txtDetYoloParams.labelPath,
        txt_idx: parseInt(formState.txtDetYoloParams.txtIdx),
      },
    };
  }

  if (formState.txtDetType === ModelAlgorithm.PaddleDbNet) {
    scriptData.txtDetModel = {
      PaddleDbNet: {
        base_model: {
          input_width: parseInt(formState.dbNetParams.inputWidth),
          input_height: parseInt(formState.dbNetParams.inputHeight),
          model_source: formState.txtDetSource,
          model_path: formState.txtDetSource === ModelSource.BuiltIn ? '' : formState.dbNetParams.modelPath,
          execution_provider: formState.dbNetParams.executionProvider,
          intra_thread_num: 4,
          intra_spinning: true,
          inter_thread_num: 1,
          inter_spinning: true,
          model_type: ModelAlgorithm.PaddleDbNet,
        },
        db_thresh: parseFloat(formState.dbNetParams.dbThresh),
        db_box_thresh: parseFloat(formState.dbNetParams.dbBoxThresh),
        unclip_ratio: parseFloat(formState.dbNetParams.unclipRatio),
        use_dilation: formState.dbNetParams.useDilation,
      },
    };
  }

  // Text Recognition Model Construction
  if (formState.txtRecType === ModelAlgorithm.PaddleCrnn) {
    scriptData.txtRecModel = {
      PaddleCrnn: {
        base_model: {
          input_width: parseInt(formState.crnnParams.inputWidth),
          input_height: parseInt(formState.crnnParams.inputHeight),
          model_source: formState.txtRecSource,
          model_path: formState.txtRecSource === ModelSource.BuiltIn ? '' : formState.crnnParams.modelPath,
          execution_provider: formState.crnnParams.executionProvider,
          intra_thread_num: 4,
          intra_spinning: true,
          inter_thread_num: 1,
          inter_spinning: true,
          model_type: ModelAlgorithm.PaddleCrnn,
        },
        dict_path: formState.crnnParams.dictPath || null,
        dict: [],
      },
    };
  }

  emit('save', scriptData);
};

const handleSelectFile = async (type = 'model') => {
  try {
    const filters =
      type === 'model'
        ? [{ name: 'ONNX Model', extensions: ['onnx'] }]
        : [{ name: '配置/字典文件', extensions: ['yaml'] }];

    const selected = await open({
      multiple: false,
      filters: filters,
    });

    if (selected) {
      return selected;
    }
  } catch (e) {
    console.error('File selection failed', e);
  }
  return null;
};

const resetForm = () => {
  formState.name = '';
  formState.description = '';
  formState.pkgName = '';
  formState.imgDetType = ModelAlgorithm.None;
  formState.imgDetSource = ModelSource.Custom;
  formState.txtDetType = ModelAlgorithm.None;
  formState.txtDetSource = ModelSource.Custom;
  formState.txtRecType = ModelAlgorithm.PaddleCrnn;
  formState.txtRecSource = ModelSource.BuiltIn;
  activeTab.value = 'basic';

  // Reset Yolo Params
  formState.yoloParams.modelPath = '';
  formState.yoloParams.labelPath = '';
  formState.yoloParams.executionProvider = executionProviders[0];
  formState.yoloParams.inputWidth = yoloDefaultParams.inputWidth;
  formState.yoloParams.inputHeight = yoloDefaultParams.inputHeight;
  formState.yoloParams.classCount = yoloDefaultParams.classCount;
  formState.yoloParams.confidenceThresh = yoloDefaultParams.confidenceThresh;
  formState.yoloParams.iouThresh = yoloDefaultParams.iouThresh;

  // Reset TxtDet Yolo Params
  formState.txtDetYoloParams.modelPath = '';
  formState.txtDetYoloParams.labelPath = '';
  formState.txtDetYoloParams.executionProvider = executionProviders[0];
  formState.txtDetYoloParams.inputWidth = yoloDefaultParams.inputWidth;
  formState.txtDetYoloParams.inputHeight = yoloDefaultParams.inputHeight;
  formState.txtDetYoloParams.classCount = yoloDefaultParams.classCount;
  formState.txtDetYoloParams.confidenceThresh = yoloDefaultParams.confidenceThresh;
  formState.txtDetYoloParams.iouThresh = yoloDefaultParams.iouThresh;
  formState.txtDetYoloParams.txtIdx = 0;

  // Reset DBNet params
  formState.dbNetParams.modelPath = '';
  formState.dbNetParams.executionProvider = executionProviders[0];
  formState.dbNetParams.inputWidth = dbNetDefaultParams.inputWidth;
  formState.dbNetParams.inputHeight = dbNetDefaultParams.inputHeight;
  formState.dbNetParams.dbThresh = dbNetDefaultParams.dbThresh;
  formState.dbNetParams.dbBoxThresh = dbNetDefaultParams.dbBoxThresh;
  formState.dbNetParams.unclipRatio = dbNetDefaultParams.unclipRatio;
  formState.dbNetParams.useDilation = dbNetDefaultParams.useDilation;

  // Reset CRNN params
  formState.crnnParams.modelPath = '';
  formState.crnnParams.executionProvider = executionProviders[0];
  formState.crnnParams.inputWidth = crnnDefaultParams.inputWidth;
  formState.crnnParams.inputHeight = crnnDefaultParams.inputHeight;
  formState.crnnParams.dictPath = '';
};

// Populate form from editing script
const populateFormFromScript = (script) => {
  if (!script) return;

  formState.name = script.name || '';
  formState.description = script.description || '';
  formState.pkgName = script.pkgName || '';

  // Image Detection Model
  if (script.imgDetModel?.Yolo11) {
    formState.imgDetType = ModelAlgorithm.Yolo11;
    const yolo = script.imgDetModel.Yolo11;
    formState.imgDetSource = yolo.base_model?.model_source || ModelSource.Custom;
    formState.yoloParams.modelPath = yolo.base_model?.model_path || '';
    formState.yoloParams.labelPath = yolo.label_path || '';
    formState.yoloParams.executionProvider = yolo.base_model?.execution_provider || executionProviders[0];
    formState.yoloParams.inputWidth = yolo.base_model?.input_width || yoloDefaultParams.inputWidth;
    formState.yoloParams.inputHeight = yolo.base_model?.input_height || yoloDefaultParams.inputHeight;
    formState.yoloParams.classCount = yolo.class_count || 80;
    formState.yoloParams.confidenceThresh = yolo.confidence_thresh || yoloDefaultParams.confidenceThresh;
    formState.yoloParams.iouThresh = yolo.iou_thresh || yoloDefaultParams.iouThresh;
  } else {
    formState.imgDetType = ModelAlgorithm.None;
    formState.imgDetSource = ModelSource.Custom;
  }

  // Text Detection Model
  if (script.txtDetModel?.Yolo11) {
    formState.txtDetType = ModelAlgorithm.Yolo11;
    const yolo = script.txtDetModel.Yolo11;
    formState.txtDetSource = yolo.base_model?.model_source || ModelSource.Custom;
    formState.txtDetYoloParams.modelPath = yolo.base_model?.model_path || '';
    formState.txtDetYoloParams.labelPath = yolo.label_path || '';
    formState.txtDetYoloParams.executionProvider = yolo.base_model?.execution_provider || executionProviders[0];
    formState.txtDetYoloParams.inputWidth = yolo.base_model?.input_width || yoloDefaultParams.inputWidth;
    formState.txtDetYoloParams.inputHeight = yolo.base_model?.input_height || yoloDefaultParams.inputHeight;
    formState.txtDetYoloParams.classCount = yolo.class_count || 80;
    formState.txtDetYoloParams.confidenceThresh = yolo.confidence_thresh || yoloDefaultParams.confidenceThresh;
    formState.txtDetYoloParams.iouThresh = yolo.iou_thresh || yoloDefaultParams.iouThresh;
    formState.txtDetYoloParams.txtIdx = yolo.txt_idx || 0;
  } else if (script.txtDetModel?.PaddleDbNet) {
    formState.txtDetType = ModelAlgorithm.PaddleDbNet;
    const dbnet = script.txtDetModel.PaddleDbNet;
    formState.txtDetSource = dbnet.base_model?.model_source || ModelSource.Custom;
    formState.dbNetParams.modelPath = dbnet.base_model?.model_path || '';
    formState.dbNetParams.executionProvider = dbnet.base_model?.execution_provider || executionProviders[0];
    formState.dbNetParams.inputWidth = dbnet.base_model?.input_width || dbNetDefaultParams.inputWidth;
    formState.dbNetParams.inputHeight = dbnet.base_model?.input_height || dbNetDefaultParams.inputHeight;
    formState.dbNetParams.dbThresh = dbnet.db_thresh || dbNetDefaultParams.dbThresh;
    formState.dbNetParams.dbBoxThresh = dbnet.db_box_thresh || dbNetDefaultParams.dbBoxThresh;
    formState.dbNetParams.unclipRatio = dbnet.unclip_ratio || dbNetDefaultParams.unclipRatio;
    formState.dbNetParams.useDilation = dbnet.use_dilation || dbNetDefaultParams.useDilation;
  } else {
    formState.txtDetType = ModelAlgorithm.None;
    formState.txtDetSource = ModelSource.Custom;
  }

  // Text Recognition Model
  if (script.txtRecModel?.PaddleCrnn) {
    formState.txtRecType = ModelAlgorithm.PaddleCrnn;
    const crnn = script.txtRecModel.PaddleCrnn;
    formState.txtRecSource = crnn.base_model?.model_source || ModelSource.BuiltIn;
    formState.crnnParams.modelPath = crnn.base_model?.model_path || '';
    formState.crnnParams.executionProvider = crnn.base_model?.execution_provider || executionProviders[0];
    formState.crnnParams.inputWidth = crnn.base_model?.input_width || crnnDefaultParams.inputWidth;
    formState.crnnParams.inputHeight = crnn.base_model?.input_height || crnnDefaultParams.inputHeight;
    formState.crnnParams.dictPath = crnn.dict_path || '';
  } else {
    formState.txtRecType = ModelAlgorithm.None;
    formState.txtRecSource = ModelSource.BuiltIn;
  }

  activeTab.value = 'basic';
};

watch(
  () => props.isOpen,
  (newVal) => {
    if (newVal) {
      // When opening, check if we're editing
      if (props.editingScript) {
        populateFormFromScript(props.editingScript);
      } else {
        resetForm();
      }
    }
  }
);

// Also watch for editingScript changes while modal is open
watch(
  () => props.editingScript,
  (newScript) => {
    if (props.isOpen && newScript) {
      populateFormFromScript(newScript);
    }
  }
);
</script>

<template>
  <dialog class="modal" :class="{ 'modal-open': isOpen }">
    <div
      class="modal-box w-11/12 max-w-4xl bg-base-100 p-0 overflow-hidden border border-base-content/10 flex flex-col h-[85vh]"
    >
      <!-- Header -->
      <div class="p-4 border-b border-base-content/5 flex items-center justify-between bg-base-200/50 flex-none">
        <h3 class="font-bold flex items-center gap-2 text-lg">
          <Edit v-if="isEditMode" class="w-5 h-5 text-info" />
          <Plus v-else class="w-5 h-5 text-primary" />
          {{ isEditMode ? '编辑脚本信息' : '新建' }}
        </h3>
        <button @click="$emit('close')" class="btn btn-ghost btn-sm btn-square">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="flex grow min-h-0">
        <!-- Sidebar Tabs -->
        <div class="w-48 flex-none bg-base-200/30 border-r border-base-content/5 p-2 space-y-1">
          <div
            class="btn btn-sm w-full justify-start gap-2"
            :class="activeTab === 'basic' ? 'btn-primary' : 'btn-ghost'"
            @click="activeTab = 'basic'"
          >
            <div class="w-4 h-4 rounded-full border border-current flex items-center justify-center text-[10px]">1</div>
            基础信息
          </div>
          <div class="divider my-1 opacity-50 text-[10px]">模型配置</div>
          <button
            class="btn btn-sm w-full justify-start gap-2"
            :class="activeTab === 'det' ? 'btn-active' : 'btn-ghost'"
            @click="activeTab = 'det'"
          >
            <Eye class="w-4 h-4 opacity-70" />
            检测模型
          </button>
          <button
            class="btn btn-sm w-full justify-start gap-2"
            :class="activeTab === 'rec' ? 'btn-active' : 'btn-ghost'"
            @click="activeTab = 'rec'"
          >
            <Type class="w-4 h-4 opacity-70" />
            文本识别
          </button>
        </div>

        <!-- Content Area -->
        <div class="grow overflow-y-auto custom-scrollbar p-6">
          <!-- Basic Info Tab -->
          <div v-show="activeTab === 'basic'" class="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300">
            <div class="form-control hover:bg-base-200/50 p-2 rounded-lg transition-colors">
              <label class="label"
                ><span class="label-text font-bold">名称 <span class="text-error">*</span></span></label
              >
              <input type="text" v-model="formState.name" class="input input-bordered w-full" />
            </div>
            <div class="form-control hover:bg-base-200/50 p-2 rounded-lg transition-colors">
              <label class="label"
                ><span class="label-text font-bold">包名/主Activity<span class="text-error">*</span></span></label
              >
              <input
                type="text"
                v-model="formState.pkgName"
                placeholder="com.example.game/com.unity3d.player.UnityPlayerActivity"
                class="input input-bordered w-full"
              />
            </div>
            <div class="form-control hover:bg-base-200/50 p-2 rounded-lg transition-colors">
              <label class="label"><span class="label-text font-bold">描述</span></label
              ><br />
              <textarea v-model="formState.description" class="textarea textarea-bordered h-10"></textarea>
            </div>
          </div>

          <!-- Detection Tab -->
          <div v-show="activeTab === 'det'" class="space-y-8 animate-in fade-in slide-in-from-right-4 duration-300">
            <!-- Img Det Section -->
            <section class="space-y-4">
              <div class="flex items-center justify-between border-b border-base-content/10 pb-2">
                <h4 class="font-bold flex items-center gap-2">
                  <Eye class="w-4 h-4 text-primary" /> 目标/文本检测模型
                </h4>
                <div class="flex gap-2">
                  <select v-model="formState.imgDetType" class="select select-bordered select-sm w-32">
                    <option :value="ModelAlgorithm.None">不设置</option>
                    <option :value="ModelAlgorithm.Yolo11">YOLO11</option>
                  </select>
                  <select
                    v-if="formState.imgDetType !== ModelAlgorithm.None"
                    v-model="formState.imgDetSource"
                    class="select select-bordered select-sm w-24"
                  >
                    <option :value="ModelSource.BuiltIn">内置</option>
                    <option :value="ModelSource.Custom">自定义</option>
                  </select>
                </div>
              </div>

              <div
                v-if="formState.imgDetType === ModelAlgorithm.Yolo11"
                class="bg-base-200/50 p-4 rounded-xl space-y-4 border border-base-content/5"
              >
                <div class="grid grid-cols-2 gap-4">
                  <!-- Base Model Param Common -->
                  <div class="form-control col-span-1">
                    <label class="label text-xs font-bold opacity-70">模型路径 (ONNX)</label>
                    <div v-if="formState.imgDetSource === ModelSource.Custom" class="flex gap-2">
                      <input
                        type="text"
                        v-model="formState.yoloParams.modelPath"
                        class="input input-sm input-bordered grow text-xs"
                        placeholder="/path/to/model.onnx"
                      />
                      <button
                        @click="
                          async () => {
                            const p = await handleSelectFile('model');
                            if (p) formState.yoloParams.modelPath = p;
                          }
                        "
                        class="btn btn-sm btn-square btn-ghost border border-base-content/20"
                      >
                        ...
                      </button>
                    </div>
                    <div v-else class="text-xs opacity-50 py-1.5 italic border border-transparent px-1">
                      使用内置默认模型路径
                    </div>
                  </div>
                  <div class="form-control col-span-1">
                    <label class="label text-xs font-bold opacity-70">标签路径 (Label Path)</label>
                    <div class="flex gap-2">
                      <input
                        type="text"
                        v-model="formState.yoloParams.labelPath"
                        class="input input-sm input-bordered grow text-xs"
                        placeholder="/path/to/config.yaml"
                      />
                      <button
                        @click="
                          async () => {
                            const p = await handleSelectFile('config');
                            if (p) formState.yoloParams.labelPath = p;
                          }
                        "
                        class="btn btn-sm btn-square btn-ghost border border-base-content/20"
                      >
                        ...
                      </button>
                    </div>
                  </div>

                  <div class="form-control">
                    <label class="label text-xs font-bold opacity-70">执行器 (Provider)</label>
                    <select
                      v-model="formState.yoloParams.executionProvider"
                      class="select select-bordered select-sm text-xs"
                    >
                      <option v-for="p in executionProviders" :key="p" :value="p">
                        {{ p }}
                      </option>
                    </select>
                  </div>
                  <div class="form-control">
                    <label class="label text-xs font-bold opacity-70">输入图像宽高 (WxH)</label>
                    <div class="flex items-center gap-2">
                      <input
                        type="number"
                        v-model="formState.yoloParams.inputWidth"
                        class="input input-sm input-bordered w-full text-xs"
                      />
                      <span class="opacity-50">x</span>
                      <input
                        type="number"
                        v-model="formState.yoloParams.inputHeight"
                        class="input input-sm input-bordered w-full text-xs"
                      />
                    </div>
                  </div>
                  <div class="form-control">
                    <label class="label text-xs font-bold opacity-70">类别数量:</label>
                    <input
                      type="text"
                      disabled
                      v-model="formState.yoloParams.classCount"
                      class="input input-sm input-bordered text-xs"
                    />
                  </div>
                  <div class="form-control">
                    <label class="label text-xs font-bold opacity-70">
                      <span>置信度: {{ formState.yoloParams.confidenceThresh }}</span>
                    </label>
                    <input
                      type="range"
                      min="0"
                      max="1"
                      step="0.01"
                      v-model.number="formState.yoloParams.confidenceThresh"
                      class="range range-xs range-secondary"
                    />
                  </div>
                  <div class="form-control">
                    <label class="label text-xs font-bold opacity-70">
                      <span>IOU 阈值: {{ formState.yoloParams.iouThresh }}</span>
                    </label>
                    <input
                      type="range"
                      min="0"
                      max="1"
                      step="0.01"
                      v-model.number="formState.yoloParams.iouThresh"
                      class="range range-xs range-primary"
                    />
                  </div>
                </div>
              </div>
            </section>

            <!-- Txt Det Section -->
            <section class="space-y-4">
              <div class="flex items-center justify-between border-b border-base-content/10 pb-2">
                <h4 class="font-bold flex items-center gap-2">
                  <FileJson class="w-4 h-4 text-secondary" /> 文本检测模型
                </h4>
                <div class="flex gap-2">
                  <select v-model="formState.txtDetType" class="select select-bordered select-sm w-32">
                    <option :value="ModelAlgorithm.None">不设置</option>
                    <option :value="ModelAlgorithm.PaddleDbNet">Paddle DBNet</option>
                    <option :value="ModelAlgorithm.Yolo11">YOLO11</option>
                  </select>
                  <select
                    v-if="formState.txtDetType !== ModelAlgorithm.None"
                    v-model="formState.txtDetSource"
                    class="select select-bordered select-sm w-24"
                  >
                    <option :value="ModelSource.BuiltIn">内置</option>
                    <option :value="ModelSource.Custom">自定义</option>
                  </select>
                </div>
              </div>

              <!-- DBNet Params -->
              <div
                v-if="formState.txtDetType === ModelAlgorithm.PaddleDbNet"
                class="bg-base-200/50 p-4 rounded-xl space-y-4 border border-base-content/5"
              >
                <div class="grid grid-cols-2 gap-4">
                  <div class="form-control col-span-2">
                    <label class="label text-xs font-bold opacity-70">模型路径 (ONNX)</label>
                    <div v-if="formState.txtDetSource === ModelSource.Custom" class="flex gap-2">
                      <input
                        type="text"
                        v-model="formState.dbNetParams.modelPath"
                        class="input input-sm input-bordered grow text-xs"
                        placeholder="/path/to/dbnet.onnx"
                      />
                      <button
                        @click="
                          async () => {
                            const p = await handleSelectFile('model');
                            if (p) formState.dbNetParams.modelPath = p;
                          }
                        "
                        class="btn btn-sm btn-square btn-ghost border border-base-content/20"
                      >
                        ...
                      </button>
                    </div>
                    <div v-else class="text-xs opacity-50 py-1.5 italic border border-transparent px-1">
                      使用内置默认模型路径
                    </div>
                  </div>
                  <div class="form-control">
                    <label class="label text-xs font-bold opacity-70">执行器 (Provider)</label>
                    <select
                      v-model="formState.dbNetParams.executionProvider"
                      class="select select-bordered select-sm text-xs"
                    >
                      <option v-for="p in executionProviders" :key="p" :value="p">
                        {{ p }}
                      </option>
                    </select>
                  </div>
                  <div class="form-control">
                    <label class="label text-xs font-bold opacity-70">输入图像宽高 (WxH)</label>
                    <div class="flex items-center gap-2">
                      <input
                        type="number"
                        v-model="formState.dbNetParams.inputWidth"
                        class="input input-sm input-bordered w-full text-xs"
                      />
                      <span class="opacity-50">x</span>
                      <input
                        type="number"
                        v-model="formState.dbNetParams.inputHeight"
                        class="input input-sm input-bordered w-full text-xs"
                      />
                    </div>
                  </div>
                  <div class="form-control">
                    <label class="label text-xs font-bold opacity-70">
                      <span>二值化阈值 (Threshold): {{ formState.dbNetParams.dbThresh }}</span>
                    </label>
                    <input
                      type="range"
                      min="0"
                      max="1"
                      step="0.01"
                      v-model.number="formState.dbNetParams.dbThresh"
                      class="range range-xs range-secondary"
                    />
                  </div>
                  <div class="form-control">
                    <label class="label text-xs font-bold opacity-70">
                      <span>框扩充比例 (Unclip): {{ formState.dbNetParams.unclipRatio }}</span>
                    </label>
                    <input
                      type="range"
                      min="1.0"
                      max="4.0"
                      step="0.1"
                      v-model.number="formState.dbNetParams.unclipRatio"
                      class="range range-xs range-secondary"
                    />
                  </div>
                  <div class="form-control flex flex-row items-center gap-2 pt-8">
                    <input type="checkbox" v-model="formState.dbNetParams.useDilation" class="checkbox checkbox-sm" />
                    <span class="label text-xs font-bold opacity-70">使用膨胀 (Dilation)</span>
                  </div>
                </div>
              </div>

              <!-- YOLO Params for Text Detection -->
              <div
                v-if="formState.txtDetType === ModelAlgorithm.Yolo11"
                class="bg-base-200/50 p-4 rounded-xl space-y-4 border border-base-content/5"
              >
                <div class="grid grid-cols-2 gap-4">
                  <div class="form-control col-span-1">
                    <label class="label text-xs font-bold opacity-70">模型路径 (ONNX)</label>
                    <div v-if="formState.txtDetSource === ModelSource.Custom" class="flex gap-2">
                      <input
                        type="text"
                        v-model="formState.txtDetYoloParams.modelPath"
                        class="input input-sm input-bordered grow text-xs"
                        placeholder="/path/to/text_det_yolo.onnx"
                      />
                      <button
                        @click="
                          async () => {
                            const p = await handleSelectFile('model');
                            if (p) formState.txtDetYoloParams.modelPath = p;
                          }
                        "
                        class="btn btn-sm btn-square btn-ghost border border-base-content/20"
                      >
                        ...
                      </button>
                    </div>
                    <div v-else class="text-xs opacity-50 py-1.5 italic border border-transparent px-1">
                      使用内置默认模型路径
                    </div>
                  </div>
                  <div class="form-control col-span-1">
                    <label class="label text-xs font-bold opacity-70">标签路径 (Label Path)</label>
                    <div class="flex gap-2">
                      <input
                        type="text"
                        v-model="formState.txtDetYoloParams.labelPath"
                        class="input input-sm input-bordered grow text-xs"
                        placeholder="/path/to/config.yaml"
                      />
                      <button
                        @click="
                          async () => {
                            const p = await handleSelectFile('config');
                            if (p) formState.txtDetYoloParams.labelPath = p;
                          }
                        "
                        class="btn btn-sm btn-square btn-ghost border border-base-content/20"
                      >
                        ...
                      </button>
                    </div>
                  </div>
                  <div class="form-control">
                    <label class="label text-xs font-bold opacity-70">执行器 (Provider)</label>
                    <select
                      v-model="formState.txtDetYoloParams.executionProvider"
                      class="select select-bordered select-sm text-xs"
                    >
                      <option v-for="p in executionProviders" :key="p" :value="p">
                        {{ p }}
                      </option>
                    </select>
                  </div>
                  <div class="form-control">
                    <label class="label text-xs font-bold opacity-70">输入图像宽高 (WxH)</label>
                    <div class="flex items-center gap-2">
                      <input
                        type="number"
                        v-model="formState.txtDetYoloParams.inputWidth"
                        class="input input-sm input-bordered w-full text-xs"
                      />
                      <span class="opacity-50">x</span>
                      <input
                        type="number"
                        v-model="formState.txtDetYoloParams.inputHeight"
                        class="input input-sm input-bordered w-full text-xs"
                      />
                    </div>
                  </div>
                  <div class="form-control">
                    <label class="label text-xs font-bold opacity-70">类别数量 (Class Count)</label>
                    <input
                      type="text"
                      disabled
                      v-model="formState.txtDetYoloParams.classCount"
                      class="input input-sm input-bordered text-xs"
                    />
                  </div>

                  <!-- Text Index Specific Parameter -->
                  <div class="form-control">
                    <label class="label text-xs font-bold opacity-70">文本类别索引 (Text Class Index)</label>
                    <input
                      type="number"
                      v-model="formState.txtDetYoloParams.txtIdx"
                      class="input input-sm input-bordered text-xs"
                      placeholder="0"
                    />
                    <label class="label text-[10px] opacity-50">指定哪一个类别ID代表"文本"</label>
                  </div>

                  <div class="form-control">
                    <label class="label text-xs font-bold opacity-70">
                      <span>置信度: {{ formState.txtDetYoloParams.confidenceThresh }}</span>
                    </label>
                    <input
                      type="range"
                      min="0"
                      max="1"
                      step="0.01"
                      v-model.number="formState.txtDetYoloParams.confidenceThresh"
                      class="range range-xs range-secondary"
                    />
                  </div>
                  <div class="form-control">
                    <label class="label text-xs font-bold opacity-70">
                      <span>IOU 阈值: {{ formState.txtDetYoloParams.iouThresh }}</span>
                    </label>
                    <input
                      type="range"
                      min="0"
                      max="1"
                      step="0.01"
                      v-model.number="formState.txtDetYoloParams.iouThresh"
                      class="range range-xs range-secondary"
                    />
                  </div>
                </div>
              </div>
            </section>
          </div>

          <!-- Rec Tab -->
          <div v-show="activeTab === 'rec'" class="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300">
            <div class="flex items-center justify-between border-b border-base-content/10 pb-2">
              <h4 class="font-bold flex items-center gap-2"><Type class="w-4 h-4 text-accent" /> 文本识别模型</h4>
              <div class="flex gap-2">
                <select v-model="formState.txtRecType" class="select select-bordered select-sm w-32">
                  <option :value="ModelAlgorithm.None">不设置</option>
                  <option :value="ModelAlgorithm.PaddleCrnn">Paddle CRNN</option>
                </select>
                <select
                  v-if="formState.txtRecType !== ModelAlgorithm.None"
                  v-model="formState.txtRecSource"
                  class="select select-bordered select-sm w-24"
                >
                  <option :value="ModelSource.BuiltIn">内置</option>
                  <option :value="ModelSource.Custom">自定义</option>
                </select>
              </div>
            </div>

            <div
              v-if="formState.txtRecType === ModelAlgorithm.PaddleCrnn"
              class="bg-base-200/50 p-4 rounded-xl space-y-4 border border-base-content/5"
            >
              <div class="grid grid-cols-2 gap-4">
                <div class="form-control col-span-2">
                  <label class="label text-xs font-bold opacity-70">模型路径 (ONNX)</label>
                  <div v-if="formState.txtRecSource === ModelSource.Custom" class="flex gap-2">
                    <input
                      type="text"
                      v-model="formState.crnnParams.modelPath"
                      class="input input-sm input-bordered grow text-xs"
                      placeholder="/path/to/rec_model.onnx"
                    />
                    <button
                      @click="
                        async () => {
                          const p = await handleSelectFile('model');
                          if (p) formState.crnnParams.modelPath = p;
                        }
                      "
                      class="btn btn-sm btn-square btn-ghost border border-base-content/20"
                    >
                      ...
                    </button>
                  </div>
                  <div v-else class="text-xs opacity-50 py-1.5 italic border border-transparent px-1">
                    使用内置默认模型路径
                  </div>
                </div>
                <div class="form-control col-span-2">
                  <label class="label text-xs font-bold opacity-70">字典文件 (Keys/Dict)</label>
                  <div class="flex gap-2">
                    <input
                      type="text"
                      v-model="formState.crnnParams.dictPath"
                      class="input input-sm input-bordered grow text-xs"
                      placeholder="/path/to/keys.txt"
                    />
                    <button
                      @click="
                        async () => {
                          const p = await handleSelectFile('config');
                          if (p) formState.crnnParams.dictPath = p;
                        }
                      "
                      class="btn btn-sm btn-square btn-ghost border border-base-content/20"
                    >
                      ...
                    </button>
                  </div>
                </div>
                <div class="form-control">
                  <label class="label text-xs font-bold opacity-70">执行器 (Provider)</label>
                  <select
                    v-model="formState.crnnParams.executionProvider"
                    class="select select-bordered select-sm text-xs"
                  >
                    <option v-for="p in executionProviders" :key="p" :value="p">
                      {{ p }}
                    </option>
                  </select>
                </div>
                <div class="form-control">
                  <label class="label text-xs font-bold opacity-70">输入图像宽高 (WxH)</label>
                  <div class="flex items-center gap-2">
                    <input
                      type="number"
                      v-model="formState.crnnParams.inputWidth"
                      class="input input-sm input-bordered w-full text-xs"
                    />
                    <span class="opacity-50">x</span>
                    <input
                      type="number"
                      v-model="formState.crnnParams.inputHeight"
                      class="input input-sm input-bordered w-full text-xs"
                    />
                  </div>
                </div>
              </div>
            </div>

            <div
              v-if="
                formState.imgDetModelType === 'None' &&
                formState.txtDetModelType === 'None' &&
                formState.txtRecModelType === 'None'
              "
              class="alert alert-warning shadow-sm border-none bg-warning/10 text-warning-content py-3 rounded-xl mt-4"
            >
              <AlertTriangle class="w-5 h-5" />
              <div class="text-xs">
                <h3 class="font-bold">未配置任何模型</h3>
                <p>将无法使用截图分析、OCR 文字提取等高级功能，仅可进行基础点击操作。</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="p-4 border-t border-base-content/5 bg-base-200/30 flex justify-end gap-3 flex-none">
        <button @click="$emit('close')" class="btn btn-ghost">取消</button>
        <button @click="handleSave" class="btn btn-primary px-8" :disabled="!formState.name || !formState.pkgName">
          {{ isEditMode ? '保存' : '创建' }}
        </button>
      </div>
    </div>
    <form method="dialog" class="modal-backdrop" @click="$emit('close')">
      <button>close</button>
    </form>
  </dialog>
</template>
