/**
 * Drag and Drop Composable for Script Editor
 * 
 * 这个 composable 负责处理从 Toolbox 拖动节点到画布的逻辑。
 * 与 VueFlow 官方示例不同，它使用回调函数来添加节点，
 * 这样可以与自定义的节点管理逻辑(v-model绑定)保持一致。
 */

import { ref, watch } from 'vue';
import { useVueFlow } from '@vue-flow/core';

/**
 * 模块级别的拖动状态 (单例模式)
 * 使用单例确保 Toolbox 和 VueFlow 画布共享相同的拖动状态
 */
const dragState = {
    draggedType: ref(null),
    isDragOver: ref(false),
    isDragging: ref(false),
};

/**
 * Drag and Drop Composable
 * 
 * @param {Object} options - 配置选项
 * @param {Function} options.onAddNode - 添加节点的回调函数 (type: string, position: {x, y}) => void
 * @returns {Object} 拖放相关的状态和方法
 */
export function useDragAndDrop(options = {}) {
    const { draggedType, isDragOver, isDragging } = dragState;
    const { onAddNode } = options;

    // 获取 VueFlow 的坐标转换方法
    const { screenToFlowCoordinate } = useVueFlow();

    // 监听拖动状态，禁用文本选择
    watch(isDragging, (dragging) => {
        document.body.style.userSelect = dragging ? 'none' : '';
    });

    /**
     * 开始拖动 (从 Toolbox 触发)
     * @param {DragEvent} event 
     * @param {string} type - 节点类型
     */
    function onDragStart(event, type) {
        if (event.dataTransfer) {
            event.dataTransfer.setData('application/vueflow', type);
            event.dataTransfer.effectAllowed = 'move';
        }
        draggedType.value = type;
        isDragging.value = true;

        // 监听全局 drop 事件以清理状态
        document.addEventListener('drop', onDragEnd);
    }

    /**
     * 拖动经过画布
     * @param {DragEvent} event 
     */
    function onDragOver(event) {
        event.preventDefault();

        if (draggedType.value) {
            isDragOver.value = true;

            if (event.dataTransfer) {
                event.dataTransfer.dropEffect = 'move';
            }
        }
    }

    /**
     * 拖动离开画布
     */
    function onDragLeave() {
        isDragOver.value = false;
    }

    /**
     * 拖动结束/取消
     */
    function onDragEnd() {
        isDragging.value = false;
        isDragOver.value = false;
        draggedType.value = null;
        document.removeEventListener('drop', onDragEnd);
    }

    /**
     * 放置到画布
     * @param {DragEvent} event 
     */
    function onDrop(event) {
        // 计算放置位置 (屏幕坐标转换为画布坐标)
        const position = screenToFlowCoordinate({
            x: event.clientX,
            y: event.clientY,
        });

        const type = draggedType.value;

        // 重置状态
        onDragEnd();

        // 如果有回调函数，调用它来添加节点
        if (onAddNode && type) {
            onAddNode(type, position);
        } else if (!onAddNode) {
            console.warn('[useDragAndDrop] onAddNode callback is not provided. Node will not be added.');
        }
    }

    return {
        // 状态
        draggedType,
        isDragOver,
        isDragging,
        // 方法
        onDragStart,
        onDragOver,
        onDragLeave,
        onDrop,
    };
}

export default useDragAndDrop;
