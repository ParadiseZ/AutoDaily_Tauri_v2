use crate::app::app_error::{AppError, AppResult};
use crate::constant::project::MAIN_WINDOW;
use crate::domain::config::shortcut_conf::ShortCut;
use crate::infrastructure::logging::log_trait::Log;
use std::str::FromStr;
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

/// 从配置读取并绑定快捷键
pub fn register_short_cut_by_config(
    shortcut_conf: ShortCut,
    app_handle: &AppHandle,
) -> AppResult<String> {
    let manager = app_handle.global_shortcut();
    // 先清除所有已注册的快捷键（避免重复注册）
    unregister_all(app_handle)?;

    let mut registered_count = 0;
    let mut messages = Vec::new();

    // 注册截图快捷键
    if !shortcut_conf.capture.is_empty() {
        match register_single_shortcut(
            &manager,
            &shortcut_conf.capture,
            capture_shortcut_handler,
            "截图",
        ) {
            Ok(msg) => {
                registered_count += 1;
                messages.push(msg);
            }
            Err(e) => messages.push(format!("截图快捷键注册失败: {}", e)),
        }
    } else {
        messages.push("截图快捷键未设置（空值）".to_string());
    }

    // 注册显示/隐藏窗口快捷键
    if !shortcut_conf.toggle_window.is_empty() {
        match register_single_shortcut(
            &manager,
            &shortcut_conf.toggle_window,
            toggle_window_handler,
            "显示/隐藏窗口",
        ) {
            Ok(msg) => {
                registered_count += 1;
                messages.push(msg);
            }
            Err(e) => messages.push(format!("显示/隐藏窗口快捷键注册失败: {}", e)),
        }
    } else {
        messages.push("显示/隐藏窗口快捷键未设置（空值）".to_string());
    }

    // 注册运行/停止所有脚本快捷键
    if !shortcut_conf.toggle_all_scripts.is_empty() {
        match register_single_shortcut(
            &manager,
            &shortcut_conf.toggle_all_scripts,
            toggle_all_scripts_handler,
            "运行/停止所有脚本",
        ) {
            Ok(msg) => {
                registered_count += 1;
                messages.push(msg);
            }
            Err(e) => messages.push(format!("运行/停止所有脚本快捷键注册失败: {}", e)),
        }
    } else {
        messages.push("运行/停止所有脚本快捷键未设置（空值）".to_string());
    }

    let summary = format!("快捷键加载完成，成功注册 {} 个快捷键", registered_count);
    messages.insert(0, summary.clone());

    // 记录详细日志
    for msg in &messages {
        Log::info(msg);
    }

    Ok(summary)
}

/// 注册单个快捷键的辅助函数
fn register_single_shortcut<F>(
    manager: &tauri_plugin_global_shortcut::GlobalShortcut<Wry>,
    shortcut_str: &str,
    handler: F,
    description: &str,
) -> AppResult<String>
where
    F: Fn(&AppHandle, &Shortcut, ShortcutEvent) + Send + Sync + 'static,
{
    let shortcut = Shortcut::from_str(shortcut_str).map_err(|e| AppError::ShortCutSetFailed {
        detail: format!("解析{}快捷键 '{}' 失败", description, shortcut_str),
        e: e.to_string(),
    })?;

    manager
        .on_shortcut(shortcut, handler)
        .map_err(|e| AppError::ShortCutSetFailed {
            detail: format!("绑定{}快捷键 '{}' 失败", description, shortcut_str),
            e: e.to_string(),
        })?;

    Ok(format!("{}快捷键 '{}' 注册成功", description, shortcut_str))
}

/// 卸载所有快捷键
pub fn unregister_all(app_handle: &AppHandle) -> AppResult<()> {
    app_handle
        .global_shortcut()
        .unregister_all()
        .map_err(|e| AppError::ShortCutSetFailed {
            detail: "卸载所有快捷键失败".to_string(),
            e: e.to_string(),
        })?;
    Ok(())
}

/// 显示/隐藏主窗口快捷键处理函数
pub fn toggle_window_handler(app_handle: &AppHandle, _shortcut: &Shortcut, event: ShortcutEvent) {
    // 只在按键按下时执行操作，避免按下和松开都触发
    if event.state != ShortcutState::Pressed {
        return;
    }

    if let Some(window) = app_handle.get_webview_window(MAIN_WINDOW) {
        match window.is_visible() {
            Ok(visible) => {
                if visible {
                    if let Err(e) = window.hide() {
                        Log::error(&format!("隐藏窗口失败: {}", e));
                    } else {
                        Log::info("窗口已隐藏");
                    }
                } else {
                    if let Err(e) = window.show() {
                        Log::error(&format!("显示窗口失败: {}", e));
                    } else {
                        Log::info("窗口已显示");
                        // 将窗口带到前台
                        let _ = window.set_focus();
                    }
                }
            }
            Err(e) => {
                Log::error(&format!("获取窗口可见性失败: {}", e));
            }
        }
    } else {
        Log::error("找不到主窗口");
    }
}

/// 运行/停止所有脚本快捷键处理函数
pub fn toggle_all_scripts_handler(
    _app_handle: &AppHandle,
    _shortcut: &Shortcut,
    event: ShortcutEvent,
) {
    // 只在按键按下时执行操作，避免按下和松开都触发
    if event.state != ShortcutState::Pressed {
        return;
    }

    // TODO: 这里需要实现脚本的启动/停止逻辑
    // 现在只是占位符，输出日志
    Log::info("触发运行/停止所有脚本快捷键");
    // 这里将来可以调用脚本管理服务的相关方法
}

/// 快捷键回调函数，符合快捷键处理程序的签名要求
pub fn capture_shortcut_handler(
    _app_handle: &AppHandle,
    _shortcut: &Shortcut,
    event: ShortcutEvent,
) {
    // 只在按键按下时执行操作，避免按下和松开都触发
    if event.state != ShortcutState::Pressed {
        return;
    }
    Log::info("快捷键截图功能为实现...");
    //let _ = get_capture().is_some();
}
