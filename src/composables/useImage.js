import {BaseDirectory, readFile} from '@tauri-apps/plugin-fs'; // 如果使用 Tauri fs 读取二进制文件

function getMimeType(extension) {
    const mimeTypes = {
        'png': 'image/png',
        'jpg': 'image/jpeg',
        'jpeg': 'image/jpeg',
        'bmp': 'image/bmp',
        'webp': 'image/webp',
        'gif': 'image/gif'
    };
    return mimeTypes[extension] || 'image/png';
}

function arrayBufferToBase64(buffer) {
    let binary = '';
    const bytes = new Uint8Array(buffer);
    const len = bytes.byteLength;
    for (let i = 0; i < len; i++) {
        binary += String.fromCharCode(bytes[i]);
    }
    return window.btoa(binary);
}
export function useImage() {
    // 读取图像文件并转换为 Base64
    async function loadImage(imagePath) {
        const imageData = await readFile(imagePath, {
            baseDir: BaseDirectory.Picture,
        });
        // 将 Uint8Array 转换为 base64 字符串
        const base64String = arrayBufferToBase64(imageData);

        // 根据文件扩展名确定 MIME 类型
        const extension = imagePath.split('.').pop().toLowerCase();
        const mimeType = getMimeType(extension);

        // 创建 data URL
        return`data:${mimeType};base64,${base64String}`;
    }
    return {
        loadImage,
    };
}