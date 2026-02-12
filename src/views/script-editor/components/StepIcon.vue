<template>
  <component :is="icon" :class="className" />
</template>

<script setup lang="ts">
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

const props = defineProps<{
  type?: string;
  category?: string;
  className?: string;
}>();

const icon = computed(() => {
  const typeMap: Record<string, any> = {
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

  const lowerType = props.type?.toLowerCase() || '';
  const legacyMap: Record<string, any> = {
    click: MousePointer2,
    swipe: Move,
    wait: Clock,
  };

  return (props.type ? typeMap[props.type] : null) || legacyMap[lowerType] || Box;
});
</script>
