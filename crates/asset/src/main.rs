#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

//! Asset 示例入口，当前保持最小 Bevy 应用启动路径。

use bevy::prelude::*;

/// 启动最小 Bevy 应用。
fn main() {
    // 创建应用并进入 Bevy 主循环。
    App::new().run();
}
