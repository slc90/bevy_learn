# Bevy 0.19 中 App Settings 教程

## 执行摘要

Bevy 0.19 在官方发布说明中明确把 **App Settings** 列为新特性之一：它提供了一个官方的、通用的“应用设置”框架，可将设置从文件加载为 ECS `Resource`，并在运行时保存回去，适合图形选项、布局偏好、音量、窗口位置，或“不要再提示我”这类跨会话持久化状态。官方同时给出了 `SettingsGroup`、`SettingsPlugin`、`SaveSettingsDeferred`、`SaveSettingsSync` 等核心 API，并在示例页提供了完整演示。

从 0.19 的官方文档、发布说明与源码可归纳出一个非常重要的结论：**Bevy 0.19 内建的 App Settings 后端是 TOML**。文档和源码都围绕 `settings.toml`、TOML 表、`toml` 依赖与 TOML 路径展开，没有官方 JSON/YAML 后端，也没有官方环境变量覆盖或热重载 API。换言之，**TOML 是内建方案；JSON/YAML、环境变量覆盖、热重载需要你自己叠加一层桥接逻辑**。

就使用方式而言，`SettingsPlugin::new(app_name)` 会在插件构建阶段**立即加载**设置，并把对应资源插入到 `World`；这意味着插件顺序很关键。官方文档明确建议：如果后续插件或你的初始化逻辑依赖这些设置，应当优先初始化并加载设置。保存不会自动发生，推荐在资源更改后发出 `SaveSettingsDeferred`，并在退出前补一个 `SaveSettingsSync::IfChanged`，这样兼顾高频更新场景与退出前落盘。

本文给出的核心结论是：如果你现在在 Bevy 0.18 或更早版本里手写配置加载，**0.19 的 App Settings 非常值得用来接管“用户偏好型配置”**；但如果你需要多格式配置、严格 schema 校验、环境变量注入、配置热重载、密钥/令牌安全存储，则 0.19 的内建能力还不够，需要把它当作“持久化偏好层”，然后在其上叠加自己的配置系统。

## 背景与目的

官方把 App Settings 的目标描述得很清楚：游戏和应用都需要保存用户设置，例如图形选项、音频音量、窗口位置与大小、面板布局、工具偏好，或者是否已经看过教程。Bevy 在 0.19 之前并没有这个官方内建框架，因此 0.19 的 App Settings 本质上是在弥补“用户偏好型持久化数据”的官方空白。

官方发布说明还特别点出，**Bevy Editor 自己就需要这套系统**，因为编辑器本身也是一个 Bevy App。所以这套机制不是“只给游戏用的小工具”，而是 Bevy 打算自己长期依赖的基础设施之一。这个背景很重要，因为它解释了为什么这套 API 采用 ECS `Resource`、`Reflect`、插件式加载与平台标准配置目录，而不是单纯暴露一个“读 TOML/写 TOML”的函数。

在定位上，App Settings 更适合 **“用户偏好”**，而不是所有配置。官方 issue 对它的定义是：这类数据通常是**按用户**、**跨会话**、**不是资产**、**也不是存档**。例如音量、布局、窗口尺寸属于它；而关卡定义、地图资源、游戏内容本身，通常不属于它。

从工程角度看，使用 App Settings 的主要收益有三点。第一，它把配置数据直接暴露为 ECS 资源，让你的系统能像读取任何别的 `Res<T>` 一样读取设置。第二，它把存储路径交给平台规范处理：Linux 放配置目录、macOS 放 `~/Library/Preferences`、Windows 放 `%LOCALAPPDATA%`、WASM 放 `localStorage`。第三，它内置了去抖保存和同步保存两种路径，减少“频繁写盘”或“退出未保存”的问题。

## 功能概述与加载生命周期

App Settings 的工作流可以概括为：**注册类型 → 安装插件 → 启动时加载 → 运行时读写 Resource → 延迟或同步保存**。官方发布说明与源码都表明，`SettingsPlugin` 会扫描类型注册表，找出被标记为设置组的资源类型，按文件名分桶，然后逐个读取 TOML 文件并把值应用到 `World`。

这里有两个容易忽略但非常关键的点。第一，加载发生在**插件构建阶段**，不是在 `Startup` 系统里；所以如果你希望其他插件或启动系统一开始就读到这些设置，就要把 `SettingsPlugin` 加在前面。第二，加载后这些设置会以 **ECS `Resource`** 的形式存在，因此后续系统只需要正常使用 `Res<T>` / `ResMut<T>` 即可。官方文档明确说，`SettingsPlugin` 不依赖其他插件，建议尽量先加载设置，再让别的插件消费它们。

官方也明确说明了存储位置：桌面端会写入平台的首选项/配置目录，WASM 写入浏览器 `localStorage`，其他没有合适持久化目录的平台则**不持久化**。这意味着 Bevy 的 App Settings 不是从你的项目根目录、`assets/` 目录或可执行文件旁边读取，而是从用户级配置目录读取。很多初学者第一次“找不到配置文件”，就是因为去错了路径。

下面这张流程图把 0.19 的官方加载/保存时序串起来。图中的“PostUpdate 去抖保存”和“退出前同步保存”都直接来自源码与官方文档描述。

```mermaid
flowchart TD
    A[注册设置类型<br/>derive Resource + SettingsGroup + Reflect + Default] --> B[add_plugins(SettingsPlugin::new(app_name))]
    B --> C[插件 build 阶段扫描 TypeRegistry]
    C --> D[按 file/group/key 形成设置清单]
    D --> E[逐个加载 settings.toml 等 TOML 文件]
    E --> F[若 Resource 已存在<br/>把匹配字段应用到现有资源]
    E --> G[若 Resource 不存在<br/>以 Default 创建并与 TOML 合并]
    F --> H[运行时系统通过 Res<T> / ResMut<T> 读取与修改]
    G --> H
    H --> I[修改后 queue(SaveSettingsDeferred(Duration))]
    I --> J[PostUpdate 中计时器 tick]
    J --> K[计时器到期后 queue(SaveSettings::IfChanged)]
    H --> L[退出前 queue(SaveSettingsSync::IfChanged)]
    K --> M[异步写入 TOML]
    L --> N[同步写入 TOML]
    M --> O[原子替换旧文件]
    N --> O
```

如果你只想记一条经验法则，可以记成：**先加载、后消费；平时延迟保存，退出前同步保存**。这也是官方文档推荐的组合方式，因为某些平台的退出路径未必给你足够机会拦截窗口关闭事件。

## API 详解

### 核心类型与职责

下表总结了 Bevy 0.19 App Settings 的几个核心 API。表中所有 API 与行为均以 `bevy = "0.19"` 为准；相关代码块的 **Rust 版本：未指定**。官方材料没有为这组 API 单独给出一个专属 Rust 版本要求，因此本文按你的要求统一标注为“未指定”。

| API | 类型 | 关键参数或返回 | 默认值 | 生命周期与时机 | 错误处理与边界 | 说明 |
|---|---|---|---|---|---|---|
| `SettingsPlugin` | `struct` | `new(app_name: &str) -> Self` | 无 | 在插件 `build` 时立即加载；建议尽量放在依赖设置的插件之前 | `app_name` 需要全局唯一，官方建议用反向域名；平台无文件系统时使用其他存储或不持久化 | 负责扫描设置组、从存储加载、注册延迟保存系统。 |
| `SettingsGroup` | `trait` | `settings_group_name() -> &'static str`、`settings_key_name() -> Option<&'static str>`、`settings_source() -> Option<&'static str>` | 默认文件名为 `"settings"`，即 `settings.toml` | 类型级元数据；由 derive 宏与反射系统提供 | 不是 `dyn` compatible；若多个资源组名冲突，会合并到同一节 | 用于把某个 `Resource` 映射到设置文件中的节与键。支持 `group`、`key`、`file` 属性覆盖。 |
| `SaveSettingsDeferred` | `struct` | `SaveSettingsDeferred(Duration)` | `Default` 为 `1` 秒 | 运行时发出命令后，计时器在 `PostUpdate` 中 tick，并在到期时转成 `SaveSettings::IfChanged` | 若未安装 `SettingsPlugin`，设置注册表不存在，命令不会真正触发保存 | 用于高频变更场景的去抖保存。 |
| `SaveSettings` | `enum` | `IfChanged` / `Always` | `Default` 为 `IfChanged` | 命令执行时异步保存 | 保存发生在其他线程；若注册表不存在会发出警告并返回 | 适合平时后台落盘。 |
| `SaveSettingsSync` | `enum` | `IfChanged` / `Always` | `Default` 为 `IfChanged` | 命令执行时同步保存 | 阻塞命令队列直到保存完成；若注册表不存在会警告并返回 | 适合退出前“最后一次落盘”。 |
| `ReflectSettingsGroup` | `struct` | 反射元数据容器 | 未指定 | 内部/反射层使用 | 常规业务中通常无需直接手工操作 | 它保存组名、键名、文件名等反射元数据；源码据此构建设置文件清单。 |

### 类型约束与 derive 要求

官方例子与源码都表明，一个能被 `SettingsPlugin` 自动发现并创建/加载的设置资源，最核心的模式是：

- `#[derive(Resource, SettingsGroup, Reflect, Default)]`
- `#[reflect(Resource, SettingsGroup, Default)]`

发布说明直接给出了这套组合；源码则进一步说明，构建注册表时会跳过**没有 `ReflectDefault`** 的类型，因此只有 `SettingsGroup` 还不够，通常还必须提供 `Default` 并把它注册到反射数据里。

这也解释了为什么很多“我明明 derive 了 `SettingsGroup`，但没自动加载”的问题，本质上是**少了 `Default` 或 `#[reflect(Default)]`**。对 Bevy 0.19 而言，默认值不是可选装饰，而是“首次运行、缺字段、或文件不存在时如何建资源”的基础。

### 参数、默认值与命名规则

`SettingsPlugin::new(app_name)` 的 `app_name` 会被复制成内部字符串，随后用于生成应用专属配置目录。官方文档强调这个值要**全局唯一**，推荐使用反向域名写法，例如 `com.example.myapp`；如果你没有域名，也可以基于你的仓库托管地址倒写一个名字。

`SettingsGroup` 的三个静态方法分别定义了节名、枚举键名和来源文件名，返回值都带 `'static` 生命周期：这组名称在程序生命周期内都是编译期常量元数据。默认文件名是 `"settings"`，对应 `settings.toml`；默认组名来自 derive 生成的蛇形命名；若多个资源使用同一个组名，其字段会合并到同一个 TOML 节。对枚举来说，`key` 用来指定那一个“唯一键”。

保存命令的默认行为也值得单独记住：`SaveSettings` 与 `SaveSettingsSync` 的默认变体都是 `IfChanged`，只有检测到自上次加载/保存以来资源发生变化时才写盘；`SaveSettingsDeferred::default()` 的延迟是 `1` 秒。源码中还可以看到，去抖定时器到期后，会自动排入 `SaveSettings::IfChanged`。

### 加载语义、默认值合并与错误处理

源码把加载行为分成两类。若目标资源**已经存在**，就把 TOML 中匹配到的字段应用到现有资源；若目标资源**不存在**，就先用默认值创建，再把 TOML 中的值覆写进去。官方 issue 还进一步解释了当前反序列化语义：**缺失字段保留默认值/现有值，额外字段会被忽略**。这在容错上是友好的，但也意味着“拼错字段名”未必会直接报错。

错误处理方面，官方源码里可以看到两条最常见的警告信息。第一，如果你调用保存命令时没有安装 `SettingsPlugin`，会警告“找不到设置注册表，你是不是忘了安装 SettingsPlugin”；第二，如果启动时找不到对应 TOML 文件，会警告 `Filename xxx.toml not found`。第一种是接入错误，第二种在首次运行时通常是正常现象。

保存策略方面，官方强调“保存不是自动的”，推荐修改后发 `SaveSettingsDeferred`，退出前再发 `SaveSettingsSync::IfChanged`。此外，保存实现具备一定崩溃抗损坏能力：文档明确说明它会先写临时文件，再用原子方式替换旧文件，因此中途崩溃不容易把原文件写坏。

## 配置文件格式与完整最小示例

### 内建格式与多格式比较

先给出最重要的结论：**Bevy 0.19 的内建 App Settings 直接支持 TOML；JSON/YAML 需要自定义桥接层。** 证据有三层：官方文档通篇写的是 `settings.toml` 和 TOML table；crates 文档依赖里明确出现 `toml`；而针对 `json`、`yaml`、`environment`、`reload` 的搜索在该 crate 文档与源码中都没有匹配项。

下面这张表把三种常见配置格式放在一起比较。需要特别注意，“是否可被 `SettingsPlugin` 直接读写”这一列里，只有 TOML 是“是”。JSON/YAML 的示例片段是为了帮助你设计**等价 schema**，不是说内建插件会直接读取它们。Serde 官方文档说明，Serde 本身是格式无关的数据结构序列化/反序列化框架，因此 JSON/YAML 可以通过自定义层与同一份 Rust 结构体对接。

| 格式 | Bevy 0.19 `SettingsPlugin` 直接支持 | 优点 | 缺点 | 适合场景 | 示例片段 |
|---|---|---|---|---|---|
| TOML | 是  | 人类可读性好；节/键结构和 Bevy 的 group/key/file 模型天然贴合；官方唯一内建路径  | 相比 JSON/YAML，生态里“跨语言通用配置”感略弱 | 用户偏好、桌面应用、游戏设置 | <pre><code>[counter]\ncount = 7\nstep = 2\nenabled = true\n\n[display]\nmode = "Windowed"</code></pre> |
| JSON | 否，需要自定义桥接  | Web/服务端生态通用；工具链广泛；前后端共享 schema 比较方便  | 0.19 内建不会直接加载；可读性在带注释需求时较差 | 你本来就有 JSON 配置源，或需与外部系统共享 | <pre><code>{\n  "counter": { "count": 7, "step": 2, "enabled": true },\n  "display": { "mode": "Windowed" }\n}</code></pre> |
| YAML | 否，需要自定义桥接  | 对复杂嵌套更直观；人工编辑体验通常不错 | 0.19 内建不会直接加载；缩进敏感；官方资料未给出 YAML 后端 | 你已有 YAML 体系，且愿意加一层自定义 loader | <pre><code>counter:\n  count: 7\n  step: 2\n  enabled: true\ndisplay:\n  mode: Windowed</code></pre> |

### 完整可运行的最小示例

下面这个示例基于官方 `settings.rs` 示例的思路做了压缩与中文化整理，同时加入了三件教程里最常用的模式：

- 两个 struct 组共享同一个 `[counter]` 节；
- 一个 enum 组使用独立 `[display]` 节和 `mode` 键；
- 运行时用 `SaveSettingsDeferred` 去抖保存，窗口关闭时再用 `SaveSettingsSync::IfChanged` 同步保存。

#### Cargo.toml

**Bevy 版本：`0.19`；Rust 版本：未指定。**  
官方示例页明确写出此示例需要启用 `bevy_settings` feature；而 Bevy 的 Cargo feature 文档把 `bevy_settings` 单独列为一项，同时默认 profile 只列出 `2d`、`3d`、`ui`、`audio`，因此教程里建议显式打开它。

```toml
[package]
name = "bevy_app_settings_tutorial"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy = { version = "0.19", features = ["bevy_settings"] }
```

#### src/main.rs

**Bevy 版本：`0.19`；Rust 版本：未指定。**  
这段代码使用的 API 均来自 Bevy 0.19 官方文档与示例：`SettingsPlugin`、`SettingsGroup`、`SaveSettingsDeferred`、`SaveSettingsSync`、`WindowCloseRequested`。

```rust
use std::time::Duration;

use bevy::{
    prelude::*,
    settings::{SaveSettingsDeferred, SaveSettingsSync, SettingsGroup, SettingsPlugin},
    window::{ExitCondition, WindowCloseRequested},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            // 让我们有机会在关闭窗口前同步保存
            exit_condition: ExitCondition::DontExit,
            primary_window: Some(Window {
                title: "Bevy 0.19 App Settings 教程".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(SettingsPlugin::new("com.example.bevy.settings.tutorial"))
        .add_systems(Startup, setup)
        .add_systems(Update, (show_state, handle_input, on_window_close))
        .run();
}

#[derive(Resource, SettingsGroup, Reflect, Default)]
#[reflect(Resource, SettingsGroup, Default)]
#[settings_group(group = "counter")]
struct CounterSettings {
    count: i32,
    step: i32,
}

#[derive(Resource, SettingsGroup, Reflect)]
#[reflect(Resource, SettingsGroup, Default)]
#[settings_group(group = "counter")]
struct CounterFlags {
    enabled: bool,
}

impl Default for CounterFlags {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Resource, SettingsGroup, Reflect, Default, Clone, Copy, Debug, PartialEq, Eq)]
#[reflect(Resource, SettingsGroup, Default)]
#[settings_group(group = "display", key = "mode")]
enum DisplayMode {
    #[default]
    Windowed,
    Borderless,
}

#[derive(Component)]
struct StatusText;

fn setup(mut commands: Commands) {
    commands.spawn((Camera::default(), Camera2d));

    commands
        .spawn(Node {
            width: percent(100.0),
            height: percent(100.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(12.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("正在加载设置..."),
                TextFont {
                    font_size: FontSize::Px(26.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                StatusText,
            ));

            parent.spawn((
                Text::new("SPACE: +step | BACKSPACE: -step | F1: 启用/禁用 | F2: 切换显示模式"),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        });

    println!("程序启动：如果 settings.toml 存在，会在插件构建阶段先被加载。");
}

fn show_state(
    mut query: Query<&mut Text, With<StatusText>>,
    counter: Res<CounterSettings>,
    flags: Res<CounterFlags>,
    mode: Res<DisplayMode>,
) {
    if !(counter.is_changed() || flags.is_changed() || mode.is_changed()) {
        return;
    }

    let mode_text = match *mode {
        DisplayMode::Windowed => "Windowed",
        DisplayMode::Borderless => "Borderless",
    };

    for mut text in &mut query {
        text.0 = format!(
            "count = {}, step = {}, enabled = {}, mode = {}",
            counter.count, counter.step, flags.enabled, mode_text
        );
    }

    println!(
        "当前设置：count={}, step={}, enabled={}, mode={}",
        counter.count, counter.step, flags.enabled, mode_text
    );
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut counter: ResMut<CounterSettings>,
    mut flags: ResMut<CounterFlags>,
    mut mode: ResMut<DisplayMode>,
    mut commands: Commands,
) {
    let mut changed = false;

    if keyboard.just_pressed(KeyCode::F1) {
        flags.enabled = !flags.enabled;
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::F2) {
        *mode = match *mode {
            DisplayMode::Windowed => DisplayMode::Borderless,
            DisplayMode::Borderless => DisplayMode::Windowed,
        };
        changed = true;
    }

    if flags.enabled {
        if keyboard.just_pressed(KeyCode::Space) {
            counter.count += counter.step.max(1);
            changed = true;
        }

        if keyboard.just_pressed(KeyCode::Backspace) || keyboard.just_pressed(KeyCode::Delete) {
            counter.count -= counter.step.max(1);
            changed = true;
        }
    }

    if changed {
        // 高频输入场景下建议使用去抖保存
        commands.queue(SaveSettingsDeferred(Duration::from_millis(250)));
        println!("设置已修改：已排队去抖保存（250ms）。");
    }
}

fn on_window_close(
    mut close_events: MessageReader<WindowCloseRequested>,
    mut commands: Commands,
) {
    if close_events.read().next().is_some() {
        println!("窗口关闭：执行同步保存，然后退出。");
        commands.queue(SaveSettingsSync::IfChanged);
        commands.write_message(AppExit::Success);
    }
}
```

#### 预置配置文件示例

**Bevy 版本：`0.19`；Rust 版本：未指定。**  
这个文件名之所以叫 `settings.toml`，是因为 `SettingsGroup` 的默认 `file` 名称就是 `"settings"`。而节名 `[counter]`、`[display]` 则来自上面代码里的 `#[settings_group(group = "...")]`。同名组会被合并到同一个节，这一点与官方示例一致。

```toml
[counter]
count = 7
step = 2
enabled = true

[display]
mode = "Windowed"
```

#### 文件应该放在哪里

这份 `settings.toml` **不应该** 放在项目根目录，而应该放到应用的用户配置目录里。对于本例的 `app_name = "com.example.bevy.settings.tutorial"`，常见路径如下。官方文档与发布说明都给出了这些平台路径规则。

| 平台 | 典型路径 |
|---|---|
| Linux | `~/.config/com.example.bevy.settings.tutorial/settings.toml` |
| macOS | `~/Library/Preferences/com.example.bevy.settings.tutorial/settings.toml` |
| Windows | `%LOCALAPPDATA%\com.example.bevy.settings.tutorial\settings.toml` |
| WASM | 浏览器 `localStorage`，不是文件 |

#### 示例运行输出

如果你第一次运行，且还没有配置文件，通常会看到“找不到 `settings.toml`”相关警告；这是正常现象，因为源码里明确在缺文件时只会 `warn!`，然后继续用默认值创建资源。

一个典型运行过程大致如下：

```text
程序启动：如果 settings.toml 存在，会在插件构建阶段先被加载。
当前设置：count=7, step=2, enabled=true, mode=Windowed
设置已修改：已排队去抖保存（250ms）。
当前设置：count=9, step=2, enabled=true, mode=Windowed
设置已修改：已排队去抖保存（250ms）。
当前设置：count=9, step=2, enabled=false, mode=Borderless
窗口关闭：执行同步保存，然后退出。
```

## 进阶用法

### 热重载

先说结论：**Bevy 0.19 的官方 App Settings 文档与源码没有提供内建“配置文件热重载”API。** 在 crate 文档与源码中搜索 `reload`、`watch`、`file_watcher` 都没有针对 `bevy_settings` 的命中；而 Bevy 的 `dev` collection 里提到的 `file_watcher` 明确是用于 **asset hot-reloading**，不是设置文件热重载。

因此，如果你想演示“热重载”，最佳做法是明确标注：**官方内建未支持，下面是替代方案。** 最稳妥的替代方案有两种。第一种是“轮询文件修改时间 + 重新解析 + 覆盖资源”；第二种是使用外部 watcher crate，在收到文件变更后重新读 TOML 并写回 `ResMut<T>`。这两种方案都属于“在官方 Settings 之上再包一层”。这个判断来自“官方没有 reload API”的证据，而不是主观臆测。

一个最小替代思路如下。注意，这不是官方内建 API，而是教程中推荐的工程化补充方案：

```rust
// Bevy 版本：0.19
// Rust 版本：未指定
//
// 思路：每隔一段时间检查 settings.toml 的 modified 时间；
// 一旦变化，就用 toml 重新解析，并把值写回 Resource。

use std::{fs, time::SystemTime};

// 伪代码：省略路径拼装与错误上报
fn poll_and_reload_settings(
    // last_modified: Local<Option<SystemTime>>,
    // mut counter: ResMut<CounterSettings>,
) {
    // 1. 找到系统配置目录下的 settings.toml
    // 2. fs::metadata(path).modified()
    // 3. 如果时间戳更新，则 fs::read_to_string(path)
    // 4. 用 toml 解析到一个中间 DTO
    // 5. 把 DTO 覆盖回 CounterSettings / DisplayMode 等 Resource
}
```

如果你只想在开发阶段方便调试，而不是做生产级 watcher，可以用按钮触发“重新读取配置文件”系统，这往往比常驻 watcher 更简单、更可控。由于官方没有热重载语义，手写一个显式“Reload Settings”按钮，实际上更容易排查问题。

### 环境变量覆盖

官方文档没有提供环境变量覆盖 API，源码和文档对 `environment` 搜索也没有命中，因此在 0.19 中这属于**需要你自己实现的外层覆盖逻辑**。比较推荐的时机是：**先让 `SettingsPlugin` 加载 TOML，再在 `Startup` 早期系统里读环境变量并覆写资源**。

示例思路如下：

```rust
// Bevy 版本：0.19
// Rust 版本：未指定

fn apply_env_overrides(mut counter: ResMut<CounterSettings>) {
    if let Ok(step) = std::env::var("APP_COUNTER_STEP") {
        if let Ok(step) = step.parse::<i32>() {
            counter.step = step;
        }
    }
}
```

如果你的环境变量层比较复杂，建议引入一个**中间 DTO**，并充分利用 Serde 的 `#[serde(default)]`。Serde 官方文档说明，`#[serde(default)]` 可以在缺字段时回退到 `Default::default()` 或指定函数，这很适合做“文件配置 + 环境变量局部覆盖”的组合。

### 嵌套与复杂类型

官方文档对“复杂类型支持矩阵”没有做完整表格说明，因此这一点在严格意义上应标注为 **“官方细节未完全指定”**。不过，Bevy 当前主分支源码测试里已经覆盖了 `NestedStruct`、单字段/多字段 tuple struct、newtype、以及多种 enum 变体，这说明框架设计目标确实包含复杂类型，而不只是最简单的平面 struct。由于该证据来自发布后的主分支源码，而不是明确锚定为 0.19.0 tag 的文档，所以最谨慎的表述是：**复杂类型大概率可行，但仍建议以你的 0.19.0 实测为准。** 

在工程实践里，若你要保存嵌套设置，建议遵守三个原则。第一，所有嵌套类型都实现 `Default`，这样缺字段时才有可预测的回退。第二，尽量保持字段名稳定，因为 0.19 的反序列化策略会忽略多余字段、保留缺失字段默认值；字段重命名如果没有迁移逻辑，容易留下“旧配置悄悄失效”的问题。第三，枚举的外观最好先根据 Serde 文档明确表示方式，再决定是否要通过中间 DTO 做一层转换。

### JSON 与 YAML 桥接

虽然内建设置系统只直接支持 TOML，但如果你的项目已经有 JSON/YAML 体系，完全可以把这两者当作“输入层”，再把结果转成 App Settings Resource。Serde 官方文档强调，它本身就是“数据结构无关、格式无关”的序列化/反序列化框架，因此同一个 Rust 类型理论上可以复用于 TOML、JSON、YAML 等多个格式。

一种常见做法是：定义一个专门的 `AppConfigDto`，让它负责从 JSON/YAML 读入；随后把 DTO 映射为你的 `CounterSettings`、`DisplayMode` 等资源，再插入世界。例如：

```rust
// Bevy 版本：0.19
// Rust 版本：未指定

// 伪代码：用于说明思路，而非官方内建 API
//
// let text = std::fs::read_to_string("config.json")?;
// let dto: AppConfigDto = serde_json::from_str(&text)?;
// commands.insert_resource(CounterSettings {
//     count: dto.counter.count,
//     step: dto.counter.step,
// });
```

这种做法的优点是，你可以把 `SettingsPlugin` 当作“用户偏好持久化层”，同时保留自己原有的 JSON/YAML 配置入口。代价是你要自行处理路径、重载、错误提示与保存格式一致性。官方在 0.19 中没有替你做这一层。

## 常见问题、迁移指南与最佳实践

### 常见问题与调试技巧

最常见的第一个问题是：**配置文件为什么没生效？** 优先检查四件事：是否显式启用了 `bevy_settings` feature；是否安装了 `SettingsPlugin`；设置类型是否同时满足 `Resource + SettingsGroup + Reflect + Default` 与 `#[reflect(..., Default)]`；你是不是把 `settings.toml` 放到了项目根目录而不是用户配置目录。前两项来自官方示例页与 feature 文档，后两项来自发布说明、源码与平台路径说明。

第二个常见问题是：**首次运行为什么警告 `Filename settings.toml not found`？** 这通常是正常的。源码清楚写着：缺文件时只会给出警告，随后照常把默认值资源插入 `World`。因此首启没有设置文件不是错误，后续你修改设置并保存后，文件才会出现。

第三个问题是：**为什么我改了资源却没有落盘？** 因为官方明确说保存不是自动的。你必须自己在修改后排入 `SaveSettingsDeferred`、`SaveSettings` 或 `SaveSettingsSync` 命令。如果你没有手动发保存命令，设置仅仅是 ECS 资源发生了变化，并不会自动写回磁盘。

第四个问题是：**为什么某些配置字段拼错了，程序也没报错？** 当前 0.19 的设计就是“缺字段保留默认值、额外字段忽略”。这提升了兼容性，但也会降低“拼错字段立即炸”的显式度。实战里最有用的调试办法，是在启动时 `println!`/`info!` 打印最终加载的 Resource 内容，并在保存后检查生成的 TOML。

### 从 0.18 或无 App Settings 项目迁移

如果你是从 **Bevy 0.18** 迁移过来，最关键的事实是：**App Settings 是 0.19 新增能力**。官方 0.19 发布说明把它列为新增亮点之一，而 0.18 到 0.19 迁移指南并没有给出专门的“旧设置 API 迁移条目”，这说明它更像是一个**新增工具**，而不是一个对旧项目强制替换的破坏性变更。

对 0.18 项目，推荐的迁移步骤通常是：

1. 在 `Cargo.toml` 里显式开启 `bevy_settings`。  
2. 为原本手写持久化的“用户偏好资源”补上 `SettingsGroup + Reflect + Default`。  
3. 安装 `SettingsPlugin::new("你的反向域名")`，并把它放到依赖设置的插件之前。  
4. 把你原来“每改一下立刻写文件”的逻辑改成 `SaveSettingsDeferred` 去抖保存。  
5. 在退出路径补一个 `SaveSettingsSync::IfChanged`。

如果你的老项目原本使用 JSON/YAML 手写配置，不建议一次性全部迁移成由 `SettingsPlugin` 直接接管。更现实的做法是：把“用户偏好”这部分迁到 App Settings；把“部署配置/内容配置/服务端配置”继续留在 JSON/YAML；必要时在启动阶段把外部配置覆盖到 Resource 上。这样迁移成本最低，也最符合 Bevy 官方对“用户偏好型设置”的定位。

### 最佳实践与安全注意事项

最佳实践里最重要的一条，是**把 App Settings 当作“用户偏好层”而不是“万能配置系统”**。音量、窗口大小、UI 布局、是否显示提示，这些都很适合；而敏感凭据、访问令牌、私钥、数据库口令、联网服务密钥，不适合放在这里。原因并不复杂：官方内建后端是纯 TOML 文件或浏览器 `localStorage`，官方资料没有提供加密、密钥托管或权限隔离 API；因此把秘密信息塞进 App Settings，本质上是在把敏感数据放进普通明文偏好存储。这里的“不要存秘密”是基于官方存储介质描述作出的工程推论。

第二条最佳实践，是**显式分文件、按职责分组**。官方 `SettingsGroup` 支持 `file`、`group` 与 `key` 自定义；因此你完全可以把 `audio.toml`、`video.toml`、`ui.toml` 拆开。这样做的好处是：冲突更少、调试更清晰、未来迁移也更容易。

第三条，是**平时用 `SaveSettingsDeferred`，退出前再用 `SaveSettingsSync::IfChanged`**。这是官方文档直接推荐的组合：前者避免高频改动反复写盘，后者保证退出前尽量落盘；并且考虑到某些退出路径可能不给你机会拦截事件，双保险比只用其中一种可靠。

第四条，是**把插件顺序设计好**。由于 `SettingsPlugin` 在构建时立即加载，如果你有窗口插件、音频插件、主题插件或自定义 UI 插件依赖这些值，就应当用“设置资源 → 胶水系统 → 目标插件/系统”的方式组织顺序。官方文档甚至专门举了窗口尺寸/位置需要由“桥接插件”复制到窗口实体上的例子。

### 关键参考链接

以下资料是本文最重要的原始依据。优先给出官方/原始来源；其中中文官方资料目前主要是学习入口与既有中文站点，App Settings 本身的 0.19 一手资料仍以英文官方为主。

- Bevy 0.19 官方发布说明中的 **App Settings** 章节。
- `bevy_settings` 0.19.0 crate 文档主页。
- `SettingsPlugin` 官方 API 文档。
- `SettingsGroup` 官方 API 文档。
- Bevy 官方示例页 `Application / Settings`。
- Bevy 0.18 → 0.19 迁移指南总页。
- Bevy Project Goal / Issue：**Bevy User Settings Framework**。
- GitHub Issue：**Settings: Make serialization and deserialization more consistent**，用于理解 0.19 当前缺字段/多字段与序列化不对称的已知限制。
- Serde 官方文档：Overview、derive、default、enum representations。
