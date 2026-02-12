import { ref, watch } from 'vue';
import type { Ref } from 'vue';

const dragState = {
    draggedType: ref<string | null>(null),
    isDragOver: ref(false),
    isDragging: ref(false),
};

let registeredAddNode: ((type: string, position: { x: number, y: number }) => Promise<any>) | null = null;
let registeredScreenToFlowCoordinate: ((position: { x: number, y: number }) => { x: number, y: number }) | null = null;

interface DragOptions {
    onAddNode?: (type: string, position: { x: number, y: number }) => Promise<any>;
    screenToFlowCoordinate?: (position: { x: number, y: number }) => { x: number, y: number };
}

export function useDragAndDrop(options: DragOptions = {}) {
    const { draggedType, isDragOver, isDragging } = dragState;
    const { onAddNode, screenToFlowCoordinate } = options;

    if (onAddNode) {
        registeredAddNode = onAddNode;
    }
    if (screenToFlowCoordinate) {
        registeredScreenToFlowCoordinate = screenToFlowCoordinate;
    }

    watch(isDragging, (dragging) => {
        document.body.style.userSelect = dragging ? 'none' : '';
    });

    function onDragStart(event: DragEvent, type: string) {
        if (event.dataTransfer) {
            event.dataTransfer.setData('application/vueflow', type);
            event.dataTransfer.effectAllowed = 'move';
        }
        draggedType.value = type;
        isDragging.value = true;
        document.addEventListener('drop', onDragEnd);
    }

    function onDragOver(event: DragEvent) {
        event.preventDefault();
        if (draggedType.value) {
            isDragOver.value = true;
            if (event.dataTransfer) {
                event.dataTransfer.dropEffect = 'move';
            }
        }
    }

    function onDragLeave() {
        isDragOver.value = false;
    }

    function onDragEnd() {
        isDragging.value = false;
        isDragOver.value = false;
        draggedType.value = null;
        document.removeEventListener('drop', onDragEnd);
    }

    async function onDrop(event: DragEvent) {
        let position = { x: 200, y: 200 };
        if (registeredScreenToFlowCoordinate) {
            position = registeredScreenToFlowCoordinate({
                x: event.clientX,
                y: event.clientY,
            });
        }
        const type = draggedType.value;
        onDragEnd();
        if (registeredAddNode && type) {
            await registeredAddNode(type, position);
        }
    }

    return {
        draggedType,
        isDragOver,
        isDragging,
        onDragStart,
        onDragOver,
        onDragLeave,
        onDrop,
    };
}

export default useDragAndDrop;
