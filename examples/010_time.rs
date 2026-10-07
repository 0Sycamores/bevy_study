//! 010 · 时间与计时器
//!
//! 运行：`cargo run --example 010_time`
//!
//! 两件事：① 让运动**与帧率无关**；② 定时逻辑的**三种写法**对照。
//!
//! 本讲**不用** `DefaultPlugins`，而是手动以固定帧长推进 ——
//! 时间相关的行为只有在"帧长可控"时才看得清（否则每帧 delta 都在抖）。
//! 手动推进的写法就是 `app.update()` 直接调用，不调 `app.run()`。

use bevy::log::LogPlugin;
use bevy::prelude::*;
// `Stopwatch` 不在 prelude 里，要单独引入。
use bevy::time::Stopwatch;
use std::time::Duration;

/// 前半程每帧 100ms（10 FPS），后半程切成 50ms（20 FPS），用来暴露"帧率相关"的写法。
const SLOW_FRAME: Duration = Duration::from_millis(100);
const FAST_FRAME: Duration = Duration::from_millis(50);
/// 定时器统一用 0.25 秒，短一点才好在 10 帧里看到多次触发。
const INTERVAL: f32 = 0.25;

fn main() {
    let mut app = App::new();
    app.add_plugins(LogPlugin::default());
    // 没有 TimePlugin，`Time` 不会自己走，完全由我们推进。
    app.init_resource::<Time>();
    app.init_resource::<ManualAccumulator>();
    app.init_resource::<Watch>();
    app.insert_resource(BlinkTimer(Timer::from_seconds(
        INTERVAL,
        TimerMode::Repeating,
    )));
    app.insert_resource(OnceTimer(Timer::from_seconds(0.4, TimerMode::Once)));
    app.add_systems(Startup, (spawn_movers, spawn_blinker));
    app.add_systems(
        Update,
        // 这些系统彼此没有数据冲突、本来可以并行（004 讲的），
        // 这里 chain 起来只为让**打印顺序固定**，方便和文档里的输出对照。
        (
            move_movers,
            show_positions,
            write_manual_accumulator,
            write_resource_timer,
            write_component_timer,
            write_once_timer,
            write_stopwatch,
        )
            .chain(),
    );

    let mut elapsed = 0.0_f32;
    for frame in 1..=10 {
        let step = if frame <= 5 { SLOW_FRAME } else { FAST_FRAME };
        app.world_mut().resource_mut::<Time>().advance_by(step);
        elapsed += step.as_secs_f32();
        println!(
            "── 第 {frame:>2} 帧   帧长 {:>3.0}ms   真实累计 {:.2}s",
            step.as_secs_f32() * 1000.0,
            elapsed
        );
        app.update();
    }
}

// ── 帧率无关的两种写法 ────────────────────────────────────────────────

#[derive(Component)]
struct Mover {
    name: &'static str,
    /// 速度，单位/秒
    speed: f32,
    /// `Some(每帧秒数)` 表示"骗自己每帧都是这个长度"——也就是帧率相关的写法
    fixed_step: Option<f32>,
    pos: f32,
}

fn spawn_movers(mut commands: Commands) {
    // ① 正确的写法：每帧用真实的 `delta_secs()` 推进
    commands.spawn(Mover {
        name: "delta法",
        speed: 100.0,
        fixed_step: None,
        pos: 0.0,
    });
    // ② 错误的写法：假设每帧都是 100ms，直接乘常量
    commands.spawn(Mover {
        name: "定步法",
        speed: 100.0,
        fixed_step: Some(0.1),
        pos: 0.0,
    });
}

fn move_movers(time: Res<Time>, mut movers: Query<&mut Mover>) {
    for mut mover in &mut movers {
        // `time.delta_secs()` = 上一帧到这一帧的秒数。
        // 位移乘上它，速度就与帧率无关；写死常量则会跟着帧率一起变。
        let dt = mover.fixed_step.unwrap_or_else(|| time.delta_secs());
        mover.pos += mover.speed * dt;
    }
}

fn show_positions(movers: Query<&Mover>) {
    // 显式排序，不依赖遍历顺序（006 讲的：遍历顺序不是生成顺序）。
    let mut list: Vec<&Mover> = movers.iter().collect();
    list.sort_by_key(|mover| mover.name);
    let parts: Vec<String> = list
        .iter()
        .map(|mover| format!("{}={:>5.1}", mover.name, mover.pos))
        .collect();
    println!("        {}", parts.join("   "));
}

// ── 写法①：自己拿 delta 累加 ──────────────────────────────────────────

#[derive(Resource, Default)]
struct ManualAccumulator(f32);

/// 最原始的做法：自己攒 delta。能跑，但要自己处理"超出的部分结转"。
/// 下面这行 `-= INTERVAL` 而不是 `= 0.0` 就是在结转 ——
/// 忘了结转的话，定时会随帧长慢慢漂移。
fn write_manual_accumulator(time: Res<Time>, mut acc: ResMut<ManualAccumulator>) {
    acc.0 += time.delta_secs();
    if acc.0 >= INTERVAL {
        acc.0 -= INTERVAL;
        println!("        【写法① 手动累加】触发（结转 {:.2}s）", acc.0);
    }
}

// ── 写法②：Timer 作为资源 ────────────────────────────────────────────

#[derive(Resource)]
struct BlinkTimer(Timer);

/// 适合"全局只有一个"的定时，比如刷怪、自动存档。
/// `tick()` 返回 `&Timer`，所以可以直接链上 `just_finished()`。
fn write_resource_timer(time: Res<Time>, mut timer: ResMut<BlinkTimer>, mut count: Local<u32>) {
    if timer.0.tick(time.delta()).just_finished() {
        *count += 1;
        println!(
            "        【写法② Timer 资源】第 {} 次触发（本轮已走 {:.2}s）",
            *count,
            timer.0.elapsed().as_secs_f32()
        );
    }
}

// ── 写法③：Timer 作为组件 ────────────────────────────────────────────

#[derive(Component)]
struct Blink {
    timer: Timer,
    on: bool,
}

fn spawn_blinker(mut commands: Commands) {
    commands.spawn(Blink {
        timer: Timer::from_seconds(INTERVAL, TimerMode::Repeating),
        on: false,
    });
}

/// 适合"每个实体各有各的节奏"，比如每个敌人独立的攻击冷却。
/// 同一个 `Timer` 类型，挂在不同实体上就是各自独立的计时。
fn write_component_timer(time: Res<Time>, mut query: Query<&mut Blink>) {
    for mut blink in &mut query {
        if blink.timer.tick(time.delta()).just_finished() {
            blink.on = !blink.on;
            println!(
                "        【写法③ Timer 组件】闪烁 → {}",
                if blink.on { "亮" } else { "灭" }
            );
        }
    }
}

// ── TimerMode::Once 与 just_finished / is_finished 的区别 ─────────────

#[derive(Resource)]
struct OnceTimer(Timer);

fn write_once_timer(time: Res<Time>, mut timer: ResMut<OnceTimer>) {
    let ticked = timer.0.tick(time.delta());
    if ticked.just_finished() {
        println!("        【Once + just_finished】只在这一帧为真");
    }
    if ticked.is_finished() {
        println!("        【Once + is_finished】 从此每帧都为真（所以每帧都打印）");
    }
}

// ── Stopwatch：只计时，不触发 ────────────────────────────────────────

#[derive(Resource, Default)]
struct Watch(Stopwatch);

/// `Timer` 关心"到了没有"，`Stopwatch` 只关心"走了多久"，永远不会 finished。
/// 适合冷却显示、计时赛、性能统计。
fn write_stopwatch(time: Res<Time>, mut watch: ResMut<Watch>) {
    watch.0.tick(time.delta());
    if watch.0.elapsed().as_secs_f32() >= 0.5 {
        println!(
            "        【Stopwatch】已走 {:.2}s，reset() 重来",
            watch.0.elapsed().as_secs_f32()
        );
        watch.0.reset();
    }
}

// ─────────────────────────────────────────────────────────────────────
// 实测输出
//
//   ── 第  1 帧   帧长 100ms   真实累计 0.10s
//           delta法= 10.0   定步法= 10.0
//   ── 第  2 帧   帧长 100ms   真实累计 0.20s
//           delta法= 20.0   定步法= 20.0
//   ── 第  3 帧   帧长 100ms   真实累计 0.30s
//           delta法= 30.0   定步法= 30.0
//           【写法① 手动累加】触发（结转 0.05s）
//           【写法② Timer 资源】第 1 次触发（本轮已走 0.05s）
//           【写法③ Timer 组件】闪烁 → 亮
//   ── 第  4 帧   帧长 100ms   真实累计 0.40s
//           delta法= 40.0   定步法= 40.0
//   ── 第  5 帧   帧长 100ms   真实累计 0.50s
//           delta法= 50.0   定步法= 50.0
//           【写法① 手动累加】触发（结转 0.00s）
//           【写法② Timer 资源】第 2 次触发（本轮已走 0.00s）
//           【写法③ Timer 组件】闪烁 → 灭
//           【Once + just_finished】只在这一帧为真
//           【Once + is_finished】 从此每帧都为真（所以每帧都打印）
//           【Stopwatch】已走 0.50s，reset() 重来
//   ── 第  6 帧   帧长  50ms   真实累计 0.55s     ← 从这里帧长减半
//           delta法= 55.0   定步法= 60.0         ← 定步法开始跑偏
//           【Once + is_finished】 从此每帧都为真（所以每帧都打印）
//   ...（第 7/8/9 帧同理：delta法 每帧 +5，定步法 每帧 +10）
//   ── 第 10 帧   帧长  50ms   真实累计 0.75s
//           delta法= 75.0   定步法=100.0         ← 真实只过了 0.75s，定步法多走 33%
//           【写法① 手动累加】触发（结转 0.00s）
//           【写法② Timer 资源】第 3 次触发（本轮已走 0.00s）
//           【写法③ Timer 组件】闪烁 → 亮
//           【Once + is_finished】 从此每帧都为真（所以每帧都打印）
//
// 关键就在最后几行：真实时间只过了 0.75 秒，速度 100/s 应该走 75；
// `delta法` 正好 75.0，`定步法` 却是 100.0。
// 原因是后半程每帧只有 50ms，而它每帧都硬加 100ms 的量。
//
// 另一个现象：`Once + is_finished` 那行从第 5 帧起**每帧都在刷** ——
// 正是下面要讲的 `is_finished()` 陷阱。
//
// **所以：任何"随时间变化"的量，位移、缩放、冷却、动画进度……
// 都必须乘 `delta_secs()`，绝对不能按帧算。**
//
// 三种定时写法在同一帧触发，说明它们等价，选哪种看用途：
//
//   写法① 手动累加     最灵活也最容易写错（要自己结转），一般没必要
//   写法② Timer 资源   全局唯一的定时（刷怪、自动存档）
//   写法③ Timer 组件   每个实体各自节奏（每个敌人的攻击冷却）
//   Stopwatch         只计时不触发（冷却显示、性能统计）
//
// ─────────────────────────────────────────────────────────────────────
// Once 与 Repeating，just_finished 与 is_finished
//
//   `TimerMode::Repeating`  到点后自动重置，周期触发
//   `TimerMode::Once`       到点后停住，永远保持 finished
//
// **`is_finished()` 对 Once 计时器是"一直为真"的** —— 一旦到点，之后每一帧都返回 true。
// 拿它当"触发一次"用，就会变成每帧触发一次（上面 `写 Once + is_finished` 那行
// 会一直刷屏）。要"只触发一次"就得用 `just_finished()`，它只在跨越终点的那一帧为真。
//
// 反过来，`Repeating` + `is_finished()` 只在到点那一帧为真（因为下一帧就重置了），
// 所以那种组合下两者行为相同。但**别依赖这个巧合**，想表达"刚刚到点"就用 `just_finished()`。
//
// ─────────────────────────────────────────────────────────────────────
// 关于"暂停"和"倍速"：Time<Real> 与 Time<Virtual>
//
// 上面的 `Time` 其实是**虚拟时钟**。Bevy 里时间分三层：
//
//   `Time<Real>`     真实世界时间，只受系统时钟影响
//   `Time<Virtual>`  在 Real 之上应用"暂停/倍速"，游戏逻辑该用它
//   `Time<Fixed>`    固定步长，供 `FixedUpdate` 用
//
// 关键结论（实测）：**系统里 `Res<Time>` 拿到的就是虚拟时钟。**
// 所以在 `DefaultPlugins` 下做暂停/慢动作，改 `Time<Virtual>` 就够了：
//
//   mut virtual_time: ResMut<Time<Virtual>>
//   virtual_time.pause();                 // 暂停：Res<Time> 的 delta 立刻变成 0
//   virtual_time.unpause();
//   virtual_time.set_relative_speed(3.0); // 3 倍速
//   virtual_time.is_paused()              // 查询是否暂停
//
// 实测数据（每帧真实流逝约 100ms）：
//
//   正常      Time=100.74ms   Real=100.74ms   Virtual=100.74ms
//   暂停后    Time=  0.00ms   Real=100.49ms   Virtual=  0.00ms   ← Real 照样在走
//   3 倍速    Time=300.85ms   Real=100.28ms   Virtual=300.85ms
//
// 注意暂停时 `Real` **没有停**：这正是你要的 —— 暂停菜单的动画、无头服务器的
// 心跳、网络同步都还得按真实时间走，只有游戏世界的时间冻结。
//
// 本讲为了精确控制帧长没有装 `TimePlugin`（该插件会用自己的时钟覆盖手动推进的时间，
// 实测手动 `advance_by` 会被它盖掉），所以这一节只给结论和实测数字，
// 不在这里现场演示。要现场用，就用 `DefaultPlugins` 或 `MinimalPlugins`。
//
// ─────────────────────────────────────────────────────────────────────
// 顺带一提：`FixedUpdate`
//
// 物理、确定性逻辑这类"必须固定步长"的东西，Bevy 提供了单独的调度：
//
//   .add_systems(FixedUpdate, physics)
//   .insert_resource(Time::<Fixed>::from_hz(60.0))
//
// `FixedUpdate` 可能在一帧里跑 0 次、1 次或多次（Bevy 会补帧），
// 所以它里面的 `Res<Time>` 是 `Time<Fixed>`，delta 恒定。
// 本讲不展开，知道它以固定步长驱动即可。
// ─────────────────────────────────────────────────────────────────────
