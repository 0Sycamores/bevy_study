//! 001 · 最小 App
//!
//! 运行：`cargo run --example 001_app`
//!
//! 新增概念
//!   App        容器：装着插件、资源、系统
//!   系统       普通的 Rust 函数，参数写什么就注入什么
//!   Schedule   系统挂在哪个时间点跑：Startup 只跑一次，Update 每帧都跑
//!
//! 使用场景
//!   任何 Bevy 程序的骨架都只有这三样
//!
//! 注意：没装 DefaultPlugins 时 App **只跑一帧就退出**，不会一直运行（原因见文末）

use bevy::prelude::*;

fn main() {
    App::new()
        // Startup 调度：整个 App 生命周期里只跑一次，用来做初始化。
        .add_systems(Startup, say_hello)
        // Update 调度：每帧跑一次，游戏逻辑的主战场。
        .add_systems(Update, tick)
        // 对照：加上 DefaultPlugins 之后，runner 会被 winit 换成事件循环，
        // 程序不再跑一帧就退，末尾那行 println! 也永远不会执行。
        // .add_plugins(DefaultPlugins)
        .run();

    // 只有"跑一帧就退"的情况下，这行才会被执行到。
    println!("—— .run() 已返回，进程结束 ——");
}

fn say_hello() {
    // 这里用 println! 而不是 Bevy 的 info!，原因是：
    // 没有 DefaultPlugins 就没有 LogPlugin，也就没人安装 tracing 的订阅者，
    // info!/warn! 会被**静默丢弃**，终端上什么都看不到。
    println!("[Startup] 整个 App 只跑这一次");
}

// `Local<T>` 是"系统私有状态"：只属于这一个系统，跨帧保持，
// 别的系统看不见也拿不到。（全局共享的状态叫 Resource，009 讲。）
fn tick(mut frame: Local<u32>) {
    *frame += 1;
    // Local<u32> 是个智能指针，取值要解引用：*frame
    println!("[Update] 第 {} 帧", *frame);
}

// ─────────────────────────────────────────────────────────────────────
// 为什么会退出？
//
// `App::new()` 其实就是 `App::default()`，而它内部先调用了 `App::empty()`，
// 那里把 runner 设成了 `run_once`：
//
//     // bevy_app-0.19.1/src/app.rs:152
//     runner: Box::new(run_once),
//
// 而 `run_once` 的完整实现是：
//
//     // bevy_app-0.19.1/src/app.rs:1531
//     fn run_once(mut app: App) -> AppExit {
//         ...
//         app.update();                              // ← 只 update 这一次
//         app.should_exit().unwrap_or(AppExit::Success)
//     }
//
// 所以：没有窗口类插件时，App 跑完一帧就把控制权还给你，进程正常结束。
// 「每帧跑一次」的 Update 系统因此只跑了一次 —— 上面打印的「第 1 帧」既是
// 第一帧，也是最后一帧。
//
// 加上 `DefaultPlugins` 后，winit 插件会调用 `App::set_runner(..)` 把 runner
// 换成"事件循环"：由操作系统的窗口事件驱动 update，程序才会一直运行，
// 直到你关掉窗口。这也是为什么加上那行之后，末尾的 println! 再也不会打印。
//
// 两种模式各有用途：
//   · run_once  —— 写测试、跑工具、做数据处理（032 讲 ECS 测试就靠它）
//   · 事件循环   —— 真正的游戏
// ─────────────────────────────────────────────────────────────────────
