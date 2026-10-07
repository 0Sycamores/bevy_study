//! 002 · 窗口与相机
//!
//! 运行：`cargo run --example 002_window`
//!
//! 新增概念
//!   DefaultPlugins   引擎基础插件包：窗口、渲染、输入、时间、日志
//!   Window           窗口标题、初始分辨率、全屏模式
//!   Camera2d         2D 相机 —— 决定"往窗口里渲染什么"
//!   ClearColor       清屏色（是个 Resource）
//!
//! 使用场景
//!   想让画面出现就必须有相机；配置窗口外观在这里做
//!
//! 注意：没有相机时窗口照样弹出、程序不报错，但画面全空，连 ClearColor 都不生效

use bevy::prelude::*;

fn main() {
    App::new()
        // DefaultPlugins 一次性装好渲染、窗口、输入、日志等基础插件，
        // 同时把 runner 换成事件循环 —— 程序从此不会"跑一帧就退"。
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "002 · 窗口与相机".into(),
                // 逻辑分辨率（像素）。窗口仍可缩放，这里只是初始尺寸。
                resolution: (960, 640).into(),
                ..default()
            }),
            ..default()
        }))
        // ClearColor 是一个 Resource，存"清屏用什么颜色"。
        // 默认是 Bevy 官网代码块那种深灰，这里换成偏蓝的深色。
        .insert_resource(ClearColor(Color::srgb(0.10, 0.11, 0.15)))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // ★ 整个世界只需要这一台 2D 相机。
    //
    // 相机决定的是"往窗口里渲染什么"，而不是"窗口存不存在"。
    // 去掉这一行的后果很能说明问题：程序照常启动、窗口照常弹出、
    // 终端不报任何错，但窗口里什么都没有 —— 连 ClearColor 都不生效。
    //
    // 原因是清屏属于**相机**的渲染流程：ClearColor 只是个"默认值"，
    // 真正去读它、执行清屏动作的是相机。没有相机就没有渲染流程，
    // 也就没人去清屏。
    //
    // 所以：窗口 ≠ 画面。窗口是操作系统给的一块画布，
    // 相机才决定往这块画布上渲染什么。
    // 遇到"窗口是黑的但程序没报错"，第一反应就是去查相机。
    commands.spawn(Camera2d);
}

// ─────────────────────────────────────────────────────────────────────
// 两个常用扩展
//
// ① 启动即全屏，`Window` 的 mode 字段：
//
//     mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
//
//    无边框全屏（推荐）；想要独占全屏（会切换显示器分辨率）则用：
//
//     mode: WindowMode::Fullscreen(MonitorSelection::Primary, VideoModeSelection::Current),
//
// ② 只让某一台相机用别的清屏色 —— 多相机叠加时有用：
//
//     commands.spawn((
//         Camera2d,
//         Camera {
//             clear_color: ClearColorConfig::Custom(Color::BLACK),
//             ..default()
//         },
//     ));
//
//    `ClearColorConfig` 还有第三种取值 `None`：不清屏，直接画在已有内容之上。
//    注意区分这两个名字：
//      · `ClearColor`       —— 全局 Resource，整个世界的默认清屏色
//      · `ClearColorConfig` —— 相机身上的字段，默认值 Default 的含义正是
//                              "去读那个全局 Resource"
// ─────────────────────────────────────────────────────────────────────
