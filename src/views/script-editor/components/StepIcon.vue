<template>
  <component :is="icon" :class="className" />
</template>

<script setup>
import { computed } from 'vue';
import {
  MousePointer2,
  Move,
  Clock,
  Zap,
  Camera,
  Target,
  Type,
  Repeat,
  AlertTriangle,
  GitBranch,
  Play,
  Square,
  Box,
  Variable,
  Filter,
  Split,
  Info,
  Settings,
  Code,
  Terminal,
  ArrowRight,
  Pause,
  SkipForward,
  StopCircle,
  CheckCircle2,
  XCircle,
} from 'lucide-vue-next';

const props = defineProps({
  type: String,
  category: String,
  className: {
    type: String,
    default: 'w-4 h-4',
  },
});

const icon = computed(() => {
  // Mapping by type
  const typeMap = {
    // Interaction
    click: MousePointer2,
    swipe: Move,
    wait: Clock,

    // Vision
    capture: Camera,
    detect: Target,
    ocr: Type,
    vision_search: Zap,
    find_object: Box,
    det_rec: Target,

    // Control
    sequence: ListTodo, // fallback to Box if not listed
    if: Split,
    while: Repeat,
    for_each: Repeat,
    continue: SkipForward,
    break: StopCircle,
    wait_until: Clock,

    // Logic
    set_var: Variable,
    get_var: Variable,
    filter_hits: Filter,

    // State/Policy
    stop_policy: XCircle,
    finish_task: CheckCircle2,
    set_state: Settings,
    get_state: Info,
    inc_index: ArrowRight,
    reset_index: RotateCcw,
  };

  // Mapping by category if type not found
  const categoryMap = {
    interaction: MousePointer2,
    vision: Zap,
    control: GitBranch,
    logic: Code,
    state: Settings,
  };

  return typeMap[props.type] || categoryMap[props.category] || Box;
});
</script>
