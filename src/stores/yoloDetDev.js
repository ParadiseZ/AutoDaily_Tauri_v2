// src/stores/yoloConfigStore.js
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useYoloDetDevStore = defineStore('yoloDetDev', () => {
  // 定义状态
  const modelVersion = ref('yolov8')
  const modelPath = ref('')
  const classFilePath = ref('')
  const targetSize = ref(640)
  const confidenceThreshold = ref(0.25)
  const iouThreshold = ref(0.45)
  const device = ref('cpu')
  const imagePath = ref('')
  
  // 加载配置
  function loadConfig() {
    try {
      const savedConfig = localStorage.getItem('yoloDetDev')
      if (savedConfig) {
        const config = JSON.parse(savedConfig)
        if (config.modelVersion) modelVersion.value = config.modelVersion
        if (config.modelPath) modelPath.value = config.modelPath
        if (config.classFilePath) classFilePath.value = config.classFilePath
        if (config.targetSize) targetSize.value = config.targetSize
        if (config.confidenceThreshold) confidenceThreshold.value = config.confidenceThreshold
        if (config.iouThreshold) iouThreshold.value = config.iouThreshold
        if (config.device) device.value = config.device
        if (config.imagePath) imagePath.value = config.imagePath
      }
    } catch (error) {
      console.error('加载YOLO配置失败:', error)
    }
  }
  
  // 保存配置
  function saveConfig() {
    try {
      const config = {
        modelVersion: modelVersion.value,
        modelPath: modelPath.value,
        classFilePath: classFilePath.value,
        targetSize: targetSize.value,
        confidenceThreshold: confidenceThreshold.value,
        iouThreshold: iouThreshold.value,
        device: device.value,
        imagePath: imagePath.value
      }
      localStorage.setItem('yoloDetDev', JSON.stringify(config))
    } catch (error) {
      console.error('保存YOLO配置失败:', error)
    }
  }
  
  return {
    modelVersion,
    modelPath,
    classFilePath,
    targetSize,
    confidenceThreshold,
    iouThreshold,
    device,
    imagePath,
    loadConfig,
    saveConfig
  }
}, {
  persist: true, // 使用pinia-plugin-persistedstate插件可以自动持久化
})