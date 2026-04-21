import type { BaseModel } from '@/types/bindings/BaseModel';
import type { DetectorType } from '@/types/bindings/DetectorType';
import type { PaddleDetDbNet } from '@/types/bindings/PaddleDetDbNet';
import type { PaddleRecCrnn } from '@/types/bindings/PaddleRecCrnn';
import type { RecognizerType } from '@/types/bindings/RecognizerType';
import type { YoloDet } from '@/types/bindings/YoloDet';

export type DetectorKind = 'none' | 'Yolo11' | 'PaddleDbNet' | 'Yolo26';
export type RecognizerKind = 'none' | 'PaddleCrnn';

function clone<T>(value: T): T {
    return JSON.parse(JSON.stringify(value)) as T;
}

export function createBaseModel(
    modelType: BaseModel['modelType'],
    width: number,
    height: number,
    modelSource: BaseModel['modelSource'] = 'Custom',
): BaseModel {
    return {
        intraThreadNum: 4,
        intraSpinning: true,
        interThreadNum: 1,
        interSpinning: true,
        executionProvider: 'CPU',
        inputWidth: width,
        inputHeight: height,
        modelSource,
        modelPath: '',
        modelType,
    };
}

export function createYoloDet(kind: 'Yolo11' | 'Yolo26', textMode: boolean): YoloDet {
    return {
        baseModel: createBaseModel(kind, 640, 640, 'Custom'),
        classCount: 80,
        confidenceThresh: 0.25,
        iouThresh: 0.45,
        labelPath: null,
        txtIdx: textMode ? 0 : null,
    };
}

export function createDbNet(): PaddleDetDbNet {
    return {
        baseModel: createBaseModel('PaddleDet5', 640, 640, 'Custom'),
        dbThresh: 0.3,
        dbBoxThresh: 0.5,
        unclipRatio: 1.5,
        useDilation: false,
    };
}

export function createCrnn(): PaddleRecCrnn {
    return {
        baseModel: createBaseModel('PaddleCrnn5', 320, 48, 'BuiltIn'),
        dictPath: null,
        resizeFilter: 'Triangle',
        processingMode: 'Single',
        microBatchSize: 4,
        widthBucketStep: 32,
    };
}

export function resolveDetectorKind(model: DetectorType | null): DetectorKind {
    if (!model) return 'none';
    if ('Yolo11' in model) return 'Yolo11';
    if ('Yolo26' in model) return 'Yolo26';
    if ('PaddleDbNet' in model) return 'PaddleDbNet';
    return 'none';
}

export function resolveRecognizerKind(model: RecognizerType | null): RecognizerKind {
    if (!model) return 'none';
    if ('PaddleCrnn' in model) return 'PaddleCrnn';
    return 'none';
}

export function ensureDetectorModel(model: DetectorType | null, textMode: boolean): DetectorType {
    if (model) {
        return clone(model);
    }
    return { PaddleDbNet: createDbNet() };
}

export function ensureRecognizerModel(model: RecognizerType | null): RecognizerType {
    if (model) {
        return clone(model);
    }
    return { PaddleCrnn: createCrnn() };
}

export function extractYoloDetector(model: DetectorType | null): YoloDet | null {
    if (!model) return null;
    if ('Yolo11' in model) return model.Yolo11;
    if ('Yolo26' in model) return model.Yolo26;
    return null;
}

export function extractDbNet(model: DetectorType | null): PaddleDetDbNet | null {
    if (!model || !('PaddleDbNet' in model)) {
        return null;
    }
    return model.PaddleDbNet;
}

export function extractCrnn(model: RecognizerType | null): PaddleRecCrnn | null {
    if (!model || !('PaddleCrnn' in model)) {
        return null;
    }
    return model.PaddleCrnn;
}

export function createDetectorByKind(kind: DetectorKind, textMode: boolean): DetectorType | null {
    if (kind === 'none') return null;
    if (kind === 'Yolo11') return { Yolo11: createYoloDet('Yolo11', textMode) };
    if (kind === 'Yolo26') return { Yolo26: createYoloDet('Yolo26', textMode) };
    return { PaddleDbNet: createDbNet() };
}

export function createRecognizerByKind(kind: RecognizerKind): RecognizerType | null {
    if (kind === 'none') return null;
    return { PaddleCrnn: createCrnn() };
}

export function rewritePublishedDetectorModelPath(
    model: DetectorType | null,
    scriptId: string,
    fileName: 'det.onnx' | 'txt_det.onnx',
): DetectorType | null {
    if (!model) {
        return null;
    }
    const next = clone(model);
    if ('Yolo11' in next) {
        if (next.Yolo11.baseModel.modelSource === 'Custom') {
            next.Yolo11.baseModel.modelPath = `${scriptId}/${fileName}`;
        }
        return next;
    }
    if ('Yolo26' in next) {
        if (next.Yolo26.baseModel.modelSource === 'Custom') {
            next.Yolo26.baseModel.modelPath = `${scriptId}/${fileName}`;
        }
        return next;
    }
    if (next.PaddleDbNet.baseModel.modelSource === 'Custom') {
        next.PaddleDbNet.baseModel.modelPath = `${scriptId}/${fileName}`;
    }
    return next;
}

export function rewritePublishedRecognizerModelPath(
    model: RecognizerType | null,
    scriptId: string,
): RecognizerType | null {
    if (!model) {
        return null;
    }
    const next = clone(model);
    if ('PaddleCrnn' in next && next.PaddleCrnn.baseModel.modelSource === 'Custom') {
        next.PaddleCrnn.baseModel.modelPath = `${scriptId}/rec.onnx`;
    }
    return next;
}
