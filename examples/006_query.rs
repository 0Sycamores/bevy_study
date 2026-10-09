//! 006 · 查询的取数方式
//!
//! 运行：`cargo run --example 006_query`
//!
//! 新增概念
//!   iter / iter_mut / par_iter_mut      遍历一批（串行只读 / 串行可写 / 并行可写）
//!   single() / get(entity)              取一个，返回 Result
//!   Single / Populated / Option<Single> 取一个 / 至少一个；条件不满足则整个系统被跳过
//!
//! 使用场景
//!   遍历处理一批实体；或写成"有才处理、没有就别跑"的系统
//!
//! 注意：`Query` 命中 0 个不会报错、也不会跳过系统；缺 `Res` 才会 panic（三种反应见文末）
//! 注意：遍历顺序**不是**生成顺序，别依赖它

use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        // 本讲不需要画面，只要日志。
        // 没有 DefaultPlugins 时 App 只跑一帧就退出（001 讲），输出干净好读。
        .add_plugins(LogPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                // 先给所有 Score +1，再遍历打印。
                // bump 写 Score、后面几个都读 Score —— 数据冲突，
                // 按 004 讲的必须显式声明顺序，否则打印出的是加之前还是之后的值不定。
                bump_all_parallel,
                (
                    list_all,
                    single_via_query,
                    single_via_param,
                    single_that_gets_skipped,
                    populated_param,
                )
                    // 这几个系统都只读 Score，彼此没有数据冲突、本来可以并行（004 讲的）。
                    // 这里 chain 起来纯粹是为了让**打印顺序固定**，方便和文档里的输出对照。
                    .chain()
                    .after(bump_all_parallel),
            ),
        )
        .run();
}

#[derive(Component)]
struct Label(&'static str);

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Score(i32);

fn setup(mut commands: Commands) {
    // 两个 Player，一个 Enemy，各自带一份 Score。
    // 注意 `Single` 要的是"恰好一个" —— 所以 Player 有两个这件事很重要。
    commands.spawn((Label("玩家A"), Player, Score(10)));
    commands.spawn((Label("玩家B"), Player, Score(20)));
    commands.spawn((Label("敌人"), Enemy, Score(50)));
}

/// 写法①：并行遍历所有匹配实体。
///
/// `par_iter_mut()` 把遍历切块分给多个线程。串行版本只需把 `par_iter_mut`
/// 换成 `iter_mut`，循环体一模一样。
/// 实体少、单次迭代又轻的时候串行反而更快（线程调度本身有开销），
/// 只有迭代体足够重、实体足够多时才值得并行。
fn bump_all_parallel(mut scores: Query<&mut Score>) {
    scores.par_iter_mut().for_each(|mut score| {
        score.0 += 1;
    });
}

/// 写法②：串行遍历，逐个读取。最常用的一种。
fn list_all(scores: Query<(&Label, &Score)>) {
    println!("── 遍历全部（iter 逐个读取）");
    for (label, score) in scores.iter() {
        println!("     {:<8} Score = {}", label.0, score.0);
    }
}

/// 写法③：`single()` —— 只想要**恰好一个**匹配实体时用它。
///
/// `single()` 返回 `Result`，所以成功和失败两种情况都得处理掉。
/// 对比下面写法④的 `Single` 参数：那个在失败时直接跳过整个系统。
fn single_via_query(enemies: Query<(&Label, &Score), With<Enemy>>) {
    match enemies.single() {
        Ok((label, score)) => println!("── single() 命中：{}，Score = {}", label.0, score.0),
        Err(e) => println!("── single() 没命中：{e}"),
    }
}

/// 写法④：把 `Single` 直接写进系统参数。
/// 好处是不用自己处理错误；代价是**匹配不唯一时整个系统不执行**，
/// 而且什么都不会打印 —— 排查"我的系统怎么没跑"时要记得这一点。
fn single_via_param(enemy: Single<(&Label, &Score), With<Enemy>>) {
    let (label, score) = *enemy;
    println!("── Single 参数命中：{}，Score = {}", label.0, score.0);
}

/// 写法⑤：**反例**。`With<Player>` 有两个匹配实体，而 `Single` 要求恰好一个，
/// 所以这个系统会被静默跳过 —— 下面不会出现它的任何输出。
fn single_that_gets_skipped(_player: Single<&Score, With<Player>>) {
    println!("── 【这行永远不会打印】Single 要求恰好一个 Player，但有两个");
}

/// 写法⑥：`Populated` = "至少有一个"。有就执行，一个都没有就跳过。
fn populated_param(players: Populated<(&Label, &Score), With<Player>>) {
    println!(
        "── Populated 参数：匹配到 {} 个 Player（依次是 {}）",
        players.iter().count(),
        players
            .iter()
            .map(|(label, _)| label.0)
            .collect::<Vec<_>>()
            .join("、")
    );
}

// ─────────────────────────────────────────────────────────────────────
// 实测输出
//
//   ── 遍历全部（iter 逐个读取）
//        敌人       Score = 51
//        玩家A      Score = 11
//        玩家B      Score = 21
//   ── single() 命中：敌人，Score = 51
//   ── Single 参数命中：敌人，Score = 51
//   ── Populated 参数：匹配到 2 个 Player（依次是 玩家A、玩家B）
//
// （同一份产物连跑 8 次，输出逐字相同 —— `.chain()` 把顺序钉死了。
//   反过来说：这里"敌人排在前面"并不是生成顺序，见下。）
//
// 三点值得注意：
//   · 三个分数是 11/21/51 而不是 10/20/50，因为 `list_all` 被排在
//     `bump_all_parallel` 之后。把那句 `.after(..)` 去掉，打印的就可能是原值。
//   · **写法⑤ 一行输出都没有** —— 它被静默跳过了，而不是报错。
//   · ⚠️ **遍历顺序不是生成顺序。** 生成顺序是"玩家A、玩家B、敌人"，
//     打印出来敌人却排在最前面。原因见下。
//
// ── 为什么遍历顺序 ≠ 生成顺序 ──
//
// Bevy 按**原型(archetype)**存储实体：组件组合**完全相同**的实体放在一起。
// 这里先后创建了两批：
//
//     玩家A、玩家B  → 原型 (Label, Player, Score)
//     敌人          → 原型 (Label, Enemy, Score)
//
// 遍历时是**逐原型**走的：同一个原型内部保持生成顺序，
// 但原型之间的先后跟生成时间无关。所以三个实体被分成了两组，
// 敌人那一组恰好排在了前面。
//
// 结论：**永远不要依赖 `Query` 的遍历顺序。**
// 需要固定次序就把结果收集成 `Vec` 再 `sort_by_key`，
// 或者用某个明确的组件字段作为排序依据。
// 顺带一提，"按原型存储"正是 Bevy 查询快的原因 —— 同一原型的组件
// 在内存里连续排列，遍历时缓存友好。顺序不可依赖是它付出的代价。
//
// ─────────────────────────────────────────────────────────────────────
// 取不到数据时，Bevy 的三种反应
//
//   1. `Query` 命中 0 个 → 完全正常。
//      `for` 不执行、`iter().count()` 得 0，系统照常运行。
//      **`Query` 永远不会导致系统被跳过。**
//
//   2. `Single` / `Option<Single>` / `Populated` 的条件不满足
//      → **静默跳过整个系统**：不打印、不报错、不 panic。
//      这是"我的系统怎么没跑"最常见的原因。
//
//   3. 缺 `Res<T>` / `ResMut<T>`（资源压根没注册）
//      → **直接 panic**，不是跳过！实测报错原文（临时往这个 example 里加了一个
//        带 `Res<NeverRegistered>` 参数的系统，跑完即删；进程退出码 101）：
//
//          thread 'TaskPool (8)' (...) panicked at
//          .../bevy_ecs-0.20.0/src/error/handler.rs:128:1:
//          Encountered an error in system `006_query::need_missing`: Parameter
//          `Res<NeverRegistered>` failed validation: Resource does not exist
//          If this is an expected state, wrap the parameter in `Option<T>` and
//          handle `None` when it happens, or wrap the parameter in `If<T>` to
//          skip the system when it happens.
//
//      三点值得记：
//        · **系统名和参数名都是真的**（`006_query::need_missing`、
//          `Res<NeverRegistered>`）—— `dev` 特性打开了 `debug`，不会退化成占位文字。
//        · panic 打在 **TaskPool 工作线程**上，不是 `main`；而且同一个 Schedule 里
//          **其它系统会继续跑完**（实测终端上能看到它们的输出与这条 panic 交错），
//          进程最后以退出码 101 结束。所以"炸掉"指的是这一帧整体失败，不是立刻死。
//        · 报错源头是 `bevy_ecs` 的 `FallbackErrorHandler`
//          （`error/handler.rs` 里的 `match_severity`）—— 系统出错统一收拢到
//          这个兜底处理器，但它**默认仍然重新 panic**，不会把错误吞掉。
//
//      报错信息自己给了两条出路：要 `Option<Res<T>>` 自己处理 `None`，
//      或者用 `If<Res<T>>` 让系统在资源缺失时自动跳过。
//
// 一句话记住：**实体查询不匹配只会空手而归，资源缺失会当场炸掉。**
//
// ─────────────────────────────────────────────────────────────────────
// 取数方式对照表
//
//   query.iter()             串行遍历（只读）
//   query.iter_mut()         串行遍历（可写），需要 `mut query`
//   query.par_iter_mut()     并行遍历（可写），迭代重 / 实体多时才划算
//   query.single()           恰好一个，返回 Result，失败自己处理
//   query.get(entity)        按实体取一个，返回 Result
//   query.iter_many(..)      按一批实体取
//   Single<D, F>             恰好一个，不满足则整个系统被跳过
//   Option<Single<D, F>>     零个或一个，多于一个则系统被跳过
//   Populated<D, F>          至少一个，一个都没有则系统被跳过
//
// 前六种是"取不到就返回空/Err"，后三种是"取不到就不跑系统"。
// 选哪种取决于你的逻辑是"能处理没有的情况"还是"没有就干脆别跑"。
// ─────────────────────────────────────────────────────────────────────
