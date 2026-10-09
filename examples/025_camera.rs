//! 025 · 相机
//!
//! 运行：`cargo run --example 025_camera`
//!
//! 新增概念
//!   Camera2d                 2D 相机
//!   Projection               投影方式；2D 用 `OrthographicProjection`
//!   OrthographicProjection.scale   正交缩放：**越大看得越广**（不是"放大"）
//!   Viewport                 把一台相机的输出限制在屏幕的一块矩形里 —— 分屏靠它
//!   Camera.order             多相机时的绘制顺序
//!
//! 使用场景
//!   跟随玩家、缩放视野、分屏双人、小地图
//!
//! 注意：正交相机的 `scale` 语义容易搞反 —— `scale = 2.0` 是"视野放大两倍"，
//!       也就是**看起来东西变小了**。它与世界坐标的关系见文末

// `Viewport` 与 `AccumulatedMouseScroll` 都不在 prelude 里。
use bevy::camera::Viewport;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "025 · 相机：左=1x / 右=缩小的同一场景；滚轮调左相机".into(),
                resolution: (960, 640).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (zoom_left_camera, show_camera_state))
        .run();
}

/// 想要倍率随滚轮变化的相机。
#[derive(Component)]
struct ZoomableCamera;

/// 用来显示"当前 zoom 是多少"的文本。
#[derive(Component)]
struct ZoomText;

/// 拿来做参照物的方块：铺一片网格，这样缩放/移动一眼就能看出来。
fn setup(mut commands: Commands) {
    // ── 世界内容：一片彩色网格 ──
    // 网格是"被看的东西"，用来让相机效果可见；这里没有任何游戏逻辑。
    for y in -3..=3 {
        for x in -4..=4 {
            let hue = ((x + 4) as f32 / 8.0) * 300.0;
            commands.spawn((
                Sprite::from_color(Color::hsl(hue, 0.55, 0.55), Vec2::splat(56.0)),
                Transform::from_xyz(x as f32 * 80.0, y as f32 * 80.0, 0.0),
            ));
        }
    }

    // ── 左半边：1 倍视野的相机 ──
    //
    // `Viewport` 的两个字段是**物理像素**：位置 + 尺寸。
    // 960 宽的窗口从中间切开，就是左右各 480。
    commands.spawn((
        Camera2d,
        ZoomableCamera,
        // ★ 这一行不是可有可无的：场景里有**两台**相机时，UI 默认归 order
        //   最大的那台（也就是右边那台）管，于是左上角那行文字会跑到**右半边**
        //   的左上角去。`IsDefaultUiCamera` 明确指定"UI 归我"，它才会出现在
        //   窗口真正的左上角（细节见文末「Viewport 分屏的要点」）。
        IsDefaultUiCamera,
        Camera {
            // 左半边
            viewport: Some(Viewport {
                physical_position: UVec2::new(0, 0),
                physical_size: UVec2::new(480, 640),
                ..default()
            }),
            // 多相机同屏时，order 大的后画（也就是画在上面）
            order: 0,
            ..default()
        },
        // 正交投影，scale = 1.0 是"1 世界单位 1 像素"的原始视野
        Projection::Orthographic(OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        }),
    ));

    // ── 右半边：视野放大 2 倍（看起来东西小一半）──
    commands.spawn((
        Camera2d,
        Camera {
            viewport: Some(Viewport {
                physical_position: UVec2::new(480, 0),
                physical_size: UVec2::new(480, 640),
                ..default()
            }),
            order: 1,
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            // ⚠️ scale 是"视野的放大倍数"，不是"物体的放大倍数"。
            //    2.0 表示视野宽高各扩到 2 倍 => 画面里的东西看起来小了一半。
            scale: 2.0,
            ..OrthographicProjection::default_2d()
        }),
    ));

    // 左上角显示 zoom 数值（UI 文本用英文，原因见 022 讲的中文字体说明）
    // 注意它显示的是**左相机**的 scale，所以上面给左相机加了 `IsDefaultUiCamera`，
    // 让这行 UI 跟着左半边的 viewport 排版。
    commands.spawn((
        ZoomText,
        Text::new("left camera scale: 1.00"),
        TextFont {
            font_size: FontSize::Px(20.0),
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.95, 0.98)),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));

    println!("── 左相机 scale=1.0，右相机 scale=2.0（视野两倍宽）");
    println!("   滚轮调整左相机；两半是**同一份世界**，只是相机不同");
}

/// 滚轮缩放左相机。
///
/// ⚠️ 正交相机的 `scale` 有个容易搞反的地方：它描述的是**视野**，不是物体。
/// 想让东西看起来变大，要**减小** scale。
fn zoom_left_camera(
    scroll: Res<AccumulatedMouseScroll>,
    mut cameras: Query<&mut Projection, With<ZoomableCamera>>,
) {
    let dy = scroll.delta.y;
    if dy == 0.0 {
        return;
    }

    for mut projection in &mut cameras {
        let Projection::Orthographic(ortho) = &mut *projection else {
            continue;
        };
        // 向上滚（dy > 0）=> 缩小 scale => 看得更近（东西变大）
        ortho.scale = (ortho.scale * (1.0 - dy * 0.08)).clamp(0.25, 4.0);
    }
}

/// 把左相机当前的 scale 显示到屏幕上。
fn show_camera_state(
    cameras: Query<&Projection, With<ZoomableCamera>>,
    mut texts: Query<&mut Text, With<ZoomText>>,
) {
    for projection in &cameras {
        let Projection::Orthographic(ortho) = projection else {
            continue;
        };
        for mut text in &mut texts {
            let wanted = format!("left camera scale: {:.2}", ortho.scale);
            if text.0 != wanted {
                text.0 = wanted;
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// 跑起来会看到什么
//
//   · 窗口左右各一半，**显示的是同一份世界**
//   · 左半边是 1 倍视野，右半边视野宽两倍（同样的方块看起来小一半）
//   · 滚轮滚动 → 左半边缩放，左上角文字实时显示当前 scale
//   · 右半边始终不变（它没有 `ZoomableCamera` 标记）
//
// ─────────────────────────────────────────────────────────────────────
// ★ `scale` 是"视野"，不是"物体"
//
// 这是正交相机最容易搞反的一点：
//
//   scale = 0.5  视野只有一半宽 → 画面里的东西**变大**两倍
//   scale = 1.0  原始视野
//   scale = 2.0  视野宽两倍     → 画面里的东西**变小**一半
//
// 直观理解：`scale` 是"照相机往后退了几倍"。
// 后退 → 看到更多 → 每个东西显得更小。所以想"放大画面"要**减小** scale。
//
// 判断当前该乘还是该除，有个稳妥的办法：**先问自己"视野该变大还是变小"**。
//
// ─────────────────────────────────────────────────────────────────────
// `Viewport` 分屏的要点
//
// `Viewport { physical_position, physical_size }` 用的都是**物理像素**，
// 所以写死 `480` 只在 960 宽的窗口下正确。真实项目里应该按窗口实际尺寸算：
//
//   let window = windows.single()?;
//   let half = window.physical_width() / 2;
//
// 另外两件事：
//   · **每台相机都要有 `Camera2d`（或 `Camera3d`）**，`Camera` 组件只是配置。
//     只写 `Camera { viewport: .. }` 而没有 `Camera2d` 是不行的。
//   · 多相机同屏时用 `Camera.order` 决定谁画在上面（值大的后画）。
//   · UI 默认渲染到**默认 UI 相机**上，而"默认"的判定是：**有
//     `IsDefaultUiCamera` 标记就用它，否则用 order 最大的那台**（不是"第一台"）。
//     本讲两台相机的 order 是 0 和 1，所以不加标记的话，左上角那行文字会跟着
//     **右半边**的 viewport 排版 —— 这就是本讲给左相机加标记的原因。
//   · 想只给**某一个** UI 根节点换相机，用 `UiTargetCamera` 目标组件（022 提过）。
//
// ─────────────────────────────────────────────────────────────────────
// 相机跟随的两种做法
//
// 本讲没现场演示"让相机追着某个东西"，但两种做法的取舍值得先记下：
//
//   ① **把相机挂成目标的子实体**（014 讲的层级）
//        commands.spawn(player).with_children(|p| { p.spawn(Camera2d); });
//      优点：一行搞定，变换继承自动帮你算
//      缺点：**生硬** —— 目标动多快相机就动多快，画面会跟着抖
//
//   ② **每帧插值逼近目标**（自己写平滑）
//        camera.translation = camera.translation.lerp(target, 0.1);
//      优点：有"跟随感"，快速移动时画面不会抽
//      缺点：要自己处理边界（别让相机露出地图外）
//
// 绝大多数游戏要的是 ② —— "平滑跟随"是手感的一部分，而不只是实现细节。
// 两种都用得上，常见组合是"子实体定基准位置 + 插值做平滑"。
//
// ─────────────────────────────────────────────────────────────────────
// 顺带一提：世界坐标 ⇄ 屏幕坐标
//
// 011 讲光标时用到过 `Camera::viewport_to_world_2d(..)`，反向的是
// `Camera::world_to_viewport(..)`。这两个在做"点击选单位""血条跟着单位跑"
// 时是必需的，因为 UI 在屏幕空间、实体在世界空间，中间必须靠相机换算。
//
// 相机还能做：多相机叠加（小地图 = 第二台相机 + `Viewport` + 更高 `order`）、
// 每台相机不同的 `ClearColorConfig`（002 提过）。
// ─────────────────────────────────────────────────────────────────────
