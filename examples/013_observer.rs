//! 013 · 观察者与实体事件
//!
//! 运行：`cargo run --example 013_observer`
//!
//! 新增概念
//!   EntityEvent        带"目标实体"的事件：`#[derive(EntityEvent)]`
//!   On<E>              观察者系统的参数，能拿到事件数据与 `on.entity`
//!   observe / add_observer   把观察者挂在**某一个实体**上 / 挂成**全局**的
//!   trigger            触发事件：`commands.trigger(..)`
//!   生命周期事件       `On<Add<T>>` / `On<Remove<T>>` —— 组件增删时自动触发
//!
//! 使用场景
//!   某个实体出了事、只有关心它的那几个人要响应：被击中、被拾取、血条归零
//!   组件被加上/移除时做配套动作：进索引、清缓存、播特效
//!
//! 注意：`commands.trigger` 是**延迟**执行的（和 008 的命令一样），观察者在同步点才跑

use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(LogPlugin::default());

    // 全局观察者：任何实体触发 Hit 都能看到。
    app.add_observer(on_any_hit);
    // 生命周期观察者：任何实体被加上 Health 组件时都会跑。
    app.add_observer(on_health_added);
    app.add_observer(on_died);

    app.add_systems(Startup, spawn_enemies);
    app.add_systems(Update, fire);

    for frame in 1..=2 {
        println!("═══ 第 {frame} 帧 ═══");
        app.update();
    }
}

#[derive(Component)]
struct Enemy(&'static str);

#[derive(Component)]
struct Health(i32);

/// `EntityEvent`：事件自带一个**目标实体**，所以能精准投递。
#[derive(EntityEvent)]
struct Hit {
    entity: Entity,
    damage: i32,
}

/// 同样带目标实体：谁的血掉光了。
#[derive(EntityEvent)]
struct Died {
    entity: Entity,
}

fn spawn_enemies(mut commands: Commands) {
    // 给"敌人A"单独挂一个**只关心它自己**的观察者。
    // `observe` 写在实体命令上，所以这个观察者只监听这一个实体。
    commands
        .spawn((Enemy("敌人A"), Health(10)))
        .observe(on_hit_this_entity);

    // 这两个没有专属观察者，只会被全局观察者看到。
    commands.spawn((Enemy("敌人B"), Health(10)));
    commands.spawn((Enemy("敌人C"), Health(10)));
}

/// 对 A、B 各打一下，C 不动 —— 用来验证"精准投递"。
fn fire(mut commands: Commands, enemies: Query<(Entity, &Enemy)>) {
    // 显式排序，不依赖遍历顺序（006 讲的）。
    let mut list: Vec<(Entity, &Enemy)> = enemies.iter().collect();
    list.sort_by_key(|(_, enemy)| enemy.0);

    for (entity, enemy) in list {
        if enemy.0 == "敌人C" {
            continue;
        }
        println!("   [触发] 对 {} 造成 6 点伤害", enemy.0);
        commands.trigger(Hit { entity, damage: 6 });
    }
}

/// 全局观察者：**每个** Hit 都会跑。
fn on_any_hit(
    hit: On<Hit>,
    enemies: Query<&Enemy>,
    mut healths: Query<&mut Health>,
    mut commands: Commands,
) {
    let name = name_of(&enemies, hit.entity);
    println!("   [全局观察者] {} 被击中，伤害 {}", name, hit.damage);

    if let Ok(mut health) = healths.get_mut(hit.entity) {
        health.0 -= hit.damage;
        println!("   [全局观察者] {} 剩余血量 {}", name, health.0);
        if health.0 <= 0 {
            commands.trigger(Died { entity: hit.entity });
        }
    }
}

/// 专属观察者：只挂在"敌人A"身上，所以**只有 A 的 Hit 会跑到这里**。
fn on_hit_this_entity(hit: On<Hit>) {
    println!("   [专属观察者] 这是敌人A 自己的观察者收到了 Hit");
    let _ = hit.entity;
}

/// 生命周期事件：`On<Add<T>>` 在组件被**加上**时自动触发，不用手动 trigger。
fn on_health_added(add: On<Add<Health>>) {
    println!("   [生命周期] 有实体被加上了 Health（{:?}）", add.entity);
}

fn on_died(died: On<Died>, enemies: Query<&Enemy>, mut commands: Commands) {
    let name = name_of(&enemies, died.entity);
    println!("   [观察者] {} 阵亡，销毁实体", name);
    commands.entity(died.entity).despawn();
}

fn name_of(enemies: &Query<&Enemy>, entity: Entity) -> &'static str {
    enemies.get(entity).map(|enemy| enemy.0).unwrap_or("?")
}

// ─────────────────────────────────────────────────────────────────────
// 观察者 vs 消息（012）—— 什么时候用哪个
//
//              消息 Message（012）              观察者 Observer（本讲）
//   投递范围    全局广播，谁都能读              可精准投递给某个实体
//   谁跑        没人读就丢掉                  订阅了就跑，不看有没有人关心
//   时机        缓冲两帧，晚一帧读到也正常      触发即达（命令则在同步点）
//   典型场景    "有件事发生了"                  "这个实体出事了"
//               玩家死了、得分变了              这个敌人被击中、这个按钮被点了
//
// 一句话：**消息是"广播"，观察者是"点名"。**
//
// 两者不是替代关系：同一件事既可以发消息、也可以触发事件，看你要"广播"还是"点名"。
//
// ─────────────────────────────────────────────────────────────────────
// 本讲的四个层次
//
// 1. **全局观察者**：`app.add_observer(sys)` —— 所有同类事件都跑。
// 2. **实体专属观察者**：`commands.spawn(..).observe(sys)` —— 只监听那一个实体。
//    这正是 `EntityEvent` 存在的意义：事件带着 `entity`，才能"点名"。
// 3. **生命周期事件**：`On<Add<T>>` / `On<Remove<T>>`，组件增删时**自动**触发，
//    不需要你写 `trigger`。`Add` 在组件被加上时、`Remove` 在被移除时
//    （`despawn` 也算移除）各触发一次。
// 4. **链式反应**：观察者里可以再 `trigger` 别的事件（上面血量归零 → `Died`），
//    于是"击中 → 掉血 → 阵亡 → 销毁"整条链自动跑完，每一步的双方互不认识。
//
// ─────────────────────────────────────────────────────────────────────
// 观察者的能力
//
// 观察者**就是系统**，参数规则和普通系统一样，可以有：
//   `On<E>`            事件本身（唯一必须的参数之一）
//   `Query<..>`        查实体
//   `Res` / `ResMut`   查资源
//   `Commands`         排队改世界
//
// 所以它几乎能干系统能干的一切，区别只是"什么时候跑"。
// 它还能带运行条件：`app.add_observer(sys.run_if(某个条件))`。
//
// ─────────────────────────────────────────────────────────────────────
// 一个容易困惑的点：为什么我的改动要等到"同步点"才生效
//
// `commands.trigger(..)` 和 `commands.spawn(..)` 一样是**排队**的（008 讲的），
// 要等到同步点才真正执行，观察者也才跑。想让事件**立即**触发，得用
// `world.trigger(..)`（需要独占世界访问）。
//
// 另外观察者之间的执行顺序**不做保证**（全局观察者与专属观察者谁先跑没规定），
// 所以别写"依赖某个观察者先跑"的逻辑 —— 又是 004 讲的同一类问题。
//
// ─────────────────────────────────────────────────────────────────────
// 进阶：事件冒泡
//
// `EntityEvent` 支持沿父子层级**向上冒泡**（子实体上的事件，父实体也能观察到），
// 由 `ChildOf` 关系驱动 —— 那是 014 讲层级时才讲得清，这里先知道有这个能力。
// 另外观察者本身就是一个带 `Observer` 组件的**实体**，也可以被查询、被复用
// （`Observer::new(..)` + `watch_entity(..)`）。

// ─────────────────────────────────────────────────────────────────────
// 实测输出
//
//   ═══ 第 1 帧 ═══
//      [生命周期] 有实体被加上了 Health（10v0）
//      [生命周期] 有实体被加上了 Health（11v0）
//      [生命周期] 有实体被加上了 Health（12v0）
//      [触发] 对 敌人A 造成 6 点伤害
//      [触发] 对 敌人B 造成 6 点伤害
//      [全局观察者] 敌人A 被击中，伤害 6
//      [全局观察者] 敌人A 剩余血量 4
//      [专属观察者] 这是敌人A 自己的观察者收到了 Hit
//      [全局观察者] 敌人B 被击中，伤害 6
//      [全局观察者] 敌人B 剩余血量 4
//   ═══ 第 2 帧 ═══
//      [触发] 对 敌人A 造成 6 点伤害
//      [触发] 对 敌人B 造成 6 点伤害
//      [全局观察者] 敌人A 被击中，伤害 6
//      [全局观察者] 敌人A 剩余血量 -2
//      [专属观察者] 这是敌人A 自己的观察者收到了 Hit
//      [观察者] 敌人A 阵亡，销毁实体
//      [全局观察者] 敌人B 被击中，伤害 6
//      [全局观察者] 敌人B 剩余血量 -2
//      [观察者] 敌人B 阵亡，销毁实体
//
// 四个结论都能从输出里读出来：
//
// 1. **生命周期事件不用手动触发**：第 1 帧开头那三行 `On<Add<Health>>` 是三个敌人
//    生成时自动跑的 —— 注意它们出现在 `[触发]` 之前，因为 `Startup` 跑完就有一个
//    同步点，命令（含 spawn）在那里落地。
// 2. **精准投递**：`[专属观察者]` 只在**敌人A** 被击中时出现，敌人B 那次没有。
//    三个敌人都被全局观察者看到了，但只有 A 有自己的专属观察者。
// 3. **敌人C 全程没出现** —— 我们压根没对它 trigger，它自然什么都不会发生。
// 4. **链式反应**：第 2 帧血量掉到 -2 后，全局观察者里 `trigger(Died)`，
//    于是 `[观察者] 敌人A 阵亡，销毁实体` 紧接着跑。全程"击中"和"销毁"
//    两段逻辑互不认识，只靠事件串起来。
//
// 注意 `commands.trigger` 是**延迟**的：第 1 帧的 `[触发]` 两行先打印，
// 观察者的输出统统排在它们后面 —— 命令要等同步点（这里是 `Update` 结束时）才生效。
//
// ─────────────────────────────────────────────────────────────────────
// API 速查
//
//   #[derive(EntityEvent)] struct E { entity: Entity, .. }   带目标实体的事件
//   #[derive(Event)] struct E { .. }                         不带目标（纯全局广播）
//   commands.trigger(E { .. })                               触发（延迟，同步点生效）
//   app.add_observer(sys)                                    全局观察者
//   commands.spawn(..).observe(sys)                          只监听这个实体
//   On<E>                                                    观察者参数：事件数据 + .entity
//   On<Add<T>> / On<Remove<T>>                               组件增删时自动触发
//   sys.run_if(..)                                           观察者也能带运行条件
//
// 不在 prelude 里的：`EntityEvent` 的派生宏来自 `bevy::ecs::event`，但**通常不用手写 use** ——
// `bevy::prelude::*` 里已经有它（本讲没有额外 import）。
// ─────────────────────────────────────────────────────────────────────
