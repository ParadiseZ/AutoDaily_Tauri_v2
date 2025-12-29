<template>
  <div class="w-80 border-l border-base-300 flex flex-col bg-base-100 shadow-md z-1 h-full">
    <div class="p-3 font-bold text-sm bg-base-200 flex justify-between items-center">
      属性
      <div class="badge badge-sm" v-if="selectedNode">{{ nodeTypeDisplay }}</div>
      <div class="badge badge-sm badge-ghost" v-else>未选择</div>
    </div>
    
    <!-- Node Selected -->
    <div class="flex-1 p-4 overflow-y-auto" v-if="selectedNode">
      <!-- Common: Label/Remark -->
      <div class="form-control w-full">
        <label class="label"><span class="label-text font-bold">Label (Remark)</span></label>
        <input 
          type="text" 
          v-model="nodeLabel" 
          class="input input-bordered w-full input-sm" 
          placeholder="Enter a description..."
          @input="updateLabel"
        />
        <label class="label"><span class="label-text-alt opacity-60">Displayed on the node</span></label>
      </div>

      <div class="divider text-xs opacity-50">Configuration</div>
      
      <!-- Type: Click -->
      <div v-if="selectedNode.data?.type === 'click'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Target Type</span></label>
          <select class="select select-bordered select-sm w-full" v-model="localData.targetType">
            <option value="coordinates">Coordinates (x, y)</option>
            <option value="image">Image Match</option>
            <option value="text">OCR Text</option>
          </select>
        </div>
        
        <div class="form-control w-full" v-if="localData.targetType === 'coordinates'">
          <label class="label"><span class="label-text">Coordinates</span></label>
          <div class="join w-full">
            <input type="number" v-model="localData.x" placeholder="X" class="input input-bordered input-sm join-item w-1/2" />
            <input type="number" v-model="localData.y" placeholder="Y" class="input input-bordered input-sm join-item w-1/2" />
          </div>
        </div>
        
        <div class="form-control w-full" v-else>
          <label class="label"><span class="label-text">Target</span></label>
          <input type="text" v-model="localData.target" placeholder="Image path or text..." class="input input-bordered input-sm w-full" />
        </div>
      </div>
      
      <!-- Type: Wait -->
      <div v-if="selectedNode.data?.type === 'wait'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Duration (ms)</span></label>
          <input type="number" v-model="localData.duration" class="input input-bordered input-sm w-full" min="100" step="100" />
        </div>
        <div class="form-control w-full">
          <label class="label cursor-pointer justify-start gap-2">
            <input type="checkbox" v-model="localData.randomize" class="checkbox checkbox-sm" />
            <span class="label-text">Randomize (±20%)</span>
          </label>
        </div>
      </div>
      
      <!-- Type: Swipe -->
      <div v-if="selectedNode.data?.type === 'swipe'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Start Point (x, y)</span></label>
          <div class="join w-full">
            <input type="number" v-model="localData.startX" placeholder="X" class="input input-bordered input-sm join-item w-1/2" />
            <input type="number" v-model="localData.startY" placeholder="Y" class="input input-bordered input-sm join-item w-1/2" />
          </div>
        </div>
        <div class="form-control w-full">
          <label class="label"><span class="label-text">End Point (x, y)</span></label>
          <div class="join w-full">
            <input type="number" v-model="localData.endX" placeholder="X" class="input input-bordered input-sm join-item w-1/2" />
            <input type="number" v-model="localData.endY" placeholder="Y" class="input input-bordered input-sm join-item w-1/2" />
          </div>
        </div>
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Duration (ms)</span></label>
          <input type="number" v-model="localData.duration" class="input input-bordered input-sm w-full" min="100" />
        </div>
      </div>
      
      <!-- Type: IF Found / IF Not Found -->
      <div v-if="['if_found', 'if_not_found'].includes(selectedNode.data?.type)" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Search Type</span></label>
          <select class="select select-bordered select-sm w-full" v-model="localData.searchType">
            <option value="image">Image</option>
            <option value="text">Text (OCR)</option>
          </select>
        </div>
        
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Target</span></label>
          <input type="text" v-model="localData.target" :placeholder="localData.searchType === 'image' ? 'Image path...' : 'Text to find...'" class="input input-bordered input-sm w-full" />
        </div>
        
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Confidence (%)</span></label>
          <input type="range" v-model="localData.confidence" min="50" max="100" class="range range-sm range-primary" />
          <div class="text-right text-xs opacity-60">{{ localData.confidence || 80 }}%</div>
        </div>
        
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Timeout (ms)</span></label>
          <input type="number" v-model="localData.timeout" class="input input-bordered input-sm w-full" min="1000" step="1000" />
        </div>
      </div>
      
      <!-- Type: Find Image -->
      <div v-if="selectedNode.data?.type === 'detect'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Template Image</span></label>
          <div class="join w-full">
            <input type="text" v-model="localData.imagePath" placeholder="Select image..." class="input input-bordered input-sm join-item flex-1" readonly />
            <button class="btn btn-sm join-item btn-primary">Browse</button>
          </div>
        </div>
        
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Confidence Threshold (%)</span></label>
          <input type="range" v-model="localData.confidence" min="50" max="100" class="range range-sm range-primary" />
          <div class="text-right text-xs opacity-60">{{ localData.confidence || 80 }}%</div>
        </div>
        
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Store Result In</span></label>
          <input type="text" v-model="localData.resultVar" placeholder="Variable name..." class="input input-bordered input-sm w-full font-mono" />
        </div>
      </div>
      
      <!-- Type: OCR -->
      <div v-if="selectedNode.data?.type === 'ocr'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Region (optional)</span></label>
          <div class="grid grid-cols-2 gap-2">
            <input type="number" v-model="localData.regionX" placeholder="X" class="input input-bordered input-sm" />
            <input type="number" v-model="localData.regionY" placeholder="Y" class="input input-bordered input-sm" />
            <input type="number" v-model="localData.regionW" placeholder="Width" class="input input-bordered input-sm" />
            <input type="number" v-model="localData.regionH" placeholder="Height" class="input input-bordered input-sm" />
          </div>
        </div>
        
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Store Result In</span></label>
          <input type="text" v-model="localData.resultVar" placeholder="Variable name..." class="input input-bordered input-sm w-full font-mono" />
        </div>
      </div>
      
      <!-- Type: Loop -->
      <div v-if="selectedNode.data?.type === 'loop'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Loop Count</span></label>
          <input type="number" v-model="localData.count" class="input input-bordered input-sm w-full" min="1" />
        </div>
        
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Loop Type</span></label>
          <select class="select select-bordered select-sm w-full" v-model="localData.loopType">
            <option value="count">Fixed Count</option>
            <option value="until_found">Until Found</option>
            <option value="until_not_found">Until Not Found</option>
            <option value="infinite">Infinite (with break condition)</option>
          </select>
        </div>
        
        <div class="form-control w-full" v-if="['until_found', 'until_not_found'].includes(localData.loopType)">
          <label class="label"><span class="label-text">Break Condition</span></label>
          <input type="text" v-model="localData.breakCondition" placeholder="Image or text to find..." class="input input-bordered input-sm w-full" />
        </div>
      </div>
      
      <!-- Type: Fallback -->
      <div v-if="selectedNode.data?.type === 'fallback'" class="space-y-3">
        <div class="alert alert-info text-xs">
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current shrink-0 w-4 h-4"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
          <span>Fallback executes when all previous conditions fail.</span>
        </div>
        
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Max Retries</span></label>
          <input type="number" v-model="localData.maxRetries" class="input input-bordered input-sm w-full" min="1" max="10" />
        </div>
        
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Fallback Actions</span></label>
          <div class="space-y-2">
            <div v-for="(strategy, idx) in (localData.strategies || defaultStrategies)" :key="idx" 
                 class="flex items-center gap-2 p-2 bg-base-200 rounded">
              <span class="badge badge-sm badge-neutral">{{ idx + 1 }}</span>
              <input type="text" v-model="strategy.target" class="input input-bordered input-xs flex-1" placeholder="Target..." />
              <select v-model="strategy.action" class="select select-bordered select-xs w-20">
                <option value="click">Click</option>
                <option value="back">Back</option>
              </select>
              <button class="btn btn-xs btn-ghost btn-circle text-error" @click="removeStrategy(idx)">×</button>
            </div>
          </div>
          <button class="btn btn-xs btn-ghost mt-2" @click="addStrategy">+ Add Action</button>
        </div>
      </div>
      
      <!-- Type: Screenshot -->
      <div v-if="selectedNode.data?.type === 'screenshot'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Output Variable</span></label>
          <input type="text" v-model="localData.outputVar" placeholder="Variable name..." class="input input-bordered input-sm w-full font-mono" />
        </div>
      </div>

      <!-- Type: Variable -->
      <div v-if="selectedNode.data?.type === 'variable'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Variable Name</span></label>
          <input type="text" v-model="localData.varName" placeholder="e.g. price_str" class="input input-bordered input-sm w-full font-mono" />
        </div>
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Operation</span></label>
          <select class="select select-bordered select-sm w-full" v-model="localData.opType">
            <option value="set">Set Literal</option>
            <option value="math">Math Expression</option>
            <option value="string">String Slice/Split</option>
            <option value="regex">Regex Extract</option>
          </select>
        </div>
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Expression / Value</span></label>
          <textarea v-model="localData.expression" class="textarea textarea-bordered text-xs font-mono h-20" placeholder="e.g. input.split('/')[0]"></textarea>
        </div>
      </div>

      <!-- Type: Filter -->
      <div v-if="selectedNode.data?.type === 'filter'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Source Array</span></label>
          <input type="text" v-model="localData.sourceVar" placeholder="Variable name..." class="input input-bordered input-sm w-full font-mono" />
        </div>
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Target Variable</span></label>
          <input type="text" v-model="localData.targetVar" placeholder="Default same as source" class="input input-bordered input-sm w-full font-mono" />
        </div>
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Mode</span></label>
          <div class="join w-full">
            <button class="btn btn-xs join-item flex-1" :class="localData.mode === 'filter' ? 'btn-primary' : 'btn-ghost'" @click="localData.mode = 'filter'">Filter</button>
            <button class="btn btn-xs join-item flex-1" :class="localData.mode === 'map' ? 'btn-primary' : 'btn-ghost'" @click="localData.mode = 'map'">Map</button>
          </div>
        </div>
        <div class="form-control w-full">
          <label class="label"><span class="label-text">{{ localData.mode === 'filter' ? 'Condition (item)' : 'Logic (item)' }}</span></label>
          <textarea v-model="localData.logic" class="textarea textarea-bordered text-xs font-mono h-20" :placeholder="localData.mode === 'filter' ? 'item.score > 80' : 'item.name'"></textarea>
        </div>
      </div>

      <!-- Type: Macro Action (Unified) -->
      <div v-if="selectedNode.data?.type === 'macro_action'" class="space-y-3">
        <div class="alert alert-warning text-[10px] leading-tight p-2">
            Multi-step unified node: Screenshot -> Detect -> Click.
        </div>
        <div class="form-control w-full">
          <label class="label cursor-pointer justify-start gap-2">
            <input type="checkbox" v-model="localData.screenshot" class="checkbox checkbox-xs" />
            <span class="label-text text-xs">Auto-Screenshot</span>
          </label>
        </div>
        <div class="form-control w-full">
          <label class="label pb-1"><span class="label-text text-xs">Detection Target</span></label>
          <input type="text" v-model="localData.detectTarget" placeholder="Image or text..." class="input input-bordered input-xs w-full" />
        </div>
        <div class="form-control w-full">
          <label class="label pb-0"><span class="label-text text-xs">Confidence: {{ localData.confidence }}%</span></label>
          <input type="range" v-model="localData.confidence" min="50" max="100" class="range range-xs range-primary" />
        </div>
        <div class="form-control w-full pt-1">
          <label class="label pb-1"><span class="label-text text-xs">Click Type</span></label>
          <select class="select select-bordered select-xs w-full" v-model="localData.clickType">
            <option value="coordinates">Relative Offset</option>
            <option value="center">Center of Match</option>
          </select>
        </div>
      </div>

      <!-- Type: SubFlow -->
      <div v-if="selectedNode.data?.type === 'subflow'" class="space-y-3">
        <div class="form-control w-full">
          <label class="label"><span class="label-text">Target Task</span></label>
          <select class="select select-bordered select-sm w-full" v-model="localData.targetTaskId">
            <option :value="null">Select a task...</option>
            <!-- In real implementation, this would list other tasks -->
            <option value="1">Login</option>
            <option value="2">Sign In</option>
            <option value="3">Claim Rewards</option>
          </select>
        </div>
        
        <div class="form-control w-full">
          <label class="label cursor-pointer justify-start gap-2">
            <input type="checkbox" v-model="localData.waitForComplete" class="checkbox checkbox-sm" />
            <span class="label-text">Wait for completion</span>
          </label>
        </div>
      </div>

      <!-- Common: Advanced Options -->
      <div class="collapse collapse-arrow bg-base-200 mt-4" v-if="!['start', 'input'].includes(selectedNode.data?.type)">
        <input type="checkbox" /> 
        <div class="collapse-title text-sm font-medium">Advanced Options</div>
        <div class="collapse-content space-y-3">
          <div class="form-control w-full">
            <label class="label"><span class="label-text">Delay Before (ms)</span></label>
            <input type="number" v-model="localData.delayBefore" class="input input-bordered input-sm w-full" min="0" />
          </div>
          <div class="form-control w-full">
            <label class="label"><span class="label-text">Delay After (ms)</span></label>
            <input type="number" v-model="localData.delayAfter" class="input input-bordered input-sm w-full" min="0" />
          </div>
          <div class="form-control w-full">
            <label class="label"><span class="label-text">Condition (Rhai Script)</span></label>
            <textarea v-model="localData.condition" class="textarea textarea-bordered text-xs font-mono h-20" placeholder="// Return true to execute this node"></textarea>
          </div>
        </div>
      </div>

      <!-- Delete Button -->
      <div class="mt-8" v-if="!['start', 'input'].includes(selectedNode.data?.type)">
        <button class="btn btn-error btn-sm w-full btn-outline" @click="$emit('delete-node')">
            Delete Node
        </button>
      </div>
    </div>
    
    <!-- No Selection -->
    <div class="flex-1 p-10 flex flex-col items-center justify-center text-base-content/30" v-else>
      <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><line x1="9" y1="9" x2="15" y2="15"/><line x1="15" y1="9" x2="9" y2="15"/></svg>
      <span class="mt-2 text-sm">Select a node to edit</span>
      <span class="mt-1 text-xs opacity-50">Click on any node in the canvas</span>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue';
import {
  DEFAULT_FALLBACK_STRATEGIES,
  NODE_TYPES,
  getNodeDisplay,
  NODE_DATA_DEFAULTS,
} from './config.js';

const props = defineProps({
  selectedNode: {
    type: Object,
    default: null
  }
});

const emit = defineEmits(['delete-node', 'update-node']);

// Local state for editing
const nodeLabel = ref('');
const localData = ref({});

// 使用统一配置
const defaultStrategies = DEFAULT_FALLBACK_STRATEGIES;

// Node type display - 使用配置函数
const nodeTypeDisplay = computed(() => {
  const type = props.selectedNode?.data?.type;
  return getNodeDisplay(type) || type || 'Unknown';
});

// Watch for selected node changes
watch(() => props.selectedNode, (newNode) => {
  if (newNode) {
    nodeLabel.value = newNode.label || '';
    localData.value = { ...newNode.data };
    
    // Initialize defaults from config
    if (!localData.value.targetType) localData.value.targetType = NODE_DATA_DEFAULTS.targetType;
    if (!localData.value.searchType) localData.value.searchType = NODE_DATA_DEFAULTS.searchType;
    if (!localData.value.confidence) localData.value.confidence = NODE_DATA_DEFAULTS.confidence;
    if (!localData.value.duration) localData.value.duration = NODE_DATA_DEFAULTS.duration;
    if (!localData.value.timeout) localData.value.timeout = NODE_DATA_DEFAULTS.timeout;
    if (!localData.value.count) localData.value.count = NODE_DATA_DEFAULTS.count;
    if (!localData.value.loopType) localData.value.loopType = NODE_DATA_DEFAULTS.loopType;
    if (!localData.value.maxRetries) localData.value.maxRetries = NODE_DATA_DEFAULTS.maxRetries;
    if (!localData.value.strategies) localData.value.strategies = defaultStrategies.map(s => ({ ...s }));
    if (localData.value.waitForComplete === undefined) localData.value.waitForComplete = NODE_DATA_DEFAULTS.waitForComplete;
  }
}, { immediate: true, deep: true });

// Watch local data changes and emit updates
watch(localData, (newData) => {
  if (props.selectedNode) {
    emit('update-node', props.selectedNode.id, newData);
  }
}, { deep: true });

// Update label
const updateLabel = () => {
  if (props.selectedNode) {
    emit('update-node', props.selectedNode.id, { ...localData.value, label: nodeLabel.value });
  }
};

// Fallback strategy management
const addStrategy = () => {
  if (!localData.value.strategies) {
    localData.value.strategies = [];
  }
  localData.value.strategies.push({ target: '', action: 'click' });
};

const removeStrategy = (idx) => {
  localData.value.strategies.splice(idx, 1);
};
</script>
