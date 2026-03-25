# AutoDaily 前端 UI/UX 设计规范

本文档记录了 AutoDaily 原生桌面端的前端设计重点与工程化规范，以确保后续开发在视觉、交互与主题更换上保持高度一致。

## 1. 视觉基调 (Vibrant Apple Style)
- **无框化与呼吸感 (Frameless & Airy)**：抛弃了传统的强边框与厚重卡片，采用大面积留白与半透明的高斯模糊（`backdrop-blur-2xl`）来构建界面，使其具有 macOS/iOS 原生级别的融入感。
- **秩序与克制**：在数据展示（如设备状态、日志排版）上保持极其克制的排版，利用字重（`font-medium`, `font-semibold`）和灰度（如 `text-base-content/60`）来建立信息层级，而不是滥用颜色。
- **活力与张力 (Vibrant Accent)**：在用户交互频繁的元素（如正在运行的设备呼吸灯、聚焦的按钮、当前选中的菜单栏、主操作区）上，使用**高饱和度、带轻微发光阴影**的色彩注入活力。

## 2. 语义化色彩规范 (Semantic Colors)
**【极度重要】**：严禁在组件中硬编码具体的 Tailwind 物理颜色（如 `bg-blue-500`, `text-green-600`）。所有与主题相关的颜色必须使用 DaisyUI 提供的语义化工具类。

- **主色调 (`primary`)**：代表核心操作、当前激活的状态、强调色（对应 iOS 系统的主题蓝）。
  - 使用场景：侧边栏选中态、全局主按钮、当前在线/活跃设备的强调等。
  - 用法示例：`bg-primary text-primary-content`，发光阴影 `shadow-primary/20`。
- **次色调 (`secondary`)**：代表辅助强调、或需要构建多色阶渐变时的第二种颜色（对应高活力的紫/粉色）。
  - 使用场景：头像渐变背景、大型标志位渐变 `from-primary to-secondary`。
- **成功/运行状态 (`success`)**：代表正常运行、设备在线、执行成功（对应鲜亮的绿色）。
  - 使用场景：设备“在线”呼吸灯节点、成功状态文字。
- **错误/警告状态 (`error` / `warning`)**：代表设备掉线、任务执行失败。
- **底层与内容色 (`base-100`, `base-content`)**：
  - `bg-base-100`：主内容区域背景。
  - `bg-base-200` 或 `bg-base-300`：用于次级面板、悬浮时的背景色加深。
  - `text-base-content`：主文本色，利用 `/80`, `/60`, `/40` 等控制透明度拉开层级。

## 3. 交互与反演效果
- **悬浮 (Hover)**：为了确保操作有及时的反馈，必须给可交互元素添加明显的 Hover 样式（如 `hover:bg-base-200/80` 或 `hover:bg-primary/90`），配合 `transition-colors duration-150`。
- **缩放 (Scale)**：对于图标或者设备块等卡片，增加 `group-hover:scale-110 transition-transform` 来提供生动的微动效。
- **路由切换过渡**：在 `router-view` 挂载微弱的 `fade` 或 `slide` 动画，模拟原生 App 切换页面时的平滑感。

## 4. 主题更换支持
基于上述的语义化规范，应用通过动态改变 `style.css` 注入到底层的 `oklch(var(--p))` 变量，或在 `<html data-theme="...">` 标签中切换 DaisyUI 的主题，即可在一瞬间全盘改变应用基调（无论是深色模式、浅色模式，还是定制色）且毫无违和感。
