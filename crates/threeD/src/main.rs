//! 3D 示例入口，负责装配 Bevy 应用和全局系统。

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

mod font_loader;

use bevy::prelude::*;

use crate::font_loader::install_default_ui_font;

/// 启动 3D 示例应用。
fn main() {
    // 创建 Bevy 应用并加载默认插件集合。
    App::new()
        .add_plugins(DefaultPlugins)
        // 安装中文友好的默认 UI 字体，避免文本显示缺字。
        .add_systems(Startup, install_default_ui_font)
        // 当前示例先保持最小启动路径，后续章节可继续追加 3D 场景系统。
        .run();
}
