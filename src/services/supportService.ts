import { ApiEnvelope, createServerResponseError, invoke } from '@/utils/api';

export type SupportDialogMode = 'report' | 'product-feedback' | 'script-feedback';

export interface SupportScriptContext {
    cloudId: string;
    name: string;
    authorName?: string | null;
    runtimeType?: string | null;
}

export interface SupportSubmissionResult {
    id: string;
    uploadedScreenshots: number;
    failedScreenshots: number;
    message?: string;
}

export interface ScriptReportInput {
    category: string;
    description: string;
    screenshotPaths: string[];
}

export interface FeedbackInput {
    targetType: 'product' | 'script';
    scriptId: string | null;
    category: string;
    title: string;
    description: string;
    reproductionSteps: string | null;
    expectedBehavior: string | null;
    actualBehavior: string | null;
    runtimeType: string | null;
    screenshotPaths: string[];
}

const unwrap = (command: string, response: ApiEnvelope<SupportSubmissionResult>) => {
    if (!response.success || !response.data) {
        throw createServerResponseError(command, response);
    }
    return { ...response.data, message: response.message };
};

export function getSupportSubmissionSuccessMessage(mode: SupportDialogMode, result: SupportSubmissionResult) {
    const fallback = mode === 'report' ? '举报已提交，我们会进行核查' : '反馈已提交';
    const message = result.message?.trim() || fallback;
    const attachmentHint = result.failedScreenshots ? `，其中 ${result.failedScreenshots} 张截图上传失败` : '';
    return `${message}${attachmentHint}`;
}

export const supportService = {
    reportScript: async (scriptId: string, input: ScriptReportInput) => {
        const response = (await invoke('backend_create_script_report', {
            scriptId,
            req: { category: input.category, description: input.description },
            screenshotPaths: input.screenshotPaths,
        })) as ApiEnvelope<SupportSubmissionResult>;
        return unwrap('backend_create_script_report', response);
    },
    createFeedback: async (input: FeedbackInput) => {
        const response = (await invoke('backend_create_feedback', {
            req: {
                targetType: input.targetType,
                scriptId: input.scriptId,
                category: input.category,
                title: input.title,
                description: input.description,
                reproductionSteps: input.reproductionSteps,
                expectedBehavior: input.expectedBehavior,
                actualBehavior: input.actualBehavior,
                runtimeType: input.runtimeType,
            },
            screenshotPaths: input.screenshotPaths,
        })) as ApiEnvelope<SupportSubmissionResult>;
        return unwrap('backend_create_feedback', response);
    },
};
