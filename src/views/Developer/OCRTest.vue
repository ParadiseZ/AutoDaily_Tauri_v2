<template>
  <div class="ocr-test">
    <div class="control-panel">
      <el-card shadow="hover">
        <template #header>
          <div class="card-header">
            <h3>OCR识别测试</h3>
            <el-tag type="success">PaddleOCR</el-tag>
          </div>
        </template>
        
        <div class="settings-panel">
          <div class="form-item">
            <span class="label">模型类型:</span>
            <el-select v-model="modelType" placeholder="请选择" @change="handleModelTypeChange">
              <el-option label="内置PPOCR" value="built-in" />
              <el-option label="自定义PPOCR模型" value="custom" />
            </el-select>
          </div>
          
          <template v-if="modelType === 'built-in'">
            <div class="form-item">
              <span class="label">模型版本:</span>
              <el-select v-model="modelVersion" placeholder="请选择" @change="handleModelVersionChange">
                <el-option label="v5-中文" value="5" />
                <el-option label="v4-中文" value="4" />
                <el-option label="v3-中文" value="3" />
              </el-select>
            </div>
          </template>

          <div class="section-title">线程设置</div>
          <div class="form-item">
            <span class="label">Intra Thread Num:</span>
            <el-input-number v-model="intraThreadNum" :min="1" />
          </div>
          <div class="form-item">
            <span class="label">Intra Spinning:</span>
            <el-switch v-model="intraSpinning" />
          </div>
          <div class="form-item">
            <span class="label">Inter Thread Num:</span>
            <el-input-number v-model="interThreadNum" :min="1" />
          </div>
          <div class="form-item">
            <span class="label">Inter Spinning:</span>
            <el-switch v-model="interSpinning" />
          </div>

          <div class="section-title">检测设置</div>
          <template v-if="modelType === 'built-in'">
            <div class="form-item">
              <span class="label">检测模型:</span>
              <div class="path-input">
                <el-input v-model="detModelPath" disabled />
              </div>
            </div>
          </template>
          <template v-else>
            <div class="form-item">
              <span class="label">检测模型:</span>
              <div class="path-input">
                <el-input v-model="detModelPath" placeholder="请输入检测模型路径" />
                <el-button @click="handleDetModelFile">浏览</el-button>
              </div>
            </div>
            <div class="form-item">
              <span class="label">置信度（yolo）:</span>
              <el-slider v-model="detConfidenceThresh" :min="0" :max="1" :step="0.01" show-tooltip />
            </div>
            <div class="form-item">
              <span class="label">NMS IOU阈值（yolo）:</span>
              <el-slider v-model="detNmsIouThresh" :min="0" :max="1" :step="0.01" show-tooltip />
            </div>
          </template>
          <div class="form-item">
            <span class="label">执行器</span>
            <el-radio-group v-model="detExecutionProvider" size="small" fill="#6cf">
              <el-radio-button label="cpu" value="cpu" />
              <el-radio-button label="dml" value="dml" />
              <el-radio-button label="cuda" value="cuda" />
            </el-radio-group>
          </div>
          <div class="form-item">
            <span class="label">1[yolo] 2[ppocr] 3[合并]</span>
            <template v-if="modelType === 'built-in'">
              <div class="path-input">
                <el-input v-model="detModelType" disabled/>
              </div>
            </template>
            <template v-else>
              <el-input-number v-model="detModelType" :min="1" :max="3" />
            </template>
          </div>
          <div class="form-item">
            <span class="label">输入大小:</span>
            <el-input-number v-model="detInputSize" :min="480" :step="32" />
          </div>
          <div class="form-item">
            <span class="label">二值化阈值:</span>
            <el-slider v-model="detDbThresh" :min="0" :max="1" :step="0.01" :format-tooltip="formatConfidence" show-tooltip />
          </div>
          <div class="form-item">
            <span class="label">框置信度阈值:</span>
            <el-slider v-model="detDbBoxThresh" :min="0" :max="1" :step="0.01" :format-tooltip="formatConfidence" show-tooltip />
          </div>
          <div class="form-item">
            <span class="label">边框膨胀系数:</span>
            <el-input-number v-model="unclipRatio" :step="0.1" :min="0" />
          </div>
          <div class="form-item">
            <span class="label">二值化膨胀:</span>
            <el-switch v-model="useDilation" />
          </div>

          <div class="section-title">识别设置</div>
          <template v-if="modelType === 'built-in'">
            <div class="form-item">
              <span class="label">识别模型:</span>
              <div class="path-input">
                <el-input v-model="recModelPath" disabled />
              </div>
            </div>
            <div class="form-item">
              <span class="label">字典路径:</span>
              <div class="path-input">
                <el-input v-model="dictPath" disabled />
              </div>
            </div>
          </template>
          <template v-else>
            <div class="form-item">
              <span class="label">识别模型:</span>
              <div class="path-input">
                <el-input v-model="recModelPath" placeholder="请输入识别模型路径" />
                <el-button @click="handleRecModelFile">浏览</el-button>
              </div>
            </div>
            <div class="form-item">
              <span class="label">字典文件:</span>
              <div class="path-input">
                <el-input v-model="dictPath" placeholder="请输入字典文件路径" />
                <el-button @click="handleDictFile">浏览</el-button>
              </div>
            </div>
          </template>
          <div class="form-item">
            <span class="label">执行器</span>
            <el-radio-group v-model="recExecutionProvider" size="small" fill="#6cf">
              <el-radio-button label="cpu" value="cpu" />
              <el-radio-button label="dml" value="dml" />
              <el-radio-button label="cuda" value="cuda" />
            </el-radio-group>
          </div>
          <div class="form-item">
            <span class="label">图像输入高度</span>
            <template v-if="modelType === 'built-in'">
              <el-input-number v-model="recInputSize" :min="32" :max="64" :step="16" />
            </template>
            <template v-else>
              <el-input-number v-model="recInputSize" :min="1" />
            </template>
          </div>
          <div class="form-item">
            <span class="label">置信度:{{formatConfidence(recConfidenceThreshold)}}</span>
            <el-slider v-model="recConfidenceThreshold" :min="0" :max="1" :step="0.01" show-tooltip />
          </div>

          <div class="section-title">其他设置</div>
          
          <div class="form-item">
            <span class="label">使用GPU:</span>
            <el-switch v-model="useGpu" />
          </div>
          
          <div class="form-item" v-if="useGpu">
            <span class="label">GPU设备:</span>
            <el-select v-model="gpuDevice" placeholder="请选择">
              <el-option label="GPU:0" value="0" />
              <el-option label="GPU:1" value="1" />
            </el-select>
          </div>
          <div class="form-item">
            <span class="label">图像路径:</span>
            <div class="path-input">
              <el-input v-model="imagePath" readonly/>
              <el-button type="primary" @click="handleBrowseImage">浏览</el-button>
            </div>
          </div>

          <!-- 新增配置加载/保存按钮 -->
          <div class="button-row config-buttons">
            <el-button @click="loadConfigFromFile">加载配置</el-button>
            <el-button @click="saveConfigToFile">保存配置</el-button>
          </div>
        </div>
        
        <div class="button-row">
          <!-- <el-upload
            action=""
            :auto-upload="false"
            :show-file-list="false"
            accept="image/*"
            :on-change="handleBrowseImage"
          >
            <template #trigger>
              <el-button type="primary">选择图片</el-button>
            </template>
          </el-upload> -->
          <el-button @click="captureAndRecognize">截图并识别</el-button>
          <el-button type="success" @click="startRecognition" :disabled="!imageSource || isRecognizing" :loading="isRecognizing">
            {{ isRecognizing ? '识别中...' : '开始识别' }}
          </el-button>
        </div>
      </el-card>
    </div>
    
    <div class="result-panel">
      <el-card shadow="hover">
        <template #header>
          <div class="card-header">
            <h3>{{ showDetectionResult ? '识别结果' : '原始图像' }}</h3>
            <div class="header-controls">
              <el-switch
                v-if="ocrResults.length > 0"
                v-model="showDetectionResult"
                :active-text="'显示识别结果'"
                :inactive-text="'显示原图'"
              />
              <el-tag v-if="ocrResults.length > 0" type="success">
                检测到 {{ ocrResults.length }} 个文本区域
              </el-tag>
            </div>
          </div>
        </template>
        
        <div class="image-container">
          <div v-if="imageSource" class="result-image">
            <img :src="imageSource" alt="OCR图像" />
            <div 
              v-if="showDetectionResult"
              v-for="(box, index) in ocrResults" 
              :key="index"
              class="text-box"
              :style="getBoxStyle(box)"
              :title="box.text + ' (' + (box.confidence * 100).toFixed(2) + '%)'">
              <span class="text-overlay">{{ box.text }}</span>
            </div>
          </div>
          <el-empty v-else description="未选择图片" />
        </div>
        
        <div class="text-result" v-if="ocrResults.length && showDetectionResult">
          <div class="text-controls">
            <el-button size="small" @click="copyAllText">复制所有</el-button>
            <el-button size="small" @click="exportJson">导出JSON</el-button>
          </div>
          <el-divider content-position="left">提取文本</el-divider>
          <div class="text-list">
            <div 
              v-for="(box, index) in ocrResults" 
              :key="index"
              class="text-item"
              :class="{ 'low-confidence': box.confidence < confidenceThreshold }">
              <span class="text-content">{{ box.text }}</span>
              <span class="text-confidence">{{ (box.confidence * 100).toFixed(2) }}%</span>
            </div>
          </div>
        </div>
      </el-card>
    </div>
  </div>
</template>

<script setup>
import { ref, watch, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { writeFile, BaseDirectory } from '@tauri-apps/plugin-fs';
import { readTextFile } from '@tauri-apps/plugin-fs';
import { ElMessage } from 'element-plus';
import { usePaddleOcrDevStore } from '../../stores/paddle-ocr-dev';
import { storeToRefs } from 'pinia';

import { useImage } from '../../composables/useImage.js';
import { useDevConfig } from '../../composables/useDevConfig.js'

const { loadImage } = useImage()
const { browseModelFile,browseImageFile,browseDictFile } = useDevConfig()

// 从store获取状态
const paddleOcrDevStore = usePaddleOcrDevStore();
const {
  modelType,
  modelVersion,
  dictPath,
  useGpu,
  gpuDevice,
  
  intraThreadNum,
  intraSpinning,
  interThreadNum,
  interSpinning,
  detModelPath,
  recModelPath,
  detInputSize,
  recInputSize,
  detDbThresh,
  detDbBoxThresh,
  detConfidenceThresh,
  detNmsIouThresh,
  unclipRatio,
  useDilation,
  recConfidenceThreshold,
  detModelType,
  detExecutionProvider,
  recExecutionProvider,
  imagePath
} = storeToRefs(paddleOcrDevStore);

// 本地状态 for confidenceThreshold since removed from store
const confidenceThreshold = ref(0.5);

// 组件本地状态
const imageSource = ref(null);
const isRecognizing = ref(false);
const ocrResults = ref([]);
const showDetectionResult = ref(false);

// 组件挂载时加载配置
onMounted(() => {
  paddleOcrDevStore.loadConfig();
  // 如果有保存的图片路径，尝试加载图片
  if (imagePath.value) {
    // 这里需要实现从路径加载图片的逻辑
  }
});

// 监听配置变化并保存
watch(
  [
  modelType,
  modelVersion,
  dictPath,
  useGpu,
  gpuDevice,
  
  intraThreadNum,
  intraSpinning,
  interThreadNum,
  interSpinning,
  detModelPath,
  recModelPath,
  detInputSize,
  recInputSize,
  detDbThresh,
  detDbBoxThresh,
  detConfidenceThresh,
  detNmsIouThresh,
  unclipRatio,
  useDilation,
  recConfidenceThreshold,
  detModelType,
  detExecutionProvider,
  recExecutionProvider,
  imagePath],
  () => {
    paddleOcrDevStore.saveConfig();
  },
  { deep: true }
);

// 监听 imagePath 变化，在初始值设置后加载图像
watch(imagePath, (newPath) => {
  if (newPath && newPath.trim() !== '') {
    loadImg(newPath)
  }
}, { immediate: true })

// 处理模型类型变更
function handleModelTypeChange() {
  if (modelType.value === 'built-in') {
    paddleOcrDevStore.updateDefaultPaths();
  }
}

// 处理模型版本变更
function handleModelVersionChange() {
  paddleOcrDevStore.updateDefaultPaths();
}

async function loadImg(imagePath) {
  try{
    showDetectionResult.value = false;
    ocrResults.value = [];
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

// 浏览检测模型文件
async function handleDetModelFile() {
  try{
    const selectedPath = await browseModelFile();  // 调用并获取返回值
    if (selectedPath) {
      detModelPath.value = selectedPath;  // 使用返回值更新状态
    }
  }catch (error){
    ElMessage.error('选择文字检测模型文件失败：' + error);
  }

}

// 浏览识别模型文件
async function handleRecModelFile() {
  try{
    const selectedPath = await browseModelFile();  // 调用并获取返回值
    if (selectedPath) {
      recModelPath.value = selectedPath;  // 使用返回值更新状态
    }
  }catch (error){
    ElMessage.error('选择文字识别模型文件失败：' + error);
  }

}


// 浏览字典文件
async function handleDictFile() {
  try{
    const selectedPath = await browseDictFile();  // 调用并获取返回值
    if (selectedPath) {
      dictPath.value = selectedPath;  // 使用返回值更新状态
    }
  }catch (error){
    ElMessage.error('选择字典文件失败：' + error);
  }
}

// 格式化置信度显示
function formatConfidence(val) {
  return (val * 100).toFixed(0) + '%';
}

// 截图并识别
async function captureAndRecognize() {
  try {
    isRecognizing.value = true;
    
    // TODO: 实际项目中这里应该从Tauri API调用后端进行截图
    // const captureResult = await invoke('capture_screen_for_ocr');
    // imageSource.value = captureResult.base64Image;
    
    // 模拟截图
    imageSource.value = 'https://via.placeholder.com/800x600?text=OCR测试图像';
    ElMessage.success('截图成功');
    
    // 清除之前的结果
    ocrResults.value = [];
    showDetectionResult.value = false;
    
    // 自动开始识别
    await startRecognition();
  } catch (error) {
    ElMessage.error('截图失败：' + error);
    isRecognizing.value = false;
  }
}

// 加载配置从文件
async function loadConfigFromFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'JSON', extensions: ['json'] }]
    });
    if (selected) {
      const content = await readTextFile(selected, { dir: BaseDirectory.App });
      const config = JSON.parse(content);
      // 设置所有值
      modelType.value = config.modelType || modelType.value;
      modelVersion.value = config.modelVersion || modelVersion.value;
      dictPath.value = config.dictPath || dictPath.value;
      useGpu.value = config.useGpu || useGpu.value;
      gpuDevice.value = config.gpuDevice || gpuDevice.value;
      intraThreadNum.value = config.intraThreadNum || intraThreadNum.value;
      intraSpinning.value = config.intraSpinning || intraSpinning.value;
      interThreadNum.value = config.interThreadNum || interThreadNum.value;
      interSpinning.value = config.interSpinning || interSpinning.value;
      detModelPath.value = config.detModelPath || detModelPath.value;
      recModelPath.value = config.recModelPath || recModelPath.value;
      detInputSize.value = config.detInputSize || detInputSize.value;
      recInputSize.value = config.recInputSize || recInputSize.value;
      detDbThresh.value = config.detDbThresh || detDbThresh.value;
      detDbBoxThresh.value = config.detDbBoxThresh || detDbBoxThresh.value;
      detConfidenceThresh.value = config.detConfidenceThresh || detConfidenceThresh.value;
      detNmsIouThresh.value = config.detNmsIouThresh || detNmsIouThresh.value;
      unclipRatio.value = config.unclipRatio || unclipRatio.value;
      useDilation.value = config.useDilation || useDilation.value;
      recConfidenceThreshold.value = config.recConfidenceThreshold || recConfidenceThreshold.value;
      detModelType.value = config.detModelType || detModelType.value;
      detExecutionProvider.value = config.detExecutionProvider || detExecutionProvider.value;
      recExecutionProvider.value = config.recExecutionProvider || recExecutionProvider.value;
      imagePath.value = config.imagePath || imagePath.value;
      ElMessage.success('配置加载成功');
    }
  } catch (error) {
    ElMessage.error('加载配置失败：' + error);
  }
}

// 保存配置到文件
async function saveConfigToFile() {
  try {
    const filePath = await save({
      filters: [{ name: 'JSON', extensions: ['json'] }]
    });
    if (filePath) {
      const config = {
        modelType: modelType.value,
        modelVersion: modelVersion.value,
        dictPath: dictPath.value,
        useGpu: useGpu.value,
        gpuDevice: gpuDevice.value,
        intraThreadNum: intraThreadNum.value,
        intraSpinning: intraSpinning.value,
        interThreadNum: interThreadNum.value,
        interSpinning: interSpinning.value,
        detModelPath: detModelPath.value,
        recModelPath: recModelPath.value,
        detInputSize: detInputSize.value,
        recInputSize: recInputSize.value,
        detDbThresh: detDbThresh.value,
        detDbBoxThresh: detDbBoxThresh.value,
        detConfidenceThresh : detConfidenceThresh.value,
        detNmsIouThresh: detNmsIouThresh.value,
        unclipRatio: unclipRatio.value,
        useDilation: useDilation.value,
        recConfidenceThreshold: recConfidenceThreshold.value,
        detExecutionProvider: detExecutionProvider.value,
        recExecutionProvider: recExecutionProvider.value,
        detModelType: detModelType.value,
        imagePath: imagePath.value
      };
      await writeFile(filePath, new TextEncoder().encode(JSON.stringify(config, null, 2)), { dir: BaseDirectory.App });
      ElMessage.success('配置保存成功');
    }
  } catch (error) {
    ElMessage.error('保存配置失败：' + error);
  }
}

// 更新 startRecognition 以调用后端
async function startRecognition() {
  if (!imageSource.value) {
    return ElMessage.warning('请先选择图片或进行截图');
  }
  
  if (!detModelPath.value || !recModelPath.value || !dictPath.value) {
    return ElMessage.warning('请设置完整的模型和字典路径');
  }
  
  try {
    isRecognizing.value = true;
    
    // TODO: 使用Tauri fs保存base64到文件
    // await invoke('save_image', { base64: imageSource.value, path: tempImagePath });
    
    const result = await invoke('paddle_ocr_inference_test', {
      modelType: modelType.value,
      intraThreadNum: intraThreadNum.value,
      intraSpinning: intraSpinning.value,
      interThreadNum: interThreadNum.value,
      interSpinning: interSpinning.value,
      detModelPath: detModelPath.value,
      recModelPath: recModelPath.value,
      dictPath: dictPath.value,
      detInputSize: detInputSize.value,
      recInputSize: recInputSize.value, // array for tuple
      detDbThresh: detDbThresh.value,
      detDbBoxThresh: detDbBoxThresh.value,
      detConfidenceThresh: detConfidenceThresh.value,
      detNmsIouThresh: detNmsIouThresh.value,
      unclipRatio: unclipRatio.value,
      useDilation: useDilation.value,
      //recConfidenceThreshold: recConfidenceThreshold.value,
      detModelType: detModelType.value,
      detExecutionProvider: detExecutionProvider.value,
      recExecutionProvider: recExecutionProvider.value,
      imagePath: imagePath.value // 或 imagePath.value 如果是路径
    });
    
    // 假设 result 是 JSON 字符串
    const parsedResult = JSON.parse(result);
    if (parsedResult.status === 'ok') {
      console.log(parsedResult)
      ocrResults.value = parsedResult.ocrResults.map(ocr => ({
        text: ocr.txt,
        confidence: ocr.score,
        box: [ocr.bounding_box.x1, ocr.bounding_box.y1, ocr.bounding_box.x2, ocr.bounding_box.y2] // 调整为前端格式
      }));
      showDetectionResult.value = true;
      ElMessage.success('OCR识别完成，识别到 ' + ocrResults.value.length + ' 个文本区域');
    }else {
      ElMessage.error('OCR识别失败：' + parsedResult.ocrResults);
    }
  } catch (error) {
    ElMessage.error('OCR识别失败：' + error);
  } finally {
    isRecognizing.value = false;
  }
}

// 获取文本框的样式
function getBoxStyle(box) {
  // 文本框的坐标格式：[左上x, 左上y, 右上x, 右上y, 右下x, 右下y, 左下x, 左下y]
  const [x1, y1, x2, y2, x3, y3, x4, y4] = box.box;
  
  // 计算边界框
  const left = Math.min(x1, x4);
  const top = Math.min(y1, y2);
  const width = Math.max(x2, x3) - left;
  const height = Math.max(y3, y4) - top;
  
  // 透明度基于置信度
  const opacity = box.confidence < confidenceThreshold.value ? 0.3 : 0.7;
  
  return {
    left: left + 'px',
    top: top + 'px',
    width: width + 'px',
    height: height + 'px',
    borderColor: box.confidence < confidenceThreshold.value ? '#ff4d4f' : '#52c41a',
    backgroundColor: `rgba(82, 196, 26, ${opacity * 0.2})`
  };
}

// 复制所有文本
function copyAllText() {
  const text = ocrResults.value
    .filter(box => box.confidence >= confidenceThreshold.value)
    .map(box => box.text)
    .join('\n');
  
  // 复制到剪贴板
  navigator.clipboard.writeText(text)
    .then(() => ElMessage.success('文本已复制到剪贴板'))
    .catch(err => ElMessage.error('复制失败：' + err));
}

// 导出JSON
function exportJson() {
  const data = JSON.stringify(ocrResults.value, null, 2);
  const blob = new Blob([data], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  
  const a = document.createElement('a');
  a.href = url;
  a.download = 'ocr_results.json';
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
  
  ElMessage.success('JSON导出成功');
}
</script>

<style lang="scss" scoped>
.ocr-test {
  display: grid;
  grid-template-columns: 1fr 2fr;
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
  
  .header-controls {
    display: flex;
    align-items: center;
    gap: 10px;
  }
}

.settings-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-bottom: 20px;
}

.form-item {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  
  .label {
    min-width: 70px;
    color: var(--text-color-regular);
  }
  
  .el-select, .el-slider {
    flex: 1;
  }
  
  .path-input {
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
}

.config-buttons {
  justify-content: flex-start;
  margin-bottom: 16px;
}

.section-title {
  font-weight: bold;
  font-size: 16px;
  margin-bottom: 8px;
  color: var(--text-color-regular);
}

.result-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.image-container {
  width: 100%;
  min-height: 400px;
  display: flex;
  justify-content: center;
  align-items: center;
  overflow: auto;
  background-color: var(--bg-color-mute);
  border-radius: 4px;
  margin-bottom: 16px;
  
  .result-image {
    position: relative;
    display: inline-block;
    
    img {
      max-width: 100%;
      max-height: 100%;
      object-fit: contain;
    }
    
    .text-box {
      position: absolute;
      border: 2px solid;
      box-sizing: border-box;
      
      .text-overlay {
        position: absolute;
        top: -25px;
        left: 0;
        background-color: var(--bg-color-soft);
        padding: 2px 6px;
        border-radius: 4px;
        font-size: 12px;
        white-space: nowrap;
        box-shadow: var(--box-shadow);
      }
    }
  }
}

.text-result {
  margin-top: 16px;
  
  .text-controls {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-bottom: 8px;
  }
  
  .text-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 200px;
    overflow: auto;
  }
  
  .text-item {
    display: flex;
    justify-content: space-between;
    padding: 8px 12px;
    background-color: var(--bg-color-soft);
    border-radius: 4px;
    
    &.low-confidence {
      opacity: 0.5;
      text-decoration: line-through;
    }
    
    .text-confidence {
      color: var(--text-color-secondary);
      font-size: 12px;
    }
  }
}
</style> 