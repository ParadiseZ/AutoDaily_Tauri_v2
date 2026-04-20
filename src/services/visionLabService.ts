import { invoke } from '@/utils/api';
import type { DeviceTable } from '@/types/bindings/DeviceTable';
import type { DetectorType } from '@/types/bindings/DetectorType';
import type { OcrResult } from '@/types/bindings/OcrResult';
import type { DetResult } from '@/types/bindings/DetResult';
import type { RecognizerType } from '@/types/bindings/RecognizerType';
import type { ADBConnectConfig } from '@/types/bindings/ADBConnectConfig';

const fallbackAdbConfig: ADBConnectConfig = { directTcp: null };

const resolveCaptureMethod = (device: DeviceTable) => (device.data.capMethod === 'adb' ? 2 : 1);
const resolveCaptureType = (device: DeviceTable) => (device.data.capMethod === 'adb' ? 'adb' : 'window');

export const visionLabService = {
    listImageFiles: (dirPath: string) =>
        invoke('vision_list_image_files_cmd', { dirPath }) as Promise<string[]>,
    stageCaptureImage: (imageData: string, suggestedName?: string | null) =>
        invoke('vision_stage_capture_image_cmd', {
            imageData,
            suggestedName: suggestedName || null,
        }) as Promise<string>,
    saveStagedImage: (stagedPath: string, saveDir: string, fileName?: string | null) =>
        invoke('vision_save_staged_image_cmd', {
            stagedPath,
            saveDir,
            fileName: fileName || null,
        }) as Promise<string>,
    runDetection: (detectorConf: DetectorType, imagePath: string) =>
        invoke('yolo_inference_test', { detectorConf, imagePath }) as Promise<DetResult[]>,
    runOcr: (detModel: DetectorType, recModel: RecognizerType, imagePath: string) =>
        invoke('paddle_ocr_inference_test', { detModel, recModel, imagePath }) as Promise<OcrResult[]>,
    captureDevice: async (device: DeviceTable) => {
        const imageData = await invoke('dev_capture_test', {
            method: resolveCaptureMethod(device),
            deviceConf: device.data,
            adbConf: device.data.adbConnect ?? fallbackAdbConfig,
        }) as string;

        return {
            imageData,
            imageType: resolveCaptureType(device),
        };
    },
};
