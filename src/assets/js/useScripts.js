
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export const useScripts = () => {
    const scripts = ref([
        {
            name: '自动日常任务',
            description: '包含每日签到、体力消耗等自动化脚本',
            userId: 'user_001',
            userName: 'Admin',
            verName: 'v1.2.0',
            verNum: 120,
            latestVer: 125,
            downloadCount: 1540,
            scriptType: 'Official',
            isValid: true,
            createTime: '2023-10-01 12:00:00',
            updateTime: '2023-12-25 15:30:00',
            pkgName: 'com.game.auto',
            imgDetModel: 'YOLOv8',
            txtDetModel: 'PaddleOCR_v4',
            txtRecModel: 'PaddleOCR_v4_rec',
            tasks: [
                { id: 1, name: '加载界面检测', enabled: true, delay: 500, indent: 0 },
                { id: 2, name: '签到功能', enabled: true, delay: 1000, indent: 1 },
                { id: 3, name: '点击签到按钮', enabled: true, delay: 200, indent: 2 },
                { id: 4, name: '领取奖励', enabled: false, delay: 500, indent: 2 },
                { id: 5, name: '体力管理', enabled: true, delay: 2000, indent: 1 },
            ],
            templates: [
                { id: 't1', name: '极速模式', config: { delayMultiplier: 0.5 } },
                { id: 't2', name: '稳定模式', config: { delayMultiplier: 1.2 } }
            ]
        },
        {
            name: '自定义脚本_打怪',
            description: '用户自己录制的打怪逻辑',
            userId: 'user_001',
            userName: 'Admin',
            verName: 'v0.1.0',
            verNum: 10,
            latestVer: 10,
            downloadCount: 0,
            scriptType: 'Custom',
            isValid: true,
            createTime: '2024-01-05 09:00:00',
            updateTime: '2024-01-05 09:00:00',
            pkgName: 'custom.battle',
            imgDetModel: 'YOLOv5',
            tasks: [
                { id: 101, name: '寻找目标', enabled: true, delay: 100, indent: 0 },
                { id: 102, name: '攻击', enabled: true, delay: 50, indent: 1 }
            ],
            templates: []
        }
    ]);

    const selectedScript = ref(null);
    const selectedTemplate = ref(null);

    const selectScript = (script) => {
        selectedScript.value = script;
        if (script.templates && script.templates.length > 0) {
            selectedTemplate.value = script.templates[0].id;
        } else {
            selectedTemplate.value = null;
        }
    };

    const deleteScript = async (script) => {
        // Mock delete
        scripts.value = scripts.value.filter(s => s !== script);
        if (selectedScript.value === script) {
            selectedScript.value = null;
        }
    };

    const editScript = (script) => {
        console.log('Edit script:', script.name);
    };

    return {
        scripts,
        selectedScript,
        selectedTemplate,
        selectScript,
        deleteScript,
        editScript
    };
};
