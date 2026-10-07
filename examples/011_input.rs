//! 011 · 键盘与鼠标输入
//!
//! 运行：`cargo run --example 011_input`
//!
//! 新增概念
//!   ButtonInput<KeyCode>      键盘，按**物理位置**匹配（W 就是 W 那个位置）
//!   ButtonInput<Key>          键盘，按**实际字符**匹配（问号在哪个键上都算问号）
//!   ButtonInput<MouseButton>  鼠标键
//!   AccumulatedMouseMotion / AccumulatedMouseScroll   这一帧移动 / 滚动了多少
//!   pressed / just_pressed / just_released            按住每帧 / 按下那帧 / 松开那帧
//!
//! 使用场景
//!   键盘移动用 KeyCode、符号快捷键用 Key；鼠标点击、视角旋转、滚轮缩放
//!
//! 注意："按一次做一件事"必须用 just_pressed；用 pressed 会变成**每秒帧率次**，且不报错

use bevy::input::keyboard::Key;
// 这两个也不在 prelude 里。
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "011 · 输入：移动鼠标 / 按住左键 / 按空格、? 、+".into(),
                resolution: (960, 640).into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<Stats>()
        .insert_resource(ReportTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                // 这几个都要读写 `Stats`（或与它串行），chain 起来保证顺序确定（004 讲的）。
                (
                    cursor_follows_mouse,
                    collect_mouse_accumulators,
                    track_space_rate,
                    report_second,
                )
                    .chain(),
                // 下面几个是只读的离散事件，只在那"一帧"打印，互不冲突。
                print_key_events,
                print_char_key_events,
                print_mouse_button_events,
            ),
        )
        .run();
}

/// 跟随光标的方块。
#[derive(Component)]
struct CursorMarker;

/// 每秒汇总用的累计量。
#[derive(Resource, Default)]
struct Stats {
    space_just: u32,
    space_pressed: u32,
    motion: Vec2,
    scroll: f32,
}

#[derive(Resource)]
struct ReportTimer(Timer);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        CursorMarker,
        Sprite::from_color(Color::srgb(0.35, 0.62, 0.95), Vec2::splat(40.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    println!("── 操作提示 ──");
    println!("  移动鼠标：方块跟随（顺便看每秒那行里的「屏幕 → 世界」坐标换算）");
    println!("  按住左键：方块变橙色");
    println!("  按住空格：观察 just_pressed 与 pressed 的次数差别");
    println!("  按 ? 或 + ：体验 Key（按字符匹配）与 KeyCode（按位置匹配）的不同");
    println!();
}

// ── 鼠标：位置、移动量、滚轮 ────────────────────────────────────────────

/// 把光标位置从**屏幕坐标**换算成**世界坐标**，让方块跟着光标走。
///
/// 这一行代码正好让 003 讲过的两套坐标正面相遇：
///   · `window.cursor_position()` 给的是**屏幕坐标**（左上角原点、Y 向下）
///   · `Transform` 用的是**世界坐标**（屏幕中心原点、Y 向上）
/// `Camera::viewport_to_world_2d` 就是这次换算的桥。
fn cursor_follows_mouse(
    window: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut markers: Query<(&mut Transform, &mut Sprite), With<CursorMarker>>,
) {
    let Ok((mut transform, mut sprite)) = markers.single_mut() else {
        return;
    };

    // 按键状态直接驱动视觉 —— 这就是"输入"最直接的用途。
    sprite.color = if mouse.pressed(MouseButton::Left) {
        Color::srgb(0.91, 0.55, 0.25)
    } else {
        Color::srgb(0.35, 0.62, 0.95)
    };

    // 三个都可能"暂时没有"：窗口没了、光标移出窗口、相机还没就绪。
    // 注意这里用 `single_mut()` 自己处理 Err，而不是 `Single` 参数 ——
    // 后者在条件不满足时会跳过整个系统（006 讲的），这里不适合。
    let Ok(window) = window.single() else {
        return;
    };
    let Some(cursor_screen) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera.single() else {
        return;
    };
    if let Ok(world) = camera.viewport_to_world_2d(camera_transform, cursor_screen) {
        transform.translation = world.extend(0.0);
    }
}

/// 鼠标的**连续**输入不在 `ButtonInput` 里，而是两个独立的资源：
///   · `AccumulatedMouseMotion`   这一帧光标移动了多少（dx / dy）
///   · `AccumulatedMouseScroll`   这一帧滚轮滚了多少（`delta.y`，另有 `unit` 区分行/像素）
/// 它们是"增量"，每帧都会重置，所以要自己累加才能做每秒统计。
fn collect_mouse_accumulators(
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut stats: ResMut<Stats>,
) {
    stats.motion += motion.delta;
    stats.scroll += scroll.delta.y;
}

// ── 键盘：两种查询语义 ──────────────────────────────────────────────────

/// 同时统计 `just_pressed` 和 `pressed` 的触发次数，用来量化两者的差别。
fn track_space_rate(keyboard: Res<ButtonInput<KeyCode>>, mut stats: ResMut<Stats>) {
    // 只在"按下那一帧"为真
    if keyboard.just_pressed(KeyCode::Space) {
        stats.space_just += 1;
    }
    // 按住期间"每帧"都为真
    if keyboard.pressed(KeyCode::Space) {
        stats.space_pressed += 1;
    }
}

/// 键盘的**离散**事件：只在按下/松开的那一帧打印一次。
///
///   `just_pressed(..)`   按下那一帧      → 单次动作
///   `just_released(..)`  松开那一帧      → 松手动作
///   `pressed(..)`        按住期间每帧    → 持续动作
fn print_key_events(keyboard: Res<ButtonInput<KeyCode>>) {
    const WATCHED: [(KeyCode, &str); 4] = [
        (KeyCode::Space, "空格"),
        (KeyCode::ArrowUp, "↑"),
        (KeyCode::ArrowDown, "↓"),
        (KeyCode::Enter, "回车"),
    ];
    for (key, name) in WATCHED {
        if keyboard.just_pressed(key) {
            println!("【键盘 KeyCode】{name} 按下");
        }
        if keyboard.just_released(key) {
            println!("【键盘 KeyCode】{name} 松开");
        }
    }
}

/// `ButtonInput<KeyCode>` 按**物理位置**匹配，`ButtonInput<Key>` 按**实际字符**匹配。
///
/// 这里查的是 "?" "+" "-" 三个字符 —— 它们在键盘上的位置随布局而变
/// （美式键盘 ? 在 / 上，德语键盘在 ß 上），但"玩家想打出 ?"这件事不变。
/// 所以**符号类快捷键用 `Key`**；而 WASD 那种"手感位置"的绑定要用 `KeyCode`。
fn print_char_key_events(key_input: Res<ButtonInput<Key>>) {
    for ch in ["?", "+", "-"] {
        if key_input.just_pressed(Key::Character(ch.into())) {
            println!("【键盘 Key】字符 {ch} 按下（不管它在键盘哪个位置）");
        }
    }
}

// ── 鼠标按键 ────────────────────────────────────────────────────────────

/// 鼠标键和键盘完全同一套 API，只是资源类型换成 `ButtonInput<MouseButton>`。
fn print_mouse_button_events(mouse: Res<ButtonInput<MouseButton>>) {
    const BUTTONS: [(MouseButton, &str); 3] = [
        (MouseButton::Left, "左键"),
        (MouseButton::Right, "右键"),
        (MouseButton::Middle, "中键"),
    ];
    for (button, name) in BUTTONS {
        if mouse.just_pressed(button) {
            println!("【鼠标】{name} 按下");
        }
        if mouse.just_released(button) {
            println!("【鼠标】{name} 松开");
        }
    }
}

// ── 每秒汇总 ────────────────────────────────────────────────────────────

/// 用 010 讲的 `Timer` 把刷屏的输入压成每秒一行。
fn report_second(
    time: Res<Time>,
    mut timer: ResMut<ReportTimer>,
    mut stats: ResMut<Stats>,
    window: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let cursor_screen = window
        .single()
        .ok()
        .and_then(|window| window.cursor_position());
    let cursor_world = cursor_screen.and_then(|position| {
        let (camera, camera_transform) = camera.single().ok()?;
        camera.viewport_to_world_2d(camera_transform, position).ok()
    });

    println!(
        "【每秒】空格 just_pressed {} 次 / pressed {} 次   鼠标移动 ({:+.1}, {:+.1})   滚轮 {:+.1}   光标 屏幕 {} → 世界 {}",
        stats.space_just,
        stats.space_pressed,
        stats.motion.x,
        stats.motion.y,
        stats.scroll,
        cursor_screen.map_or("窗口外".to_string(), |p| format!(
            "({:.0}, {:.0})",
            p.x, p.y
        )),
        cursor_world.map_or("—".to_string(), |p| format!("({:.0}, {:.0})", p.x, p.y)),
    );

    stats.space_just = 0;
    stats.space_pressed = 0;
    stats.motion = Vec2::ZERO;
    stats.scroll = 0.0;
}

// ─────────────────────────────────────────────────────────────────────
// 输入从哪来：五个资源 / 两类形态
//
//   Res<ButtonInput<KeyCode>>        键盘，按物理位置        ← 移动键绑定用
//   Res<ButtonInput<Key>>            键盘，按实际字符        ← 符号快捷键用
//   Res<ButtonInput<MouseButton>>    鼠标键
//   Res<ButtonInput<GamepadButton>>  手柄键（另有 Gamepad 轴）
//   Res<AccumulatedMouseMotion>      鼠标这一帧移动了多少
//   Res<AccumulatedMouseScroll>      鼠标这一帧滚轮滚了多少
//
// 形态上分两类，别混：
//   · **离散**（按键）：用 `ButtonInput` 查询"这一刻的状态"
//   · **连续**（移动、滚轮）：用 `Accumulated*` 拿"这一帧的增量"，自己累加
//
// ─────────────────────────────────────────────────────────────────────
// 按下有"三种语义"，这是本讲的核心
//
//   `pressed(..)`        按住期间**每帧**为真   → 持续动作：加速、蓄力、拖拽
//   `just_pressed(..)`   只在**按下那一帧**为真  → 单次动作：跳跃、开菜单
//   `just_released(..)`  只在**松开那一帧**为真  → 松手动作：释放蓄力、结束拖拽
//
// 实测（固定帧长精确复现：按住空格 60 帧、每帧恰好 1/60 秒）：
//
//   just_pressed 触发 1 次
//   pressed      触发 60 次
//
// 那个 **60 就是帧率**。实际跑起来，按住空格不放时每秒那行会是这样：
//
//   【每秒】空格 just_pressed 1 次 / pressed 60 次   ...
//   【每秒】空格 just_pressed 0 次 / pressed 61 次   ...
//   【每秒】空格 just_pressed 0 次 / pressed 59 次   ...
//
// 第一秒是 1 次、之后每秒都是 0 次 —— 因为 `just_pressed` 只在按下那一帧为真，
// 一直按住不会再触发。而 `pressed` 每帧都真，所以数字跟着帧率走。
//
// **所以：所有"按一次做一件事"的逻辑都必须用 `just_pressed`。**
// 用 `pressed` 的后果不是报错，而是让帧率悄悄决定游戏平衡 ——
// 换台更快的机器，行为就变了。这类 bug 不崩溃、不报警，只是"数值不对"，
// 属于 004 讲的同一类隐性错误。
//
// 不用手动清理这些状态：装了 `InputPlugin`（`DefaultPlugins` 自带）后，
// Bevy 会在每帧末自动清掉 `just_*`。
//
// ─────────────────────────────────────────────────────────────────────
// Time、Timer 也在这张图里 —— 但只当配角
//
// 本讲为了"每秒汇总一次"用了 `Res<Time>` 和 `Timer`（上面 `report_second`）。
// 它们**不是**本讲主题，只是把刷屏的输入压成可读的一行；细节都在 010。
// 同样，方块跟随光标用到的是"光标 → 世界坐标"这个**输入话题**，
// 相机本身的玩法（跟随、缩放、分屏）是 025。
//
// ─────────────────────────────────────────────────────────────────────
// 一个常见追问：想要"按住连续触发、但速度固定"怎么办
//
// 光把 `pressed` 换成 `just_pressed` 会变成"按住只触发一次"，
// 所以真正的需求（按住连续射、但每秒固定 N 发）既不是 `pressed` 也不是
// `just_pressed` 单独能表达的 —— 它是"首次即时响应 + 之后按固定节奏"：
//
//   按下瞬间 → `just_pressed` 立刻来一发（响应最快）
//   持续按住 → 后续由冷却计时器接管，射速与帧率无关
//
// 那个冷却就是 010 的 `Timer`，组合写法在 031 综合游戏里完整用到。
// 本讲只指出这一点，不想把输入讲成"射击教程"。
// ─────────────────────────────────────────────────────────────────────
