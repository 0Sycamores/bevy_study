//! 010 · 时间与计时器
//!
//! 运行：`cargo run --example 010_time`
//!
//! 新增概念
//!   Time            这一帧过了多久（delta）、总共过了多久（elapsed）
//!   Timer           计时器：Once 到点停住 / Repeating 周期触发
//!   Stopwatch       秒表：只报"走了多久"，永不触发
//!   Time<Virtual>   可暂停、可倍速的时钟 —— 系统里的 Res<Time> 拿到的就是它
//!
//! 使用场景
//!   让运动 / 动画与帧率无关（每帧位移乘 delta_secs）；冷却、刷怪、自动存档
//!   暂停游戏但 UI 继续动 —— 改 Time<Virtual>
//!
//! 注意：Once 到点后 is_finished() **每帧都为真**，要"触发一次"得用 just_finished()

use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::time::Stopwatch;
use std::time::Duration;

/// 第一段每帧推进 100ms。
const FRAME: Duration = Duration::from_millis(100);
/// 重复触发的间隔。
const REPEAT: f32 = 0.25;
/// `Once` 计时器的时长（故意和 `REPEAT` 不同，也故意**不落在帧边界上**，
/// 否则 `0.1s × 4` 的浮点累加恰好差一点，会看起来"该完成却没完成"）。
const ONCE: f32 = 0.35;

fn main() {
    part1_timers();
    println!();
    part2_virtual_time();
}

// ═══════════════════════════════════════════════════════════════════════
// 一、Timer 与 Stopwatch
// ═══════════════════════════════════════════════════════════════════════

/// 写法①：自己拿 delta 累加。
#[derive(Resource, Default)]
struct ManualAccumulator(f32);

/// 写法②：`Timer` 作为**资源**（全局唯一的定时）。
#[derive(Resource)]
struct RepeatTimer(Timer);

/// `TimerMode::Once` 的对照样本。
#[derive(Resource)]
struct OnceTimer(Timer);

/// 写法③：`Timer` 作为**组件**。同一个类型挂在不同实体上，就是各自独立的计时。
#[derive(Component)]
struct Blink {
    name: &'static str,
    timer: Timer,
}

/// `Stopwatch` 只计时、从不"到点"。
#[derive(Resource, Default)]
struct Watch(Stopwatch);

fn part1_timers() {
    println!("═══ 一、Timer 的状态与三种驱动写法 ═══");
    let mut app = App::new();
    app.add_plugins(LogPlugin::default());
    // 注意：没有 TimePlugin。`Time` 不会自己走，完全由下面的 advance_by 推进。
    app.init_resource::<Time>();
    app.init_resource::<ManualAccumulator>();
    app.init_resource::<Watch>();
    app.insert_resource(RepeatTimer(Timer::from_seconds(
        REPEAT,
        TimerMode::Repeating,
    )));
    app.insert_resource(OnceTimer(Timer::from_seconds(ONCE, TimerMode::Once)));
    app.add_systems(Startup, spawn_blinkers);
    app.add_systems(
        Update,
        (
            manual_accumulate,
            tick_repeat,
            tick_once,
            tick_blinkers,
            stopwatch_demo,
            show_state,
        )
            .chain(),
    );

    for frame in 1..=5 {
        app.world_mut().resource_mut::<Time>().advance_by(FRAME);
        println!("── 第 {frame} 帧");
        app.update();
    }

    println!();
    println!("【小结】Repeating 到点后自动重置、周期触发；Once 到点后停住，");
    println!(
        "        之后 is_finished() **每帧都为真**（看上面 Once 那行，第 4 帧起一直是 true）——"
    );
    println!(
        "        拿它当「触发一次」用就会变成每帧触发，要「刚刚到点」必须用 just_finished()。"
    );
    println!("        另外注意 Repeating 在完成的那一帧 finished 也是 true，但 elapsed 已经绕回");
    println!("        （第 3、5 帧的 Repeating 行就是：elapsed 很小，finished 却是 true）。");
}

fn spawn_blinkers(mut commands: Commands) {
    // 两个间隔不同的计时器，证明"每个实体各走各的"。
    commands.spawn(Blink {
        name: "组件A",
        timer: Timer::from_seconds(0.25, TimerMode::Repeating),
    });
    commands.spawn(Blink {
        name: "组件B",
        timer: Timer::from_seconds(0.15, TimerMode::Repeating),
    });
}

/// 最原始的写法：自己攒 delta。能跑，但"超出的部分"要自己结转，
/// 忘了结转，定时就会随帧长慢慢漂移。
fn manual_accumulate(time: Res<Time>, mut acc: ResMut<ManualAccumulator>) {
    acc.0 += time.delta_secs();
    if acc.0 >= REPEAT {
        acc.0 -= REPEAT; // ← 结转，不是清零
        println!("     【写法① 手动累加】触发（结转 {:.2}s）", acc.0);
    }
}

/// `tick()` 返回 `&Timer`，所以可以直接链上 `just_finished()`。
fn tick_repeat(time: Res<Time>, mut timer: ResMut<RepeatTimer>) {
    if timer.0.tick(time.delta()).just_finished() {
        println!("     【写法② Timer 资源】触发");
    }
}

fn tick_once(time: Res<Time>, mut timer: ResMut<OnceTimer>) {
    if timer.0.tick(time.delta()).just_finished() {
        println!("     【Once + just_finished】只在这一帧为真");
    }
}

/// 每个实体各自 tick 自己的计时器，互不干扰。
fn tick_blinkers(time: Res<Time>, mut blinks: Query<&mut Blink>) {
    // 显式排序，不依赖遍历顺序（006 讲的：遍历顺序不是生成顺序）。
    let mut list: Vec<Mut<Blink>> = blinks.iter_mut().collect();
    list.sort_by_key(|blink| blink.name);
    for blink in &mut list {
        if blink.timer.tick(time.delta()).just_finished() {
            println!("     【写法③ Timer 组件】{} 触发", blink.name);
        }
    }
}

/// `Stopwatch` 与 `Timer` 的区别：它只回答"走了多久"，永远不会 finished。
/// `pause()` 之后 elapsed 冻结，`unpause()` 从原处继续，`reset()` 归零。
fn stopwatch_demo(time: Res<Time>, mut watch: ResMut<Watch>, mut call: Local<u32>) {
    *call += 1;
    watch.0.tick(time.delta());
    match *call {
        2 => {
            println!(
                "     【Stopwatch】pause()  —— 此时 elapsed = {:.2}s，之后冻结",
                watch.0.elapsed().as_secs_f32()
            );
            watch.0.pause();
        }
        4 => {
            println!(
                "     【Stopwatch】unpause() —— 还是从 {:.2}s 继续",
                watch.0.elapsed().as_secs_f32()
            );
            watch.0.unpause();
        }
        5 => {
            println!(
                "     【Stopwatch】reset()  —— 归零前 elapsed = {:.2}s",
                watch.0.elapsed().as_secs_f32()
            );
            watch.0.reset();
        }
        _ => {}
    }
}

/// 把这一帧的"时间现场"完整打出来：`Time` 的读数，加两个 `Timer` 的全部字段。
fn show_state(time: Res<Time>, repeat: Res<RepeatTimer>, once: Res<OnceTimer>, watch: Res<Watch>) {
    println!(
        "     Time      delta {:.2}s   elapsed {:.2}s",
        time.delta_secs(),
        time.elapsed_secs()
    );
    for (label, timer) in [("Repeating", &repeat.0), ("Once", &once.0)] {
        println!(
            "     {:<9} elapsed {:.2}/{:.2}s   remaining {:.2}s   fraction {:>3.0}%   finished {}",
            label,
            timer.elapsed().as_secs_f32(),
            timer.duration().as_secs_f32(),
            timer.remaining().as_secs_f32(),
            timer.fraction() * 100.0,
            timer.is_finished()
        );
    }
    println!(
        "     Stopwatch elapsed {:.2}s",
        watch.0.elapsed().as_secs_f32()
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 二、暂停与倍速：Time<Real> / Time<Virtual>
// ═══════════════════════════════════════════════════════════════════════

/// 这一段必须请出 `TimePlugin` —— `Time<Virtual>` 由它提供。
///
/// 代价是：装了插件之后，**手动 `advance_by` 会被插件用真实时钟覆盖掉**（实测无效），
/// 所以这一段只能按真实时间走，靠 `sleep` 控制帧间距。
/// 这正是本讲分成两段、两个 App 的原因：一段要精确，一段要插件。
fn part2_virtual_time() {
    println!("═══ 二、暂停与倍速（Time<Virtual>）═══");
    let mut app = App::new();
    // MinimalPlugins 里含 TimePlugin，但它不会自己跑循环 —— 我们仍然手动 update。
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, report_clocks);

    // 先空走一帧：时间插件的**首帧 delta 一定是 0**（它在这一帧才建立基准），
    // 不预热的话第一行输出全是 0，容易误读。
    app.update();

    let phases: [(&str, fn(&mut App)); 3] = [("正常", |_| {}), ("暂停", pause), ("3 倍速", triple)];
    for (label, setup) in phases {
        setup(&mut app);
        println!("── {label}");
        for _ in 0..2 {
            std::thread::sleep(Duration::from_millis(100));
            app.update();
        }
    }

    println!();
    println!("【小结】系统里 `Res<Time>` 拿到的就是**虚拟时钟**：");
    println!("        暂停时它变成 0，而 `Time<Real>` 照走 —— 这正是暂停菜单要的效果：");
    println!("        游戏世界的时间冻结，而 UI 动画、心跳、网络同步仍按真实时间走。");
    println!("        倍速同理，改 `Time<Virtual>` 的 set_relative_speed 即可。");
}

fn pause(app: &mut App) {
    app.world_mut().resource_mut::<Time<Virtual>>().pause();
}

fn triple(app: &mut App) {
    app.world_mut().resource_mut::<Time<Virtual>>().unpause();
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_relative_speed(3.0);
}

fn report_clocks(
    time: Res<Time>,
    real: Res<Time<Real>>,
    virtual_time: Res<Time<Virtual>>,
    mut skipped_baseline: Local<bool>,
) {
    // 跳过预热那一帧：它的 delta 是基准值 0（时间插件在这一帧才建立基准），
    // 打出来只会让人以为"时间没走"。
    if !*skipped_baseline {
        *skipped_baseline = true;
        return;
    }
    println!(
        "     Time.delta {:>7.1}ms   Real.delta {:>7.1}ms   Virtual.delta {:>7.1}ms   paused={}  speed={}",
        time.delta_secs() * 1000.0,
        real.delta_secs() * 1000.0,
        virtual_time.delta_secs() * 1000.0,
        virtual_time.is_paused(),
        virtual_time.relative_speed()
    );
}

// ─────────────────────────────────────────────────────────────────────
// 实测输出（第一段：逐帧稳定，可逐字复核）
//
//   ═══ 一、Timer 的状态与三种驱动写法 ═══
//   ── 第 1 帧
//        Time      delta 0.10s   elapsed 0.10s
//        Repeating elapsed 0.10/0.25s   remaining 0.15s   fraction  40%   finished false
//        Once      elapsed 0.10/0.35s   remaining 0.25s   fraction  29%   finished false
//        Stopwatch elapsed 0.10s
//   ── 第 2 帧
//        【写法③ Timer 组件】组件B 触发
//        【Stopwatch】pause()  —— 此时 elapsed = 0.20s，之后冻结
//        Time      delta 0.10s   elapsed 0.20s
//        Repeating elapsed 0.20/0.25s   remaining 0.05s   fraction  80%   finished false
//        Once      elapsed 0.20/0.35s   remaining 0.15s   fraction  57%   finished false
//        Stopwatch elapsed 0.20s
//   ── 第 3 帧
//        【写法① 手动累加】触发（结转 0.05s）
//        【写法② Timer 资源】触发
//        【写法③ Timer 组件】组件A 触发
//        Time      delta 0.10s   elapsed 0.30s
//        Repeating elapsed 0.05/0.25s   remaining 0.20s   fraction  20%   finished true
//        Once      elapsed 0.30/0.35s   remaining 0.05s   fraction  86%   finished false
//        Stopwatch elapsed 0.20s
//   ── 第 4 帧
//        【Once + just_finished】只在这一帧为真
//        【写法③ Timer 组件】组件B 触发
//        【Stopwatch】unpause() —— 还是从 0.20s 继续
//        Time      delta 0.10s   elapsed 0.40s
//        Repeating elapsed 0.15/0.25s   remaining 0.10s   fraction  60%   finished false
//        Once      elapsed 0.35/0.35s   remaining 0.00s   fraction 100%   finished true
//        Stopwatch elapsed 0.20s
//   ── 第 5 帧
//        【写法① 手动累加】触发（结转 0.00s）
//        【写法② Timer 资源】触发
//        【写法③ Timer 组件】组件A 触发
//        【写法③ Timer 组件】组件B 触发
//        【Stopwatch】reset()  —— 归零前 elapsed = 0.30s
//        Time      delta 0.10s   elapsed 0.50s
//        Repeating elapsed 0.00/0.25s   remaining 0.25s   fraction   0%   finished true
//        Once      elapsed 0.35/0.35s   remaining 0.00s   fraction 100%   finished true
//        Stopwatch elapsed 0.00s
//
// （以上是第一段的**完整**输出，除末尾的【小结】外没有省略。同一份产物连跑 5 次逐字相同。）
//
// 三件事值得盯住：
//   1. 第 3 帧：三种写法**在同一帧触发** —— 它们等价，只是组织方式不同。
//      写法③ 的两个组件计时器间隔不同（0.25s / 0.15s），所以触发次数不一样，
//      这就是"每个实体各自节奏"。
//   2. 第 4 帧起 Once 的 `finished` 一直是 true，而 `just_finished()` 只在第 4 帧响过一次。
//   3. 第 3、5 帧的 Repeating：`finished` 是 true，但 `elapsed` 已经绕回很小的值 ——
//      Repeating 在"完成的那一帧"同时满足"刚完成"和"已重置"。
//
// ── 实测输出（第二段：按真实时间走，数字会有 ±2ms 浮动，看趋势即可）
//
//   ── 正常
//        Time.delta   100.8ms   Real.delta   100.8ms   Virtual.delta   100.8ms   paused=false  speed=1
//        Time.delta   100.8ms   Real.delta   100.8ms   Virtual.delta   100.8ms   paused=false  speed=1
//   ── 暂停
//        Time.delta     0.0ms   Real.delta   100.7ms   Virtual.delta     0.0ms   paused=true  speed=1
//        Time.delta     0.0ms   Real.delta   100.7ms   Virtual.delta     0.0ms   paused=true  speed=1
//   ── 3 倍速
//        Time.delta   250.0ms   Real.delta   100.7ms   Virtual.delta   250.0ms   paused=false  speed=3
//        Time.delta   250.0ms   Real.delta   100.5ms   Virtual.delta   250.0ms   paused=false  speed=3
//
// 每个阶段都会 `update` 两次，所以每个标题下面是两行。
//
// ⚠️ **3 倍速那一格不是 300ms，而是 250ms 封顶** —— 这不是笔误，是 0.20 的一处真实行为变化。
//    `Time<Virtual>` 有一个 `max_delta`（默认 `DEFAULT_MAX_DELTA = 250ms`），
//    用来防止长时间卡顿后一帧推进太多。两个版本的裁剪时机不同：
//
//        // 0.20：先乘倍速，再裁剪（bevy_time-0.20.0/src/virt.rs:249，略去 tracing）
//        let scaled = raw_delta.mul_f64(speed);
//        let (effective_speed, delta) = if scaled > max_delta {
//            (max_delta.as_secs_f64() / raw_delta.as_secs_f64(), max_delta)
//        } else {
//            (speed, scaled)
//        };
//
//        // 0.19：先裁剪，再乘倍速（所以 100ms × 3 = 300ms 能突破 250ms）
//        // bevy_time-0.19.1/src/virt.rs:240，同样略去 tracing
//        let clamped_delta = if raw_delta > max_delta { max_delta } else { raw_delta };
//        let delta = clamped_delta.mul_f64(effective_speed);
//
//    结果就是：0.20 里**倍速放大后的结果同样受 `max_delta` 限制**，
//    真实帧长约 100ms 时设 `set_relative_speed(3.0)`，实际只推进约 250ms（≈2.5 倍）。
//    注意 `effective_speed` 这时会被改写成 `max_delta / raw_delta`
//    （按实测的 raw ≈ 100.7ms 算约 2.48），而 `relative_speed()` 依然报 3.0 ——
//    一个"名义倍速"、一个"实际倍速"。
//    想让高倍速真正生效，得把上限一起调大（对 `Time<Virtual>` 资源调用）：
//
//        app.world_mut().resource_mut::<Time<Virtual>>()
//            .set_max_delta(Duration::from_secs(1));
//
// 三行的对比就是全部结论：**暂停只冻结虚拟时钟，真实时间照走**；
// 倍速只放大虚拟时钟，而且放大幅度会被 `max_delta` 截断。
//
// 稳定性：第一段完全由 `advance_by(100ms)` 驱动、没有真实时钟参与，同一份产物连跑 5 次
// 逐字相同，可以逐字复核；第二段靠 `sleep` 计时，数字有 ±2ms 浮动，只看趋势
// （暂停那一行 `Real.delta` 照走才是结论的关键）。
//
// ─────────────────────────────────────────────────────────────────────
// Timer 的完整 API 一览（本讲出现过的）
//
//   Timer::from_seconds(时长, TimerMode)    建一个计时器
//   TimerMode::Once / Repeating             到点后停住 / 自动重置、周期触发
//   timer.tick(delta)                       推进一帧，返回 &Timer（所以能链式调用）
//   just_finished()                         只在跨越终点那一帧为真   ← 触发一次用它
//   is_finished()                           到点后持续为真（Once 会一直真）
//   elapsed() / remaining() / fraction()    走了多久 / 还剩多久 / 进度百分比
//   duration()                              总时长
//   pause() / unpause() / is_paused()       暂停这一个计时器
//   reset()                                 归零重来
//   set_duration(..) / set_elapsed(..)      运行时改时长 / 改进度
//
//   Stopwatch                               只计时、永不 finished；有 elapsed() / pause() / reset()
//
// Time<Virtual> 专用（第二段用到的都在这里）：
//
//   pause() / unpause() / is_paused()       冻结 / 恢复 / 查是否暂停
//   set_relative_speed(f64) / relative_speed()
//                                           设置 / 读取名义倍速（上面打印的 speed=3 就是这个）
//   effective_speed()                       本帧实际生效的倍速；
//                                           被 `max_delta` 截断时它与 relative_speed 不同
//   max_delta() / set_max_delta(Duration)   单帧推进上限，**默认 250ms**（0.20 里倍速结果也受它限制）
//   from_max_delta(Duration)                带自定义上限构造一个 Time<Virtual>
//
// 注意 `Time::<Fixed>::from_hz(60.0)` 这类"构造器"是关联函数，
// 而 `set_max_delta(..)` 是方法，要用 `resource_mut::<Time<Virtual>>()` 拿到后再调。
//
// ─────────────────────────────────────────────────────────────────────
// 三种驱动写法怎么选
//
//   写法① 手动累加     `acc += delta; if acc >= X { acc -= X; ... }`
//                     最灵活（能表达"攒够 N 次才触发"这类逻辑），也最容易写错 ——
//                     必须**结转**超出的部分（用 `-=` 而不是 `= 0`），否则定时会随帧长漂移。
//   写法② Timer 资源   适合全局唯一的定时：刷怪、自动存档、每秒结算。
//   写法③ Timer 组件   适合每个实体各自节奏：每个敌人的攻击冷却、每个 buff 的剩余时间。
//
// ─────────────────────────────────────────────────────────────────────
// 还有一层：Time<Fixed> 与 FixedUpdate
//
// 物理、确定性逻辑这类"必须固定步长"的东西，Bevy 给了单独的调度：
//
//   .add_systems(FixedUpdate, physics)
//   .insert_resource(Time::<Fixed>::from_hz(60.0))
//
// 它一帧里可能跑 0 次、1 次或多次（Bevy 会自动补帧），所以里面的 `Res<Time>`
// 是 `Time<Fixed>`，delta 恒定。这跟 `Update` 里的"真实帧长"是两回事。
// ─────────────────────────────────────────────────────────────────────
