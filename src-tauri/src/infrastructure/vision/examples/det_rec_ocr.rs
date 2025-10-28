/*use std::path::Path;
use image::DynamicImage;
/// OCR服务使用示例
///
/// 本文件展示了如何使用新的基于trait的OCR架构
use crate::infrastructure::entities::vision::{OcrService};
use crate::infrastructure::factory::ocr_factory::{OcrModelFactory};
use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::config::log_conf::{Log, Logger};

/// 示例1: 使用YOLO检测器 + CRNN识别器
pub async fn example_yolo_crnn_ocr() -> AppResult<()> {
    // 1. 创建OCR服务实例
    let mut ocr_service = OcrService::new();

    // 2. 配置并初始化YOLO检测器
    let detector_config = OcrModelFactory::yolo_detector_config(
        "models/ppocr/ch_mobile_v5_det.onnx".into(),
        Some("cuda".into())
    );
    ocr_service.init_detector(detector_config).await?;

    // 3. 配置并初始化CRNN识别器
    let recognizer_config = OcrModelFactory::crnn_recognizer_config(
        "models/ppocr/ch_mobile_v5_rec.onnx".into(),
        Some("models/ppocr/ch_v5_dict.txt".into()),
        Some("cuda".into())
    );
    ocr_service.init_recognizer(recognizer_config).await?;

    // 4. 检查服务是否就绪
    if !ocr_service.is_ready() {
        return Err("OCR服务未就绪".into());
    }

    // 5. 加载图像数据 (这里用示例数据)
    let image_data = load_image_example()?;

    // 6. 执行完整的OCR流程
    let results = ocr_service.ocr(&image_data).await?;

    // 7. 处理结果
    for result in results {
        println!("检测结果: {:?}", result);
    }

    // 8. 获取服务信息
    if let Some(detector_info) = ocr_service.detector_info() {
        println!("检测器信息: {}", detector_info);
    }

    if let Some(recognizer_info) = ocr_service.recognizer_info() {
        println!("识别器信息: {}", recognizer_info);
    }

    Ok(())
}

/// 示例2: 使用DBNet检测器 + CRNN识别器
pub async fn example_dbnet_crnn_ocr() -> AppResult<()> {
    let mut ocr_service = OcrService::new();

    // 配置DBNet检测器
    let detector_config = OcrModelFactory::dbnet_detector_config(
        "models/ppocr/ch_mobile_v5_det.onnx".to_string(),
        Some("cpu".into())
    );
    ocr_service.init_detector(detector_config).await?;

    // 配置CRNN识别器
    let recognizer_config = OcrModelFactory::crnn_recognizer_config(
        "models/ppocr/ch_mobile_v5_rec.onnx".into(),
        None,  // 使用默认字典
        Some("cpu".into())
    );
    ocr_service.init_recognizer(recognizer_config).await?;

    let image_data = load_image_example()?;
    let results = ocr_service.ocr(&image_data).await?;

    for result in results {
        println!("检测结果: {:?}", result);
    }

    Ok(())
}

/// 示例4: 自定义配置参数
pub async fn example_custom_configuration() -> AppResult<()> {
    use crate::infrastructure::factory::ocr_factory::{DetectorConfig, RecognizerConfig, DetectorType, RecognizerType};

    let mut ocr_service = OcrService::new();

    // 自定义YOLO检测器配置
    let custom_detector_config = DetectorConfig {
        detector_type: DetectorType::Yolo11,
        model_path: "models/custom_yolo.onnx".into(),
        execution_provider: "cpu".into(),
        input_width: 1024,  // 更高分辨率
        input_height: 1024,

        intra_thread_num: 4,
        intra_spinning: false,
        inter_thread_num: 1,
        inter_spinning: false,
        confidence_thresh: Some(0.7),  // 更高置信度阈值
        iou_thresh: Some(0.3),         // 更严格的IOU阈值
        class_count: Some(1),
        class_labels: Some(vec!["text".into()]),
        db_thresh: None,
        db_box_thresh: None,
        unclip_ratio: None,
        use_dilation: None,
    };

    // 自定义CRNN识别器配置
    let custom_recognizer_config = RecognizerConfig {
        recognizer_type: RecognizerType::PaddleCrnn,
        model_path: "models/custom_crnn.onnx".into(),
        execution_provider: "cpu".into(),
        input_width: 256,   // 自定义输入尺寸
        input_height: 32,
        dict_path: Some("models/custom_dict.txt".into()),
        beam_width: Some(5),  // 使用束搜索
        intra_thread_num: 4,
        intra_spinning: false,
        inter_thread_num: 1,
        inter_spinning: false,
    };

    ocr_service.init_detector(custom_detector_config).await?;
    ocr_service.init_recognizer(custom_recognizer_config).await?;

    let image_data = load_image_example()?;
    let results = ocr_service.ocr(&image_data).await?;

    for result in results {
        println!("检测结果: {:?}", result);
    }

    Ok(())
}



/// 运行所有示例的测试函数
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_all_examples() {
        // 注意: 这些测试需要实际的模型文件才能运行
        // 在实际环境中运行前需要确保模型文件存在

        println!("测试示例需要实际的模型文件，请在部署时添加模型文件路径");

        // example_yolo_crnn_ocr().await.expect("YOLO+CRNN示例失败");
        // example_dbnet_crnn_ocr().await.expect("DBNet+CRNN示例失败");
        // example_separate_detection_recognition().await.expect("分离式示例失败");
        // example_custom_configuration().await.expect("自定义配置示例失败");
    }
}
*/
/*use std::path::Path;
use image::DynamicImage;
use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::config::log_conf::{Log, Logger};

/// 加载示例图像数据
pub fn load_image_example() -> AppResult<DynamicImage> {
    // 这里应该加载实际的图像文件
    // 示例中返回空数据
    let image = match image::open(Path::new("D:/Database/Project/YOLO/AutoDailyTKFM/all_images/0014.jpg")) {
        Ok(img) => img,
        Err(e) => {
            let err_msg = format!("无法读取图像: {}", e);
            Log::error(&err_msg);
            return Err(AppError::IoError(err_msg));
        }
    };
    Ok(image) // 示例数据
}*/