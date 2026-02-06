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
  List,
  RotateCcw,
  Search,
  LayoutGrid,
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
  // Mapping by exact PascalCase OPs from Rust
  const typeMap = {
    // Interaction
    ClickAction: MousePointer2,
    SwipePoint: Move,
    SwipePercent: Move,
    WaitMs: Clock,

    // Vision
    VisionSearch: Zap,
    Ocr: Type,
    FindObject: Box,
    DetRec: Target,
    TakeScreenshot: Camera,

    // Control
    If: Split,
    While: Repeat,
    Sequence: LayoutGrid,
    ForEachActivity: List,
    Continue: SkipForward,
    Break: StopCircle,
    WaitUntil: Clock,

    // Logic
    SetVar: Variable,
    GetVar: Terminal,
    FilterHits: Filter,
    IncIndex: ArrowRight,
    ResetIndex: RotateCcw,

    // State
    SetState: Settings,
    GetState: Info,
    StopPolicy: XCircle,
    FinishTask: CheckCircle2,
  };

  // Case-insensitive fallback for legacy or simplified calls
  const lowerType = props.type?.toLowerCase();
  const legacyMap = {
    click: MousePointer2,
    swipe: Move,
    wait: Clock,
  };

  return typeMap[props.type] || legacyMap[lowerType] || Box;
});
</script>
