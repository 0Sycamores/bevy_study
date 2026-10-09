//! 026 · Gizmos 调试绘制
//!
//! 运行：`cargo run --example 026_gizmos`
//!
//! 新增概念
//!   Gizmos             系统参数，提供画线 / 圆 / 矩形 / 箭头等方法
//!   GizmoConfigStore   全局配置：线宽、是否开启、渲染层级
//!   只画一帧           Gizmos **不留痕**，所以必须每帧重画
//!
//! 使用场景
//!   把看不见的东西画出来：碰撞范围、索敌半径、速度向量、寻路路径、网格线
//!
//! 注意：Gizmos 走的是**渲染管线之外的调试路径** —— 它没有实体、不受精灵层级影响，
//!       也**不能** `despawn` 掉（压根没有实体可销毁），只能靠"这一帧不画"来让它消失

use bevy::gizmos::config::{DefaultGizmoConfigGroup, GizmoConfigStore};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "026 · Gizmos：索敌圈 / 速度箭头 / 世界边界".into(),
                resolution: (960, 640).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup, configure_gizmos))
        // 先更新位置与方向，再画 —— 否则箭头可能显示的是上一帧的方向
        .add_systems(Update, (drift, draw_debug).chain())
        .run();
}

/// 索敌半径。这个数据在画面上"看不见"，正是 Gizmos 要解决的问题。
#[derive(Component)]
struct Sight(f32);

/// 漂移方向与速度。也同样看不见，用箭头画出来。
#[derive(Component)]
struct Drift(Vec2);

/// 世界边界的**半宽**（画出来的方框从中心到边线的距离）。
const WORLD_HALF: f32 = 440.0;

/// 哨兵方块边长。
const SENTRY_SIZE: f32 = 36.0;

/// 反弹点：让方块的**边缘**正好贴到边界线上。
///
/// 这个值必须由 `WORLD_HALF` 和 `SENTRY_SIZE` **算出来**，不能另写一个数 ——
/// 逻辑边界和画出来的边界一旦各写各的，就会悄悄错开，画面上很难发现。
/// 这正是 Gizmos 存在的理由：把看不见的边界画出来，不一致才看得见。
const BOUNCE_AT: f32 = WORLD_HALF - SENTRY_SIZE / 2.0;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // 三个"哨兵"，各自有不同大小的索敌半径与漂移方向。
    let specs = [
        (-220.0, -80.0, 150.0, Vec2::new(60.0, 25.0)),
        (60.0, 120.0, 220.0, Vec2::new(-40.0, 55.0)),
        (260.0, -140.0, 110.0, Vec2::new(-70.0, -30.0)),
    ];
    for (x, y, radius, drift) in specs {
        commands.spawn((
            Sprite::from_color(Color::srgb(0.35, 0.62, 0.95), Vec2::splat(SENTRY_SIZE)),
            Transform::from_xyz(x, y, 0.0),
            Sight(radius),
            Drift(drift),
        ));
    }

    println!("── 三个哨兵：方块是实体，圈和箭头是 Gizmos 另画的");
    println!("   圈 = 索敌半径，箭头 = 漂移速度，外框 = 世界边界");
    println!("   反弹点在 ±{BOUNCE_AT}，方块边缘正好贴到 ±{WORLD_HALF} 的线上");
}

/// 调整 Gizmos 的全局参数。本讲只改线宽，让它看得清楚些。
fn configure_gizmos(mut store: ResMut<GizmoConfigStore>) {
    let (config, _) = store.config_mut::<DefaultGizmoConfigGroup>();
    // 默认线宽只有 1 像素，在高分屏上几乎看不见
    config.line.width = 3.0;
}

/// 让哨兵在世界里漂啊漂（碰到边界就反弹）—— 只是给 Gizmos 提供"会动的东西"。
///
/// 反弹判据用的是 `BOUNCE_AT`（= 边界线 − 半个方块），所以撞上时**边缘**贴线，
/// 而不是中心贴在线上、半个身子探出框外。
fn drift(time: Res<Time>, mut sentries: Query<(&mut Drift, &mut Transform)>) {
    for (mut drift, mut transform) in &mut sentries {
        transform.translation += (drift.0 * time.delta_secs()).extend(0.0);

        // 碰到哪个轴就翻转**那个轴**的速度分量，另一个轴保持不变
        if transform.translation.x.abs() > BOUNCE_AT {
            transform.translation.x = transform.translation.x.clamp(-BOUNCE_AT, BOUNCE_AT);
            drift.0.x = -drift.0.x;
            println!("   [反弹] 撞到左右边界，水平速度改为 {:.0}", drift.0.x);
        }
        if transform.translation.y.abs() > BOUNCE_AT {
            transform.translation.y = transform.translation.y.clamp(-BOUNCE_AT, BOUNCE_AT);
            drift.0.y = -drift.0.y;
            println!("   [反弹] 撞到上下边界，垂直速度改为 {:.0}", drift.0.y);
        }
    }
}

/// 每帧重画全部调试图形 —— **这就是 Gizmos 的工作方式**。
///
/// 它不像精灵那样"生成一次就一直在"：上一帧画的东西下一帧自动清空，
/// 所以想让某条线一直显示，就得每帧都画一遍。
fn draw_debug(mut gizmos: Gizmos, sentries: Query<(&Transform, &Sight, &Drift)>) {
    // ① 世界边界：一个矩形。注意 `rect_2d` 的第二个参数是**全宽**（内部会 /2），
    //    所以这里传 `WORLD_HALF * 2.0`，画出来正好经过 ±WORLD_HALF。
    gizmos.rect_2d(
        Vec2::ZERO,
        Vec2::splat(WORLD_HALF * 2.0),
        Color::srgb(0.35, 0.38, 0.45),
    );

    // ② 原点十字：两根线交叉
    gizmos.line_2d(
        Vec2::new(-20.0, 0.0),
        Vec2::new(20.0, 0.0),
        Color::srgb(0.9, 0.9, 0.9),
    );
    gizmos.line_2d(
        Vec2::new(0.0, -20.0),
        Vec2::new(0.0, 20.0),
        Color::srgb(0.9, 0.9, 0.9),
    );

    // ③ 每个哨兵：索敌圈 + 速度箭头
    for (transform, sight, drift) in &sentries {
        let position = transform.translation.truncate();
        let velocity_end = position + drift.0 * 0.6;

        // 索敌半径：画个圈。颜色随半径变化，一眼能区分谁的范围大。
        let color = Color::hsl(sight.0 / 3.0, 0.75, 0.6);
        gizmos.circle_2d(position, sight.0, color);
        // 箭头：从实体指向"速度方向再走一段"，长度正比于速度
        gizmos.arrow_2d(position, velocity_end, Color::srgb(0.95, 0.75, 0.25));
    }
}

// ─────────────────────────────────────────────────────────────────────
// 跑起来会看到什么
//
//   · 三个蓝方块在世界里漂移，**碰到边界会反弹**（速度取反后往回走）
//     —— 反弹判据是"方块**边缘**贴到边线"，所以不会半个身子探出框外
//   · 每个方块外面套一个彩色的圈 —— 那是它的索敌半径（数据本身看不见）
//   · 每个方块伸出一支黄色箭头 —— 那是它的速度方向与大小
//     （反弹时箭头会立刻掉头，因为方向数据真的变了）
//   · 灰白色的外框标出世界边界，中心的十字标出原点
//
// 控制台在每次反弹时打印一行（实测：7 秒时 1 次、14 秒时 5 次；下面是头 4 次）：
//
//   ── 三个哨兵：方块是实体，圈和箭头是 Gizmos 另画的
//      圈 = 索敌半径，箭头 = 漂移速度，外框 = 世界边界
//      反弹点在 ±422，方块边缘正好贴到 ±440 的线上
//      [反弹] 撞到上下边界，垂直速度改为 -55
//      [反弹] 撞到上下边界，垂直速度改为 30
//      [反弹] 撞到左右边界，水平速度改为 70
//      [反弹] 撞到左右边界，水平速度改为 -60
//
// 顺带注意箭头**会越过边界线**而方块不会 —— 箭头画的是"接下来要往哪走"，
// 它的长度是 `|速度| × 0.6`，所以尖端本来就领先于方块本身。
//
// 值得注意的是：**方块是"实体"，圈和箭头不是。**
// 你无法用 `Query` 找到那些圈，也无法 `despawn` 它们。
//
// ─────────────────────────────────────────────────────────────────────
// ★ 为什么必须每帧重画
//
// Gizmos 的图形**只在当前帧存在**，下一帧开始时会全部清空。
// 所以画调试图形时要问的不是"我什么时候生成它"，而是
// **"哪一帧需要看到它"** —— 想一直看到就每帧画。
//
// 这跟精灵的思维正好相反：
//
//   精灵      spawn 一次 → 一直存在 → 想让它消失要 `despawn`
//   Gizmos    每帧画     → 只活一帧 → 想让它消失就**这一帧别画**
//
// 也正因如此，"开关调试显示"只需要一个 `if`：
//
//   fn draw_debug(mut gizmos: Gizmos, debug: Res<DebugFlag>) {
//       if !debug.0 { return; }        // 不画 = 消失，干净利落
//       ...
//   }
//
// ─────────────────────────────────────────────────────────────────────
// 常用的绘制方法
//
//   `line_2d(a, b, color)`              两点连线
//   `circle_2d(center, radius, color)`  空心圆（碰撞半径、索敌范围）
//   `rect_2d(center, size, color)`      矩形（边界、触发区）
//   `arrow_2d(from, to, color)`         箭头（速度、朝向、受力）
//   `linestrip_2d(points, color)`       折线（路径、轨迹）
//
// 3D 版本把 `_2d` 换成 `_3d` 即可（球、立方体、坐标轴…），用法一致。
//
// ─────────────────────────────────────────────────────────────────────
// 全局配置 `GizmoConfigStore`
//
// 本讲把线宽从默认的 1 改成了 3 —— 高分屏上 1 像素的线基本看不见：
//
//   let (config, _) = store.config_mut::<DefaultGizmoConfigGroup>();
//   config.line.width = 3.0;
//
// 常用的还有：
//   `config.enabled`        整体开关（比在每个系统里加 `if` 更省事）
//   `config.line.perspective`  透视相机的线宽是否随距离变化
//   `config.depth_bias`     与场景重叠时的深度偏移（解决 z-fighting）
//
// `config_mut::<G>()` 的第二个返回值是**自定义配置组**：
// 你可以定义 `#[derive(GizmoConfigGroup)]` 的类型，给自己的调试图形单独一套开关，
// 这样"只关掉寻路显示、保留碰撞显示"就很好实现。
//
// ─────────────────────────────────────────────────────────────────────
// 什么时候值得画 Gizmos
//
//   ✅ 碰撞体、触发区、索敌范围 —— 这些数值调错了很难从画面上发现
//   ✅ 速度 / 加速度向量 —— 单位"飘"或者"抖"时一眼看出问题
//   ✅ 寻路路径、AI 目标点
//   ✅ 网格 / 坐标轴 —— 搭建场景时定位用
//
//   ❌ 别拿它做正式美术。Gizmos 是调试工具：线宽、颜色、层级都不适合最终画面，
//      而且每帧重画的成本不低（几百条线没问题，几万条就该换方案了）。
//
// 顺带一提：`TransformGizmo` 那种"拖拽手柄"是**编辑工具**（`bevy_gizmos` 里的
// 交互式 gizmo），跟本讲的"调试绘制"不是一回事，别混。
//
// ─────────────────────────────────────────────────────────────────────
// 到这里，阶段五（2D 表现层）结束
//
// 回头串一下这一阶段：020 把素材从磁盘装进来，021 把图切成帧做成动画，
// 022/023 把界面画出来并能点，024 给它配上声音，025 让镜头能动，
// 026 则是"看不见的东西怎么看见"。
//
// 有了这七讲，一个 2D 游戏该有的表现层零件就齐了 —— 031 的综合游戏会全部用上。
// ─────────────────────────────────────────────────────────────────────
