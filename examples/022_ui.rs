//! 022 · UI 布局
//!
//! 运行：`cargo run --example 022_ui`
//!
//! 新增概念
//!   Node                UI 元素的布局属性：宽高、内外边距、flex 方向、绝对定位
//!   px(..) / percent(..)  长度单位：逻辑像素 / 父容器的百分比
//!   Text / TextFont / TextColor   文字
//!   ImageNode           在 UI 里显示图片
//!   BackgroundColor / BorderColor / BorderColor   底色与边框
//!   ZIndex              层叠顺序（UI 内部的 z）
//!
//! 使用场景
//!   分数、血条、提示文字、菜单面板 —— 一切"贴在屏幕上"的东西
//!
//! 注意：**UI 用的是另一套坐标系** —— 原点在**左上角**、**Y 轴向下**、长度按像素算。
//!       这跟 003 讲的精灵世界坐标（原点在中心、Y 向上）正好相反

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "022 · UI 布局：左上图片 / 右上文字 / 底部血条".into(),
                resolution: (960, 640).into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<Health>()
        .add_systems(Startup, setup)
        .add_systems(Update, (drain_health, update_hud).chain())
        .run();
}

/// 血量，0~100。UI 只负责显示它。
#[derive(Resource)]
struct Health(f32);

impl Default for Health {
    fn default() -> Self {
        Self(100.0)
    }
}

#[derive(Component)]
struct HealthBarFill;

#[derive(Component)]
struct HealthText;

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    // 注意：UI 不需要相机也能显示吗？—— 需要。UI 默认渲染到"主相机"上，
    // 没有相机就没有渲染目标，所以这里照旧要一台 2D 相机。
    commands.spawn(Camera2d);

    // ── ① 左上角：一张图片 ──
    // `ImageNode` 是 UI 版的"图片组件"（`Sprite` 是世界空间版的）。
    // `position_type: Absolute` 让它脱离 flex 流，按 top/left 直接定位。
    commands.spawn((
        ImageNode::new(assets.load("textures/logo.png")),
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
            width: px(64),
            height: px(64),
            ..default()
        },
    ));

    // ── ② 右上角：文字 ──
    // 用 `right` 而不是 `left` 来定位，这样窗口变宽时它会贴着右边。
    commands.spawn((
        HealthText,
        Text::new("HP 100"),
        TextFont {
            font_size: FontSize::Px(28.0),
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.95, 0.98)),
        Node {
            position_type: PositionType::Absolute,
            top: px(24),
            right: px(24),
            ..default()
        },
    ));

    // ── ③ 底部居中：血条 ──
    // 这一块**不用**绝对定位，而是用 flex 布局把它推到屏幕底部中间 ——
    // 这正是 UI 与精灵最大的不同：UI 是"排版"，不是"摆坐标"。
    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            // 主轴纵向，从下往上排 => 子元素被推到屏幕底部
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexEnd,
            // 交叉轴居中 => 水平居中
            align_items: AlignItems::Center,
            padding: UiRect::bottom(px(36)),
            ..default()
        },
        children![(
            // 血条外框
            Node {
                width: px(360),
                height: px(26),
                border: UiRect::all(px(2)),
                padding: UiRect::all(px(2)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.85, 0.85, 0.90)),
            BackgroundColor(Color::srgb(0.13, 0.13, 0.16)),
            children![(
                // 血条填充：宽度用**百分比**，改一个数就能表现血量
                HealthBarFill,
                Node {
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.85, 0.25, 0.28)),
            )],
        )],
    ));

    println!("── UI 已铺好：左上图片、右上文字、底部血条（flex 推到中间）");
}

/// 血量每 0.4 秒掉一点，掉到 0 就回满 —— 让画面一直有变化。
fn drain_health(time: Res<Time>, mut health: ResMut<Health>, mut timer: Local<Timer>) {
    if timer.is_finished() {
        *timer = Timer::from_seconds(0.4, TimerMode::Repeating);
    }
    if timer.tick(time.delta()).just_finished() {
        health.0 -= 15.0;
        if health.0 <= 0.0 {
            health.0 = 100.0;
        }
    }
}

/// 只在血量**变过**的时候才去改 UI（015 讲的 `is_changed`）—— 每帧无脑刷新是浪费。
fn update_hud(
    health: Res<Health>,
    mut fills: Query<&mut Node, With<HealthBarFill>>,
    mut texts: Query<&mut Text, With<HealthText>>,
) {
    if !health.is_changed() {
        return;
    }
    for mut node in &mut fills {
        // 百分比宽度就是血条的全部秘密：把数值直接当成宽度百分比
        node.width = percent(health.0.clamp(0.0, 100.0));
    }
    for mut text in &mut texts {
        text.0 = format!("HP {:.0}", health.0);
    }
}

// ─────────────────────────────────────────────────────────────────────
// 跑起来会看到什么
//
//   左上角：一张 64×64 的小图片（就是 020 加载的那个 logo）
//   右上角：文字「血量 xx」，每 0.4 秒跟着数字变一次
//   底部中间：一条血条，红色填充随血量伸缩；掉到 0 会回满再来一轮
//
// 三块东西用的三种定位方式，正好覆盖 UI 的常见套路：
//   图片   `position_type: Absolute` + `left/top` —— 钉死在某个角
//   文字   `position_type: Absolute` + `right/top` —— 钉在另一个角
//   血条   交给 flex（纵向排列 + 主轴推到末端 + 交叉轴居中）
//
// ─────────────────────────────────────────────────────────────────────
// ★ UI 是另一套坐标系（这条最容易错）
//
//   世界坐标（003 讲的精灵 / 相机 / Transform）
//     原点在**屏幕中心**，X 向右、Y **向上**，单位是"世界单位"
//
//   UI 坐标（本讲的 Node / Text）
//     原点在**左上角**，X 向右、Y **向下**，单位是**逻辑像素**
//
// 所以同一个"往上"：世界坐标里 y 增大，UI 里却是 y 减小（比如 `bottom` 变小）。
// 从精灵切到 UI 时，脑子里要换一把尺子。
//
// 顺带一提：`px(..)` 是逻辑像素，会按系统 DPI 缩放自动换算，
// 所以高分屏上不会变糊也不会变小 —— 这也是 UI 用像素而不是世界单位的原因。
//
// ─────────────────────────────────────────────────────────────────────
// flex 布局：UI 是"排版"不是"摆坐标"
//
// Bevy 的 UI 用 flexbox（和网页 CSS 同一套模型），常用属性：
//
//   `flex_direction`    主轴方向：Row（默认，横向）/ Column（纵向）
//   `justify_content`   主轴对齐：FlexStart / Center / FlexEnd / SpaceBetween ..
//   `align_items`       交叉轴对齐：FlexStart / Center / FlexEnd / Stretch
//   `width` / `height`  尺寸，可用 `px(..)` 写死或 `percent(..)` 跟父容器
//   `padding` / `margin`  内边距 / 外边距，用 `UiRect::all(px(4))` 这样的写法
//   `position_type`     Absolute 可以让元素脱离排版流、回到"按坐标钉"
//
// 本讲底部那条血条就是典型用法：外层铺满全屏做容器，用
// `Column + FlexEnd + Center` 把血条推到"底部居中"——**没有一个坐标是手算的**，
// 窗口怎么缩放它都在那儿。
//
// ─────────────────────────────────────────────────────────────────────
// ⚠️ 中文字体：Bevy 默认字体**没有中文字形**
//
// 本讲 UI 上的文字全是英文，这不是偷懒，而是**必须**：
// Bevy 内置的默认字体是 `FiraMono-subset.ttf`（拉丁字母等宽字体），
// **不含任何汉字字形**。直接写 `Text::new("血量")` 的结果是：
//
//   · 终端刷一片警告（实测）：
//       ICU4X data error: No segmentation model for complex script: Chinese/Japanese
//   · 画面上那几个字根本渲染不出来（空白或豆腐块）
//
// 这不属于"配置一下就好"，而是**必须自己提供一份中文字体**：
//
//   let font = asset_server.load("fonts/某中文字体.ttf");
//   commands.spawn((
//       Text::new("血量 100"),
//       TextFont {
//           font: font.clone(),            // ← 换成自己的字体
//           font_size: FontSize::Px(28.0),
//           ..default()
//       },
//   ));
//
// 字体从哪来（都不用下载）：
//   · **系统字体**：开 `system_font_discovery` feature，用
//     `Font::from_system(..)` 引用系统里已有的中文字体，不必把字体文件放进仓库
//   · 自己放一份进 `assets/fonts/`（注意字体授权）
//
// 本项目为了不引入二进制依赖，UI 文本统一用英文。
// 但要分清一件事：**控制台的 `println!` 完全不受影响** —— 那是终端在渲染。
//
// 一句话：**`println!` 的字由终端画，`Text` 的字由 Bevy 画。**
// 前者什么语言都行，后者要求字体里有对应字形。
//
// ─────────────────────────────────────────────────────────────────────
// 几个实际会用到的点
//
// · **UI 需要相机**。UI 默认渲染到主相机上，场景里没有相机会什么都看不到。
//   多相机时可以用 `UiTargetCamera` 指定"这套 UI 渲染到哪台相机"。
// · **`ZIndex` 管的是 UI 内部的层叠**，跟精灵的 `Transform.z` 是两套体系，不能互相压。
//   想让整块 UI 盖在世界之上，UI 本身就在世界之上（UI 始终渲染在最上层）。
// · **改文字就是改 `String`**：`text.0 = format!(..)` —— `Text` 就是个 `String` 包装。
// · **别每帧无脑刷新 UI**。布局重排是有成本的，用 015 的变更检测
//   （像本讲的 `update_hud` 那样先问一句 `is_changed()`）能省下大量重复计算。
// ─────────────────────────────────────────────────────────────────────
