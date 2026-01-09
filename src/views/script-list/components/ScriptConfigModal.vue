<script setup>
import { reactive, watch, ref } from 'vue';
import { X, Plus, Eye, FileJson, Type, AlertTriangle, Cpu as CpuIcon } from 'lucide-vue-next';
import { open } from '@tauri-apps/plugin-dialog';

const props = defineProps({
  isOpen: Boolean,
});

const emit = defineEmits(['close', 'save']);

const activeTab = ref('basic'); // basic, det, rec

const formState = reactive({
  name: '',
  pkgName: '',
  description: '',

  // Image Detection
  imgDetModelType: 'None', // 'None' | 'Yolo11' | 'PaddleDbNet'
  yoloParams: {
    // Base Model Params
    modelPath: '',
    executionProvider: 'cpu', // Cpu, DirectMl, Cuda, TensorRt
    inputWidth: 640,
    inputHeight: 640,

    // Yolo Specific
    classCount: 80,
    confidenceThresh: 0.25,
    iouThresh: 0.45,
    labelPath: '',
  },

  // Text Detection - YOLO params
  txtDetYoloParams: {
    // Base Model Params
    modelPath: '',
    executionProvider: 'cpu',
    inputWidth: 640,
    inputHeight: 640,

    // Yolo Specific
    classCount: 80,
    confidenceThresh: 0.25,
    iouThresh: 0.45,
    labelPath: '',
    txtIdx: 0, // Index of the class representing 'text'
  },

  // Text Detection
  txtDetModelType: 'None', // 'None' | 'PaddleDbNet' | 'Yolo11'
  dbNetParams: {
    // Base Model Params
    modelPath: '',
    executionProvider: 'cpu',
    inputWidth: 640, // Usually 640 or 960
    inputHeight: 640,

    // DBNet Specific
    dbThresh: 0.3,
    dbBoxThresh: 0.6,
    unclipRatio: 1.5,
    useDilation: false,
  },

  // Text Recognition
  txtRecModelType: 'None', // 'None' | 'PaddleCrnn'
  crnnParams: {
    // Base Model Params
    modelPath: '',
    executionProvider: 'cpu',
    inputWidth: 320, // Standard CRNN width, usually resized keeping ratio
    inputHeight: 48, // Standard CRNN height

    // CRNN Specific
    dictPath: '',
  },
});

const executionProviders = ['cpu', 'directml', 'cuda'];

const handleSave = () => {
  if (!formState.name) return;

  // Construct the data structure matching Rust structs
  // Note: ScriptInfo is camelCase, but internal structs are snake_case (standard Rust serde default)

  const scriptData = {
    userId: '019b82ca280377a09eeb95dbdca056cc',
    name: formState.name,
    description: formState.description || null,
    pkgName: formState.pkgName || null,
    scriptType: 'custom',
    verName: 'v1.0.0',
    verNum: 1,
    latestVer: 1,
    downloadCount: 0,
    isValid: true,
    createTime: new Date().toISOString(),
    updateTime: new Date().toISOString(),
    userName: 'Local User',
    tasks: [],
    templates: [],
  };

  // Image Detection Model Construction
  if (formState.imgDetModelType === 'Yolo11') {
    scriptData.imgDetModel = {
      Yolo11: {
        base_model: {
          input_width: parseInt(formState.yoloParams.inputWidth),
          input_height: parseInt(formState.yoloParams.inputHeight),
          model_path: formState.yoloParams.modelPath,
          execution_provider: formState.yoloParams.executionProvider,
          intra_thread_num: 4,
          intra_spinning: true,
          inter_thread_num: 1,
          inter_spinning: true,
          model_type: 'Yolo11',
        },
        class_count: parseInt(formState.yoloParams.classCount),
        class_labels: [],
        confidence_thresh: parseFloat(formState.yoloParams.confidenceThresh),
        iou_thresh: parseFloat(formState.yoloParams.iouThresh),
        label_path: formState.yoloParams.labelPath,
        txt_idx: null,
      },
    };
  }

  // Text Detection Model Construction
  if (formState.txtDetModelType === 'Yolo11') {
    scriptData.txtDetModel = {
      Yolo11: {
        base_model: {
          input_width: parseInt(formState.txtDetYoloParams.inputWidth),
          input_height: parseInt(formState.txtDetYoloParams.inputHeight),
          model_path: formState.txtDetYoloParams.modelPath,
          execution_provider: formState.txtDetYoloParams.executionProvider,
          intra_thread_num: 4,
          intra_spinning: true,
          inter_thread_num: 1,
          inter_spinning: true,
          model_type: 'Yolo11',
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

  if (formState.txtDetModelType === 'PaddleDbNet') {
    scriptData.txtDetModel = {
      PaddleDbNet: {
        base_model: {
          input_width: parseInt(formState.dbNetParams.inputWidth),
          input_height: parseInt(formState.dbNetParams.inputHeight),
          model_path: formState.dbNetParams.modelPath,
          execution_provider: formState.dbNetParams.executionProvider,
          intra_thread_num: 4,
          intra_spinning: true,
          inter_thread_num: 1,
          inter_spinning: true,
          model_type: 'PaddleDet5',
        },
        db_thresh: parseFloat(formState.dbNetParams.dbThresh),
        db_box_thresh: parseFloat(formState.dbNetParams.dbBoxThresh),
        unclip_ratio: parseFloat(formState.dbNetParams.unclipRatio),
        use_dilation: formState.dbNetParams.useDilation,
      },
    };
  }

  // Text Recognition Model Construction
  if (formState.txtRecModelType === 'PaddleCrnn') {
    scriptData.txtRecModel = {
      PaddleCrnn: {
        base_model: {
          input_width: parseInt(formState.crnnParams.inputWidth),
          input_height: parseInt(formState.crnnParams.inputHeight),
          model_path: formState.crnnParams.modelPath,
          execution_provider: formState.crnnParams.executionProvider,
          intra_thread_num: 4,
          intra_spinning: true,
          inter_thread_num: 1,
          inter_spinning: true,
          model_type: 'PaddleCrnn5',
        },
        dict_path: formState.crnnParams.dictPath || null,
        dict: [], // Backend should load this if path is provided, or we pass empty
      },
    };
  }

  emit('save', scriptData);
};

const handleSelectFile = async (target, type = 'model') => {
  try {
    const filters =
      type === 'model'
        ? [{ name: 'ONNX Model', extensions: ['onnx'] }]
        : [{ name: 'Config File', extensions: ['yaml'] }];

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
  formState.imgDetModelType = 'None';
  formState.txtDetModelType = 'None';
  formState.txtRecModelType = 'None';
  activeTab.value = 'basic';
};

watch(
  () => props.isOpen,
  (newVal) => {
    if (!newVal) {
      // Optional: setup a slight delay or just keep state if user cancels?
      // Usually better to clear on re-open or explicit success, but here specific reset might be needed
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
          <Plus class="w-5 h-5 text-primary" />
          新建
        </h3>
        <button @click="$emit('close')" class="btn btn-ghost btn-sm btn-square">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="flex grow min-h-0">
        <!-- Sidebar Tabs -->
        <div class="w-48 flex-none bg-base-200/30 border-r border-base-content/5 p-2 space-y-1">
          <button
            class="btn btn-sm w-full justify-start gap-2"
            :class="activeTab === 'basic' ? 'btn-primary' : 'btn-ghost'"
            @click="activeTab = 'basic'"
          >
            <div class="w-4 h-4 rounded-full border border-current flex items-center justify-center text-[10px]">1</div>
            基础信息
          </button>
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
                <select v-model="formState.imgDetModelType" class="select select-bordered select-sm w-48">
                  <option value="None">不设置</option>
                  <option value="Yolo11">YOLO11</option>
                </select>
              </div>

              <div
                v-if="formState.imgDetModelType === 'Yolo11'"
                class="bg-base-200/50 p-4 rounded-xl space-y-4 border border-base-content/5"
              >
                <div class="grid grid-cols-2 gap-4">
                  <!-- Base Model Param Common -->
                  <div class="form-control col-span-1">
                    <label class="label text-xs font-bold opacity-70">模型路径 (ONNX)</label>
                    <div class="flex gap-2">
                      <input
                        type="text"
                        v-model="formState.yoloParams.modelPath"
                        class="input input-sm input-bordered grow text-xs"
                        placeholder="/path/to/model.onnx"
                      />
                      <button
                        @click="
                          async () => {
                            const p = await handleSelectFile(null, 'model');
                            if (p) formState.yoloParams.modelPath = p;
                          }
                        "
                        class="btn btn-sm btn-square btn-ghost border border-base-content/20"
                      >
                        ...
                      </button>
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
                            const p = await handleSelectFile(null, 'config');
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
                <select v-model="formState.txtDetModelType" class="select select-bordered select-sm w-48">
                  <option value="None">不设置</option>
                  <option value="DbNet">内置Paddle DBNet V5</option>
                  <option value="PaddleDbNet">自定义Paddle DBNet(v3-v5)</option>
                  <option value="Yolo11">YOLO11</option>
                </select>
              </div>

              <!-- DBNet Params -->
              <div
                v-if="formState.txtDetModelType === 'PaddleDbNet'"
                class="bg-base-200/50 p-4 rounded-xl space-y-4 border border-base-content/5"
              >
                <div class="grid grid-cols-2 gap-4">
                  <div class="form-control col-span-2">
                    <label class="label text-xs font-bold opacity-70">模型路径 (ONNX)</label>
                    <div class="flex gap-2">
                      <input
                        type="text"
                        v-model="formState.dbNetParams.modelPath"
                        class="input input-sm input-bordered grow text-xs"
                        placeholder="/path/to/dbnet.onnx"
                      />
                      <button
                        @click="
                          async () => {
                            const p = await handleSelectFile(null, 'model');
                            if (p) formState.dbNetParams.modelPath = p;
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
                v-if="formState.txtDetModelType === 'Yolo11'"
                class="bg-base-200/50 p-4 rounded-xl space-y-4 border border-base-content/5"
              >
                <div class="grid grid-cols-2 gap-4">
                  <div class="form-control col-span-1">
                    <label class="label text-xs font-bold opacity-70">模型路径 (ONNX)</label>
                    <div class="flex gap-2">
                      <input
                        type="text"
                        v-model="formState.txtDetYoloParams.modelPath"
                        class="input input-sm input-bordered grow text-xs"
                        placeholder="/path/to/text_det_yolo.onnx"
                      />
                      <button
                        @click="
                          async () => {
                            const p = await handleSelectFile(null, 'model');
                            if (p) formState.txtDetYoloParams.modelPath = p;
                          }
                        "
                        class="btn btn-sm btn-square btn-ghost border border-base-content/20"
                      >
                        ...
                      </button>
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
                            const p = await handleSelectFile(null, 'config');
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
              <select v-model="formState.txtRecModelType" class="select select-bordered select-sm w-48">
                <option value="None">不设置</option>
                <option value="ResourceCrnn">内置Paddle CRNN v5</option>
                <option value="PaddleCrnn">自定义Paddle CRNN (v3-v5)</option>
              </select>
            </div>

            <div
              v-if="formState.txtRecModelType === 'PaddleCrnn'"
              class="bg-base-200/50 p-4 rounded-xl space-y-4 border border-base-content/5"
            >
              <div class="grid grid-cols-2 gap-4">
                <div class="form-control col-span-2">
                  <label class="label text-xs font-bold opacity-70">模型路径 (ONNX)</label>
                  <div class="flex gap-2">
                    <input
                      type="text"
                      v-model="formState.crnnParams.modelPath"
                      class="input input-sm input-bordered grow text-xs"
                      placeholder="/path/to/rec_model.onnx"
                    />
                    <button
                      @click="
                        async () => {
                          const p = await handleSelectFile(null, 'model');
                          if (p) formState.crnnParams.modelPath = p;
                        }
                      "
                      class="btn btn-sm btn-square btn-ghost border border-base-content/20"
                    >
                      ...
                    </button>
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
                          const p = await handleSelectFile(null, 'config');
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
          创建
        </button>
      </div>
    </div>
    <form method="dialog" class="modal-backdrop" @click="$emit('close')">
      <button>close</button>
    </form>
  </dialog>
</template>
