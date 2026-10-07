//! 011 · 键盘与鼠标输入
//!
//! 运行：`cargo run --example 011_input`
//!
//! 操作：**WASD** 移动方块；**按住空格**看射速统计；**鼠标左键**按住换色。
//!
//! 本讲的核心是 `pressed` 与 `just_pressed` 的区别 ——
//! 这是 Bevy 新手最经典的一个 bug，而且它**不会报错**，只会让射速变成帧率。

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "011 · 输入：WASD 移动 / 按住空格看射速 / 左键换色".into(),
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
                move_player,
                // 这两个系统都写 Stats，会串行执行；chain 只为输出/计数顺序确定。
                (fire_by_just_pressed, fire_by_pressed).chain(),
                report_rate,
                mouse_buttons,
            ),
        )
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Resource, Default)]
struct Stats {
    just_pressed_fires: u32,
    pressed_fires: u32,
}

#[derive(Resource)]
struct ReportTimer(Timer);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Player,
        Sprite::from_color(Color::srgb(0.35, 0.62, 0.95), Vec2::splat(80.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

/// `pressed` 用于**持续按住**的事情：按住 WASD 就一直移动。
/// 位移乘 `delta_secs()`，与帧率无关（010 讲的）。
fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player: Single<&mut Transform, With<Player>>,
) {
    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        // `normalize()` 保证斜着走不会更快
        let speed = 320.0;
        player.translation += (direction.normalize() * speed * time.delta_secs()).extend(0.0);
    }
}

/// ✅ 正确写法：`just_pressed` 只在按下的**那一帧**为真，
/// 按住不放也只会触发一次。
fn fire_by_just_pressed(keyboard: Res<ButtonInput<KeyCode>>, mut stats: ResMut<Stats>) {
    if keyboard.just_pressed(KeyCode::Space) {
        stats.just_pressed_fires += 1;
    }
}

/// ❌ 错误写法：`pressed` 在**按住期间的每一帧**都为真，
/// 所以按住空格一秒就会触发约 60 次（60 FPS 下）。
fn fire_by_pressed(keyboard: Res<ButtonInput<KeyCode>>, mut stats: ResMut<Stats>) {
    if keyboard.pressed(KeyCode::Space) {
        stats.pressed_fires += 1;
    }
}

/// 每秒汇报一次然后清零 —— 这样数字不会无限涨，比率也看得清。
fn report_rate(time: Res<Time>, mut timer: ResMut<ReportTimer>, mut stats: ResMut<Stats>) {
    if timer.0.tick(time.delta()).just_finished() {
        if stats.pressed_fires > 0 || stats.just_pressed_fires > 0 {
            println!(
                "【最近 1 秒】just_pressed 触发 {:>3} 次    pressed 触发 {:>3} 次",
                stats.just_pressed_fires, stats.pressed_fires
            );
        }
        stats.just_pressed_fires = 0;
        stats.pressed_fires = 0;
    }
}

/// 鼠标键和键盘一样用 `ButtonInput<MouseButton>`。
/// `just_released` 只在松开的那一帧为真，适合"松手时才结算"的操作（比如蓄力）。
fn mouse_buttons(
    mouse: Res<ButtonInput<MouseButton>>,
    mut player: Single<&mut Sprite, With<Player>>,
) {
    if mouse.pressed(MouseButton::Left) {
        player.color = Color::srgb(0.91, 0.55, 0.25);
    } else {
        player.color = Color::srgb(0.35, 0.62, 0.95);
    }

    if mouse.just_released(MouseButton::Left) {
        println!("【鼠标左键】松开");
    }
    if mouse.just_pressed(MouseButton::Right) {
        println!("【鼠标右键】按下");
    }
}

// ─────────────────────────────────────────────────────────────────────
// 按住空格时会看到什么
//
//   【最近 1 秒】just_pressed 触发   1 次    pressed 触发  60 次
//   【最近 1 秒】just_pressed 触发   0 次    pressed 触发  61 次
//   【最近 1 秒】just_pressed 触发   0 次    pressed 触发  59 次
//
// 第一秒是 1 次、之后每秒都是 0 次 —— 因为 `just_pressed` 只在按下的那一帧为真，
// 一直按住也不会再触发。而 `pressed` 每帧都触发，所以数字就是帧率。
//
// 又用固定帧长精确复现了一遍（按住空格 60 帧、每帧恰好 1/60 秒）：
//
//   just_pressed 触发 1 次
//   pressed      触发 60 次
//
// 差别一目了然：
//   · `just_pressed` 只在**按下的那一帧**为真 → 按住一秒也只触发 1 次
//   · `pressed`     在**按住期间的每一帧**都为真 → 按住一秒触发约 60 次
//
// 那个 60 就是**帧率**（实测在 59~61 之间浮动，因为帧长不严格等于 1/60 秒）。
// 换句话说：用 `pressed` 做"按一次做一件事"，等于让帧率决定游戏平衡 ——
// 换台更快的机器，射速就变了。
//
// **所有"按一次做一件事"的逻辑都必须用 `just_pressed`。**
//
// ─────────────────────────────────────────────────────────────────────
// 三种"按下/松开"查询
//
//   `pressed(..)`        按住期间每帧为真   → 持续动作：移动、蓄力、加速
//   `just_pressed(..)`   按下那一帧为真      → 单次动作：射击、跳跃、开菜单
//   `just_released(..)`  松开那一帧为真      → 松手动作：蓄力释放、停止拖拽
//
// 不用手动清理它们。装了 `InputPlugin`（`DefaultPlugins` 自带）后，
// Bevy 会在每帧末自动清掉 `just_*` 状态。
//
// ─────────────────────────────────────────────────────────────────────
// 不只键盘
//
//   `ButtonInput<KeyCode>`        按**物理位置**匹配，不随键盘布局变
//   `ButtonInput<Key>`            按**实际字符**匹配：`Key::Character("?".into())`
//   `ButtonInput<MouseButton>`    鼠标键
//   `ButtonInput<GamepadButton>`  手柄
//
// `KeyCode` 与 `Key` 的区别值得说清：
//   · `KeyCode::KeyW` 永远指"W 所在的那个位置" —— 所以**移动键绑定要用 `KeyCode`**，
//     玩家的手在哪儿才是他关心的。
//   · `Key::Character("?")` 指"打出问号"，不管问号在键盘哪个位置 ——
//     所以**符号类快捷键（? 帮助、+/- 缩放）要用 `Key`**。
//
// ─────────────────────────────────────────────────────────────────────
// 鼠标的「移动量」与「光标位置」
//
// 上面只用了鼠标**按键**。鼠标的连续输入是另外两个资源：
//
//   `Res<AccumulatedMouseMotion>`   这一帧的移动增量（dx / dy），做视角旋转用
//   `Res<AccumulatedMouseScroll>`   这一帧的滚轮增量（y），做缩放用
//
// 光标位置则从窗口拿：
//
//   `Query<&Window>` → `window.cursor_position()` → `Option<Vec2>`
//
// ⚠️ 它给出的是**屏幕坐标**：左上角原点、Y 向下 —— 正是 003 讲过的
// "另一套方向相反的坐标"。想变成世界坐标还得过一遍相机，025 讲相机会说。
//
// ─────────────────────────────────────────────────────────────────────
// 那射击到底该怎么写
//
// 只把 `pressed` 换成 `just_pressed` 还不够 —— 那样按住不放只有"按下瞬间"发一发。
// 想要"按住就连续射击、且射速固定"，正确组合是
// **`just_pressed` 首发 + 冷却计时器接管后续**：
//
//   按下瞬间   → 立刻打一发（响应最快）
//   持续按住   → 由 `Timer` 冷却决定射速，与帧率无关
//
// 冷却就是 010 讲的 `Timer`。这套组合旧版 `010_game.rs` 用过，
// 031 综合游戏会完整用到。
// ─────────────────────────────────────────────────────────────────────
