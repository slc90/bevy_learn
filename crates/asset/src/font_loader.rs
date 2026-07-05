//! 默认 UI 字体安装逻辑，用于保证示例中的中文文本可以正常渲染。

use bevy::{
    asset::{AssetId, Assets},
    ecs::system::ResMut,
    log::warn,
    text::Font,
};

/// 内嵌到程序中的默认 UI 字体字节。
const DEFAULT_UI_FONT_BYTES: &[u8] = include_bytes!(concat!(
    // 字体在编译期嵌入二进制，运行时不再依赖外部路径。
    env!("CARGO_MANIFEST_DIR"),
    "/assets/SmileySans-Oblique.ttf"
));

/// 将内嵌字体注册为 Bevy 默认 UI 字体。
pub fn install_default_ui_font(mut fonts: ResMut<Assets<Font>>) {
    // 使用默认 AssetId 替换默认字体，后续 UI 文本可直接复用。
    if let Err(error) = fonts.insert(
        AssetId::default(),
        // Bevy 资产系统接收拥有所有权的字体字节。
        Font::from_bytes(DEFAULT_UI_FONT_BYTES.to_vec()),
    ) {
        // 记录替换失败原因，方便定位字体资源或资产系统问题。
        warn!("替换默认 UI 字体失败: {error}");
    }
}
