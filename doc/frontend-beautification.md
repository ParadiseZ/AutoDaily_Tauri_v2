# AutoDaily 前端美化与调整记录 (Frontend Beautification)

本项目旨在通过分析各组件，提取出能够用 图标（Icon） 或 SVG 辅助展示的按钮或具有标志性意义的内容块，使用充满活力的色彩设计对其进行替换式美化。

## 1. Icon / SVG 使用原则与规划清单

- **选型原则**：常规通用图标（如设置、删除、播放）使用已安装的 `lucide-vue-next` 库。如果遇到特殊功能或更具表现力的地方，将直接手绘自定义的高清 SVG 矢量图形。
- **目标组件清单**：

### 1.1 核心布局与导航
- **`Layout/MainLayout.vue`**
  - 左侧侧边栏导航图标 (Dashboard, Devices, Scripts, Market, Settings)
  - 顶部操作栏图标

### 1.2 设备管理 (Device Management)
- **`DeviceList.vue` / `DeviceEditorDialog.vue`**
  - 设备状态指示灯/图标 (Online, Offline, Busy)
  - 操作按钮图标 (Edit, Delete, Add Device, Connect)

### 1.3 任务管理 (Task Management)
- **`TaskManagement.vue` / `TaskDevicePanel.vue`**
  - 任务运行状态图标 (Play, Pause, Stop, Loading)
  - 设备任务详情展开/折叠图标

### 1.4 脚本列表与详情 (Script List)
- **`ScriptListSidebar.vue`**
  - 列表项的类型前缀图标
- **`ScriptDetailPanel.vue` / `ScriptInfoDialog.vue`**
  - 详细信息栏目的描述性图标 (Author, Version, Logs, Setup)
  - 按钮组图标 (Run, Edit, Delete)

### 1.5 脚本编辑器 (Script Editor)
- **`EditorTaskSidebar.vue` / `EditorModeRail.vue` / etc.**
  - 节点类型对应的图标
  - 拖拽手柄、折叠/展开逻辑控制图标
  - 顶部工具栏 (Save, Test, Back)

### 1.6 设置与杂项 (Settings & Misc)
- **`Settings.vue`**
  - 配置组图标 (General, Theme, ADB, Profile)
- **`About.vue` / `Logs.vue`**
  - 日志级别图标 (Error, Warn, Info)
  - 外部链接图标

## 2. 配色与活力设计

为保持 Apple 质感的干净基调，同时增加界面的呼吸感和活力：

- **基础图标色**：随文字颜色变化（`currentColor` 或 `var(--app-text-soft)`）。
- **活力强调色（用于图标状态和特征装饰）**：
  - **Dynamic Blue (动态蓝)**：用于主要操作、进行中的状态。
  - **Lively Emerald (鲜活绿)**：用于成功、在线、完成状态。
  - **Energetic Amber (活力琥珀)**：用于警告、忙碌状态、重要提示。
  - **Vibrant Rose (灵动玫瑰/品红)**：用于错误、离线、危险操作。
  - **Electric Purple (电音紫)**：用于脚本、模型等有技术感/智能感的特殊节点。
- **动效配合**：保留克制的过渡时间（`0.15s` - `0.2s`），在按钮 `hover` 和 `active` 状态时增加轻微的缩放（`transform: scale(0.96)`）或颜色焕发。

## 3. Icon 统一管理策略

- 采用 **`lucide-vue-next`** 作为主要实现方式。
- **单独封装外壳组件 (`AppIcon.vue`)**：为了统一管理统一控制尺寸、呼吸颜色特效。将存放在 `src/components/shared/AppIcon.vue`。
- Icon 配色和排版样式在 `src/style.css` 中增加一套 `.app-icon-*` 的工具类。

## 4. 排版调整说明 (Layout Enhancements)

目前发现以下几类常见的由于肉眼不协调或缺乏美感引起的排版问题，将进行同步修正：
1. **拥挤与留白不足**：面板内部元素距离边框过近。调整 `padding` 和 `gap`，增加呼吸感。
2. **对齐不当**：文字与图标未垂直居中 (需要使用 `flex items-center`)。
3. **视觉层级扁平**：次要信息（如时间、描述）不够收敛。需调整字号和颜色。
4. **组件包裹**：表单和按钮区域未做明确区分。
