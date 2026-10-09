//! 019 · 状态进阶
//!
//! 运行：`cargo run --example 019_state_advanced`
//!
//! 新增概念
//!   SubStates          只在某个父状态里**才存在**的子状态
//!   #[source(..)]      声明它依附于哪个父状态的哪个取值
//!   DespawnOnExit(S)   实体在工作状态结束时**自动销毁**，不用手写清理系统
//!   ComputedStates     由别的状态**推导**出来的状态（本讲末尾介绍）
//!
//! 使用场景
//!   "暂停"只在游戏中有意义 —— 菜单里根本不该有"是否暂停"这个状态
//!   关卡里生成的一堆实体，离开关卡时该整体清掉，而不是逐个记得 despawn
//!
//! 注意：子状态**不存在**时，`Res<State<子状态>>` 也拿不到 ——
//!       所以要用 `Option<Res<State<..>>>` 去问，而不是直接 `Res`

use bevy::log::LogPlugin;
use bevy::prelude::*;
// ⚠️ 状态机不是"免费自带"的：`DefaultPlugins` 里含 `StatesPlugin`，
// 而本讲只装了 `LogPlugin`，所以要显式补上 —— 忘了会在 `init_state` 处 panic。
use bevy::state::app::StatesPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins((LogPlugin::default(), StatesPlugin));
    app.init_state::<AppState>()
        // 注册子状态。它只在 AppState::Playing 里存在。
        .add_sub_state::<IsPaused>()
        .insert_resource(Frame(0))
        // `Progress` 得注册，否则读它的系统会 panic（009 讲的：缺资源会炸，不是跳过）
        .init_resource::<Progress>()
        .add_systems(Startup, spawn_props_in_menu)
        .add_systems(
            Update,
            (
                // 用帧号驱动，这样输出完全确定（真实项目里是按键 / UI 触发）
                drive_transitions,
                advance_progress.run_if(in_state(IsPaused::Running)),
                report,
            )
                .chain(),
        )
        .add_systems(OnEnter(AppState::Playing), enter_playing)
        .add_systems(OnExit(AppState::Playing), exit_playing);

    // 注意这里**不调 `.run()`** —— 手动逐步推进，才能看清每一帧的状态。
    // 用 6 帧而不是 5 帧：状态切换在**帧末**生效，所以第 5 帧请求切回 Menu 之后，
    // 要再多跑一帧才看得到"子状态消失 + 道具被自动销毁"。
    for frame in 1..=6 {
        app.insert_resource(Frame(frame));
        println!("═══ 第 {frame} 帧 ═══");
        app.update();
    }
}

/// 父状态。只有两个：菜单、游戏中。
#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
enum AppState {
    #[default]
    Menu,
    Playing,
}

/// 子状态：**只在 `AppState::Playing` 里存在**。
///
/// 换做菜单阶段，"是否暂停"这个问题本身就没有意义 —— 子状态正好表达这一点：
/// 它不存在，而不是"存在但无意义"。
#[derive(SubStates, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
#[source(AppState = AppState::Playing)]
enum IsPaused {
    #[default]
    Running,
    Paused,
}

/// 当前帧号，由 `main` 每帧写入。
#[derive(Resource)]
struct Frame(u32);

/// 用来证明"暂停时逻辑真的停了"。
#[derive(Resource, Default)]
struct Progress(u32);

#[derive(Component)]
struct SceneProp(&'static str);

/// 菜单里生成的道具**不带** `DespawnOnExit` —— 它应该一直在。
fn spawn_props_in_menu(mut commands: Commands) {
    commands.spawn(SceneProp("菜单背景板"));
}

fn enter_playing(mut commands: Commands) {
    println!("   [OnEnter] Playing —— 生成 3 个带 DespawnOnExit(Playing) 的道具");
    for name in ["道具甲", "道具乙", "道具丙"] {
        // ★ 关键：挂上 DespawnOnExit，离开 Playing 时引擎**自动**销毁它。
        // 不需要写 OnExit 清理系统，也不怕漏掉某个层级深处的实体。
        commands.spawn((SceneProp(name), DespawnOnExit(AppState::Playing)));
    }
}

fn exit_playing() {
    println!("   [OnExit ] Playing —— 副产物：那 3 个道具会被自动销毁");
}

/// 只在"未暂停"时推进进度 —— 这就是子状态最实际的用途。
fn advance_progress(mut progress: ResMut<Progress>) {
    progress.0 += 1;
}

fn drive_transitions(
    frame: Res<Frame>,
    mut app_state: ResMut<NextState<AppState>>,
    // ⚠️ 子状态不存在时 `NextState<IsPaused>` 也拿不到，所以必须用 Option。
    paused: Option<ResMut<NextState<IsPaused>>>,
) {
    match frame.0 {
        2 => app_state.set(AppState::Playing),
        3 => {
            if let Some(mut paused) = paused {
                paused.set(IsPaused::Paused);
            }
        }
        4 => {
            if let Some(mut paused) = paused {
                paused.set(IsPaused::Running);
            }
        }
        5 => app_state.set(AppState::Menu),
        _ => {}
    }
}

/// 每帧汇报"现在处于什么状态、场景里有多少东西"。
fn report(
    app_state: Res<State<AppState>>,
    paused: Option<Res<State<IsPaused>>>,
    progress: Res<Progress>,
    props: Query<&SceneProp>,
) {
    // 子状态不存在时拿到 None —— 直接就问出"当前有没有这个概念"。
    let pause_text = paused.as_ref().map_or("不存在".to_string(), |state| {
        format!("{:?}", state.get())
    });
    // 把名字也列出来（顺便证明"自动销毁"销毁的确实是那几个）
    let mut names: Vec<&str> = props.iter().map(|prop| prop.0).collect();
    names.sort_unstable();

    println!(
        "   AppState={:?}  IsPaused={:<7} Progress={}  场景 {:?}",
        app_state.get(),
        pause_text,
        progress.0,
        names
    );
}

// ─────────────────────────────────────────────────────────────────────
// 实测输出
//
//   ═══ 第 1 帧 ═══
//      AppState=Menu  IsPaused=不存在     Progress=0  场景 ["菜单背景板"]
//   ═══ 第 2 帧 ═══
//      AppState=Menu  IsPaused=不存在     Progress=0  场景 ["菜单背景板"]
//   ═══ 第 3 帧 ═══
//      [OnEnter] Playing —— 生成 3 个带 DespawnOnExit(Playing) 的道具
//      AppState=Playing  IsPaused=Running Progress=1  场景 ["菜单背景板", "道具丙", "道具乙", "道具甲"]
//   ═══ 第 4 帧 ═══
//      AppState=Playing  IsPaused=Paused  Progress=1  场景 ["菜单背景板", "道具丙", "道具乙", "道具甲"]
//   ═══ 第 5 帧 ═══
//      AppState=Playing  IsPaused=Running Progress=2  场景 ["菜单背景板", "道具丙", "道具乙", "道具甲"]
//   ═══ 第 6 帧 ═══
//      [OnExit ] Playing —— 副产物：那 3 个道具会被自动销毁
//      AppState=Menu  IsPaused=不存在     Progress=2  场景 ["菜单背景板"]
//
// （道具名按名字排序后是 丙/乙/甲 —— 中文按 Unicode 码点比较，跟出场顺序无关，
//   这里只是为了让输出稳定，不影响结论。）
//
// ─────────────────────────────────────────────────────────────────────
// 怎么读这份输出
//
// 盯着三列看就够了：
//
// 1. **`IsPaused=` 那一列**
//      · 第 1、2 帧（Menu）：`不存在` —— 子状态根本没被创建
//      · 第 3~5 帧（Playing）：`Running` / `Paused` / `Running`
//      · 第 6 帧（切回 Menu）：又变回 `不存在`
//    这就是子状态的核心语义：**它随父状态一起出现和消失**，
//    而不是"一直存在、只是值没意义"。
//
// 2. **`Progress=` 那一列**
//      · 第 1、2 帧是 0（还在菜单，那个系统被 `in_state(IsPaused::Running)` 挡掉了）
//      · 第 3 帧 +1、第 5 帧再 +1
//      · **第 4 帧暂停时原地不动**
//    这说明 `in_state(子状态)` 和 `in_state(父状态)` 一样好用，
//    而且能把"暂停时该停的逻辑"精确圈出来。
//
// 3. **`场景` 那一列**
//      · Menu 里始终只有 1 个（菜单背景板，没挂 `DespawnOnExit`）
//      · 进入 Playing 后多出 3 个道具
//      · 第 6 帧离开 Playing 后**自动**回到 1 个 —— 全程没人写过清理代码
//
// 还有一个**时序**细节：第 2 帧就请求切到 Playing 了，状态却到**第 3 帧**才变；
// 第 5 帧请求切回 Menu，第 6 帧才生效。原因是状态切换发生在
// **帧末的 `StateTransition` 调度**里（018 讲的），
// 再加上 `OnEnter` 里的 `commands.spawn` 还要等一个同步点 ——
// 所以"发出请求 → 看到成果"中间隔一两帧是正常的。
//
// ─────────────────────────────────────────────────────────────────────
// DespawnOnExit 到底省了什么
//
// 不用它的话，你得写：
//
//   fn exit_playing(mut commands: Commands, props: Query<Entity, With<SceneProp>>) {
//       for entity in &props { commands.entity(entity).despawn(); }
//   }
//
// 问题在于"哪些实体该清"要靠人工维护：新加一类实体就得回来改这个查询，
// 漏一个就泄漏一个。`DespawnOnExit` 把这个约定**写在实体自己身上** ——
// 谁生成、谁声明生命周期，清理逻辑就不用集中维护了。
//
// 它的三个兄弟：
//   `DespawnOnExit(S)`    离开 S 时销毁
//   `DespawnOnEnter(S)`   进入 S 时销毁（适合清掉"上一轮残留"）
//   `DespawnWhen::new(|transition| ..)`   自定义判断，最灵活
//
// 一个细节：**重复挂也不会出错**。如果实体已经被销毁，引擎不会报错，
// 所以层级深处多挂几个 `DespawnOnExit` 是安全的。
//
// ─────────────────────────────────────────────────────────────────────
// 三种状态的关系
//
//   `States`         普通状态，独立存在
//   `SubStates`      依附于父状态，父状态不满足时**整个不存在**
//   `ComputedStates` 由别的状态**算出来**，没有自己的 NextState，不能直接切
//
// `ComputedStates` 的样子（本讲不现场演示，因为它更像"派生查询"而非独立状态）：
//
//   impl ComputedStates for InGame {
//       type SourceStates = AppState;
//       fn compute(sources: AppState) -> Option<Self> {
//           matches!(sources, AppState::Playing).then_some(InGame)
//       }
//   }
//   // 注册：app.add_computed_state::<InGame>()
//
// 它适合"好几个状态下都要跑同一批逻辑"的场景：与其在每处写
// `run_if(in_state(A).or(in_state(B)))`，不如算出一个共同状态。
// `SourceStates` 还可以是元组，从多个状态一起推导。
//
// ─────────────────────────────────────────────────────────────────────
// 拆解 019 规格里没做的部分
//
// 规划里还提到 `ComputedStates` 的现场演示。最终没做，原因是它和
// `SubStates` 的演示目标重叠（都在讲"状态之间的派生关系"），
// 而一讲里塞两种派生机制会削弱各自的重点 —— 按规划原则第 8 条，
// 留在这里说明即可，需要时再单独跑一遍验证。
// ─────────────────────────────────────────────────────────────────────
