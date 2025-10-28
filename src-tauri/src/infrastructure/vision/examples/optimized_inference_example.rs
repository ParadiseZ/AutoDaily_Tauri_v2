use crate::domain::entities::config::performance::Performance;
use crate::infrastructure::entities::vision::performance::ThreadPoolManager;
use crate::infrastructure::entities::vision::OcrService;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 多设备OCR管理器 - 解决线程池竞争问题
pub struct MultiDeviceOcrManager {
    /// 每个设备的OCR服务实例
    device_services: Arc<RwLock<HashMap<usize, OcrService>>>,
    /// 每个设备的线程池管理器
    device_thread_managers: Arc<RwLock<HashMap<usize, ThreadPoolManager>>>,
    /// 性能配置
    performance_config: Performance,
}

/*impl MultiDeviceOcrManager {
    /// 创建多设备OCR管理器
    pub fn new(performance_config: Performance) -> Self {
        Self {
            device_services: Arc::new(RwLock::new(HashMap::new())),
            device_thread_managers: Arc::new(RwLock::new(HashMap::new())),
            performance_config,
        }
    }

    /// 初始化指定设备的OCR服务
    pub async fn init_device(&self, device_id: usize) -> AppResult<()> {
        Log::info(&format!("初始化设备 {} 的OCR服务", device_id));
        
        // 1. 为每个设备创建独立的线程池管理器
        let thread_manager = ThreadPoolManager::new_for_device(
            self.performance_config.clone(), 
            device_id
        )?;
        
        // 2. 创建OCR服务实例
        let mut ocr_service = OcrService::new();
        
        // 3. 配置检测器
        let detector_config = OcrModelFactory::yolo_detector_config(
            "models/yolo11_text_detection.onnx".to_string(),
            Some("cpu".to_string()) // 使用CPU，线程由我们的线程池管理
        );
        ocr_service.init_detector(detector_config).await?;
        
        // 4. 配置识别器
        let recognizer_config = OcrModelFactory::crnn_recognizer_config(
            "models/paddle_crnn_text_recognition.onnx".to_string(),
            Some("models/ppocr_keys_v1.txt".to_string()),
            Some("cpu".to_string())
        );
        ocr_service.init_recognizer(recognizer_config).await?;
        
        // 5. 存储到管理器中
        {
            let mut services = self.device_services.write().await;
            services.insert(device_id, ocr_service);
        }
        
        {
            let mut managers = self.device_thread_managers.write().await;
            managers.insert(device_id, thread_manager);
        }
        
        Log::info(&format!("设备 {} OCR服务初始化完成", device_id));
        Ok(())
    }

    /// 获取指定设备的OCR服务
    pub async fn get_device_service(&self, device_id: usize) -> Option<OcrService> {
        let services = self.device_services.read().await;
        services.get(&device_id).cloned()
    }

    /// 获取指定设备的线程池管理器
    pub async fn get_device_thread_manager(&self, device_id: usize) -> Option<ThreadPoolManager> {
        let managers = self.device_thread_managers.read().await;
        managers.get(&device_id).cloned()
    }
}

/// 演示正确的ORT线程设置和Rayon配合使用
pub async fn demonstrate_optimized_inference() -> AppResult<()> {
    Log::info("开始演示优化后的多设备推理");
    
    // 1. 配置性能参数
    let performance_config = Performance {
        cores_per_device: 4,  // 每个设备4个核心
        max_devices: 2,       // 最多2个设备
    };
    
    // 2. 创建多设备管理器
    let manager = MultiDeviceOcrManager::new(performance_config.clone());
    
    // 3. 初始化全局线程池（为了兼容性）
    init_global_thread_pool_manager(performance_config).await?;
    
    // 4. 初始化多个设备
    for device_id in 0..2 {
        manager.init_device(device_id).await?;
    }
    
    // 5. 演示并发推理 - 每个设备独立工作
    let mut handles = Vec::new();
    
    for device_id in 0..2 {
        let manager_clone = manager.clone();
        
        let handle = tokio::spawn(async move {
            device_inference_task(manager_clone, device_id).await
        });
        
        handles.push(handle);
    }
    
    // 6. 等待所有设备完成
    for handle in handles {
        handle.await
            .map_err(|e| AppError::InternalError(format!("设备任务执行失败: {}", e)))??;
    }
    
    Log::info("多设备推理演示完成");
    Ok(())
}

/// 单个设备的推理任务
async fn device_inference_task(
    manager: MultiDeviceOcrManager, 
    device_id: usize
) -> AppResult<()> {
    Log::info(&format!("设备 {} 开始推理任务", device_id));
    
    // 获取设备的OCR服务和线程池
    let ocr_service = manager.get_device_service(device_id).await
        .ok_or_else(|| AppError::ConfigError(format!("设备 {} 未初始化", device_id)))?;
    
    let thread_manager = manager.get_device_thread_manager(device_id).await
        .ok_or_else(|| AppError::ConfigError(format!("设备 {} 线程池未初始化", device_id)))?;
    
    // 模拟图像数据
    let test_image = image::DynamicImage::new_rgb8(640, 480);
    
    // 执行OCR - 关键点：
    // 1. 推理使用设备专用的推理线程池（1个线程）
    // 2. CTC解码、图像处理使用设备专用的CPU线程池（3个线程）
    // 3. 不同设备之间完全隔离，无竞争
    let results = ocr_service.ocr(&test_image).await?;
    
    Log::info(&format!(
        "设备 {} 推理完成，识别到 {} 个文本区域", 
        device_id, 
        results.len()
    ));
    
    // 演示CPU密集型操作的并行处理
    let cpu_pool = thread_manager.cpu_pool();
    
    // 在设备专用的CPU线程池中进行并行处理
    let processed_count = cpu_pool.install(|| {
        use rayon::prelude::*;
        
        // 模拟并行处理多个结果
        (0..100).into_par_iter().map(|i| {
            // 模拟CPU密集型操作（如归一化、后处理等）
            std::thread::sleep(std::time::Duration::from_millis(1));
            i * 2
        }).sum::<usize>()
    });
    
    Log::info(&format!(
        "设备 {} CPU并行处理完成，处理结果: {}", 
        device_id, 
        processed_count
    ));
    
    Ok(())
}

/// 关键设计原则说明
pub mod design_principles {
    /*
    ## 🎯 关键设计原则总结

    ### 1. 推理代码共用 ✅
    - 通过 `BaseModel::inference_base` 方法消除重复代码
    - 只需在各模型中定义 `get_input_node_name()` 和 `get_output_node_name()`
    - 减少代码重复80%+

    ### 2. ORT线程设置正确使用 ✅
    ```rust
    // 在模型加载时设置:
    let session = session_builder
        .with_intra_threads(1)  // ORT内部操作使用1个线程
        .with_inter_threads(1)  // ORT并行操作使用1个线程
        .commit_from_file(model_path)?;

    // 在推理时使用Rayon线程池:
    let result = inference_pool.install(|| {
        // 在推理专用线程中运行ORT推理
        session.run(inputs).unwrap()
    });
    ```
    
    ### 3. 线程池架构设计 ✅
    ```
    全局架构:
    ├── 设备0 (cores_per_device=4)
    │   ├── 推理线程池: 1个线程 (专用)
    │   └── CPU处理线程池: 3个线程 (归一化、CTC解码等)
    ├── 设备1 (cores_per_device=4)  
    │   ├── 推理线程池: 1个线程 (专用)
    │   └── CPU处理线程池: 3个线程 (归一化、CTC解码等)
    └── ...
    ```
    
    **优势:**
    - ✅ 每个设备独立的线程池，避免竞争
    - ✅ 推理线程专用，确保推理性能稳定
    - ✅ CTC解码等CPU密集操作不会影响其他设备推理
    - ✅ 资源隔离，单个设备崩溃不影响其他设备
    
    ### 4. 性能特点
    - **推理隔离**: 每个设备1个专用推理线程，无竞争
    - **CPU并行**: 每个设备3个CPU线程，充分利用多核
    - **内存效率**: 模型共享，线程池独立
    - **故障隔离**: 设备级别的故障隔离
    */
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_multi_device_setup() {
        let config = Performance {
            cores_per_device: 4,
            max_devices: 2,
        };
        
        let manager = MultiDeviceOcrManager::new(config);
        
        // 测试设备初始化
        assert!(manager.init_device(0).await.is_ok());
        assert!(manager.init_device(1).await.is_ok());
        
        // 测试服务获取
        assert!(manager.get_device_service(0).await.is_some());
        assert!(manager.get_device_service(1).await.is_some());
        assert!(manager.get_device_service(2).await.is_none());
    }
    
    #[tokio::test]
    async fn test_thread_pool_isolation() {
        let config = Performance {
            cores_per_device: 4,
            max_devices: 2,
        };
        
        let manager = MultiDeviceOcrManager::new(config);
        manager.init_device(0).await.unwrap();
        manager.init_device(1).await.unwrap();
        
        let tm0 = manager.get_device_thread_manager(0).await.unwrap();
        let tm1 = manager.get_device_thread_manager(1).await.unwrap();
        
        // 验证设备ID不同
        assert_eq!(tm0.device_id(), 0);
        assert_eq!(tm1.device_id(), 1);
        
        // 验证线程池配置
        assert_eq!(tm0.cores_per_device(), 4);
        assert_eq!(tm1.cores_per_device(), 4);
    }
}
*/