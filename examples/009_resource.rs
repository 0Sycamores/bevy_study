//! 009 · 资源
//!
//! 运行：`cargo run --example 009_resource`
//!
//! 新增概念
//!   Resource         全局唯一的数据
//!   init_resource    用 Default 建（资源已存在时**不覆盖**）
//!   insert_resource  自己给初值（会覆盖，所以"重置资源"用它）
//!   Res / ResMut     只读 / 可写地访问它
//!
//! 使用场景
//!   存"整个世界只有一份"的状态：分数、配置、游戏阶段、计时器
//!
//! 注意：资源没注册会**直接 panic**，不是跳过 —— 三种应对见文末（对比：查询命中 0 个不报错）

use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(LogPlugin::default())
        // `init_resource` 用 `Default::default()` 建一个，所以 Score 必须实现 Default。
        .init_resource::<Score>()
        // `insert_resource` 自己给初值。`Multiplier` 故意不实现 Default，只能用这种。
        .insert_resource(Multiplier(2.5))
        .add_systems(
            Update,
            (
                // 这两个系统都写 Score → 数据冲突 → 必须显式声明顺序（004 讲的）。
                gain_score,
                award_bonus.after(gain_score),
                report.after(award_bonus),
                // 下面两个演示"资源可能不存在"时的两种安全写法。
                //
                // `report_optional` 读的是另一个资源，和上面三个**没有数据冲突**，
                // 本来位置随意（甚至可能排在最前面）。这里显式排到最后，
                // 只为让输出顺序固定 —— 同样是在用 004 讲的那套显式定序。
                report_optional.after(report),
                skip_if_missing.run_if(resource_exists::<NeverRegistered>),
            ),
        )
        .run();
}

/// 资源就是一个普通 struct，derive 一下即可。
#[derive(Resource, Default)]
struct Score(i32);

/// 故意**不**实现 `Default`：这种资源只能用 `insert_resource` 注册。
#[derive(Resource)]
struct Multiplier(f32);

/// 故意**不注册**，用来演示"资源不存在"会怎样。
#[derive(Resource)]
struct NeverRegistered(u32);

fn gain_score(mut score: ResMut<Score>) {
    let before = score.0;
    score.0 += 10;
    println!("── gain_score        Score {before} → {}", score.0);
}

/// `Res<T>` 只读、`ResMut<T>` 可写。同一个系统里两者**不能同时出现**。
fn award_bonus(mut score: ResMut<Score>, multiplier: Res<Multiplier>) {
    score.0 = (score.0 as f32 * multiplier.0) as i32;
    println!("── award_bonus       ×{} → {}", multiplier.0, score.0);
}

fn report(score: Res<Score>) {
    println!("── report            最终 Score = {}", score.0);
}

/// 写法①：`Option<Res<T>>` —— 允许资源不存在，自己处理 `None`。
fn report_optional(missing: Option<Res<NeverRegistered>>) {
    match missing {
        Some(value) => println!("── report_optional   NeverRegistered = {}", value.0),
        None => println!("── report_optional   NeverRegistered 未注册 → 拿到 None，没有 panic"),
    }
}

/// 写法②：`run_if(resource_exists::<T>)` —— 资源不存在就整个系统不跑。
/// 下面不会有它的输出。
fn skip_if_missing(_value: Res<NeverRegistered>) {
    println!("── 【这行永远不会打印】NeverRegistered 存在才会跑");
}

// ─────────────────────────────────────────────────────────────────────
// 实测输出
//
//   ── gain_score        Score 0 → 10
//   ── award_bonus       ×2.5 → 25
//   ── report            最终 Score = 25
//   ── report_optional   NeverRegistered 未注册 → 拿到 None，没有 panic
//
// 注意 `skip_if_missing` **一行输出都没有** —— 运行条件不成立，整个系统被跳过。
//
// （0.20 实跑，同一份产物连跑 6 次输出逐字相同。）
//
// ─────────────────────────────────────────────────────────────────────
// 组件 vs 资源
//
//   |          | 组件 Component        | 资源 Resource          |
//   |----------|----------------------|-----------------------|
//   | 数量      | 每个实体各一份          | 整个世界一份            |
//   | 怎么存    | 挂在实体上（spawn 时给） | 存在 World 里          |
//   | 怎么取    | `Query<&T>`           | `Res<T>` / `ResMut<T>` |
//   | 怎么注册  | `derive(Component)`   | `init_resource` / `insert_resource` |
//   | 典型用途  | 位置、血量、速度        | 分数、配置、计时器、游戏状态 |
//
// 判断标准就一句：**这个数据是"每个实体各有一份"还是"全局只有一份"？**
// 敌人各自的血量是组件；玩家总分是资源。
//
// ─────────────────────────────────────────────────────────────────────
// init_resource 还是 insert_resource
//
//   `init_resource::<T>()`  要求 `T: Default`，用默认值建
//   `insert_resource(v)`    自己给值；`T` 没有 `Default` 时只能用它
//
// 实测（0.20）两者在"资源已存在"时的行为**不同**：
//
//   insert_resource(Counter(42)) → 再 init_resource::<Counter>()  → 仍是 42（**不覆盖**）
//   init_resource::<Counter>()   → 再 insert_resource(Counter(7)) → 变成 7（**覆盖**）
//
// 也就是说 **`init_resource` 只在资源不存在时才创建**，它不会重置已有资源。
// 想重置一个资源，得用 `insert_resource`（会覆盖），或者直接改 `ResMut<T>`。
//
// ─────────────────────────────────────────────────────────────────────
// ⚠️ 资源不存在时：直接 panic
//
// 这是 006 讲过的"三种反应"里最凶的一种。把 `skip_if_missing` 的运行条件去掉、
// 直接注册进 `Update`，一运行就 panic。0.20 实测原文（跑完立刻还原；退出码 101）：
//
//   thread 'TaskPool (2)' (...) panicked at
//   .../bevy_ecs-0.20.0/src/error/handler.rs:128:1:
//   Encountered an error in system `009_resource::skip_if_missing`: Parameter
//   `Res<NeverRegistered>` failed validation: Resource does not exist
//   If this is an expected state, wrap the parameter in `Option<T>` and
//   handle `None` when it happens, or wrap the parameter in `If<T>` to
//   skip the system when it happens.
//
// 三点值得注意（和 0.19 时代的写法不同）：
//   · 系统名、参数名都是真的（`009_resource::skip_if_missing`、
//     `Res<NeverRegistered>`）—— `dev` 特性打开了 `debug`。
//   · panic 打在 **TaskPool 工作线程**上，而且同一 Schedule 里**其它系统照样跑完**：
//     实测这一次 `gain_score` 之后炸，`award_bonus` / `report` /
//     `report_optional` 三行输出仍然照常打印，最后才以退出码 101 收场。
//   · 报错源头是 `bevy_ecs` 的 `FallbackErrorHandler`
//     （`error/handler.rs` 里的 `match_severity`），它**默认仍然重新 panic**，
//     所以"缺资源会炸"这条结论在 0.20 没有变软。
//
// 三种应对方式：
//   ① `Option<Res<T>>`                允许不存在，自己处理 None        ← report_optional
//   ② `run_if(resource_exists::<T>)`  不存在就整个系统跳过             ← skip_if_missing
//   ③ `If<Res<T>>`                    参数级写法，效果同 ②
//
// 对比组件：`Query<&T>` 命中 0 个**永远**不会 panic（006 讲的）。
// **资源缺失会炸，实体查询不会** —— 这个不对称要记牢。
//
// ─────────────────────────────────────────────────────────────────────
// 两个系统同时写一个资源会怎样
//
// `gain_score` 和 `award_bonus` 都要 `ResMut<Score>`，Bevy 会把它们**串行**执行，
// 但**顺序不做保证**（004 讲的顺序歧义）。所以这里显式写了 `.after(gain_score)`。
// 去掉它，最终得分就可能是 25（先加分再乘）或 25 之外的另一个值，且没有报错。
// ─────────────────────────────────────────────────────────────────────
