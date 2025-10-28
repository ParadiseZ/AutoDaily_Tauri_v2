/*use crate::domain::entities::app_result::AppResult;
use crate::domain::entities::config::performance::Performance;
use crate::infrastructure::entities::vision::examples::det_rec_ocr::load_image_example;
/// 多脚本多设备OCR服务使用示例
///
/// 此文件展示了如何在多线程环境中安全地使用OCR服务，
/// 解决资源错位和模型共享问题
use crate::infrastructure::entities::vision::OcrService;
use crate::infrastructure::factory::ocr_factory::{
    DetectorConfig, OcrModelFactory, RecognizerConfig
};
use image::DynamicImage;
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::Semaphore;

/// 脚本任务配置
#[derive(Debug, Clone)]
pub struct ScriptTaskConfig {
    pub script_id: String,
    pub device_ids: Vec<u32>,
    pub detector_config: DetectorConfig,
    pub recognizer_config: RecognizerConfig,
}

/// OCR任务管理器 - 解决多脚本多设备的资源管理问题
pub struct OcrTaskManager {
    app: Arc<AppHandle>,
    performance_config: Performance,
    // 限制并发设备数量
    device_semaphore: Arc<Semaphore>,
}

impl OcrTaskManager {
    pub fn new(app: AppHandle, performance_config: Performance) -> Self {
        //let max_concurrent_devices = performance_config.max_devices * performance_config.cores_per_device;
        let max_concurrent_devices = performance_config.max_devices;
        Self {
            app: Arc::new(app),
            performance_config,
            device_semaphore: Arc::new(Semaphore::new(max_concurrent_devices)),
        }
    }

    /// 启动多个脚本任务，每个脚本可以在多个设备上运行
    pub async fn run_scripts(&self, script_configs: Vec<ScriptTaskConfig>) -> AppResult<()> {
        let mut script_handles = Vec::new();

        for script_config in script_configs {
            let script_handle = self.spawn_script_task(script_config).await?;
            script_handles.push(script_handle);
        }

        // 等待所有脚本任务完成
        for handle in script_handles {
            if let Err(e) = handle.await {
                eprintln!("脚本任务执行失败: {}", e);
            }
        }

        Ok(())
    }

    /// 为单个脚本启动任务，支持多设备运行
    async fn spawn_script_task(&self, script_config: ScriptTaskConfig) -> AppResult<tokio::task::JoinHandle<()>> {
        let app = self.app.clone();
        let device_semaphore = self.device_semaphore.clone();
        
        let handle = tokio::spawn(async move {
            let mut device_handles = Vec::new();

            // 为每个设备创建独立的OCR服务
            for device_id in script_config.device_ids {
                let device_handle = Self::spawn_device_task(
                    app.clone(),
                    device_semaphore.clone(),
                    script_config.script_id.clone(),
                    device_id,
                    script_config.detector_config.clone(),
                    script_config.recognizer_config.clone(),
                ).await;

                device_handles.push(device_handle);
            }

            // 等待所有设备任务完成
            for handle in device_handles {
                if let Err(e) = handle.await {
                    eprintln!("设备任务执行失败: {}", e);
                }
            }
        });

        Ok(handle)
    }

    /// 为单个设备创建OCR服务并执行任务
    async fn spawn_device_task(
        app: Arc<AppHandle>,
        device_semaphore: Arc<Semaphore>,
        script_id: String,
        device_id: u32,
        detector_config: DetectorConfig,
        recognizer_config: RecognizerConfig,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            // 获取设备信号量，限制并发数
            let _permit = device_semaphore.acquire().await.unwrap();
            
            println!("🚀 启动脚本 {} 在设备 {} 上的OCR任务", script_id, device_id);

            // 创建独立的OCR服务实例
            let mut ocr_service = OcrService::new();

            // 初始化检测器（可能与其他实例共享模型）
            match ocr_service.init_detector(detector_config).await {
                Ok(_) => println!("✅ 脚本 {} 设备 {} 检测器初始化成功", script_id, device_id),
                Err(e) => {
                    eprintln!("❌ 脚本 {} 设备 {} 检测器初始化失败: {}", script_id, device_id, e);
                    return;
                }
            }

            // 初始化识别器（可能与其他实例共享模型）
            match ocr_service.init_recognizer(recognizer_config).await {
                Ok(_) => println!("✅ 脚本 {} 设备 {} 识别器初始化成功", script_id, device_id),
                Err(e) => {
                    eprintln!("❌ 脚本 {} 设备 {} 识别器初始化失败: {}", script_id, device_id, e);
                    return;
                }
            }

            // 模拟OCR任务执行
            for task_id in 0..5 {
                let img_date = load_image_example().unwrap();
                match Self::execute_ocr_task(&ocr_service, &script_id, device_id, task_id, &img_date).await {
                    Ok(_) => println!("✅ 脚本 {} 设备 {} 任务 {} 完成", script_id, device_id, task_id),
                    Err(e) => eprintln!("❌ 脚本 {} 设备 {} 任务 {} 失败: {}", script_id, device_id, task_id, e),
                }

                // 模拟任务间隔
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            println!("🏁 脚本 {} 设备 {} 所有任务完成", script_id, device_id);
        })
    }

    /// 执行具体的OCR任务
    async fn execute_ocr_task(
        ocr_service: &OcrService,
        script_id: &str,
        device_id: u32,
        task_id: usize,
        image_data : &DynamicImage
    ) -> AppResult<()> {
        // 模拟图像数据
        //let image_data = Self::generate_mock_image_data(task_id);

        // 执行OCR处理
        let results = ocr_service.ocr(image_data).await?;

        println!(
            "📄 脚本 {} 设备 {} 任务 {} OCR结果: {} 个文本区域",
            script_id, device_id, task_id, results.len()
        );

        Ok(())
    }

    /// 生成模拟图像数据
    fn generate_mock_image_data(task_id: usize) -> Vec<u8> {
        // 在实际应用中，这里会是真实的图像数据
        vec![task_id as u8; 1024]
    }
}


/// 主函数示例 - 演示如何运行多脚本多设备OCR任务
pub async fn run_multi_script_ocr_example(app: AppHandle) -> AppResult<()> {
    println!("🎬 开始多脚本多设备OCR任务示例");

    // 性能配置（每设备4核心，最多2个设备）
    let performance_config = Performance {
        cores_per_device: 4,
        max_devices: 2,
    };

    // 创建任务管理器
    let task_manager = OcrTaskManager::new(app, performance_config);

    // 创建脚本配置
    //let script_configs = create_example_script_configs();
    let script_configs = Vec::new();

    println!("📊 配置概览:");
    for config in &script_configs {
        println!(
            "  - 脚本 {}: {} 检测器, 设备 {:?}",
            config.script_id,
            format!("{:?}", config.detector_config.detector_type),
            config.device_ids
        );
    }

    // 显示缓存统计
    let (detector_count, recognizer_count) = OcrModelFactory::get_cache_statistics().await;
    println!("📈 当前缓存: {} 个检测器, {} 个识别器", detector_count, recognizer_count);

    // 运行所有脚本
    task_manager.run_scripts(script_configs).await?;

    // 显示最终缓存统计
    let (final_detector_count, final_recognizer_count) = OcrModelFactory::get_cache_statistics().await;
    println!("📈 最终缓存: {} 个检测器, {} 个识别器", final_detector_count, final_recognizer_count);

    println!("🎉 多脚本多设备OCR任务示例完成");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_script_config_creation() {
        let configs =  Vec::new();;
        assert_eq!(configs.len(), 3);
        
        // 验证脚本A和脚本C使用相同的检测器配置（应该共享模型）
        assert_eq!(
            configs[0].detector_config.detector_type,
            configs[2].detector_config.detector_type
        );
        assert_eq!(
            configs[0].detector_config.model_path,
            configs[2].detector_config.model_path
        );
    }

    #[test]
    fn test_performance_calculation() {
        let performance = Performance {
            cores_per_device: 4,
            max_devices: 2,
        };
        
        let max_concurrent = performance.max_devices;
        assert_eq!(max_concurrent, 8);
    }
}
*/