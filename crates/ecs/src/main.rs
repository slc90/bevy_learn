#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

mod components;

use bevy::prelude::*;
use components::{DemoState, Health, Player, Velocity};

/// 这是最小可运行的 Bevy 0.19 程序：
/// - 创建 App
/// - 设置窗口清屏色
/// - 加载默认插件
/// - 在启动阶段创建一个 2D 相机
fn main() {
    App::new()
        // DefaultPlugins 会带上窗口、输入、时间、日志、渲染等常用功能。
        .add_plugins(DefaultPlugins)
        // ClearColor 是一个全局资源，控制窗口背景色。
        // 接近黑夜的蓝色背景
        .insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.12)))
        .init_resource::<DemoState>()
        // Startup 调度只在启动时运行一次，适合做初始化。
        .add_systems(Startup, setup)
        .add_systems(Update, keyboard_demo)
        .run();
}

/// 初始化系统：生成一个 2D 相机。
fn setup(mut commands: Commands) {
    info!("创建2D相机");
    commands.spawn(Camera2d);
    info!("按 S 创建玩家，按 Q 查询玩家，按 D 删除玩家。");
}

fn keyboard_demo(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DemoState>,
    mut players: Query<(Entity, &Player, &Health, &mut Velocity)>,
) {
    if !state.help_printed {
        state.help_printed = true;
        debug!("示例开始：S=spawn, Q=query, D=despawn");
    }

    // 创建实体：只在还没有玩家时创建。
    if keys.just_pressed(KeyCode::KeyS) && state.player.is_none() {
        let entity = commands
            .spawn((
                Player {
                    name: "Alice".to_string(),
                },
                Health(100),
                Velocity(Vec2::new(1.0, 0.0)),
            ))
            .id();

        state.player = Some(entity);
        info!("已创建玩家实体: {:?}", entity);
    }
    // 查询实体：遍历所有匹配 Player + Health + Velocity 的实体。
    else if keys.just_pressed(KeyCode::KeyQ) {
        for (entity, player, health, mut velocity) in &mut players {
            // 这里演示可以在查询时修改组件。
            velocity.0.x += 0.5;

            info!(
                "查询到实体 {:?}: name={}, hp={}, vel=({}, {})",
                entity, player.name, health.0, velocity.0.x, velocity.0.y
            );
        }
    }
    // 删除实体：如果保存过 Entity ID，就可以直接按 ID 删除。
    else if keys.just_pressed(KeyCode::KeyD) {
        if let Some(entity) = state.player.take() {
            commands.entity(entity).despawn();
            info!("已删除玩家实体: {:?}", entity);
        } else {
            info!("当前没有玩家实体可删除。");
        }
    }
}
