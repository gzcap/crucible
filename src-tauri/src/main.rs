//! ROC 应用入口文件
//!
//! 这是 Tauri 应用的主入口，负责启动整个应用程序。
//! 实际的应用逻辑在 `roc_lib` crate 的 `run()` 函数中实现。

// 防止在 Windows 发布版本中弹出额外的控制台窗口，请勿删除！
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// 应用主函数
///
/// 调用 `roc_lib::run()` 启动 Tauri 应用，该函数在 `lib.rs` 中定义。
fn main() {
    roc_lib::run()
}