//! 016 · 插件
//!
//! 运行：`cargo run --example 016_plugin`
//!
//! 新增概念
//!   Plugin          把"一组资源 + 一组系统 + 一组事件"打包成一个可复用单元
//!   Plugin::build   注册的地方：`init_resource` / `add_systems` / `add_message` 都在这
//!   PluginGroup     把多个插件再打包成一个，对外只暴露一个名字
//!   SystemSet       插件之间也要声明顺序 —— 给一组系统起名，再对整组排序
//!   DefaultPlugins.set(..) / .disable::<T>()   覆盖或关掉引擎自带的插件
//!
//! 使用场景
//!   功能模块化：敌人一套、分数一套、音频一套，各自成插件
//!   `main` 里只剩一张插件清单，完全看不出实现细节
//!
//! 注意：插件之间的协作靠**共享资源 / 消息 / 事件**，不是靠互相调用 ——
//!       所以"分数"这类跨界状态必须有一个明确的归属插件，顺序也要显式声明

use bevy::app::PluginGroupBuilder;
use bevy::log::LogPlugin;
use bevy::prelude::*;

/// 插件之间也会互相依赖顺序。给系统分组起名，然后对**整组**声明先后 ——
/// 这样每个插件只管把自己的系统丢进对应的组，顺序约束集中在一处声明。
///
/// 本讲里 `ScorePlugin` 的展示系统必须在 `EnemyPlugin` 的结算之后，
/// 否则本帧加的分要等下一帧才显示（甚至因为看不到而永远不显示）。
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GameSet {
    /// 敌人推进与结算
    Enemy,
    /// 分数展示
    Score,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(LogPlugin::default());

    // ★ 整个游戏就这一行 —— 有什么功能、怎么组织，全在插件里。
    app.add_plugins(GamePlugins);

    // 跨插件的顺序约束在这里统一声明：谁都不需要知道对方的内部结构，
    // 只需要知道对方属于哪个 SystemSet。
    app.configure_sets(Update, GameSet::Score.after(GameSet::Enemy));

    for frame in 1..=3 {
        println!("═══ 第 {frame} 帧 ═══");
        app.update();
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 插件一：分数
// ═══════════════════════════════════════════════════════════════════════

/// `Score` 归属于 `ScorePlugin` —— 别的插件只管写，怎么展示由它自己决定。
#[derive(Resource, Default)]
struct Score(i32);

struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>()
            .add_systems(Update, report_score.in_set(GameSet::Score));
    }
}

/// 只在分数**变过**的时候打印（015 讲的 `is_changed`），避免每帧刷屏。
fn report_score(score: Res<Score>) {
    if score.is_changed() {
        println!("   [分数插件] Score = {}", score.0);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 插件二：敌人
// ═══════════════════════════════════════════════════════════════════════

#[derive(Component)]
struct Enemy {
    /// 出场顺序。**不要靠名字或遍历顺序排序**（006 讲的），显式给个号最稳。
    id: u32,
    name: &'static str,
    /// 走到 9 就算抵达终点
    progress: i32,
}

struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_enemies).add_systems(
            Update,
            // 先移动、再判定抵达 —— 组内顺序也要显式声明（004 讲的）
            (move_enemies, check_arrival).chain().in_set(GameSet::Enemy),
        );
    }
}

fn spawn_enemies(mut commands: Commands) {
    commands.spawn(Enemy {
        id: 1,
        name: "小兵甲",
        progress: 0,
    });
    commands.spawn(Enemy {
        id: 2,
        name: "小兵乙",
        progress: 0,
    });
}

fn move_enemies(mut enemies: Query<&mut Enemy>) {
    for mut enemy in &mut enemies {
        enemy.progress += 3;
    }
}

/// 抵达终点就加分并销毁。
///
/// 注意这里直接写了 `ResMut<Score>` —— 而 `Score` 是 `ScorePlugin` 注册的。
/// **插件之间就是这样协作的**：通过共享的资源，而不是拿到对方的内部引用。
/// 代价是"谁拥有 Score"变成了一个需要约定的问题，所以要在注释里写清归属。
fn check_arrival(
    mut commands: Commands,
    enemies: Query<(Entity, &Enemy)>,
    mut score: ResMut<Score>,
) {
    // 按自己的 `id` 排序，不依赖遍历顺序（006 讲的）
    let mut arrived: Vec<(Entity, &Enemy)> = enemies
        .iter()
        .filter(|(_, enemy)| enemy.progress >= 9)
        .collect();
    arrived.sort_by_key(|(_, enemy)| enemy.id);

    for (entity, enemy) in arrived {
        println!("   [敌人插件] {} 抵达终点，+10 分", enemy.name);
        score.0 += 10;
        commands.entity(entity).despawn();
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 把上面两个插件再打包成一个
// ═══════════════════════════════════════════════════════════════════════

/// 对外只需要记住这一个名字。
struct GamePlugins;

impl PluginGroup for GamePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(ScorePlugin)
            .add(EnemyPlugin)
    }
}

// ─────────────────────────────────────────────────────────────────────
// 实测输出
//
//   ═══ 第 1 帧 ═══
//      [分数插件] Score = 0
//   ═══ 第 2 帧 ═══
//   ═══ 第 3 帧 ═══
//      [敌人插件] 小兵甲 抵达终点，+10 分
//      [敌人插件] 小兵乙 抵达终点，+10 分
//      [分数插件] Score = 20
//
// 两处值得说明：
//
// 1. 第 1 帧那行 `Score = 0` 有点意外 —— 分数明明没加过。
//    因为**资源刚被创建的那一帧也算"变过"**（`Added` 是 `Changed` 的子集，015 讲的）。
//
// 2. `Score = 20` 出现在**同一帧**的敌人结算之后，这不是自动的。
//    初版没写 `.in_set` / `configure_sets` 时，`report_score` 与 `check_arrival`
//    顺序未定，结果那一帧分数加了却**永远没被打印出来**（只有 3 帧，错过了就没了）。
//    这正是 004 讲的顺序问题，只不过跨了插件边界 —— 修法是给系统分组、
//    在**一个地方**声明组间顺序。
//
// ─────────────────────────────────────────────────────────────────────
// 插件到底做了什么
//
// `Plugin::build(&self, app: &mut App)` 里能做的，就是你在 `main` 里能做的一切：
//
//   app.init_resource::<T>() / insert_resource(..)     注册资源
//   app.add_systems(Update, ..)                        注册系统
//   app.add_message::<T>() / add_event::<T>()          注册消息 / 事件
//   app.add_observer(..)                               注册观察者
//   app.init_state::<S>()                              注册状态（018 讲）
//   app.add_plugins(别的插件)                          插件还能装插件
//
// 所以插件不是一个新概念，它只是**把一段配置搬了个家** ——
// 从"堆在 main 里"变成"归属于某个功能模块"。
//
// `Plugin` trait 还有两个可选方法：
//   `finish(&self, app)`   所有插件的 `build` 都跑完之后再跑（适合做收尾校验）
//   `cleanup(&self, app)`  插件被移除时跑
// 日常几乎用不到，知道有就行。
//
// ─────────────────────────────────────────────────────────────────────
// 插件组的调整方法
//
// `PluginGroupBuilder` 提供了一串方法，`DefaultPlugins` 就是靠它们定制的：
//
//   .add(SomePlugin)                     追加
//   .disable::<SomePlugin>()             关掉其中一个
//   .add_before::<SomePlugin>(other)     插在某个插件之前
//   .add_after::<SomePlugin>(other)      插在某个插件之后
//
// 典型用法：
//
//   app.add_plugins(
//       DefaultPlugins
//           .set(WindowPlugin { .. })                 // 覆盖某个插件的配置
//           .disable::<bevy::log::LogPlugin>(),       // 关掉日志
//   );
//
// 注意 `.set(..)` 与 `.disable::<..>()` 作用在**插件组**上；
// 自定义组则从 `PluginGroupBuilder::start::<Self>()` 开始搭。
//
// ─────────────────────────────────────────────────────────────────────
// 什么时候该拆插件
//
//   ✅ 值得：一个功能有**自己的资源 + 多个系统**，且希望"装上就有一整套"
//   ✅ 值得：这段逻辑可能被别的项目复用
//   ❌ 不值：只有一两个系统、也用不到专属资源 —— 直接写在 `main` 里更清楚
//
// 一个实用的判断：**如果你要为这组东西起个名词（"敌人"、"分数"、"音频"），
// 那它多半就该是个插件。**
//
// ─────────────────────────────────────────────────────────────────────
// 下一步：017 讲多文件
//
// 本讲所有插件都挤在一个文件里。真实项目里每个插件通常独占一个文件
// （甚至一个目录），`main.rs` 只负责把插件装起来 —— 那是 017 的内容。
// ─────────────────────────────────────────────────────────────────────
