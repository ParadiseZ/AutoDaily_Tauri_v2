import { invoke } from '@/utils/api';
import type { DeviceTable } from '@/types/bindings/DeviceTable';
import type { DetectorType } from '@/types/bindings/DetectorType';
import type { OcrResult } from '@/types/bindings/OcrResult';
import type { DetResult } from '@/types/bindings/DetResult';
import type { RecognizerType } from '@/types/bindings/RecognizerType';
const resolveCaptureType = (device: DeviceTable) => (device.data.capMethod.type === 'adb' ? 'adb' : 'window');

export const visionLabService = {
    listImageFiles: (dirPath: string) =>
        invoke('vision_list_image_files_cmd', { dirPath }) as Promise<string[]>,
    saveCaptureImage: (imageData: string, saveDir: string, fileName?: string | null) =>
        invoke('vision_save_capture_image_cmd', {
            imageData,
            saveDir,
            fileName: fileName || null,
        }) as Promise<string>,
    runDetection: (detectorConf: DetectorType, imagePath: string) =>
        invoke('yolo_inference_test', { detectorConf, imagePath }) as Promise<DetResult[]>,
    runDetectionForImageData: (detectorConf: DetectorType, imageData: string) =>
        invoke('yolo_inference_image_data_test', { detectorConf, imageData }) as Promise<DetResult[]>,
    runOcr: (detModel: DetectorType, recModel: RecognizerType, imagePath: string) =>
        invoke('paddle_ocr_inference_test', { detModel, recModel, imagePath }) as Promise<OcrResult[]>,
    runOcrForImageData: (detModel: DetectorType, recModel: RecognizerType, imageData: string) =>
        invoke('paddle_ocr_inference_image_data_test', { detModel, recModel, imageData }) as Promise<OcrResult[]>,
    captureDevice: async (device: DeviceTable) => {
        const imageData = await invoke('cmd_capture_device_image', {
            deviceId: device.id,
        }) as string;

        return {
            imageData,
            imageType: resolveCaptureType(device),
        };
    },
};
