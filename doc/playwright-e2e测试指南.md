# AutoDaily Playwright E2E 测试指南

本文档整理当前仓库里的 Playwright E2E 测试实现方式、浏览器 mock 方案、selector 约定，以及后续新增测试时的推荐写法。

目标不是覆盖所有业务，而是先把“前端页面主链路”稳定测起来，尤其是本地脚本相关流程。

## 1. 当前测试范围

当前已经落地的业务测试：

- `tests/script-create.spec.ts`
  - 校验“脚本名称为空时不可提交”
  - 校验“新建脚本”时可完整保存：
    - 基本信息
    - 模型信息
    - 赞助信息

当前已验证通过的执行命令：

```bash
npm run type-check
npx playwright test tests/script-create.spec.ts --project=chromium
```

说明：

- 当前重点先放在 `chromium`
- `firefox` / `webkit` 项目仍保留在 `playwright.config.ts` 中，但本轮只实际验证了 `chromium`

## 2. 运行方式

### 2.1 Playwright 配置

`playwright.config.ts` 已配置：

- `baseURL: http://127.0.0.1:4173`
- `webServer.command: npm run dev -- --host 127.0.0.1 --port 4173`

因此运行测试时会自动启动前端开发服务器，不需要手动先开一个 Vite 窗口。

### 2.2 常用命令

跑单个文件：

```bash
npx playwright test tests/script-create.spec.ts --project=chromium
```

打开 Playwright UI：

```bash
npm run test:e2e:ui
```

首次缺少浏览器时：

```bash
npx playwright install chromium
```

## 3. 为什么当前可以直接跑 Web 版 E2E

项目本身是 `Vue + Tauri`，但当前 E2E 没有直接拉起真实 Tauri 桌面壳，而是走浏览器模式。

为此，前端入口 `src/main.ts` 会加载：

- `src/mockTauri.ts`

这个文件现在不再只是“占位假实现”，而是一个可运行的浏览器 mock 后端，基于官方：

- `@tauri-apps/api/mocks`

它负责兜住浏览器模式下页面初始化和测试用到的调用，包括：

- store 读写
- 设备/任务初始化
- 脚本列表读取
- 保存脚本
- 删除脚本
- UUID 生成
- 事件监听 mock

mock 状态会存到浏览器 `localStorage`：

- key: `autodaily.mock.state`

同时它会在 `window` 上暴露测试辅助对象：

```ts
window.__AUTODAILY_MOCK__
```

当前可用能力：

- `getState()`
- `reset()`
- `seed(partial)`

测试里通常在 `beforeEach` 先调用：

```ts
window.__AUTODAILY_MOCK__.reset()
```

然后 `page.reload()`，确保每个用例都是干净状态。

## 4. 类型约定

测试里涉及结构断言时，优先从真实绑定类型导入，不要自己手写一套“差不多”的接口。

当前约定来源：

```ts
import type { ScriptTable } from '../src/types/bindings';
```

注意：

- 真实绑定里的 `ScriptTable.data.verNum / latestVer / downloadCount` 是 `bigint`
- 但浏览器端 mock 里存的是 JSON 可序列化值，所以当前测试里会基于 `ScriptTable` 再派生一个“可 JSON 存储”的局部类型
- 这种派生是允许的，但必须以 `bindings` 类型为基础，而不是完全独立定义

## 5. Selector 约定

### 5.1 不要主要依赖页面文案

如果测试大部分依赖：

- 中文按钮文案
- 表单标签文字
- 提示语

那么页面 copy 一改，测试就会一起碎。

因此当前约定是：

- 页面语义断言可以适度使用 `role`
- 关键交互控件优先使用 `data-testid`

例如当前已经补好的节点：

- `script-list-create-button`
- `script-dialog-tab-basic`
- `script-dialog-tab-models`
- `script-dialog-tab-support`
- `script-basic-name`
- `script-basic-runtime-type`
- `script-models-img-det-kind`
- `script-models-txt-rec-kind`
- `script-support-sponsorship-qr-input`
- `script-submit`

### 5.2 自定义下拉统一走 test id

项目里的 `AppSelect` 不是原生 `select`，而是自绘弹层组件。

当前统一支持：

- 触发按钮：`data-testid="<testId>"`
- 选项弹层：`data-testid="<testId>-menu"`
- 选项按钮：`data-testid="<testId>-option-<value>"`

示例：

```ts
const selectOptionByValue = async (page: Page, testId: string, value: string) => {
  await page.getByTestId(testId).click();
  await page.getByTestId(`${testId}-option-${value}`).click();
};
```

这样就不需要依赖“JavaScript”“Paddle CRNN”这些显示文字本身。

## 6. 当前“新建脚本”测试覆盖了什么

`tests/script-create.spec.ts` 的完整创建用例现在会验证：

### 6.1 基本信息

- 脚本名称
- 描述
- 运行时
- 包名
- 版本名称
- 版本号
- 允许克隆

### 6.2 模型信息

- 图像检测模型
  - 选择 `Yolo11`
  - 设置模型来源、推理后端、模型路径
  - 设置类别数量、标签路径、阈值
- 文本检测模型
  - 选择 `PaddleDbNet`
  - 设置模型来源、模型路径
  - 设置阈值、扩张比例、膨胀开关
- 文本识别模型
  - 选择 `PaddleCrnn`
  - 设置模型来源、模型路径、字典路径

### 6.3 赞助信息

- 联系方式
- 赞助链接
- 赞助二维码地址

当前赞助二维码测试使用 `data URL`，原因是：

- 不需要真实文件选择器
- 不依赖本地图片路径
- 不会额外触发 `convert_img_to_base64_cmd`

## 7. 为什么断言优先看“保存后的结构”

这类表单测试如果只断言“页面上看起来有字”，保护能力不够。

例如：

- 看到了标题，不代表真正保存进了结构体
- 看到了预览，不代表 `imgDetModel` / `txtRecModel` 真按预期写入

因此当前策略是：

1. 用 UI 走完整条业务链路
2. 用 `window.__AUTODAILY_MOCK__.getState()` 读取最终保存结果
3. 对结构体字段做断言

这能同时覆盖：

- 表单交互
- 提交逻辑
- 序列化结果

## 8. 后续新增测试怎么写

推荐模板：

1. 在 `tests/` 下新建 `*.spec.ts`
2. `beforeEach` 中：
   - `page.goto(...)`
   - `window.__AUTODAILY_MOCK__.reset()`
   - `page.reload()`
3. 优先通过 `data-testid` 操作关键控件
4. 最终优先断言 mock 状态，不要只看文案

建议优先补这些流程：

- 编辑脚本信息
- 删除脚本
- 克隆脚本
- 脚本搜索过滤
- 任务页给设备追加脚本

## 9. 如果后面要测更多 Tauri 相关流程

当前 mock 只覆盖了“脚本创建”所需的最小可运行集合。

如果后面要补更多业务，请按新增测试需要扩展 `src/mockTauri.ts`，而不是在测试里到处临时 patch。

优先扩的方向：

- 任务 assignment 保存与删除
- 设备列表 CRUD
- 市场脚本搜索 / 下载
- 图片转 base64
- 登录态和 profile mock

原则：

- mock 的能力跟着业务测试需求增长
- 不要先写一套过度庞大的假后端

## 10. 结论

当前仓库里的 Playwright E2E 已经形成了一个明确约定：

- 浏览器模式跑测试
- 用 `src/mockTauri.ts` 兜住 Tauri 能力
- 类型以 `src/types/bindings` 为准
- 交互优先用 `data-testid`
- 结果优先断言保存后的结构体

后续只要沿着这套方式补测试，维护成本会明显低于“全靠页面文案定位”的写法。
