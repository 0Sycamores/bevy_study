//! 004 · 系统执行顺序
//!
//! 运行：`cargo run --example 004_schedule`
//!
//! 本讲回答一个大多数人靠猜的问题：**到底谁先跑？**
//!
//! 三句话版本：
//!
//! 1. 元组里并列的系统默认**并行**执行。互不冲突时，谁先谁后观察不到差别。
//! 2. 一旦两个系统访问同一份数据（一个读一个写、或者都写），引擎会自动把它们
//!    **串行**执行，但**先后顺序仍然不做保证** —— 这叫「顺序歧义」
//!    (system execution order ambiguity)。
//! 3. 需要确定顺序时必须**显式声明**：`.chain()` / `.before()` / `.after()` / `SystemSet`。
//!
//! ⚠️ 第 2 条是最阴的一类 bug：它不报错、不崩溃，而且**在同一个构建里往往稳定
//! 复现** —— 要么一直对，要么一直错，错的时候看起来还像"故意这么设计的"。
//! 可一旦你改动了系统注册（加一个系统、调一下顺序），它就可能悄悄翻过来。
//! 于是 bug 出现在"某次改动"之后，而你根本不会怀疑到几个月前写的那两个系统头上。
//! 好在 Bevy 可以**主动检测**这种歧义，本讲就把它打开给你看。

use bevy::ecs::schedule::{LogLevel, ScheduleBuildSettings};
use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        // 本讲不需要窗口，只要日志。单独装 LogPlugin 就够了。
        // 没有 DefaultPlugins 时 App 只跑一帧就退出（001 讲），
        // 正好适合看这种一次性的调度报告。
        .add_plugins(LogPlugin::default())
        .init_resource::<Wave>()
        .init_resource::<Score>()
        // 开局 5 点血，而一波伤害是 10 —— 保证一击致死，
        // 这样 check_death 看到的是 5 还是 -5，会直接改变最终得分。
        .insert_resource(Health(5))
        // ★ 打开顺序歧义检测。
        // 调度每次构建时，把"数据冲突但没声明顺序"的系统对 WARN 出来。
        // 这是排查隐性顺序 bug 最有效的一招，建议在自己的项目里也开着。
        .edit_schedule(Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        })
        .add_systems(
            Update,
            (
                // 用 SystemSet 给这组系统起个名字，之后可以对"整组"声明顺序。
                new_wave.in_set(Battle),
                // `.after(..)` = "我排在某个系统之后"。
                apply_damage.in_set(Battle).after(new_wave),
                // ⚠️ 这里就是歧义所在：
                // apply_damage 写 Health，check_death 读 Health —— 数据冲突，
                // 引擎会自动串行它们，但两者之间**没有任何顺序声明**。
                //
                // 只声明了它们都在 new_wave 之后，彼此之间是自由的。
                check_death.in_set(Battle).after(new_wave),
                // 正确的排列顺序
                // check_death.in_set(Battle).after(apply_damage),

                // report 要读最终结果，所以必须排在**整个 Battle 组**之后。
                // 对 SystemSet 声明顺序，就等于对它里面所有系统声明顺序。
                report.after(Battle),
            ),
        )
        .run();
}

/// 用 SystemSet 给一组系统起名，好处是以后加系统不用改外面的顺序声明。
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct Battle;

#[derive(Resource, Default)]
struct Wave(i32);

#[derive(Resource)]
struct Health(i32);

#[derive(Resource, Default)]
struct Score(i32);

fn new_wave(mut wave: ResMut<Wave>) {
    wave.0 += 1;
    println!("[new_wave]     第 {} 波敌人来袭", wave.0);
}

// 读 Wave、写 Health —— 和 check_death 抢 Health。
fn apply_damage(wave: Res<Wave>, mut hp: ResMut<Health>) {
    let dmg = 10 * wave.0;
    hp.0 -= dmg;
    println!("[apply_damage] 受到 {} 点伤害，剩余血量 {}", dmg, hp.0);
}

// 读 Health、写 Score —— 和 apply_damage 抢 Health。
fn check_death(hp: Res<Health>, mut score: ResMut<Score>) {
    if hp.0 <= 0 {
        score.0 += 100;
        println!("[check_death]  血量 {} ≤ 0，判定死亡，+100 分", hp.0);
    } else {
        println!("[check_death]  血量 {} > 0，还活着", hp.0);
    }
}

fn report(hp: Res<Health>, score: Res<Score>) {
    println!(
        "[report]       —— 本帧结算：血量 {}，得分 {} ——",
        hp.0, score.0
    );
}

// ─────────────────────────────────────────────────────────────────────
// 跑一遍你会看到什么
//
// 实测输出（`cargo run --example 004_schedule`）：
//
//   [new_wave]     第 1 波敌人来袭
//   [check_death]  血量 5 > 0，还活着              ← 先判死，看到的还是旧血量
//   [apply_damage] 受到 10 点伤害，剩余血量 -5      ← 后扣血，扣完其实已经死了
//   [report]       —— 本帧结算：血量 -5，得分 0 ——  ← 血量 -5，却一分没加
//
// 外加一条 WARN：
//
//   Update schedule built successfully, however: 1 pairs of systems with
//   conflicting data access have indeterminate execution order. Consider
//   adding `before`, `after`, or `ambiguous_with` relationships between these:
//    -- <Enable the debug feature to see the name> (in set Battle) and
//       <Enable the debug feature to see the name> (in set Battle)
//       conflict on: ["<Enable the debug feature to see the name>"]
//
// 这就是"顺序歧义"的真面目：**没有崩溃、没有报错、每个系统单独看都对**，
// 但玩家吃到了致命伤害，既没死也没加分。
//
// Bevy 默认把系统名藏起来了（那几处 `<Enable the debug feature ...>`）。
// 想让它直接点名是哪两个系统，在 `Cargo.toml` 里给 bevy 打开 debug feature：
//
//     bevy = { version = "0.19.1", features = ["debug"] }
//
// ⚠️ 一个反直觉、但很重要的点：**这个错误顺序是稳定的。**
// 上面这个例子连着跑 5 次，check_death 每次都排在 apply_damage 前面。
// Bevy 的调度在同一个构建里通常表现一致，所以这类 bug 不会"随机复现"——
// 它要么一直对、要么一直错，错的时候看起来还挺像"故意这么设计的"。
//
// 真正危险的地方是：它**没有任何保证**。你加一个系统、调整一次注册顺序、
// 甚至只改一个无关模块，都可能让它翻过来。
// 所以判断标准不是"我现在跑着是对的"，而是"我有没有显式声明顺序"。
//
// ── 怎么修 ──
//
// 改法 ①：最小改动，只声明它俩的先后（先扣血、再判死）：
//
//     check_death.in_set(Battle).after(apply_damage),
//
//   实测：得分变成 100，那条 WARN 也消失了。
//
// 改法 ②：整组一次性定死顺序，最省心：
//
//     (new_wave, apply_damage, check_death).chain().in_set(Battle),
//
// 两种改法效果一样，之后输出顺序变成 apply_damage → check_death，
// 得分稳定在 100。
// ─────────────────────────────────────────────────────────────────────
// 三种定序写法，什么时候用哪个
//
// 1. `.chain()` —— 让元组里的系统严格按书写顺序串行。
//    适合"这几步就是一条流水线"的场景，最直观。
//
//         (new_wave, apply_damage, check_death).chain()
//
// 2. `.before(..)` / `.after(..)` —— 只声明需要的约束，其余保持自由。
//    适合系统很多、只想固定其中几条关键依赖的场景，
//    能保留更多并行度。注意它是**传递**的：
//    A before B、B before C，则 A 自动 before C，不用重复写。
//
// 3. `SystemSet` + `configure_sets` —— 给一组系统起名，对整组声明顺序。
//    适合模块化：比如"输入组必须在物理组之前"，加了新系统只需丢进对应组，
//    不用回头改顺序声明。这也是 016 讲插件时会配合使用的写法。
//
// ── 什么时候"不用管" ──
//
// 两个系统访问的是完全不同的数据（比如一个写 Transform、一个写 Score），
// 它们的执行顺序观察不到任何差别，此时的"任意顺序"是**好事**：
// 引擎可以贪心地按数据空闲情况并行调度，白拿性能。
// 歧义检测报告的是"冲突且有歧义"的情况，不是所有无序都值得修。
//
// ── 误报怎么办 ──
//
// 极少数情况你确知某对系统虽然数据冲突、但顺序真的无所谓。
// 这时用 `.ambiguous_with(另一个系统)` 显式压掉这条警告 ——
// 但请务必在代码里写清理由，否则等于把真正的 bug 也一起藏了：
//
//     reads_everything.ambiguous_with(apply_damage) // 只读快照，晚一帧无所谓
// ─────────────────────────────────────────────────────────────────────
