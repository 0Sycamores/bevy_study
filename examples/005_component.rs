//! 005 · 组件：挂在实体上的数据
//!
//! 运行：`cargo run --example 005_component`
//!
//! 组件(Component)就是"挂在实体上的数据"。实体(Entity)本身只是个 id，
//! 它的全部含义都来自身上挂了哪些组件。
//! 查询(Query)则是"按组件组合筛选实体"。

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "005 · 组件".into(),
                resolution: (960, 640).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (move_players, damage_enemy))
        .run();
}

// ── 两种组件 ──────────────────────────────────────────────────────────
// ① 标记组件(marker)：没有字段，只用来给实体贴标签，方便查询筛选。
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

// ② 数据组件：每个实体各存一份值，字段就是这个实体自己的状态。
#[derive(Component)]
struct Speed(f32);

#[derive(Component)]
struct Health(f32);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // 实体 = 一组组件的集合。spawn 时给什么，这个实体就"是什么"。
    // 下面两个实体都有 (Player, Speed, Sprite, Transform)，属于同一"类"。
    commands.spawn((
        Player,
        Speed(1.2),
        Sprite::from_color(Color::srgb(0.35, 0.62, 0.95), Vec2::splat(70.0)),
        Transform::from_xyz(-400.0, 130.0, 1.0),
    ));
    commands.spawn((
        Player,
        Speed(2.6),
        Sprite::from_color(Color::srgb(0.36, 0.82, 0.66), Vec2::splat(70.0)),
        Transform::from_xyz(-400.0, -30.0, 1.0),
    ));

    // 这个是另一"类"：有 Enemy 和 Health，但没有 Player、也没有 Speed。
    commands.spawn((
        Enemy,
        Health(100.0),
        Sprite::from_color(Color::srgb(0.91, 0.30, 0.31), Vec2::splat(90.0)),
        Transform::from_xyz(300.0, 40.0, 1.0),
    ));

    // 注意这里从来没写过 `Visibility`。它和 `Transform` 都是 `Sprite` 的
    // **必需组件**(required components)，引擎会自动补上（003 讲过）。
}

/// 查询 = "给我所有同时满足这些组件要求的实体"。
///
/// 参数分两截：
///   · 元组 `(&Speed, &mut Transform)` —— 要读/写哪些组件的数据
///   · 过滤器 `With<Player>`          —— 额外的筛选条件
///
/// `&mut players` 要求查询本身可变，因为里面含 `&mut Transform`。
fn move_players(mut players: Query<(&Speed, &mut Transform), With<Player>>) {
    for (speed, mut transform) in &mut players {
        transform.translation.x += speed.0;
        // 跑出右边界就绕回左边，好让演示一直进行下去
        if transform.translation.x > 460.0 {
            transform.translation.x = -460.0;
        }
    }
}

/// 这个查询没有任何过滤器：所有带 `Health` 的实体都会被命中。
/// 用 `&mut Health` 就能改写组件里的数据。
///
/// （每帧固定扣 0.4，所以掉血速度与帧率有关。010 讲 `Time` 时会改成帧率无关。）
fn damage_enemy(mut enemies: Query<(&mut Health, &mut Sprite)>) {
    for (mut health, mut sprite) in &mut enemies {
        health.0 -= 0.4;
        if health.0 <= 0.0 {
            health.0 = 100.0; // 扣光了就回满，方便反复观察
        }
        // 血量越低颜色越暗 —— 组件数据被改写后立刻反映到画面上
        let ratio = health.0 / 100.0;
        sprite.color = Color::srgb(0.91 * ratio, 0.30 * ratio, 0.31 * ratio);
    }
}

// ─────────────────────────────────────────────────────────────────────
// 跑起来会看到什么
//
//   · 蓝色、绿色两个方块持续向右移动，且**速度不同**（Speed 各存各的值）
//   · 红色方块**原地不动**，但颜色由红逐渐变暗，然后回满、循环
//
// "不动"就是本讲的核心证据：
//   `move_players` 要求实体带 `Player` 标记，而红方块只有 `Enemy`。
//   它的 `Transform` 组件明明也在，但**没有 `Player` 就不会被命中** ——
//   查询匹配的是"组件组合"，不是"某一个组件"。
//
// 同理，`damage_enemy` 没有过滤器，但只带 `Health` 的红方块被命中，
// 两个蓝绿方块（有 Sprite 和 Transform、没有 Health）碰都不碰。
//
// ── 组件 vs 资源 ──
//
// 组件是"每个实体各有一份"的数据；资源(Resource)是"整个世界只有一份"的数据。
// 上面的 `Speed`、`Health` 是组件；分数、游戏配置那一类东西是资源，009 讲。
//
// ── 几个容易踩的点 ──
//
// 1. 一个实体上同一种组件**只能有一份**。想存两个数就定义两个组件，
//    或者让组件里带多个字段，不能给一个实体挂两个 `Speed`。
// 2. `derive(Component)` 的类型必须 `Send + Sync + 'static`。
//    普通结构体一般都满足，除非里面塞了 `Rc` 这类东西。
// 3. 查询命中 0 个实体完全正常：`for` 直接不执行，`iter().count()` 得 0，
//    系统照常运行、不会报错，也**不会**被跳过。
//    （对比 006 要讲的 `Single`：它要求"恰好一个"，不满足会跳过整个系统。）
//
// ── 这两个系统会并行跑 ──
//
// `move_players` 写 `Transform`，`damage_enemy` 写 `Health` 和 `Sprite`，
// 两者没有共同数据，所以引擎会把它们**并行**调度。
// 按 004 讲的，这种"顺序随意"是好事，不是隐患。
//
// ── 本讲出现的查询写法，后面会逐个展开 ──
//
//   Query<(&Speed, &mut Transform), With<Player>>   读写数据 + 过滤器   ← 本讲
//   Query<(&mut Health, &mut Sprite)>               没有过滤器          ← 本讲
//   query.single() / Single<&T>                     要求恰好一个        → 006
//   query.iter() / par_iter_mut()                   遍历的几种写法      → 006
//   Query<&mut Transform, Without<Player>>          反向过滤            → 007
//   Query<Entity, Or<(With<A>, With<B>)>>           或过滤              → 007
//   Query<(&Label, Has<Health>)>                    探测有没有某组件    → 007
//   commands.spawn(..) / entity(e).insert(..)       增删改              → 008
// ─────────────────────────────────────────────────────────────────────
