//! 018 · 状态机
//!
//! 运行：`cargo run --example 018_states`
//!
//! 新增概念
//!   States                    用枚举描述"游戏现在处于哪个阶段"
//!   init_state::<S>()         注册，初始值取 `Default`
//!   OnEnter(S) / OnExit(S)    进入 / 离开某状态时**跑一次**的调度
//!   in_state(S)               运行条件：只在某状态里跑
//!   NextState<S>::set(..)     请求切换状态
//!
//! 使用场景
//!   菜单 → 游戏中 → 结算 这类流程控制
//!   每个阶段该跑哪些系统、该显示什么，用它划分最清楚
//!
//! 注意：`set()` 只是**请求**，切换发生在帧末的 `StateTransition` 调度里 ——
//!       所以 `OnExit(旧)` 一定排在 `OnEnter(新)` 之前，且当帧的 `Update` 里状态还没变

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "018 · 状态机：菜单 → 游戏中 → 结算，自动循环".into(),
                resolution: (960, 640).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .insert_resource(StateTimer(Timer::from_seconds(1.5, TimerMode::Repeating)))
        .add_systems(Startup, setup)
        .add_systems(Update, request_next_state)
        // ── 进入 / 离开，每个状态一对 ──
        .add_systems(OnEnter(AppState::Menu), enter_menu)
        .add_systems(OnExit(AppState::Menu), exit_menu)
        .add_systems(OnEnter(AppState::Playing), enter_playing)
        .add_systems(OnExit(AppState::Playing), exit_playing)
        .add_systems(OnEnter(AppState::GameOver), enter_game_over)
        .add_systems(OnExit(AppState::GameOver), exit_game_over)
        // ── 只在"游戏中"才跑的系统 ──
        .add_systems(
            Update,
            spin_only_while_playing.run_if(in_state(AppState::Playing)),
        )
        .run();
}

/// 三个状态。`Default` 决定启动时处于哪个。
#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
enum AppState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

/// 本讲用一个计时器自动推进状态，这样不用按键也能看完整循环。
/// 真实项目里通常是按键或 UI 按钮触发（011 / 023 讲）。
#[derive(Resource)]
struct StateTimer(Timer);

#[derive(Component)]
struct Spinner;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    // 这个方块一直存在，只是**只在 Playing 状态里才会转** ——
    // 转不转就是 `in_state` 是否生效的肉眼证据。
    commands.spawn((
        Spinner,
        Sprite::from_color(Color::srgb(0.95, 0.85, 0.35), Vec2::new(160.0, 40.0)),
        Transform::default(),
    ));
}

/// 每 1.5 秒请求切到下一个状态，形成 `菜单 → 游戏中 → 结算 → 菜单` 的循环。
fn request_next_state(
    time: Res<Time>,
    mut timer: ResMut<StateTimer>,
    current: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let target = match current.get() {
            AppState::Menu => AppState::Playing,
            AppState::Playing => AppState::GameOver,
            AppState::GameOver => AppState::Menu,
        };
        // 只是"请求"，不是"立刻切换" —— 见文件末尾的说明。
        println!("── 请求切换到 {target:?}");
        next.set(target);
    }
}

/// 只有 Playing 状态才跑 —— 靠 `in_state` 这个运行条件筛掉其余状态。
fn spin_only_while_playing(time: Res<Time>, mut spinners: Query<&mut Transform, With<Spinner>>) {
    for mut transform in &mut spinners {
        transform.rotate_z(2.0 * time.delta_secs());
    }
}

// ── 三个状态的进入 / 离开 ──────────────────────────────────────────────

fn enter_menu(mut clear: ResMut<ClearColor>) {
    clear.0 = Color::srgb(0.10, 0.12, 0.18);
    println!("   [OnEnter] Menu     背景变深蓝");
}

fn exit_menu() {
    println!("   [OnExit ] Menu");
}

fn enter_playing(mut clear: ResMut<ClearColor>) {
    clear.0 = Color::srgb(0.08, 0.18, 0.12);
    println!("   [OnEnter] Playing  背景变深绿，方块开始旋转");
}

fn exit_playing() {
    println!("   [OnExit ] Playing");
}

fn enter_game_over(mut clear: ResMut<ClearColor>) {
    clear.0 = Color::srgb(0.22, 0.09, 0.10);
    println!("   [OnEnter] GameOver 背景变暗红，方块停转");
}

fn exit_game_over() {
    println!("   [OnExit ] GameOver");
}

// ─────────────────────────────────────────────────────────────────────
// 实测输出（每个状态 1.5 秒，下面是头一个完整循环）
//
//   [OnEnter] Menu     背景变深蓝          ← 启动时也会跑一次，见下
//   ── 请求切换到 Playing
//      [OnExit ] Menu
//      [OnEnter] Playing  背景变深绿，方块开始旋转
//   ── 请求切换到 GameOver
//      [OnExit ] Playing
//      [OnEnter] GameOver 背景变暗红，方块停转
//   ── 请求切换到 Menu
//      [OnExit ] GameOver
//      [OnEnter] Menu     背景变深蓝
//
// 画面上：背景色随状态变化，黄色方块只在 Playing 那 1.5 秒里转，其余时间静止。
//
// 第一行 `[OnEnter] Menu` 值得注意：**`init_state` 注册完也会触发一次 `OnEnter`**，
// 哪怕"什么都没有切过去"。所以初始状态的 `OnEnter` 系统一定会跑，
// 你可以放心把"进入菜单要做的准备"全放进去。
//
// ─────────────────────────────────────────────────────────────────────
// 三条容易搞混的规则
//
// 1. **`set()` 是请求，不是执行。**
//    它只是把"下一个状态"写进 `NextState<S>` 资源；真正的切换发生在
//    帧末的 `StateTransition` 调度里。所以：
//      · 调用 `set()` 之后的同一帧，`State<S>` 还是旧值
//      · `OnExit(旧)` 先跑，`OnEnter(新)` 后跑（输出里能看到这个顺序）
//    连调两次 `set()`，只有最后一次算数。
//
// 2. **`OnEnter` / `OnExit` 只跑一次。**
//    它们是"状态切换那一帧"的调度，不是"该状态下每帧"。想每帧跑就用
//    `Update` + `in_state(..)`（本讲的 `spin_only_while_playing`）。
//    用错的话表现为"我的初始化只跑了一次"或"我的每帧逻辑压根没跑"。
//
// 3. **状态是资源，不是组件。**
//    整个 App 只有一份 `State<AppState>`，用 `Res<State<AppState>>` 读、
//    `ResMut<NextState<AppState>>` 写。所以状态描述的是"世界整体的阶段"，
//    而不是"某个实体的状态" —— 后者该用组件（配合 015 的 `Changed` 处理变化）。
//
// ─────────────────────────────────────────────────────────────────────
// 状态 vs 布尔标志
//
// 常见疑问："用 `bool` 资源也能表示菜单/游戏，为什么要用 `States`？"
//
//   · `States` 自带 **`OnEnter`/`OnExit` 调度** —— 进出时做一次初始化/清理，
//     不用自己判断"上一帧是不是还在这个状态"
//   · `in_state(..)` 是现成的运行条件，比每帧 `if flag` 更省（条件是调度级别）
//   · 状态多了以后，枚举比一堆互相约束的 bool 清楚得多
//   · 引擎会**主动检测状态变化**，不需要你手写"变了没有"的判断
//
// 一句话：只要这个"阶段"**进出时有事要做**，就该用 `States`。
//
// ─────────────────────────────────────────────────────────────────────
// 下一步：019 讲进阶
//
// 本讲只有一个状态维度。019 会讲：
//   · `SubStates`      只在某个父状态里才存在的子状态（比如"暂停"只在游戏中有意义）
//   · `ComputedStates` 由别的状态推导出来的状态
//   · `DespawnOnExit`  实体在工作状态结束时**自动销毁**，不用手写 `OnExit` 清理
// ─────────────────────────────────────────────────────────────────────
