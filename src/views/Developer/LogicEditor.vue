<script setup>
import { onMounted, ref, onBeforeUnmount } from 'vue';
import { Graph, Shape } from '@antv/x6';

const container = ref(null);
const graph = ref(null);

onMounted(() => {
  graph.value = new Graph({
    container: container.value,
    grid: true,
    background: {
      color: '#f8f9fa', // Light background for contrast, or use variable for dark mode
    },
    mousewheel: {
      enabled: true,
      zoomAtMousePosition: true,
      modifiers: 'ctrl',
      minScale: 0.5,
      maxScale: 3,
    },
    connecting: {
      router: 'manhattan',
      connector: {
        name: 'rounded',
        args: {
          radius: 8,
        },
      },
      anchor: 'center',
      connectionPoint: 'anchor',
      allowBlank: false,
      snap: {
        radius: 20,
      },
      createEdge() {
        return new Shape.Edge({
          attrs: {
            line: {
              stroke: '#A2B1C3',
              strokeWidth: 2,
              targetMarker: {
                name: 'block',
                width: 12,
                height: 8,
              },
            },
          },
          zIndex: 0,
        });
      },
      validateConnection({ targetMagnet }) {
        return !!targetMagnet;
      },
    },
    highlighting: {
      magnetAdsorbed: {
        name: 'stroke',
        args: {
          attrs: {
            fill: '#5F95FF',
            stroke: '#5F95FF',
          },
        },
      },
    },
  });

  // Add some demo nodes
  const start = graph.value.addNode({
    x: 80,
    y: 80,
    width: 100,
    height: 40,
    label: 'Start',
    attrs: {
      body: {
        fill: '#eff4ff',
        stroke: '#5F95FF',
        rx: 20,
        ry: 20,
      },
      label: {
        fill: '#5F95FF',
        fontSize: 14,
        fontWeight: 'bold',
      },
    },
    ports: {
      groups: {
        out: {
          position: 'right',
          attrs: {
            circle: {
              r: 4,
              magnet: true,
              stroke: '#5F95FF',
              strokeWidth: 1,
              fill: '#fff',
              style: {
                visibility: 'hidden',
              },
            },
          },
        },
      },
      items: [{ id: 'port1', group: 'out' }],
    },
  });

  const process = graph.value.addNode({
    x: 240,
    y: 80,
    width: 100,
    height: 40,
    label: 'Process',
    attrs: {
      body: {
        fill: '#fff',
        stroke: '#5F95FF',
        rx: 6,
        ry: 6,
      },
      label: {
        fill: '#333',
        fontSize: 14,
      },
    },
    ports: {
      groups: {
        in: { position: 'left' },
        out: { position: 'right' },
      },
      items: [
        { id: 'in', group: 'in' },
        { id: 'out', group: 'out' },
      ],
    },
  });

  graph.value.addEdge({
    source: { cell: start, port: 'port1' },
    target: { cell: process, port: 'in' },
  });
  
  graph.value.centerContent();
});

onBeforeUnmount(() => {
  if (graph.value) {
    graph.value.dispose();
  }
});
</script>

<template>
  <div class="h-full flex flex-col">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-xl font-bold">Logic Editor</h2>
      <div class="flex gap-2">
        <button class="btn btn-sm btn-outline">Save</button>
        <button class="btn btn-sm btn-primary">Run</button>
      </div>
    </div>
    
    <div class="flex-1 border border-base-300 rounded-xl overflow-hidden shadow-inner bg-base-100 relative">
      <div ref="container" class="w-full h-full"></div>
      
      <!-- Floating Palette (Mock) -->
      <div class="absolute top-4 left-4 bg-base-100/90 backdrop-blur shadow-lg rounded-lg p-2 border border-base-200 flex flex-col gap-2">
        <div class="w-10 h-10 rounded bg-blue-100 border border-blue-500 flex items-center justify-center cursor-move" title="Start Node">
          <div class="w-6 h-6 rounded-full border-2 border-blue-500"></div>
        </div>
        <div class="w-10 h-10 rounded bg-white border border-gray-400 flex items-center justify-center cursor-move" title="Process Node">
          <div class="w-6 h-4 border border-gray-500 rounded-sm"></div>
        </div>
      </div>
    </div>
  </div>
</template>
