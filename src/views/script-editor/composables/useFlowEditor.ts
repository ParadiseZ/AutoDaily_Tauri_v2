import { ref } from 'vue';
import type { Ref } from 'vue';
import { useVueFlow } from '@vue-flow/core';
import type { Node, Edge, Connection } from '@vue-flow/core';
import {
    getNodeDefaults,
    NODE_TEMPLATES,
    SOURCE_HANDLE,
    TARGET_HANDLE
} from '../config';

interface FlowOptions {
    addLog?: (message: string, level: any) => void;
    logLevel?: any;
    getUuidV7?: () => Promise<string>;
}

export function useFlowEditor(options: FlowOptions = {}) {
    const { addLog = () => { }, logLevel = {}, getUuidV7 = async () => '' } = options;

    // 节点和边的响应式数据
    const nodes = ref<Node[]>([]) as Ref<Node[]>;
    const edges = ref<Edge[]>([]) as Ref<Edge[]>;

    // 选中的节点
    const selectedNode = ref<Node | null>(null);

    // 删除确认
    const showDeleteConfirm = ref(false);
    const nodesToDelete = ref<Node[]>([]);

    // 获取 VueFlow 的方法
    const {
        onNodeClick,
        removeNodes,
        getSelectedNodes,
        fitView: flowFitView
    } = useVueFlow();

    // 监听节点点击
    onNodeClick((event) => {
        selectedNode.value = event.node;
    });

    // ============================================
    // 节点创建
    // ============================================

    async function createNode(type: string, position: { x: number, y: number }, customData: any = null) {
        const newNode: Node = {
            id: await getUuidV7(),
            type: 'custom',
            label: customData?.label || '',
            position,
            data: customData || getNodeDefaults(type),
        };

        nodes.value.push(newNode);
        addLog(`添加节点: ${type}`, logLevel.INFO);
        return newNode;
    }

    async function addNodeToCanvas(type: string, position: { x: number, y: number } | null = null) {
        if (NODE_TEMPLATES[type]) {
            await expandTemplate(type, position);
            return;
        }

        let finalPosition = position || { x: 200, y: 200 };

        if (!position) {
            if (selectedNode.value) {
                finalPosition = {
                    x: selectedNode.value.position.x,
                    y: selectedNode.value.position.y + 120
                };
            } else if (nodes.value.length > 0) {
                const lastNode = nodes.value[nodes.value.length - 1];
                finalPosition = {
                    x: lastNode.position.x,
                    y: lastNode.position.y + 120
                };
            }
        }

        const newNode = await createNode(type, finalPosition);

        if (selectedNode.value && !position && (selectedNode.value as any).sourceHandle === 'out') {
            const newEdge: Connection = {
                source: selectedNode.value.id,
                target: newNode.id,
                sourceHandle: (selectedNode.value as any).sourceHandle,
                targetHandle: 'in'
            };
            onConnect(newEdge);
            addLog(`自动连接: ${selectedNode.value.id} → ${newNode.id}`, logLevel.INFO);
        }

        selectedNode.value = newNode;
        return newNode;
    }

    // ============================================
    // 连接逻辑
    // ============================================

    function onConnect(params: Connection | any) {
        const canConnect = SOURCE_HANDLE[params.sourceHandle || 'out'] && TARGET_HANDLE[params.targetHandle || 'in'];
        if (!canConnect || (params.source === params.target)) {
            addLog(`不支持的连接：${params.sourceHandle} -> ${params.targetHandle}`, logLevel.ERROR);
            return;
        }

        let info = SOURCE_HANDLE[params.sourceHandle || 'out'];
        if (!info.animated) info = TARGET_HANDLE[params.targetHandle || 'in'];

        const newEdge: Edge = {
            id: `e-${params.source}-${params.sourceHandle || 'out'}-${params.target}-${params.targetHandle || 'in'}`,
            source: params.source,
            target: params.target,
            sourceHandle: params.sourceHandle || 'out',
            targetHandle: params.targetHandle || 'in',
            label: info.label as string,
            animated: info.animated,
        };

        edges.value.push(newEdge);
        if (params.showLog === false) return;
        addLog(`连接: ${newEdge.source} [${newEdge.sourceHandle}] → ${newEdge.target} [${newEdge.targetHandle}]`, logLevel.SUCCESS);
    }

    // ============================================
    // 删除逻辑
    // ============================================

    function requestDeleteSelected() {
        const selected = getSelectedNodes.value;
        const deletable = selected.filter(n => n.data?.type !== 'start' && n.data?.type !== 'end');
        if (deletable.length > 0) {
            nodesToDelete.value = deletable;
            showDeleteConfirm.value = true;
        }
    }

    function confirmDelete() {
        if (nodesToDelete.value.length > 0) {
            removeNodes(nodesToDelete.value);
            addLog(`删除 ${nodesToDelete.value.length} 个节点`, logLevel.WARN);
            selectedNode.value = null;
            showDeleteConfirm.value = false;
            nodesToDelete.value = [];
        }
    }

    function cancelDelete() {
        showDeleteConfirm.value = false;
        nodesToDelete.value = [];
    }

    // ============================================
    // 模板展开
    // ============================================

    async function expandTemplate(templateKey: string, basePosition: { x: number, y: number } | null = null) {
        const template = NODE_TEMPLATES[templateKey];
        if (!template) return;

        const pos = basePosition || { x: 200, y: 200 };
        const createdNodes: Node[] = [];

        for (const nodeSpec of template.nodes) {
            const nodePos = {
                x: pos.x + nodeSpec.position.x,
                y: pos.y + nodeSpec.position.y,
            };
            const node = await createNode(nodeSpec.type, nodePos, {
                ...getNodeDefaults(nodeSpec.type),
                label: nodeSpec.label
            });
            createdNodes.push(node);
        }

        template.edges.forEach((edgeSpec: any) => {
            const sourceNode = createdNodes[edgeSpec.sourceIdx];
            const targetNode = createdNodes[edgeSpec.targetIdx];

            if (sourceNode && targetNode) {
                const sourceHandle = edgeSpec.handle || 'out';
                const targetHandle = edgeSpec.targetHandle || 'in';

                onConnect({
                    source: sourceNode.id,
                    target: targetNode.id,
                    sourceHandle,
                    targetHandle,
                    showLog: false
                });
            }
        });

        addLog(`展开模板: ${templateKey}`, logLevel.SUCCESS);
    }

    // ============================================
    // 其他操作
    // ============================================

    function updateNodeData(nodeId: string, updates: any) {
        const node = nodes.value.find(n => n.id === nodeId);
        if (node) {
            Object.assign(node.data, updates);
            if (updates.label !== undefined) {
                node.label = updates.label;
            }
        }
    }

    function onPaneClick() {
        selectedNode.value = null;
    }

    function fitView() {
        flowFitView({ padding: 0.2 }).then();
    }

    return {
        // 状态
        nodes,
        edges,
        selectedNode,
        showDeleteConfirm,
        nodesToDelete,

        // 节点操作
        addNodeToCanvas,
        updateNodeData,

        // vue-flow 内容
        onNodeClick,
        removeNodes,
        getSelectedNodes,
        onPaneClick,
        fitView,

        // 连接
        onConnect,

        // 删除
        requestDeleteSelected,
        confirmDelete,
        cancelDelete,
    };
}

export default useFlowEditor;
