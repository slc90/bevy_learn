#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, install_default_ui_font)
        .run();
}

const DEFAULT_UI_FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/SmileySans-Oblique.ttf"
));

fn install_default_ui_font(mut fonts: ResMut<Assets<Font>>) {
    if let Err(error) = fonts.insert(
        AssetId::default(),
        Font::from_bytes(DEFAULT_UI_FONT_BYTES.to_vec()),
    ) {
        warn!("替换默认 UI 字体失败: {error}");
    }
}
