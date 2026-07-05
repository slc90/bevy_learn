//! Gizmo 辅助绘制系统，用于观察世界坐标轴和实体局部坐标。
//! 当前文件保留为 3D 学习示例的调试绘制片段。

/// 调整 Gizmo 默认配置，让调试线条更容易观察。
fn setup_gizmo_config(mut config_store: ResMut<GizmoConfigStore>) {
    // 获取默认 Gizmo 配置组，统一影响后续所有默认调试绘制。
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();

    // -1.0 表示总是画在几何体前面
    // 这能避免坐标轴被模型表面遮挡。
    config.depth_bias = -1.0;

    // 可选：线粗一点更容易看
    // 线宽保持固定，避免调试线在高分辨率屏幕上过细。
    config.line.width = 3.0;
}

/// 绘制世界坐标轴，并让 XYZ 标签朝向当前 3D 相机。
fn draw_axes(mut gizmos: Gizmos, camera_query: Query<&Transform, With<Camera3d>>) {
    // 如果当前场景没有 3D 相机，跳过依赖相机朝向的标签绘制。
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    // 轴长、标签偏移和标签高度共同决定坐标标记的视觉间距。
    let len = 1.6;
    let label_offset = 0.25;
    let label_height = 0.18;

    // 使用标准 RGB 颜色区分 X、Y、Z 三个方向。
    // 颜色约定和常见 3D 编辑器保持一致，方便快速识别方向。
    let red = Color::srgb(1.0, 0.0, 0.0);
    let green = Color::srgb(0.0, 1.0, 0.0);
    let blue = Color::srgb(0.0, 0.25, 1.0);

    // 从世界原点向三个正方向绘制箭头。
    gizmos.arrow(Vec3::ZERO, Vec3::X * len, red);
    gizmos.arrow(Vec3::ZERO, Vec3::Y * len, green);
    gizmos.arrow(Vec3::ZERO, Vec3::Z * len, blue);

    // 复用相机旋转，让文字标签始终面向当前观察方向。
    let text_rotation = camera_transform.rotation;

    // X 标签：沿 X 放，额外向上抬一点
    gizmos.text(
        Isometry3d::new(
            Vec3::X * (len + label_offset) + Vec3::Y * label_height,
            text_rotation,
        ),
        "X",
        0.25,
        Vec2::ZERO,
        red,
    );

    // Y 标签：本来就在空中，不需要额外抬高
    gizmos.text(
        Isometry3d::new(Vec3::Y * (len + label_offset), text_rotation),
        "Y",
        0.25,
        Vec2::ZERO,
        green,
    );

    // Z 标签：沿 Z 放，额外向上抬一点
    gizmos.text(
        Isometry3d::new(
            Vec3::Z * (len + label_offset) + Vec3::Y * label_height,
            text_rotation,
        ),
        "Z",
        0.25,
        Vec2::ZERO,
        blue,
    );
}

/// 为每个网格实体绘制局部原点和局部三轴方向。
fn draw_mesh_origins(mut gizmos: Gizmos, query: Query<&GlobalTransform, With<Mesh3d>>) {
    // 每条局部坐标轴使用固定长度，避免调试线条干扰主体模型。
    let len = 0.5;

    // 遍历场景中的网格实体，读取它们的世界空间变换。
    for global in &query {
        let origin = global.translation();

        // Entity 的局部 X/Y/Z 方向，在世界坐标中的方向
        let x_axis = global.right().as_vec3();
        let y_axis = global.up().as_vec3();
        let z_axis = global.back().as_vec3(); // Bevy 的 +Z 方向；forward 是 -Z

        gizmos.line(
            origin,
            origin + x_axis * len,
            Color::srgb(1.0, 0.0, 0.0), // X 红
        );

        gizmos.line(
            origin,
            origin + y_axis * len,
            Color::srgb(0.0, 1.0, 0.0), // Y 绿
        );

        gizmos.line(
            origin,
            origin + z_axis * len,
            Color::srgb(0.0, 0.0, 1.0), // Z 蓝
        );

        // 可选：在原点画一个小球
        // 小球标记能帮助区分实体原点和轴线端点。
        gizmos.sphere(Isometry3d::from_translation(origin), 0.04, Color::WHITE);
    }
}
