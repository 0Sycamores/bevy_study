//! 015 · 变更检测
//!
//! 运行：`cargo run --example 015_change_detection`
//!
//! 新增概念
//!   Changed<T>            查询过滤器：只留"这一帧被写过"的实体
//!   Added<T>              查询过滤器：只留"这一帧刚被加上"的实体
//!   Ref<T>                不过滤，但能问 is_added / is_changed / last_changed
//!   Res::is_changed()     资源版的变更检测
//!   RemovedComponents<T>  这一帧哪些实体的 T 被移除了
//!
//! 使用场景
//!   只对"变过的数据"做重活：血条只在掉血时重画、UI 只在数值变了时刷新、
//!   物理只在物体动了时重算 —— 省下的都是每帧重复劳动
//!
//! 注意：`Changed` 判的是"**被可变访问过**"，不是"值真的不同" —— 见文末那个反例

use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(LogPlugin::default());
    app.insert_resource(Score(0));
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            damage_a,
            rewrite_b_unchanged,
            show_changed,
            show_added,
            bump_score,
            show_score,
            remove_b_health,
            show_removed,
        )
            .chain(),
    );

    for frame in 1..=5 {
        app.insert_resource(Frame(frame));
        println!("═══ 第 {frame} 帧 ═══");
        app.update();
    }
}

/// 当前是第几帧，由 `main` 每帧写入。用它来安排"哪一帧发生什么"。
#[derive(Resource)]
struct Frame(u32);

#[derive(Resource)]
struct Score(i32);

#[derive(Component)]
struct Label(&'static str);

/// 要检测变化，组件得能比较相等 —— 这样才有 `set_if_neq` 可用。
#[derive(Component, PartialEq, Clone, Copy, Debug)]
struct Health(i32);

#[derive(Component)]
struct EnemyA;

#[derive(Component)]
struct EnemyB;

fn setup(mut commands: Commands) {
    commands.spawn((Label("敌人A"), EnemyA, Health(100)));
    commands.spawn((Label("敌人B"), EnemyB, Health(100)));
}

// ── 两个"写组件"的系统：一个真的改，一个改了等于没改 ──────────────────

/// 只在偶数帧扣血 —— 用来展示 `Changed` **会**命中真正改过的那个。
///
/// `set_if_neq` = "值不一样才写"。它内部先比较、相等就跳过，
/// 于是不会白白触发变更检测（对比下面那个系统）。
fn damage_a(frame: Res<Frame>, mut enemies: Query<&mut Health, With<EnemyA>>) {
    if frame.0 % 2 != 0 {
        return;
    }
    for mut health in &mut enemies {
        health.set_if_neq(Health(health.0 - 10));
    }
}

/// ⚠️ 反例：每一帧都"写"一遍，但写进去的值和原来**一模一样**。
///
/// 可 `Changed` 照样会命中它 —— 因为变更检测看的是
/// "有没有发生可变借用"（`DerefMut`），**不看最终值是否相同**。
/// 现实里这种代码很常见，比如无脑 `clamp`、无脑重新赋值：
///
///     health.0 = health.0.clamp(0, 100);   // 值没变，但已经可变借用了
///
/// 结果是：这个实体的血条会被每帧重画，白白干活。
fn rewrite_b_unchanged(mut enemies: Query<&mut Health, With<EnemyB>>) {
    for mut health in &mut enemies {
        health.0 = health.0.clamp(0, 100);
    }
}

// ── 四种"看变化"的写法 ────────────────────────────────────────────────

/// 写法①：`Changed<T>` 当**过滤器** —— 只有变过的实体才会出现在结果里。
/// 同时用 `Ref<T>` 拿到变更信息。
fn show_changed(changed: Query<(&Label, Ref<Health>), Changed<Health>>) {
    for (label, health) in &changed {
        println!(
            "   [Changed] {:<6} 值={:<4} 是新加的={}",
            label.0,
            health.0,
            health.is_added()
        );
    }
}

/// 写法②：`Added<T>` 只看"刚被加上"的。组件新加的那一帧，它和 `Changed` 都会命中。
fn show_added(added: Query<&Label, Added<Health>>) {
    for label in &added {
        println!("   [Added]   {} 的 Health 是本帧新加的", label.0);
    }
}

/// 写法③：资源版 —— `Res::is_changed()`。
/// 注意它问的是"**自从我这个系统上次运行以来**变过没有"，所以每个系统各看各的。
fn bump_score(frame: Res<Frame>, mut score: ResMut<Score>) {
    if frame.0 == 3 {
        score.0 += 100;
    }
}

fn show_score(score: Res<Score>) {
    if score.is_changed() {
        println!("   [Res]     Score = {}（本帧有变更）", score.0);
    }
}

/// 写法④：`RemovedComponents<T>` —— 组件被**移除**时（`despawn` 也算），
/// 这里能读到那些实体。它和消息一样是缓冲的，读者各自记录进度。
fn remove_b_health(
    frame: Res<Frame>,
    enemies: Query<Entity, With<EnemyB>>,
    mut commands: Commands,
) {
    if frame.0 == 4 {
        for entity in &enemies {
            println!("   [移除]    把敌人B 的 Health 摘掉");
            commands.entity(entity).remove::<Health>();
        }
    }
}

fn show_removed(mut removed: RemovedComponents<Health>) {
    let gone: Vec<Entity> = removed.read().collect();
    if !gone.is_empty() {
        println!("   [Removed] 本帧有 {} 个实体的 Health 被移除", gone.len());
    }
}

// ─────────────────────────────────────────────────────────────────────
// 实测输出
//
//   ═══ 第 1 帧 ═══
//      [Changed] 敌人A    值=100  是新加的=true
//      [Changed] 敌人B    值=100  是新加的=true
//      [Added]   敌人A 的 Health 是本帧新加的
//      [Added]   敌人B 的 Health 是本帧新加的
//      [Res]     Score = 0（本帧有变更）
//   ═══ 第 2 帧 ═══
//      [Changed] 敌人A    值=90   是新加的=false      ← 真扣了 10
//      [Changed] 敌人B    值=100  是新加的=false      ← 值没动，却报了
//   ═══ 第 3 帧 ═══
//      [Changed] 敌人B    值=100  是新加的=false      ← 只有 B 在"谎报"
//      [Res]     Score = 100（本帧有变更）
//   ═══ 第 4 帧 ═══
//      [Changed] 敌人A    值=80   是新加的=false      ← 又真扣了 10
//      [Changed] 敌人B    值=100  是新加的=false
//      [移除]    把敌人B 的 Health 摘掉
//      [Removed] 本帧有 1 个实体的 Health 被移除
//   ═══ 第 5 帧 ═══
//      （敌人A 这帧不扣血，敌人B 已经没 Health 了 —— 什么都没有）
//
// ─────────────────────────────────────────────────────────────────────
// 怎么看这份输出
//
// 盯着**敌人B** 那一行走势就够了：
//
//   · 它的值从头到尾都是 100，一次都没变
//   · 但 `[Changed] 敌人B` 会**每一帧都出现**
//
// 而敌人A 只在第 2、4 帧（真正扣了血的那两帧）出现。
// 同一个 `Changed<Health>` 过滤器，一个"按预期报"，一个"谎报"——
// 差别只在于代码有没有白白发生可变借用。
//
// 这就是 Bevy 变更检测最容易误解的一点：
//
//   `Changed<T>` 说的是"这帧有人可变访问过 T"，
//   **不是** "T 的值和上帧不同"。
//
// 想表达"值真的不同才处理"，就用 `set_if_neq` 去写
// （它需要组件实现 `PartialEq`）。这也是为什么 Bevy 的组件常顺手 derive 上它。
//
// ─────────────────────────────────────────────────────────────────────
// API 速查
//
//   查询过滤器（会**筛掉**实体）
//     `Changed<T>`              这一帧被写过（含刚加上）
//     `Added<T>`                这一帧刚被加上
//     `Changed<T>` 可与别的过滤组合：`(Changed<Health>, With<EnemyA>)`
//
//   取值不改筛选（能**问**，但不筛）
//     `Ref<T>`                  有 `.is_added()` / `.is_changed()` / `.last_changed()`
//     `Mut<T>`（就是 `&mut T`）  也有同样的三个方法
//
//   资源
//     `Res::is_added()` / `Res::is_changed()`     资源版；`ResMut` 同样可用
//
//   移除
//     `RemovedComponents<T>`     `.read()` 迭代出被移除的那些实体
//
// 提醒两点：
//   1. 变更检测是**按系统**记进度的。同一个变更，A 系统看到了，B 系统也照样能看到 ——
//      不存在"被谁读掉了"。所以 `show_changed` 和别的系统互不干扰。
//   2. 想定位"到底哪一行改的"，可以开 `track_location` feature，然后用
//      `Ref::changed_by()` 拿到文件与行号（调试用，别带到发布版）。
//
// ─────────────────────────────────────────────────────────────────────
// 什么时候值得用它
//
// 变更检测省的是"重复劳动"，所以只在**活够重**的时候才划算：
//
//   ✅ 值得：重建 UI 布局、重算寻路、上传 GPU 缓冲、写日志
//   ❌ 不值：`x += 1` 这种本来就极便宜的操作 ——
//           过滤器本身也有开销，得不偿失
//
// 换来的另一个好处是**语义更准**："血量变了才刷新血条"
// 比"每帧都刷新"既省又对。
//
// ─────────────────────────────────────────────────────────────────────
// 踩坑清单
//
// 1. **无谓的可变借用会谎报变更**（本讲的反例）。写组件前先想想：
//    我是"可能改"还是"真的改"？后者用 `set_if_neq`。
// 2. **`Changed` 在组件刚加上的那一帧也是真**（`Added` 是它的子集）。
//    不想混在一起，就把两者分开判断。
// 3. **`Res::is_changed()` 是"相对我这个系统"的**，不是全局时间戳。
//    系统跑得少，看到的变化就"攒"得多。
// 4. **`RemovedComponents` 是缓冲的**（和 012 的消息同源），
//    读者各自记录进度，所以多个系统都能读到同一次移除。
// ─────────────────────────────────────────────────────────────────────
