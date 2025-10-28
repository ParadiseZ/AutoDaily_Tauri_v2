import {open} from "@tauri-apps/plugin-dialog";


export function useDevConfig() {
    async function browseModelFile() {
        return await open({
            multiple: false,
            filters: [{ name: 'ONNX模型文件', extensions: ['onnx'] }]
        });
    }
    async function browseClassFile() {
        return await open({
            multiple: false,
            filters: [{name: '标签配置文件', extensions: ['yaml']}]
        });
    }
    async function browseImageFile() {
        return await open({
            multiple: false,
            filters: [{
                name: '图像文件',
                extensions: ['png', 'jpg', 'jpeg', 'bmp', 'webp']
            }]
        });
    }

    async function browseDictFile() {
        return await open({
            multiple: false,
            filters: [{ name: '字典文件', extensions: ['txt'] }]
        });
    }



    return {
        browseClassFile,
        browseModelFile,
        browseImageFile,
        browseDictFile
    };
}