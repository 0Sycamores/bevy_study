//! 008 · 命令与延迟执行
//!
//! 运行：`cargo run --example 008_commands`
//!
//! `Commands` 是 Bevy 里"排队等着的操作"。这一讲只讲清一件事：
//! **你调用 `commands.spawn(..)` 的那一刻，实体并没有被创建。**
//! 命令要等到下一个**同步点**(sync point)才真正落地。

use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(LogPlugin::default())
        .add_systems(
            Update,
            (
                spawn_enemies,
                count_enemies,
                add_ranks,
                report_ranks,
                despawn_weakest,
                count_final,
            )
                .chain(),
        )
        .run();
}

#[derive(Component)]
struct Label(&'static str);

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Rank(u32);

const NAMES: [&str; 3] = ["小兵甲", "小兵乙", "小兵丙"];

/// ① 排队创建 3 个敌人。
///
/// 注意最后那行：`spawn` 之后**立刻**用查询去看，还是 0 个 ——
/// 命令还躺在队列里，没进 World。
fn spawn_enemies(mut commands: Commands, enemies: Query<&Enemy>) {
    println!("① spawn_enemies");
    println!("     排队前查到 {} 个敌人", enemies.iter().count());

    for name in NAMES {
        commands.spawn((Label(name), Enemy, Rank(0)));
    }

    println!(
        "     排队了 3 个 spawn，立刻再查：{} 个（命令还没落地）",
        enemies.iter().count()
    );
}

/// ② `.chain()` 会在上一步和这一步之间**自动插入同步点**，
/// 所以这里能看到上一步排队的命令已经生效。
fn count_enemies(enemies: Query<&Label, With<Enemy>>) {
    let names: Vec<&str> = enemies.iter().map(|label| label.0).collect();
    println!(
        "② count_enemies        → 已创建 {} 个：{}",
        names.len(),
        names.join("、")
    );
}

/// ③ 排队给每个敌人插入 `Rank` 组件。
/// `insert` 既能添加原本没有的组件，也能覆盖已有组件的值。
fn add_ranks(mut commands: Commands, enemies: Query<(Entity, &Label), With<Enemy>>) {
    println!(
        "③ add_ranks            排队给 {} 个敌人插入 Rank",
        enemies.iter().count()
    );
    for (index, (entity, _)) in enemies.iter().enumerate() {
        commands.entity(entity).insert(Rank(index as u32 + 1));
    }
}

/// ④ 同步点之后，`Rank` 已经挂上了。
/// 这里特意用 `Option<&Rank>`：因为"有没有 Rank"正是要展示的重点。
fn report_ranks(enemies: Query<(&Label, Option<&Rank>), With<Enemy>>) {
    println!("④ report_ranks");
    for (label, rank) in enemies.iter() {
        match rank {
            Some(rank) => println!("     {:<6} Rank = {}", label.0, rank.0),
            None => println!("     {:<6} 还没有 Rank", label.0),
        }
    }
}

/// ⑤ 排队销毁 Rank 最小的那个敌人。
/// `despawn()` 会把实体连同它身上的全部组件一起移除。
fn despawn_weakest(mut commands: Commands, enemies: Query<(Entity, &Rank), With<Enemy>>) {
    let weakest = enemies
        .iter()
        .min_by_key(|(_, rank)| rank.0)
        .map(|(entity, _)| entity);

    if let Some(entity) = weakest {
        println!("⑤ despawn_weakest      排队销毁 {entity:?}");
        commands.entity(entity).despawn();
    }
}

/// ⑥ 再数一次：比 ② 少一个。
fn count_final(enemies: Query<&Enemy>) {
    println!(
        "⑥ count_final          → 还剩 {} 个敌人",
        enemies.iter().count()
    );
}

// ─────────────────────────────────────────────────────────────────────
// 实测输出
//
//   ① spawn_enemies
//        排队前查到 0 个敌人
//        排队了 3 个 spawn，立刻再查：0 个（命令还没落地）
//   ② count_enemies        → 已创建 3 个：小兵甲、小兵乙、小兵丙
//   ③ add_ranks            排队给 3 个敌人插入 Rank
//   ④ report_ranks
//        小兵甲    Rank = 1
//        小兵乙    Rank = 2
//        小兵丙    Rank = 3
//   ⑤ despawn_weakest      排队销毁 7v0
//   ⑥ count_final          → 还剩 2 个敌人
//
// ① 里的 "0 → 0" 是本讲的核心：命令是延迟的，同一系统内排队后立刻查也查不到。
//
// ⑤ 里那个 `7v0` 就是实体的 Debug 格式，写作 `索引v代数`：
//   `Entity` 本身只是个"索引 + 代数"的句柄，没有任何名字或类型信息，
//   它的含义完全由身上挂的组件决定（呼应 005）。
//
// ─────────────────────────────────────────────────────────────────────
// 同步点从哪来
//
// 命令真正落地发生在**同步点**上。Bevy 有两处会产生同步点：
//
//   1. 一个 Schedule 跑完时（比如整个 Update 结束时，以及 Startup 结束时）
//   2. 排序把"带延迟参数的系统"排在"会读相关数据的系统"之前时，
//      自动插入一个 `ApplyDeferred` 系统
//
// 第 2 条由 `ScheduleBuildSettings::auto_insert_apply_deferred` 控制，
// **默认就是 `true`**。所以上面 `.chain()` 之后，每一步都能看到上一步的结果。
//
// 实测对比：把 `.chain()` 去掉，写成 `(spawn_enemies, count_enemies)`，
// `② count_enemies` 会打印 **0 个** —— 因为它可能排在 `spawn_enemies` 之前。
// 换句话说，**不声明顺序，命令什么时候生效就不确定**，这又是 004 讲的顺序问题。
//
// ── 三种控制同步点的方式 ──
//
//   `.chain()`                   排序 + 自动插 `ApplyDeferred`（本讲用的）
//   `.chain_ignore_deferred()`   只排序，**不**插 `ApplyDeferred`
//   显式插入 `ApplyDeferred`      在序列里自己放，手动控制落点：
//
//       (spawn_enemies, ApplyDeferred, count_enemies).chain()
//
// 一般来说默认行为就够用；只有在"我明确知道不需要中间落地、想省一次同步"
// 的时候才需要 `chain_ignore_deferred`。
//
// ─────────────────────────────────────────────────────────────────────
// Commands 的常用操作
//
//   commands.spawn(..)                  创建实体
//   commands.entity(e).insert(..)       加组件 / 覆盖组件值
//   commands.entity(e).remove::<T>()    移除某个组件
//   commands.entity(e).despawn()        销毁实体（连同身上全部组件）
//   commands.get_entity(e)              拿到 Option<EntityCommands>，
//                                       实体已不存在时是 None 而不是 panic
//
// 因为都是"写进队列"，所以下面这段看起来自相矛盾的代码其实完全合法：
//
//     commands.spawn(Enemy);
//     let entity = commands.spawn(Enemy).id();   // id() 立刻返回，实体还没建
//     commands.entity(entity).insert(Health(100.0));  // 依然只是排队
//
// ─────────────────────────────────────────────────────────────────────
// 回看 005：为什么 Startup 里 spawn 的东西，Update 能查到？
//
// 因为 Startup 这个 Schedule 跑完时就产生了一次同步点，
// 命令在进入 Update 之前已经全部落地了。
// 这不是特例，正是上面第 1 条同步点规则在起作用。
// ─────────────────────────────────────────────────────────────────────
