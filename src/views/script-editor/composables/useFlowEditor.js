/**
 * Flow Editor Composable
 * 
 * 核心节点/边操作逻辑
 * - 创建节点
 * - 添加节点到画布
 * - 连接节点
 * - 删除节点
 * - 模板展开
 */

import { ref } from 'vue';
import { useVueFlow } from '@vue-flow/core';
import {
    getNodeDefaults,
    NODE_TEMPLATES,
    SOURCE_HANDLE,
    TARGET_HANDLE
} from '../config.js';

/**
 * Flow Editor Composable
 * 
 * @param {Object} options - 配置选项
 * @param {Function} options.addLog - 日志函数
 * @returns {Object} 节点编辑相关的状态和方法
 */
export function useFlowEditor(options = {}) {
    const { addLog = () => { },logLevel = {} } = options;

    // 节点和边的响应式数据
    const nodes = ref([]);
    const edges = ref([]);

    // 选中的节点
    const selectedNode = ref(null);

    // 删除确认
    const showDeleteConfirm = ref(false);
    const nodesToDelete = ref([]);

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

    /**
     * 生成唯一节点ID
     * @returns {string}
     */
    function generateNodeId() {
        return `node-${Date.now()}-${Math.floor(Math.random() * 1000)}`;
    }

    /**
     * 创建新节点并添加到画布
     * @param {string} type - 节点类型
     * @param {Object} position - 位置 {x, y}
     * @param {Object} customData - 自定义数据 (可选)
     * @returns {Object} 创建的节点
     */
    function createNode(type, position, customData = null) {
        const newNode = {
            id: generateNodeId(),
            type: 'custom', // 使用自定义渲染器
            label: customData?.label || '',
            position,
            data: customData || getNodeDefaults(type),
        };

        nodes.value.push(newNode);
        addLog(`添加节点: ${type}`, logLevel.INFO);
        return newNode;
    }

    /**
     * 添加节点到画布 (用于工具箱点击和拖放)
     * 如果有选中节点，新节点会添加到选中节点下方并自动连接
     * 
     * @param {string} type - 节点类型
     * @param {Object} position - 可选的位置，如果不提供则自动计算
     */
    function addNodeToCanvas(type, position = null) {
        // 检查是否为模板
        if (NODE_TEMPLATES[type]) {
            expandTemplate(type, position);
            return;
        }

        // 计算位置
        let finalPosition = position || { x: 200, y: 200 };

        if (!position) {
            if (selectedNode.value) {
                // 添加到选中节点下方
                finalPosition = {
                    x: selectedNode.value.position.x,
                    y: selectedNode.value.position.y + 120
                };
            } else if (nodes.value.length > 0) {
                // 添加到最后一个节点下方
                const lastNode = nodes.value[nodes.value.length - 1];
                finalPosition = {
                    x: lastNode.position.x,
                    y: lastNode.position.y + 120
                };
            }
        }

        const newNode = createNode(type, finalPosition);

        // 如果有选中节点，自动连接
        if (selectedNode.value && !position && selectedNode.value.sourceHandle==='out') {

            const newEdge = {
                source: selectedNode.value.id,
                target: newNode.id,
                sourceHandle: selectedNode.value.sourceHandle,
                targetHandle: TARGET_HANDLE['in']
            };
            onConnect(newEdge)
            addLog(`自动连接: ${selectedNode.value.id} → ${newNode.id}`, logLevel.INFO);
        }

        // 选中新节点
        selectedNode.value = newNode;

        return newNode;
    }

    // ============================================
    // 连接逻辑
    // ============================================

    /**
     * 处理节点连接
     * @param {Object} params - 连接参数
     */
    function onConnect(params) {
        const canConnect = SOURCE_HANDLE[params.sourceHandle] && TARGET_HANDLE[params.targetHandle];
        if (!canConnect || (params.source === params.target)) {
            addLog(`不支持的连接：${params.sourceHandle} -> ${params.targetHandle}`, logLevel.ERROR);
            return;
        }

        let info = SOURCE_HANDLE[params.sourceHandle];
        if (!info.animated) info = TARGET_HANDLE[params.targetHandle];

        const newEdge = {
            id: `e-${params.source}-${params.sourceHandle || 'out'}-${params.target}-${params.targetHandle || 'in'}`,
            source: params.source,
            target: params.target,
            sourceHandle: params.sourceHandle,
            targetHandle: params.targetHandle,
            label: info.label,
            animated: info.animated,
        };

        edges.value.push(newEdge);
        if(params.showLog === false) return;
        addLog(`连接: ${newEdge.source} [${newEdge.sourceHandle}] → ${newEdge.target} [${newEdge.targetHandle}]`, logLevel.SUCCESS);
    }

    // ============================================
    // 删除逻辑
    // ============================================

    /**
     * 请求删除选中的节点
     */
    function requestDeleteSelected() {
        const selected = getSelectedNodes.value;
        // 过滤掉 start 和 end 节点
        const deletable = selected.filter(n => n.data?.type !== 'start' && n.data?.type !== 'end');
        if (deletable.length > 0) {
            nodesToDelete.value = deletable;
            showDeleteConfirm.value = true;
        }
    }

    /**
     * 确认删除
     */
    function confirmDelete() {
        removeNodes(nodesToDelete.value);
        addLog(`删除 ${nodesToDelete.value.length} 个节点`, logLevel.WARN);
        selectedNode.value = null;
        showDeleteConfirm.value = false;
        nodesToDelete.value = [];
    }

    /**
     * 取消删除
     */
    function cancelDelete() {
        showDeleteConfirm.value = false;
        nodesToDelete.value = [];
    }

    // ============================================
    // 模板展开
    // ============================================

    /**
     * 展开模板
     * @param {string} templateKey - 模板键
     * @param {Object} basePosition - 基础位置
     */
    function expandTemplate(templateKey, basePosition = null) {
        const template = NODE_TEMPLATES[templateKey];
        if (!template) return;

        const pos = basePosition || { x: 200, y: 200 };
        const createdNodes = [];

        // 1. 创建节点
        template.nodes.forEach((nodeSpec) => {
            const nodePos = {
                x: pos.x + nodeSpec.position.x,
                y: pos.y + nodeSpec.position.y,
            };
            const node = createNode(nodeSpec.type, nodePos, {
                ...getNodeDefaults(nodeSpec.type),
                label: nodeSpec.label
            });
            createdNodes.push(node);
        });

        // 2. 创建边
        template.edges.forEach((edgeSpec) => {
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

    /**
     * 更新节点数据
     * @param {string} nodeId 
     * @param {Object} updates 
     */
    function updateNodeData(nodeId, updates) {
        const node = nodes.value.find(n => n.id === nodeId);
        if (node) {
            Object.assign(node.data, updates);
            if (updates.label !== undefined) {
                node.label = updates.label;
            }
        }
    }

    /**
     * 点击画布空白处
     */
    function onPaneClick() {
        selectedNode.value = null;
    }

    /**
     * 适应视图
     */
    function fitView() {
        flowFitView({padding: 0.2}).then();
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

        //vue-flow 内容
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