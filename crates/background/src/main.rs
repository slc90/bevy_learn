#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

//! 后台任务学习示例入口。
//! 当前入口保持最小 App，用于配合文档逐步扩展不同任务管理方案。

use bevy::prelude::*;

/// 启动 Bevy 应用，后续章节会在这个入口上逐步挂载插件、资源和系统。
fn main() {
    // 最小运行循环便于验证 crate 能独立编译和启动。
    App::new().run();
}
