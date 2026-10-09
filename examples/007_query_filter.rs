//! 007 · 查询过滤
//!
//! 运行：`cargo run --example 007_query_filter`
//!
//! 新增概念
//!   With / Without    实体必须有 / 必须没有某个组件
//!   Or<(..)>          满足其中任意一个即可
//!   Has<T>            在查询里返回"有没有 T"（bool），但不读它的值
//!   查询不相交        用 With + Without 向 Bevy 证明两个查询碰不到同一实体
//!
//! 使用场景
//!   按角色筛选实体（玩家 / 敌人 / 中立）；让一个系统同时读一批、写另一批
//!
//! 注意：两个查询的借用冲突是**运行时 panic**（错误码 B0001），编译能过（见文末）

use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(LogPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            // 这几个系统里只有 `mirror_enemies` 会写数据，其余都只读，
            // 彼此没有冲突、本来可以并行（004 讲的）。
            // 这里 chain 起来纯粹是为了让**打印顺序固定**，方便和文档里的输出对照。
            (
                mirror_enemies,
                list_combatants,
                report_health,
                list_healths,
                list_props,
            )
                .chain(),
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
struct Health(f32);

fn setup(mut commands: Commands) {
    commands.spawn((
        Label("玩家"),
        Player,
        Health(100.0),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    commands.spawn((
        Label("敌人A"),
        Enemy,
        Health(30.0),
        Transform::from_xyz(220.0, 120.0, 0.0),
    ));
    commands.spawn((
        Label("敌人B"),
        Enemy,
        Health(60.0),
        Transform::from_xyz(-160.0, -90.0, 0.0),
    ));
    // 这根石柱既不是玩家也不是敌人，也没有 Health。
    // 它是用来验证各个过滤器到底筛掉了谁的对照组。
    commands.spawn((Label("石柱"), Transform::from_xyz(0.0, 260.0, 0.0)));
}

/// 把每个敌人以玩家为中心做点对称（镜像）—— 就是 009 里 `award_hit` 的雏形。
///
/// 关键在第二个参数末尾的 `Without<Player>`：
///
/// 这个系统里有两个查询都要碰 `Transform`，一个读（玩家）、一个写（敌人）。
/// Bevy 只按组件的读写集合判断冲突，**不认"数学上大概不会重叠"**：
/// 一个实体完全可以同时拥有 `Player` 和 `Enemy`（比如被策反的敌人），
/// 所以它无法确定这两个查询不相交，于是判定可变借用会冲突。
///
/// 加上 `Without<Player>` 就等于明确告诉它："这个查询的实体一定没有 Player"，
/// 与 `With<Player>` 构成互斥，冲突随即消失。
///
/// ⚠️ 这个冲突是**运行时 panic**，不是编译错误。详见文件末尾。
fn mirror_enemies(
    player: Query<&Transform, With<Player>>,
    mut enemies: Query<(&Label, &mut Transform), (With<Enemy>, Without<Player>)>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    for (label, mut transform) in &mut enemies {
        let before = transform.translation;
        // 以玩家位置为中心的点对称
        transform.translation = player_transform.translation * 2.0 - before;
        println!(
            "── 镜像 {:<6}：({:>5.0}, {:>4.0}) → ({:>5.0}, {:>4.0})",
            label.0, before.x, before.y, transform.translation.x, transform.translation.y
        );
    }
}

/// `Or<(..)>`：满足其中**任意一个**条件即可命中。
/// 这里把"玩家或敌人"合起来当作"战斗单位"，石柱被排除在外。
fn list_combatants(units: Query<&Label, Or<(With<Player>, With<Enemy>)>>) {
    let names: Vec<&str> = units.iter().map(|label| label.0).collect();
    println!(
        "── Or<(With<Player>, With<Enemy>)> 命中 {} 个：{}",
        names.len(),
        names.join("、")
    );
}

/// `Has<T>`：不读组件数据，只回答"这个实体有没有 T"，返回 bool。
///
/// 它写在查询**元组里**（因为它要返回一个值），而不是像 `With` 那样
/// 放在过滤器位置 —— 而且不需要实体真的拥有 `Health` 也能匹配。
fn report_health(entities: Query<(&Label, Has<Health>)>) {
    println!("── Has<Health> 逐个探测：");
    for (label, has_health) in entities.iter() {
        println!("     {:<6} 有 Health? {}", label.0, has_health);
    }
}

/// 对比 `Has<Health>`：`&Health` 会把**数据本身**取出来。
///
/// 想让系统"只在实体有 Health 时才处理它"，用 `&Health` 就够了 ——
/// 没有这个组件的实体根本不会命中，不必再问一句"有没有"。
/// `Has<Health>` 真正的用处是：你已经为了别的原因在遍历这批实体，
/// 只想顺便问一句"它带不带 Health"，而且**不关心具体数值**。
fn list_healths(entities: Query<(&Label, &Health)>) {
    println!("── &Health 取到实际数值（对比上面的 Has<Health>）：");
    for (label, health) in entities.iter() {
        println!("     {:<6} HP = {:.0}", label.0, health.0);
    }
}

/// `Without` 也能单独当过滤条件用，不只是为了解决冲突。
/// `(Without<Player>, Without<Enemy>)` 就是"既不是玩家也不是敌人"。
fn list_props(props: Query<&Label, (Without<Player>, Without<Enemy>)>) {
    let names: Vec<&str> = props.iter().map(|label| label.0).collect();
    println!(
        "── (Without<Player>, Without<Enemy>) 命中 {} 个：{}",
        names.len(),
        names.join("、")
    );
}

// ─────────────────────────────────────────────────────────────────────
// 实测输出
//
//   ── 镜像 敌人A   ：(  220,  120) → ( -220, -120)
//   ── 镜像 敌人B   ：( -160,  -90) → (  160,   90)
//   ── Or<(With<Player>, With<Enemy>)> 命中 3 个：敌人A、敌人B、玩家
//   ── Has<Health> 逐个探测：
//        敌人A    有 Health? true
//        敌人B    有 Health? true
//        玩家     有 Health? true
//        石柱     有 Health? false
//   ── &Health 取到实际数值（对比上面的 Has<Health>）：
//        敌人A    HP = 30
//        敌人B    HP = 60
//        玩家     HP = 100
//   ── (Without<Player>, Without<Enemy>) 命中 1 个：石柱
//
// （同一份产物连跑 8 次，输出逐字相同 —— `.chain()` 把系统顺序钉死了；
//   但下面要说的"原型分组"只保证稳定，不保证等于生成顺序。）
//
// 四个查询、四种筛法，石柱该被排除时被排除、该被单独挑出来时被挑出来。
//
// ⚠️ 注意遍历顺序同样**不是生成顺序**：生成顺序是"玩家、敌人A、敌人B、石柱"，
// 但每个查询都是先出敌人、再出玩家。原因见 006 讲的**原型(archetype)**分组
// 存储 —— 组件组合相同的实体归为一组，遍历逐组进行。
// 任何依赖遍历顺序的写法都不可靠。
//
// ─────────────────────────────────────────────────────────────────────
// With / Without / Or / Has 的分工
//
//   With<T>        实体必须有 T，但**不读**它的数据
//   Without<T>     实体必须没有 T
//   Or<(..)>       括号里任意一个满足即可
//   Has<T>         放在查询元组里，返回 bool 表示"有没有 T"
//
// 位置上的区别很重要：
//   · `With` / `Without` / `Or` 是**过滤器**，写在查询的第二个参数位置
//   · `Has<T>` 是**取数项**，写在元组里，因为它要产出一个值
//
// 另外注意 `With` / `Without` 不会触发变更检测（015 讲），
// 因为它们根本不访问数据，只是筛选。
//
// ─────────────────────────────────────────────────────────────────────
// 冲突：这是运行时 panic，不是编译错误
//
// 把 `mirror_enemies` 里的 `Without<Player>` 去掉，代码**照样编译通过**
// （`cargo check` 不会有任何意见），但一运行就 panic。
// 下面是 0.20 实测原文（临时去掉那半句，跑完立刻还原；退出码 101）：
//
//   thread 'main' (...) panicked at
//   .../bevy_ecs-0.20.0/src/query/state.rs:216:13:
//   error[B0001]: Query<(&Label, &mut Transform), With<Enemy>> in system
//   007_query_filter::mirror_enemies accesses component(s) Transform in a way that
//   conflicts with a previous system parameter. Consider using `Without<T>` to
//   create disjoint Queries or merging conflicting Queries into a `ParamSet`.
//   See: https://bevy.org/learn/errors/b0001
//
// 注意两点：
//   · `Query<...>` 里印的是**真实的查询类型**，`in system` 后面是
//     **真实的系统路径** `007_query_filter::mirror_enemies`。
//     `dev` 特性打开了 `debug`，所以不再有 `<Enable the debug feature to see the name>`
//     这类占位文字（这点和 004 讲的调度警告是同一个机制）。
//   · 这次 panic 在 `main` 线程上，且发生在系统**初始化**阶段 ——
//     所以什么都还没来得及打印就结束了。
//
// 所以"编译过了"不代表查询没问题 —— 这类冲突只在系统真正跑起来时才暴露。
// 报错信息自己给了两种修法：
//
//   ① `Without<T>`：让两个查询在过滤条件上互斥（本讲用的这种）
//   ② `ParamSet`：把互相冲突的查询塞进一个 `ParamSet`，用到哪个取哪个。
//      它内部保证同一时刻只借出一种，所以冲突被结构性消除。
//      适合"我确实需要遍历两批可能重叠的实体"的场景。
//
// ── 为什么 Bevy 不认"数学上的不相交" ──
//
// 因为一个实体可以同时拥有 `Player` 和 `Enemy`，`With<Player>` 与
// `With<Enemy>` 本身并不互斥。Bevy 只做静态可判定的推断：
// 你显式写了 `Without<Player>`，它才敢认定两者不相交。
//
// 这也是 `Without` 在本讲里出现两次的原因：
// 一次是为了**让代码能跑**（消除冲突），一次是**纯粹当过滤条件**（挑出石柱）。
// ─────────────────────────────────────────────────────────────────────
