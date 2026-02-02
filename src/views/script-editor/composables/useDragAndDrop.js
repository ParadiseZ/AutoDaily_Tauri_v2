/**
 * Drag and Drop Composable for Script Editor
 * 
 * 这个 composable 负责处理从 Toolbox 拖动节点到画布的逻辑。
 * 与 VueFlow 官方示例不同，它使用回调函数来添加节点，
 * 这样可以与自定义的节点管理逻辑(v-model绑定)保持一致。
 * 
 * 使用方式：
 * - 在 Toolbox 端：const { onDragStart } = useDragAndDrop();
 * - 在 Canvas 端：const { onDragOver, onDrop, onDragLeave, isDragOver } = useDragAndDrop({ 
 *     onAddNode: addNodeToCanvas,
 *     screenToFlowCoordinate: vueFlowInstance.screenToFlowCoordinate 
 *   });
 */

import { ref, watch } from 'vue';

/**
 * 模块级别的拖动状态 (单例模式)
 * 使用单例确保 Toolbox 和 VueFlow 画布共享相同的拖动状态
 */
const dragState = {
    draggedType: ref(null),
    isDragOver: ref(false),
    isDragging: ref(false),
};

// 模块级别存储回调和坐标转换函数
let registeredAddNode = null;
let registeredScreenToFlowCoordinate = null;

/**
 * Drag and Drop Composable
 * 
 * @param {Object} options - 配置选项
 * @param {Function} options.onAddNode - 添加节点的回调函数 (type: string, position: {x, y}) => void
 * @param {Function} options.screenToFlowCoordinate - VueFlow 的坐标转换函数
 * @returns {Object} 拖放相关的状态和方法
 */
export function useDragAndDrop(options = {}) {
    const { draggedType, isDragOver, isDragging } = dragState;
    const { onAddNode= async()=>{}, screenToFlowCoordinate } = options;

    // 如果提供了 onAddNode（画布端），注册它和坐标转换函数
    if (onAddNode) {
        registeredAddNode = onAddNode;
    }
    if (screenToFlowCoordinate) {
        registeredScreenToFlowCoordinate = screenToFlowCoordinate;
    }

    // 监听拖动状态，禁用文本选择
    watch(isDragging, (dragging) => {
        document.body.style.userSelect = dragging ? 'none' : '';
    });

    /**
     * 开始拖动 (从 Toolbox 触发)
     * 这个函数不需要 VueFlow 上下文
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
     * 使用注册的回调和坐标转换函数
     * @param {DragEvent} event
     */
    async function onDrop(event) {
        // 计算放置位置 (屏幕坐标转换为画布坐标)
        let position = {x: 200, y: 200};

        if (registeredScreenToFlowCoordinate) {
            position = registeredScreenToFlowCoordinate({
                x: event.clientX,
                y: event.clientY,
            });
        }

        const type = draggedType.value;

        // 重置状态
        onDragEnd();

        // 使用注册的回调函数添加节点
        if (registeredAddNode && type) {
            await registeredAddNode(type, position);
        } else if (!registeredAddNode) {
            console.warn('[useDragAndDrop] onAddNode callback is not registered. Make sure to call useDragAndDrop({ onAddNode }) in ScriptEditor.vue first.');
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
