import { defineStore } from 'pinia'
import { ref } from 'vue'

export const usePaddleOcrDevStore = defineStore('ppocrDev', () => {
  // 定义状态
  const modelType = ref('built-in') // 'built-in' 或 'custom'
  const modelVersion = ref('5') // 'v5', 'v4', 'v3'
  const dictPath = ref('')
  const useGpu = ref(false)
  const gpuDevice = ref('0')
  const intraThreadNum  = ref(1);
  const intraSpinning  = ref(false);
  const interThreadNum = ref(1);
  const interSpinning = ref(false);
  const detModelPath = ref('')
  const recModelPath = ref('')
  const detInputSize = ref(640);
  const recInputSize = ref(32);
  const detDbThresh = ref(0.3);
  const detDbBoxThresh = ref(0.5);
  const detConfidenceThresh = ref(0.25);
  const detNmsIouThresh = ref(0.45);
  const detMinArea = ref(256.0)
  const unclipRatio = ref(1.5);
  const useDilation = ref(false);
  const recConfidenceThreshold = ref(0.5);
  const detModelType = ref(2); // assuming u8 as number
  const detExecutionProvider = ref('cpu');
  const recExecutionProvider = ref('cpu');
  const imagePath = ref('')

  // 加载配置
  function loadConfig() {
    try {
      const savedConfig = localStorage.getItem('ppocrDev')
      if (savedConfig) {
        const config = JSON.parse(savedConfig)
        if (config.modelType) modelType.value = config.modelType
        if (config.modelVersion) modelVersion.value = config.modelVersion
        if (config.dictPath) dictPath.value = config.dictPath
        if (config.useGpu) useGpu.value = config.useGpu
        if (config.gpuDevice) gpuDevice.value = config.gpuDevice
        if (config.intraThreadNum) intraThreadNum.value = config.intraThreadNum
        if (config.intraSpinning) intraSpinning.value = config.intraSpinning
        if (config.interThreadNum) interThreadNum.value = config.interThreadNum
        if (config.interSpinning) interSpinning.value = config.interSpinning
        if (config.detModelPath) detModelPath.value = config.detModelPath
        if (config.recModelPath) recModelPath.value = config.recModelPath
        if (config.detInputSize) detInputSize.value = config.detInputSize
        if (config.recInputSize) recInputSize.value = config.recInputSize
        if (config.detDbThresh) detDbThresh.value = config.detDbThresh
        if (config.detDbBoxThresh) detDbBoxThresh.value = config.detDbBoxThresh
        if (config.detConfidenceThresh) detConfidenceThresh.value = config.detConfidenceThresh
        if (config.detNmsIouThresh) detNmsIouThresh.value = config.detNmsIouThresh
        if (config.detMinArea) detMinArea.value = config.detMinArea
        if (config.unclipRatio) unclipRatio.value = config.unclipRatio
        if (config.useDilation) useDilation.value = config.useDilation
        if (config.recConfidenceThreshold) recConfidenceThreshold.value = config.recConfidenceThreshold
        if (config.detModelType) detModelType.value = config.detModelType
        if (config.detExecutionProvider) detExecutionProvider.value = config.detExecutionProvider
        if (config.recExecutionProvider) recExecutionProvider.value = config.recExecutionProvider
        if (config.imagePath) imagePath.value = config.imagePath
      }
      
      // 如果是内置模型且路径为空，设置默认路径
      updateDefaultPaths()
    } catch (error) {
      console.error('加载PPOCR配置失败:', error)
    }
  }
  
  // 保存配置
  function saveConfig() {
    try {
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
        detConfidenceThresh: detConfidenceThresh.value,
        detNmsIouThresh: detNmsIouThresh.value,
        detMinArea: detMinArea.value,
        unclipRatio: unclipRatio.value,
        useDilation: useDilation.value,
        recConfidenceThreshold: recConfidenceThreshold.value,
        detModelType: detModelType.value,
        detExecutionProvider: detExecutionProvider.value,
        recExecutionProvider: recExecutionProvider.value,
        imagePath: imagePath.value
      }
      localStorage.setItem('ppocrDev', JSON.stringify(config))
    } catch (error) {
      console.error('保存PPOCR配置失败:', error)
    }
  }
  
  // 根据模型版本更新默认路径
  function updateDefaultPaths() {
    if (modelType.value === 'built-in') {
      // 使用相对于应用根目录的路径
      dictPath.value = `models/ppocr/ch_v${modelVersion.value}_dict.txt`
      useGpu.value = false
      gpuDevice.value = '0'
      intraThreadNum.value = 1
      intraSpinning.value = false
      interThreadNum.value = 1
      interSpinning.value = false
      detModelPath.value = `models/ppocr/ch_mobile_v${modelVersion.value}_det.onnx`
      recModelPath.value = `models/ppocr/ch_mobile_v${modelVersion.value}_rec.onnx`
      detInputSize.value = 640
      recInputSize.value = 48,
      detDbThresh.value = 0.3,
      detDbBoxThresh.value = 0.5,
      detConfidenceThresh.value = 0.25,
      detNmsIouThresh.value = 0.45,
      detMinArea.value = 256.0,
      unclipRatio.value = 1.5,
      useDilation.value = false,
      recConfidenceThreshold.value = 0.5,
      detModelType.value = 2,
      detExecutionProvider.value = 'cpu',
      recExecutionProvider.value = 'cpu'
      //imagePath.value = ''
    }
  }
  
  return {
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
    detMinArea,
    unclipRatio,
    useDilation,
    recConfidenceThreshold,
    detModelType,
    detExecutionProvider,
    recExecutionProvider,
    imagePath,
    loadConfig,
    saveConfig,
    updateDefaultPaths
  }
}, {
  persist: true, // 使用pinia-plugin-persistedstate插件可以自动持久化
})