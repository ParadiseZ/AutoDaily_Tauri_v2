# Rhai 步骤函数手册

这份文档给在脚本编辑器里编写 Rhai 代码的开发者用户使用。

目标很简单：

- 你只需要记“能力函数”。
- 不需要了解内部步骤 JSON。
- 任务、策略、策略组、策略集这类对象，优先传名称，不传内部 id。

## 1. 什么时候该直接写 Rhai，什么时候该调用函数

直接写 Rhai 的场景：

- 变量读写
- 条件分支
- 循环
- 普通字符串 / 数字 / 布尔 /数组处理

示例：

```rhai
runtime.retry = (runtime.retry ?? 0) + 1;

if runtime.retry > 3 {
  stop_script();
}
```

调用内建函数的场景：

- 设备动作
- 显式等待
- 任务跳转 / 停止脚本
- 任务状态 / 策略状态修改
- 视觉能力
- 策略绑定与策略执行

## 2. 参数设计规则

函数按“最小必填参数”和“扩展参数”拆成多个版本。

也就是说，优先写最短能跑通的版本：

```rhai
click(point(120, 320));
swipe(point(100, 300), point(600, 300));
launch_app("com.demo.app");
```

需要更细控制时，再用扩展版本：

```rhai
swipe(point(100, 300), point(600, 300), 500);
launch_app("com.demo.app", "com.demo.app.MainActivity");
add_policies("主策略集", "兜底策略集", true, false);
```

## 3. 基础构造器

### `point(x, y)`

像素坐标点。

```rhai
let p = point(120, 320);
```

### `percent(x, y)`

百分比坐标点。

```rhai
let p = percent(0.5, 0.5);
```

### `rgb(r, g, b)`

RGB 颜色值。

```rhai
let red = rgb(255, 0, 0);
```

## 4. 动作函数

### 点击

最小版：

```rhai
click(point(120, 320));
click(percent(0.5, 0.5));
```

按 OCR 结果点击：

```rhai
click_text("runtime.ocrResults");
click_text("runtime.ocrResults", "开始");
```

按检测结果点击：

```rhai
click_label("runtime.detResults");
click_label("runtime.detResults", 7);
```

说明：

- `click_text(input_var)`
  - 点击输入结果集里的首个可用文本目标。
- `click_text(input_var, text)`
  - 先按文本筛选，再点击。
- `click_label(input_var)`
  - 点击输入结果集里的首个可用检测目标。
- `click_label(input_var, idx)`
  - 先按标签索引筛选，再点击。

### 长按

```rhai
long_click(point(120, 320));
long_click(percent(0.5, 0.5));

long_click_text("runtime.ocrResults");
long_click_text("runtime.ocrResults", "开始");

long_click_label("runtime.detResults");
long_click_label("runtime.detResults", 7);
```

### 滑动

坐标滑动：

```rhai
swipe(point(100, 300), point(600, 300));
swipe(point(100, 300), point(600, 300), 500);
```

OCR 结果滑动：

```rhai
swipe_text("runtime.ocrResults", "左侧按钮", "右侧按钮");
swipe_text("runtime.ocrResults", "左侧按钮", "右侧按钮", 500);
```

检测结果滑动：

```rhai
swipe_label("runtime.detResults", 1, 2);
swipe_label("runtime.detResults", 1, 2, 500);
```

### 其它动作

```rhai
capture("runtime.capture");
input_text("hello");
back();
home();
reboot();

launch_app("com.demo.app");
launch_app("com.demo.app", "com.demo.app.MainActivity");
stop_app("com.demo.app");
```

## 5. 流程函数

### 显式等待

```rhai
wait_ms(1000);
wait_ms(runtime.dynamic_wait_ms);
```

### 控制流

```rhai
stop_script();
link_task("领取奖励");
```

说明：

- `link_task(task_name)`
  - 参数传任务名称，不传 id。

## 6. 任务与策略状态函数

任务状态：

```rhai
set_task_enabled("签到任务", true);
set_task_skip("签到任务", false);
set_task_done("签到任务", true);
```

策略状态：

```rhai
set_policy_skip("弹窗关闭策略", true);
set_policy_done("弹窗关闭策略", true);
```

说明：

- 参数统一传名称。
- 后台会先按当前脚本 bundle 查名称，再进入内部执行器。

## 7. 视觉函数

```rhai
capture("runtime.capture");
detect("runtime.capture", "runtime.detResults");
ocr("runtime.capture", "runtime.ocrResults");
```

常见组合：

```rhai
capture("runtime.capture");
ocr("runtime.capture", "runtime.ocrResults");
click_text("runtime.ocrResults", "开始");
```

## 8. 策略绑定与执行函数

所有名称参数都传“脚本编辑器里显示给你的名字”。

### 策略集绑定

```rhai
add_policies("主策略集", "兜底策略集");
add_policies("主策略集", "兜底策略集", true, false);
remove_policies("主策略集", "兜底策略集");
```

### 策略组与策略集绑定

```rhai
bind_policy_group("弹窗策略组", "兜底策略集");
bind_policy_group("弹窗策略组", "兜底策略集", true, false);
remove_policy_group("弹窗策略组", "兜底策略集");
```

### 策略组之间绑定

```rhai
add_policy_groups("前置策略组", "主策略组");
add_policy_groups("前置策略组", "主策略组", true, false);
unload_policy_group("前置策略组", "主策略组");
```

### 策略与策略组绑定

```rhai
bind_policy("关闭弹窗策略", "主策略组");
bind_policy("关闭弹窗策略", "主策略组", true, false);
unload_policy("关闭弹窗策略", "主策略组");
```

### 执行策略集 / 策略

执行策略集：

```rhai
handle_policy_set(["主策略集"], "runtime.policySetResult");

handle_policy_set(
  ["主策略集", "兜底策略集"],
  "runtime.detResults",
  "runtime.ocrResults",
  "runtime.searchHits",
  "runtime.policySetResult",
);
```

执行策略：

```rhai
handle_policy(["关闭弹窗策略"], "runtime.searchHits", "runtime.policyResult");
```

## 9. 变量与数据传递规则

Rhai 步骤中最重要的一点，是函数调用和普通 Rhai 语句不是同一个时机执行。

执行顺序是：

1. Rhai 普通代码先跑。
2. 代码里调用的内建函数先进入队列。
3. Rhai 代码块结束后，队列里的函数才真正执行。
4. 最后一行返回值再写入“输出变量”。

这意味着：

- 你在 Rhai 里先写的 `runtime.xxx`，后面的函数能读到。
- 函数执行后的结果，不能在同一个 Rhai 代码块后面的语句里立刻读到。

正确示例：

```rhai
runtime.target_text = "开始";
capture("runtime.capture");
ocr("runtime.capture", "runtime.ocrResults");
click_text("runtime.ocrResults", runtime.target_text);
```

不该这样理解：

```rhai
ocr("runtime.capture", "runtime.ocrResults");
// 这里不要假设下一行已经拿到了 ocr 的新结果
```

如果你要消费函数执行后的新结果，请放到下一个步骤里处理。

## 10. 推荐写法

### 先让脚本能跑

```rhai
capture("runtime.capture");
ocr("runtime.capture", "runtime.ocrResults");
click_text("runtime.ocrResults", "开始");
wait_ms(1200);
```

### 再逐步加筛选

```rhai
runtime.button_text = input.action_button_text ?? "开始";

capture("runtime.capture");
ocr("runtime.capture", "runtime.ocrResults");
click_text("runtime.ocrResults", runtime.button_text);
```

### 控制逻辑优先用原生 Rhai

```rhai
for attempt in 0..3 {
  capture("runtime.capture");
  ocr("runtime.capture", "runtime.ocrResults");

  if runtime.should_stop == true {
    break;
  }
}
```

## 11. 脚本编辑器里的 If / While 条件类型

脚本编辑器里的 `If` 和 `While` 步骤，目前条件面板实际支持这些类型：

### 表达式

直接写一段返回 `bool` 的 Rhai 表达式。

```rhai
runtime.retry < 3
```

适合：

- 简单判断
- 临时逻辑
- 已经在 `input` / `runtime` 里有足够数据的场景

### 条件组

用 `AND / OR / NOT` 组合多个子条件。

适合：

- 同时判断多个变量
- 把任务状态、变量比较、策略结果这些条件组合起来

### 执行次数

按任务或策略的执行次数判断。

适合：

- 限制某任务最多执行几次
- 判断某策略是否已经执行到上限

### 任务状态

按任务或策略的 `enabled / done / skip` 状态判断。

适合：

- 某任务完成后再继续
- 某策略被跳过后切换分支

### 当前任务

判断当前正在执行的任务是否属于指定任务集合。

适合：

- 同一段公共逻辑在不同任务里复用
- 当前任务命中某个组时走专门分支

### 变量比较

比较 `input.xxx` 或 `runtime.xxx` 变量。

支持的比较运算：

- 等于
- 不等于
- 小于
- 小于等于
- 大于
- 大于等于
- 包含
- 不包含

适合：

- OCR 文本命中判断
- 计数值判断
- 布尔开关判断

### 判断数量大小

统计检测标签或 OCR 文字的匹配数量，再和目标值比较。

适合：

- 某个文字是否至少出现 1 次
- 某类检测框数量是否超过阈值

### 策略集结果

按 `handle_policy_set(...)` 的结果对象判断。

当前可判断字段：

- `matched`
- `policySetId`
- `policyGroupId`
- `policyId`

适合：

- 某轮策略集是否命中
- 命中的到底是哪条策略 / 哪个策略组 / 哪个策略集

### 当前不建议使用的旧条件

内部还存在 `ColorCompare` 条件结构，但当前编辑器条件类型面板没有把它作为公开可选项。

如果要做颜色判断，当前应优先使用独立步骤能力，再把结果写入变量后用 If / While 条件继续判断。
