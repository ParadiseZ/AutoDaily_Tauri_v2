<template>
  <div class="yolo-test">
    <div class="control-panel">
      <el-card shadow="hover">
        <template #header>
          <div class="card-header">
            <h3>YOLO目标检测测试</h3>
            <el-tag type="primary">{{ modelVersion }}</el-tag>
          </div>
        </template>
        
        <div class="settings-panel">
          <div class="form-item">
            <span class="label">图像缩放默认640，置信度默认0.25，IoU阈值默认0.45</span>
          </div>

          <div class="form-item">
            <span class="label">模型版本:</span>
            <el-select v-model="modelVersion" placeholder="请选择">
              <el-option value="YOLOv11" />
              <el-option value="YOLOv8" />
              <!-- <el-option label="YOLOv5" value="yolov5" /> -->
            </el-select>
          </div>
          
          <!-- <div class="form-item">
            <span class="label">模型大小:</span>
            <el-select v-model="modelSize" placeholder="请选择">
              <el-option label="纳米级 (Nano)" value="n" />
              <el-option label="小型 (Small)" value="s" />
              <el-option label="中型 (Medium)" value="m" />
              <el-option label="大型 (Large)" value="l" />
              <el-option label="超大型 (XLarge)" value="x" />
            </el-select>
          </div> -->
          
          <div class="form-item">
            <span class="label">模型路径:</span>
            <div class="path-input">
              <el-input v-model="modelPath" placeholder="请输入模型路径" />
              <el-button @click="handleBrowseModel">浏览</el-button>
            </div>
          </div>

          <div class="form-item">
            <span class="label">标签配置路径:</span>
            <div class="path-input">
              <el-input v-model="classFilePath" placeholder="请输入标签配置路径" />
              <el-button @click="handleBrowseClass">浏览</el-button>
            </div>
          </div>

          <div class="form-item">
            <span class="label">图像缩放【{{ targetSize }}】px:</span>
            <div class="resize-controls">
              <!-- <el-select v-model="targetSize" placeholder="选择或输入">
                <el-option :value="480" label="480px" />
                <el-option :value="640" label="640px (推荐)" />
                <el-option :value="672" label="672px" />
              </el-select> -->
              <el-input-number 
                v-model="targetSize" 
                :min="224" 
                :max="1280" 
                placeholder="自定义大小"
                :step="32"
              />
            </div>
          </div>
          
          <div class="form-item">
            <span class="label">置信度【{{ confidenceThreshold }}】</span>
            <el-slider
              v-model="confidenceThreshold"
              :min="0.05"
              :max="0.95"
              :step="0.05"
              :format-tooltip="formatConfidence"
              show-tooltip
            />
          </div>
          
          <div class="form-item">
            <span class="label">IoU阈值【{{ iouThreshold }}】</span>
            <el-slider
              v-model="iouThreshold"
              :min="0.05"
              :max="0.95"
              :step="0.05"
              :format-tooltip="formatConfidence"
              show-tooltip
            />
          </div>
          
          <div class="form-item">
            <span class="label">推理方式:</span>
            <el-radio-group v-model="device">
              <el-radio value="cpu">CPU</el-radio>
              <el-radio value="cuda">CUDA (GPU)</el-radio>
            </el-radio-group>
          </div>
        </div>

        <div class="form-item">
          <span class="label">图像路径:</span>
          <div class="path-input">
            <el-input v-model="imagePath" readonly/>
            <el-button type="primary" @click="handleBrowseImage">浏览</el-button>
          </div>
        </div>
        
        
        <div class="button-row">
          <!-- <el-button type="primary" @click="selectImage">选择图片</el-button> -->
          <!-- <el-button type="success" @click="detectObjects" :loading="isDetecting" :disabled="!imageSource"> -->
          <el-button type="success" @click="detectObjects" :loading="isDetecting" :disabled="!imageSource">
            检测目标
          </el-button>
        </div>
      </el-card>
    </div>
    
    <div class="result-panel">
      <el-card shadow="hover" class="result-card">
        <template #header>
          <div class="card-header">
            <h3>{{ showDetectionResult ? '检测结果' : '当前图像' }}</h3>
            <div>
              <el-tag v-if="detectionResults.length && showDetectionResult" type="success">
                检测到 {{ detectionResults.length }} 个目标
              </el-tag>
            </div>
          </div>
        </template>
        
        <div class="detection-container">
          <div class="image-container" v-if="imageSource">
            <div class="result-image">
              <img :src="displayedImage" alt="检测图像" />
              <div
                v-for="(box, index) in detectionResults"
                :key="index"
                class="detection-box"
                :style="getBoxStyle(box)"
                v-if="showDetectionResult"
              >
                <div class="box-label" :style="{ backgroundColor: getClassColor(box.class) }">
                  {{ box.className }} {{ (box.confidence * 100).toFixed(0) }}%
                </div>
              </div>
            </div>
          </div>
          <el-empty v-else description="未选择图片" />
        </div>
        
        <!-- <div class="results-table" v-if="detectionResults.length && showDetectionResult">
          <el-divider>检测详情</el-divider>
          <el-table :data="detectionResults" style="width: 100%" max-height="250">
            <el-table-column prop="className" label="类别" width="120" />
            <el-table-column prop="confidence" label="置信度" width="100">
              <template #default="scope">
                {{ (scope.row.confidence * 100).toFixed(2) }}%
              </template>
            </el-table-column>
            <el-table-column prop="box" label="边界框 [x, y, w, h]">
              <template #default="scope">
                <span>
                  [{{ scope.row.box.map(v => Math.round(v)).join(', ') }}]
                </span>
              </template>
            </el-table-column>
          </el-table>
          
          <div class="button-row export-row">
            <el-button size="small" @click="exportResults">导出结果</el-button>
            <el-button size="small" @click="copyToClipboard">复制到剪贴板</el-button>
          </div>
        </div> -->
      </el-card>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';

import { useImage } from '../../composables/useImage.js';
import { useDevConfig } from '../../composables/useDevConfig.js'

// YoloTest.vue
import { useYoloDetDevStore } from '../../stores/yoloDetDev'
import { storeToRefs } from 'pinia'
import { onMounted, onUnmounted, watch } from 'vue'

const { loadImage } = useImage()
const { browseClassFile,browseModelFile,browseImageFile } = useDevConfig()

const yoloDetDevStore = useYoloDetDevStore()
const {
  modelVersion,
  modelPath,
  classFilePath,
  targetSize,
  confidenceThreshold,
  iouThreshold,
  device,
  imagePath
} = storeToRefs(yoloDetDevStore)
// 组件挂载时加载配置
onMounted(() => {
  yoloDetDevStore.loadConfig()
})

// 监听配置变化并保存
watch(
  [modelVersion, modelPath, classFilePath, targetSize, confidenceThreshold, iouThreshold, device, imagePath],
  () => {
    yoloDetDevStore.saveConfig()
  },
  { deep: true }
)
// 图像和结果
const imageSource = ref(null);
const resultImage = ref(null);
const detectionResults = ref([]);
const isDetecting = ref(false);
const showDetectionResult = ref(false);

// 监听 imagePath 变化，在初始值设置后加载图像
watch(imagePath, (newPath) => {
  if (newPath && newPath.trim() !== '') {
    loadImg(newPath)
  }
}, { immediate: true })

// 组件卸载时保存配置
onUnmounted(() => {
  yoloDetDevStore.saveConfig()
})




// 计算属性：显示的图片
const displayedImage = computed(() => {
  if (showDetectionResult.value && resultImage.value) {
    return resultImage.value;
  }
  return imageSource.value;
});

// 读取图像文件并转换为base64
async function loadImg(imagePath) {
  try{
    showDetectionResult.value = false;
    const selectedImg = await loadImage(imagePath);  // 设置图像源
    if(selectedImg){
      imageSource.value = selectedImg
    }
    //showDetectionResult.value = true;
  }catch (error) {
    if (error.toString().includes('forbidden path') || 
        error.toString().includes('permission denied') ||
        error.toString().includes('access is denied'))
    {
      //imagePath.value = "";
      ElMessage.error({
        message: '无法访问图像文件，请重新选择',
        duration: 2500
      });
    }else {
      ElMessage.error({
        message: `图像加载失败：${error.message}`,
        duration: 2500
      })
    }
  }
}

// 选择图像文件
async function handleBrowseImage() {
  try{
    const selectedPath = await browseImageFile();
    if(selectedPath){
      imagePath.value = selectedPath;
    }
  }catch (error){
    ElMessage.error('选择图像文件失败：' + error);
  }
}

// 处理模型文件浏览
async function handleBrowseModel() {
  try{
    const selectedPath = await browseModelFile();  // 调用并获取返回值
    if (selectedPath) {
      modelPath.value = selectedPath;  // 使用返回值更新状态
    }
  }catch (error){
    ElMessage.error('选择模型文件失败：' + error);
  }

}

// 处理类文件浏览
async function handleBrowseClass() {
  try{
    const selectedPath = await browseClassFile();  // 调用并获取返回值
    if (selectedPath) {
      classFilePath.value = selectedPath;  // 使用返回值更新状态
    }
  }catch (error){
    ElMessage.error('选择标签文件失败：' + error);
  }

}
// 格式化置信度显示
function formatConfidence(val) {
  return (val * 100).toFixed(0) + '%';
}

// 获取边界框样式
function getBoxStyle(box) {
  const color = getClassColor(box.class);
  return {
    left: `${box.box[0]}px`,
    top: `${box.box[1]}px`,
    width: `${box.box[2]}px`,
    height: `${box.box[3]}px`,
    borderColor: color
  };
}

// 获取类别颜色
function getClassColor(classId) {
  const colors = [
    '#FF3B30', '#FF9500', '#FFCC00', '#4CD964',
    '#5AC8FA', '#007AFF', '#5856D6', '#FF2D55'
  ];
  return colors[classId % colors.length];
}

// 检测目标
async function detectObjects() {
  if (!imageSource.value) {
    return ElMessage.warning('请先选择图片');
  }
  
  if (!modelPath.value || !classFilePath.value) {
    return ElMessage.warning('请选择模型文件和标签配置文件');
  }
  
  try {
    isDetecting.value = true;
    
    // 调用后端进行推理
    const result = await invoke('yolo_inference_test', {
      modelPath: modelPath.value,
      classFilePath: classFilePath.value,
      imagePath: imagePath.value,
      targetSize: targetSize.value,
      confidenceThreshold: confidenceThreshold.value,
      iouThreshold: iouThreshold.value,
    });
    
    // 处理结果
    // 前端修改
    const data = JSON.parse(result);
    if (data.status === "ok") {
      const detections = data.detections;
      // 解析检测结果
      try {
        //const detections = JSON.parse(detectionsJson);
        // 转换为前端需要的格式
        detectionResults.value = detections.map(det => ({
          className: det.label,
          confidence: det.score,
          class: det.class_id,
          box: [det.x1, det.y1, det.x2 - det.x1, det.y2 - det.y1]
        }));
      } catch (error) {
        ElMessage.error('解析检测结果失败:' + error);
      }
      //resultImage.value = `data:image/png;base64,${base64Image}`;
      resultImage.value = imageSource.value;
      showDetectionResult.value = true;
      ElMessage.success('检测完成');
    } else if (data.status === "error") {
      ElMessage.error('检测失败：' + data.message);
    } else {
      ElMessage.error('未知响应格式');
    }
  } catch (error) {
    ElMessage.error('解析返回结果失败' + error);
  } finally {
    isDetecting.value = false;
  }
}
</script>

<style lang="scss" scoped>
.yolo-test {
  display: grid;
  grid-template-columns: 1fr 1.6fr;
  gap: 20px;
  height: 100%;
  
  @media (max-width: 1200px) {
    grid-template-columns: 1fr;
  }
}

.control-panel {
  display: flex;
  flex-direction: column;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  
  h3 {
    margin: 0;
    font-size: 16px;
  }
}

.settings-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-bottom: 16px;
}

.form-item {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  
  .label {
    min-width: 80px;
    color: var(--text-color-regular);
  }
  
  .el-select, .el-input, .path-input, .el-slider {
    flex: 1;
  }
  
  .path-input {
    display: flex;
    gap: 8px;
  }

  .resize-controls {
    display: flex;
    gap: 8px;
    flex: 1;
  }
}

.button-row {
  display: flex;
  justify-content: center;
  flex-wrap: wrap;
  gap: 16px;
  margin-top: 16px;
  
  &.export-row {
    justify-content: flex-end;
    margin-top: 16px;
  }
}

.result-panel {
  display: flex;
  flex-direction: column;
  
  .result-card {
    height: 100%;
    display: flex;
    flex-direction: column;
    
    :deep(.el-card__body) {
      flex: 1;
      display: flex;
      flex-direction: column;
      overflow: hidden;
    }
  }
}

.detection-container {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: auto;
  background-color: var(--bg-color-mute);
  border-radius: 4px;
  min-height: 300px;
}

.image-container {
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  overflow: auto;
  
  .result-image {
    position: relative;
    max-width: 100%;
    max-height: 100%;
    
    img {
      max-width: 100%;
      max-height: 100%;
      object-fit: contain;
    }
  }
}

.detection-box {
  position: absolute;
  border: 2px solid;
  box-sizing: border-box;
  
  .box-label {
    position: absolute;
    top: 0;
    left: 0;
    transform: translateY(-100%);
    padding: 2px 6px;
    color: white;
    font-size: 12px;
    font-weight: bold;
    border-radius: 2px 2px 0 0;
  }
}

.results-table {
  margin-top: 16px;
  max-height: 300px;
}
</style> 