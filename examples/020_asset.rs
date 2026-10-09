//! 020 · 资产加载
//!
//! 运行：`cargo run --example 020_asset`
//!
//! 新增概念
//!   AssetServer::load   从磁盘加载，**立刻**返回句柄
//!   Handle<T>           资产的句柄 —— 拿到句柄 ≠ 加载完成
//!   Assets<T>           资产的仓库：`get(handle)` 是 `None` 就说明还没到位
//!   LoadState           加载进度：`NotLoaded` / `Loading` / `Loaded` / `Failed`
//!   AssetEvent<A>       资产"加载好了 / 被改了"的消息 —— 热重载靠它
//!
//! 使用场景
//!   加载图片、音频、字体、模型；在资源就绪前先显示占位内容
//!
//! 注意：`load` 是**异步**的，返回的句柄只是"取货凭证"。本讲摆三个精灵，
//!       把"不判断状态""规范做法""加载失败"三种情况放在一起

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "020 · 资产加载：左=不判断 / 右=占位替换 / 下=路径不存在".into(),
                resolution: (960, 640).into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<FrameCount>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                tick_frame,
                report_load_state,
                swap_in_image,
                watch_asset_events,
            )
                .chain(),
        )
        .run();
}

/// 每帧 +1，用来在输出里标出"第几帧发生了什么"。
#[derive(Resource, Default)]
struct FrameCount(u32);

/// 存着 logo 的句柄。① 让资产保持存活（`Handle` 是引用计数）；
/// ② 后面过滤资产事件时用它比对 id。
#[derive(Resource)]
struct Logo(Handle<Image>);

/// 挂在"先占位、后替换"的那个精灵上。
#[derive(Component)]
struct LogoImage(Handle<Image>);

#[derive(Component)]
struct Placeholder;

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(Camera2d);

    // `load` 的参数是相对 **`assets/`** 的路径。
    // 这个调用**立刻**返回句柄 —— 内容稍后才到。
    let handle: Handle<Image> = assets.load("textures/logo.png");
    println!("── assets.load(\"textures/logo.png\") 已返回句柄（内容此刻还没到）");

    // ① 反面示范：拿到句柄就直接用。内容到位之前，这个精灵**什么都不显示**。
    commands.spawn((
        Sprite::from_image(handle.clone()),
        Transform::from_xyz(-180.0, 60.0, 0.0),
    ));

    // ② 规范做法：先摆占位方块，等加载好了再换成真图。
    commands.spawn((
        Placeholder,
        LogoImage(handle.clone()),
        Sprite::from_color(Color::srgb(0.32, 0.32, 0.38), Vec2::splat(128.0)),
        Transform::from_xyz(180.0, 60.0, 0.0),
    ));

    // ③ 故意加载一个**不存在**的路径：它会永远停在 `Failed`，精灵永远不会出现。
    //    这不是笔误，是要演示"加载失败不会 panic"—— 只是什么都没有。
    //    （终端里那条 `ERROR bevy_asset::server: Path not found` 是本例故意触发的。）
    let missing: Handle<Image> = assets.load("textures/这个文件不存在.png");
    commands.spawn((
        Sprite::from_image(missing),
        Transform::from_xyz(0.0, -180.0, 0.0),
    ));

    commands.insert_resource(Logo(handle));

    println!("   左：直接使用句柄（内容到位前是空白）");
    println!("   右：占位方块（到位后自动换成真图）");
    println!("   下：不存在的路径（永远不出现，但不会崩）");
}

fn tick_frame(mut frame: ResMut<FrameCount>) {
    frame.0 += 1;
}

/// 直接问引擎"这几个资产现在什么状态"。前几帧打印，看它怎么变。
fn report_load_state(server: Res<AssetServer>, frame: Res<FrameCount>, logo: Res<Logo>) {
    if frame.0 > 4 {
        return;
    }
    let loaded = server.get_load_state(logo.0.id());
    println!("   第 {} 帧  LoadState = {loaded:?}", frame.0);
}

/// 每帧检查内容到了没有，到了就把占位方块换成真图。
fn swap_in_image(
    assets: Res<Assets<Image>>,
    frame: Res<FrameCount>,
    mut pending: Query<(Entity, &LogoImage, &mut Sprite), With<Placeholder>>,
    mut commands: Commands,
) {
    for (entity, logo, mut sprite) in &mut pending {
        // ★ 关键判断：`Assets::get` 返回 `None` 就说明"还没有内容"。
        //   这就是异步的代价 —— "还没有"的那段时间必须自己处理。
        if assets.get(&logo.0).is_none() {
            continue;
        }
        sprite.image = logo.0.clone();
        sprite.color = Color::WHITE;
        commands.entity(entity).remove::<Placeholder>();
        println!("   第 {} 帧：内容到位，右边从占位方块换成真图", frame.0);
    }
}

/// 监听资产事件。**只关心 logo 那一个**，否则会把引擎内部所有图片的事件都打出来。
///
/// 热重载的链路就是：文件被改 → 引擎重新读盘 → 发一个 `Modified`。
fn watch_asset_events(mut events: MessageReader<AssetEvent<Image>>, logo: Res<Logo>) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id } if *id == logo.0.id() => {
                println!("   [AssetEvent] Added    —— 图片加载完成");
            }
            AssetEvent::Modified { id } if *id == logo.0.id() => {
                println!("   [AssetEvent] Modified —— 文件变了，热重载！");
            }
            _ => {}
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// 实测输出
//
//   ── assets.load("textures/logo.png") 已返回句柄（内容此刻还没到）
//      左：直接使用句柄（内容到位前是空白）
//      右：占位方块（到位后自动换成真图）
//      下：不存在的路径（永远不出现，但不会崩）
//      第 1 帧  LoadState = Some(Loaded)
//      第 1 帧：内容到位，右边从占位方块换成真图
//      第 2 帧  LoadState = Some(Loaded)
//      [AssetEvent] Added    —— 图片加载完成
//      第 3 帧  LoadState = Some(Loaded)
//      第 4 帧  LoadState = Some(Loaded)
//
// （`get_load_state` 返回的是 `Option<LoadState>`，所以打印出来带 `Some(..)`。）
//
// ⚠️ 注意：**本机实测第 1 帧就已经 `Loaded`** —— 1.7 KB 的小图放在本地 SSD 上，
//    读盘比第一帧还快，所以"Loading"那一段根本没被观察到。
//
//    这不代表可以省掉判断。恰恰相反：
//      · 换成几 MB 的模型、机械硬盘、或 Web 端，等待就是几帧甚至几秒
//      · 加载速度**没有保证**，而你的代码不能只在"快的机器上"正确
//      · 下面的第三个精灵就是反例：它永远停在 `Failed`，永远不出现
//
// ─────────────────────────────────────────────────────────────────────
// 三种判断"好了没有"的办法
//
//   ① `Assets<T>::get(handle) -> Option<&T>`
//      最直接，`None` 就是还没有 —— 适合"我现在就要用它"（本讲用的这个）。
//
//   ② `AssetServer::is_loaded_with_dependencies(handle) -> bool`
//      连**依赖**一起算。图片通常没有依赖，但场景 / 模型会引用别的资产，
//      加载"一整套东西"时用这个更稳妥。
//
//   ③ `AssetServer::get_load_state(handle) -> Option<LoadState>`
//      想知道**卡在哪一步**就用它，四个取值：
//        `NotLoaded`  还没开始
//        `Loading`    正在读
//        `Loaded`     好了
//        `Failed(..)` 失败了 —— 注意它**不会 panic**
//
// ⚠️ 所以"加载失败"最典型的表现是：**东西一直不出现，程序也不报错**。
//    本讲第三个精灵就是这样，终端里只有一行 ERROR 日志，画面上一片空白。
//    排查这类问题的第一步永远是：把 `LoadState` 打出来看。
//
// ─────────────────────────────────────────────────────────────────────
// 路径怎么找：`assets/` 在哪
//
// `load` 的参数是相对 **`assets/` 目录**的路径，写 `"textures/logo.png"`，
// 不是 `"assets/textures/logo.png"`。
//
// 而 `assets/` 这个"根"是按下面的顺序确定的（已实测）：
//
//   1. 环境变量 `BEVY_ASSET_ROOT`
//   2. 环境变量 `CARGO_MANIFEST_DIR`（`cargo run` 会自动设置 → 就是 crate 根）
//   3. 都没有 → **可执行文件所在目录**
//
// ⚠️ 第 3 条是最容易踩的：直接双击 / 直接跑 `target/debug/examples/020_asset.exe`
//    时，引擎会去 `target/debug/examples/assets/` 找 —— 当然找不到，
//    于是所有资产都是 `Failed`，画面全空。
//    本讲开发过程中就踩了这一次，报错是：
//
//      Path not found: ...\target\debug\examples\assets\textures/logo.png
//
//    解决办法二选一：用 `cargo run` 跑，或者跑之前设好 `BEVY_ASSET_ROOT`：
//
//      export BEVY_ASSET_ROOT=$(pwd)
//
// 本项目 `assets/` 下的文件全部由脚本生成，没有下载来的二进制：
//
//     python tools/make_assets.py
//
// 想换素材就改脚本再跑一遍，可复现。
//
// ─────────────────────────────────────────────────────────────────────
// 还有两个概念
//
// **热重载**：运行中改 `assets/textures/logo.png`，画面会在几帧内自动更新 ——
// 但**前提是开启 `file_watcher` feature**。
//
// ⚠️ 注意 `file_watcher` **不是**默认 feature，它属于 `dev` 特性组：
//
//     default = ["2d", "3d", "ui", "audio"]                  ← 只有这几个
//     dev     = ["debug", "bevy_dev_tools", "render_dev_tools", "file_watcher"]
//                                                              ↑ 热重载在这里
//
// **本项目已经开了 `dev`**：
//
//     bevy = { version = "0.20", features = ["dev"] }
//
// 所以热重载是**生效**的 —— 本讲实测：运行中改 PNG，几秒内就打出
// `[AssetEvent] Modified`，画面同步更新。
//
// 如果你的项目没开 `dev`，改素材不会触发任何更新；那时要么加上这个 feature，
// 要么每次重启程序。
//
// 建议开 `dev` 而不是只开 `file_watcher`，因为它一次给四样东西：
//   · `file_watcher`      —— 热重载（改素材不用重启）
//   · `debug`             —— 让 panic 与顺序歧义警告**显示系统名**
//     （否则只能看到 `<Enable the debug feature to see the name>`，004 讲过）
//   · `bevy_dev_tools`    —— 状态切换日志、世界检查等调试工具
//   · `render_dev_tools`  —— 也就是 `bevy_dev_tools/render`：FPS 悬浮窗、渲染调试
//     面板、连拍截图、无限网格 —— 这一组在 0.20 才从 `bevy_dev_tools` 里拆出来，
//     所以 0.19 的 `dev` 里没有它
//
// 代价是编译更久、二进制更大。**学习期间开 `dev` 相当划算**，
// 因为几乎所有报错信息都会清楚得多。
//
// **`Handle` 是引用计数**：
//   `assets.load(..)` 返回**强句柄**，它让资产保持存活
//   克隆强句柄只是加计数；`clone_weak()` 则不阻止卸载
// 所以"我的资产怎么被卸载了"通常意味着：所有强句柄都丢了。
// 本讲把句柄放进 `Logo` 资源（和 `LogoImage` 组件）里，它就会一直活着。
//
// 另外 `Assets<T>::add(..)` 能把**代码里造出来**的资产直接塞进仓库（不需要文件）——
// 写程序化贴图时会用到。
// ─────────────────────────────────────────────────────────────────────
