import { invoke } from '@/utils/api';
import type { VisionLabModelConfig } from '@/types/app/domain';

export const visionLabConfigService = {
    getModelConfig: () =>
        invoke('get_vision_lab_model_config_cmd') as Promise<VisionLabModelConfig>,
    setModelConfig: (config: VisionLabModelConfig) =>
        invoke('set_vision_lab_model_config_cmd', { config }) as Promise<string>,
};
