<template>
  <div class="onnx-test">
    <div class="control-panel">
      <el-card shadow="hover">
        <template #header>
          <div class="card-header">
            <h3>ONNX推理测试</h3>
            <el-tag type="warning">ONNX Runtime</el-tag>
          </div>
        </template>
        
        <div class="settings-panel">
          <div class="form-item">
            <span class="label">ONNX模型:</span>
            <div class="path-input">
              <el-input v-model="modelPath" placeholder="请输入ONNX模型路径" />
              <el-button @click="browseModel">浏览</el-button>
            </div>
          </div>
          
          <div class="model-info" v-if="modelInfo.loaded">
            <el-descriptions 
              title="模型信息" 
              :column="1" 
              border 
              size="small"
            >
              <el-descriptions-item label="模型名称">
                {{ modelInfo.name || '未知' }}
              </el-descriptions-item>
              <el-descriptions-item label="版本">
                {{ modelInfo.version || '未知' }}
              </el-descriptions-item>
              <el-descriptions-item label="输入节点">
                <div class="node-item" v-for="(node, index) in modelInfo.inputs" :key="'input-'+index">
                  {{ node.name }} ({{ node.shape.join(' × ') }})
                </div>
              </el-descriptions-item>
              <el-descriptions-item label="输出节点">
                <div class="node-item" v-for="(node, index) in modelInfo.outputs" :key="'output-'+index">
                  {{ node.name }} ({{ node.shape.join(' × ') }})
                </div>
              </el-descriptions-item>
            </el-descriptions>
          </div>
          
          <el-divider>推理配置</el-divider>
          
          <div class="form-item">
            <span class="label">运行设备:</span>
            <el-select v-model="executionProvider" placeholder="请选择">
              <el-option label="CPU" value="cpu" />
              <el-option label="CUDA (GPU)" value="cuda" />
              <el-option label="DirectML" value="dml" />
              <el-option label="OpenVINO" value="openvino" />
            </el-select>
          </div>
          
          <div class="form-item">
            <span class="label">线程数:</span>
            <el-input-number v-model="threads" :min="1" :max="32" />
          </div>
          
          <div class="form-item">
            <span class="label">内存优化:</span>
            <el-select v-model="memoryOptimization" placeholder="请选择">
              <el-option label="无" value="none" />
              <el-option label="保守模式" value="conservative" />
              <el-option label="激进模式" value="aggressive" />
            </el-select>
          </div>
          
          <div class="form-item">
            <span class="label">预热次数:</span>
            <el-input-number v-model="warmupRuns" :min="0" :max="10" />
          </div>
        </div>
        
        <div class="button-row">
          <el-button type="primary" @click="loadModel">加载模型</el-button>
          <el-button type="success" @click="runBenchmark" :disabled="!modelInfo.loaded" :loading="isBenchmarking">
            开始基准测试
          </el-button>
        </div>
      </el-card>
    </div>
    
    <div class="result-panel">
      <el-card shadow="hover" class="result-card">
        <template #header>
          <div class="card-header">
            <h3>性能测试结果</h3>
            <div>
              <el-select v-model="chartType" size="small" style="width: 120px">
                <el-option label="柱状图" value="bar" />
                <el-option label="折线图" value="line" />
              </el-select>
            </div>
          </div>
        </template>
        
        <div class="chart-container" v-if="benchmarkResults.length > 0">
          <div ref="chartContainer" class="chart"></div>
        </div>
        
        <el-empty v-else description="未进行基准测试" />
        
        <div v-if="benchmarkResults.length > 0" class="results-data">
          <el-divider>详细结果</el-divider>
          
          <div class="summary-metrics">
            <div class="metric-card">
              <div class="metric-value">{{ averageLatency.toFixed(2) }}ms</div>
              <div class="metric-label">平均推理延迟</div>
            </div>
            <div class="metric-card">
              <div class="metric-value">{{ (1000 / averageLatency).toFixed(1) }}</div>
              <div class="metric-label">FPS</div>
            </div>
            <div class="metric-card">
              <div class="metric-value">{{ minLatency.toFixed(2) }}ms</div>
              <div class="metric-label">最小延迟</div>
            </div>
            <div class="metric-card">
              <div class="metric-value">{{ maxLatency.toFixed(2) }}ms</div>
              <div class="metric-label">最大延迟</div>
            </div>
          </div>
          
          <el-table :data="benchmarkResults" style="width: 100%" height="200" size="small">
            <el-table-column prop="run" label="运行次数" width="100" />
            <el-table-column prop="latency" label="推理延迟 (ms)" width="120">
              <template #default="scope">
                {{ scope.row.latency.toFixed(2) }}
              </template>
            </el-table-column>
            <el-table-column prop="memory" label="内存使用 (MB)" width="120">
              <template #default="scope">
                {{ scope.row.memory.toFixed(1) }}
              </template>
            </el-table-column>
            <el-table-column prop="device" label="运行设备" />
          </el-table>
          
          <div class="button-row export-row">
            <el-button size="small" @click="exportResults">导出结果</el-button>
          </div>
        </div>
      </el-card>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import * as echarts from 'echarts/core';
import { BarChart, LineChart } from 'echarts/charts';
import { 
  TitleComponent, 
  TooltipComponent,
  GridComponent,
  DatasetComponent,
  LegendComponent
} from 'echarts/components';
import { CanvasRenderer } from 'echarts/renderers';

echarts.use([
  BarChart,
  LineChart,
  TitleComponent,
  TooltipComponent,
  GridComponent,
  DatasetComponent,
  LegendComponent,
  CanvasRenderer
]);

// 模型设置
const modelPath = ref('');
const executionProvider = ref('cpu');
const threads = ref(4);
const memoryOptimization = ref('none');
const warmupRuns = ref(3);

// 图表和结果
const chartType = ref('bar');
const chartContainer = ref(null);
const benchmarkResults = ref([]);
const isBenchmarking = ref(false);
let chart = null;

// 模型信息
const modelInfo = ref({
  loaded: false,
  name: '',
  version: '',
  inputs: [],
  outputs: []
});

// 计算平均延迟
const averageLatency = computed(() => {
  if (benchmarkResults.value.length === 0) return 0;
  const sum = benchmarkResults.value.reduce((acc, curr) => acc + curr.latency, 0);
  return sum / benchmarkResults.value.length;
});

// 计算最小延迟
const minLatency = computed(() => {
  if (benchmarkResults.value.length === 0) return 0;
  return Math.min(...benchmarkResults.value.map(r => r.latency));
});

// 计算最大延迟
const maxLatency = computed(() => {
  if (benchmarkResults.value.length === 0) return 0;
  return Math.max(...benchmarkResults.value.map(r => r.latency));
});

// 浏览选择ONNX模型文件
async function browseModel() {
  try {
    // TODO: 实际项目中这里应该从Tauri API调用文件选择器
    // const path = await invoke('select_file', { filter: '*.onnx' });
    // if (path) modelPath.value = path;
    
    // 模拟选择文件
    modelPath.value = 'C:\\models\\yolov8s.onnx';
    ElMessage.info('请选择ONNX模型文件');
  } catch (error) {
    ElMessage.error('选择文件失败：' + error);
  }
}

// 加载模型
async function loadModel() {
  if (!modelPath.value) {
    return ElMessage.warning('请先选择ONNX模型文件');
  }
  
  try {
    ElMessage.info('正在加载模型...');
    
    // TODO: 实际项目中这里应该从Tauri API调用后端
    // const info = await invoke('load_onnx_model', {
    //   modelPath: modelPath.value
    // });
    
    // 模拟加载结果
    setTimeout(() => {
      // 模拟模型信息
      modelInfo.value = {
        loaded: true,
        name: 'YOLOv8s',
        version: '1.0',
        inputs: [
          { name: 'images', shape: [1, 3, 640, 640] }
        ],
        outputs: [
          { name: 'output0', shape: [1, 84, 8400] }
        ]
      };
      
      ElMessage.success('模型加载成功');
    }, 1000);
  } catch (error) {
    ElMessage.error('模型加载失败：' + error);
  }
}

// 运行基准测试
async function runBenchmark() {
  if (!modelInfo.value.loaded) {
    return ElMessage.warning('请先加载模型');
  }
  
  try {
    isBenchmarking.value = true;
    
    // TODO: 实际项目中这里应该从Tauri API调用后端
    // const results = await invoke('run_onnx_benchmark', {
    //   modelPath: modelPath.value,
    //   executionProvider: executionProvider.value,
    //   threads: threads.value,
    //   memoryOptimization: memoryOptimization.value,
    //   warmupRuns: warmupRuns.value,
    //   benchmarkRuns: 20
    // });
    
    // 模拟基准测试结果
    const demoResults = [];
    const runs = 20;
    
    // 模拟基准测试数据
    let baseLatency = executionProvider.value === 'cpu' ? 40 : 10;
    if (executionProvider.value === 'dml') baseLatency = 15;
    if (executionProvider.value === 'openvino') baseLatency = 20;
    
    // 内存基准
    const baseMemory = 500 + Math.random() * 100;
    
    for (let i = 0; i < runs; i++) {
      // 生成带有一些波动的延迟数据
      const latency = baseLatency + (Math.random() - 0.5) * 5;
      // 随着运行次数略微增加的内存使用
      const memory = baseMemory + i * 0.5;
      
      demoResults.push({
        run: i + 1,
        latency: latency,
        memory: memory,
        device: executionProvider.value.toUpperCase()
      });
      
      // 模拟异步更新图表
      if (i % 4 === 0) {
        benchmarkResults.value = [...demoResults];
        updateChart();
        // 给UI线程一些时间更新
        await new Promise(resolve => setTimeout(resolve, 100));
      }
    }
    
    benchmarkResults.value = demoResults;
    updateChart();
    
    isBenchmarking.value = false;
    ElMessage.success('基准测试完成');
  } catch (error) {
    isBenchmarking.value = false;
    ElMessage.error('基准测试失败：' + error);
  }
}

// 导出结果
function exportResults() {
  if (benchmarkResults.value.length === 0) return;
  
  const summary = {
    model: modelPath.value,
    provider: executionProvider.value,
    threads: threads.value,
    memoryOptimization: memoryOptimization.value,
    warmupRuns: warmupRuns.value,
    avgLatency: averageLatency.value,
    minLatency: minLatency.value,
    maxLatency: maxLatency.value,
    fps: 1000 / averageLatency.value,
    results: benchmarkResults.value
  };
  
  const data = JSON.stringify(summary, null, 2);
  const blob = new Blob([data], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  
  const a = document.createElement('a');
  a.href = url;
  a.download = 'onnx_benchmark_results.json';
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
  
  ElMessage.success('结果已导出为JSON文件');
}

// 初始化图表
function initChart() {
  if (chart) {
    chart.dispose();
  }
  
  chart = echarts.init(chartContainer.value);
  updateChart();
}

// 更新图表
function updateChart() {
  if (!chart || !chartContainer.value) return;
  
  const latencyData = benchmarkResults.value.map(item => item.latency);
  const memoryData = benchmarkResults.value.map(item => item.memory);
  const xAxisData = benchmarkResults.value.map(item => `运行 ${item.run}`);
  
  const option = {
    title: {
      text: '模型性能基准测试',
      left: 'center'
    },
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'shadow'
      }
    },
    legend: {
      data: ['推理延迟 (ms)', '内存使用 (MB)'],
      bottom: '0%'
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '10%',
      top: '15%',
      containLabel: true
    },
    xAxis: {
      type: 'category',
      data: xAxisData
    },
    yAxis: [
      {
        type: 'value',
        name: '延迟 (ms)',
        position: 'left',
        axisLine: {
          show: true,
          lineStyle: {
            color: '#5470c6'
          }
        },
        axisLabel: {
          formatter: '{value} ms'
        }
      },
      {
        type: 'value',
        name: '内存 (MB)',
        position: 'right',
        axisLine: {
          show: true,
          lineStyle: {
            color: '#91cc75'
          }
        },
        axisLabel: {
          formatter: '{value} MB'
        }
      }
    ],
    series: [
      {
        name: '推理延迟 (ms)',
        type: chartType.value,
        data: latencyData,
        yAxisIndex: 0,
        itemStyle: {
          color: '#5470c6'
        }
      },
      {
        name: '内存使用 (MB)',
        type: chartType.value,
        data: memoryData,
        yAxisIndex: 1,
        itemStyle: {
          color: '#91cc75'
        }
      }
    ]
  };
  
  chart.setOption(option);
}

// 监听图表类型变化
watch(chartType, () => {
  updateChart();
});

// 监听容器大小
window.addEventListener('resize', () => {
  if (chart) {
    chart.resize();
  }
});

// 组件挂载
onMounted(() => {
  if (chartContainer.value) {
    initChart();
  }
});
</script>

<style lang="scss" scoped>
.onnx-test {
  display: grid;
  grid-template-columns: 1fr 1.5fr;
  gap: 20px;
  height: 100%;
  
  @media (max-width: 1200px) {
    grid-template-columns: 1fr;
  }
}

.control-panel {
  display: flex;
  flex-direction: column;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  
  h3 {
    margin: 0;
    font-size: 16px;
  }
}

.settings-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-bottom: 16px;
}

.form-item {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  
  .label {
    min-width: 90px;
    color: var(--text-color-regular);
  }
  
  .el-select, .el-input, .path-input, .el-input-number {
    flex: 1;
  }
  
  .path-input {
    display: flex;
    gap: 8px;
  }
}

.model-info {
  margin: 10px 0;
  
  :deep(.el-descriptions__title) {
    font-size: 14px;
    color: var(--text-color-primary);
  }
  
  .node-item {
    font-size: 12px;
    font-family: monospace;
    margin-bottom: 4px;
    
    &:last-child {
      margin-bottom: 0;
    }
  }
}

.button-row {
  display: flex;
  justify-content: center;
  flex-wrap: wrap;
  gap: 16px;
  margin-top: 16px;
  
  &.export-row {
    justify-content: flex-end;
    margin-top: 16px;
  }
}

.result-panel {
  display: flex;
  flex-direction: column;
  
  .result-card {
    height: 100%;
    display: flex;
    flex-direction: column;
    
    :deep(.el-card__body) {
      flex: 1;
      display: flex;
      flex-direction: column;
      overflow: hidden;
    }
  }
}

.chart-container {
  height: 300px;
  width: 100%;
  
  .chart {
    width: 100%;
    height: 100%;
  }
}

.results-data {
  margin-top: 16px;
}

.summary-metrics {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 16px;
  margin: 20px 0;
  
  .metric-card {
    background: var(--bg-color-soft);
    padding: 16px;
    border-radius: 8px;
    text-align: center;
    box-shadow: var(--box-shadow-light);
    
    .metric-value {
      font-size: 24px;
      font-weight: bold;
      margin-bottom: 4px;
      background: var(--primary-gradient);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
    }
    
    .metric-label {
      font-size: 14px;
      color: var(--text-color-secondary);
    }
  }
}
</style> 