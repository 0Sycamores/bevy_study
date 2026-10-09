//! 003 · 精灵与坐标系
//!
//! 运行：`cargo run --example 003_sprite`
//!
//! 新增概念
//!   Sprite        一个矩形色块（或贴图片）
//!   Transform     位置 / 旋转 / 缩放
//!   坐标系        右手系 Y-up：+X 右、+Y 上、+Z 朝你；2D 原点在窗口中心
//!   必需组件      Sprite 会自动补上 Transform、Visibility 等（见文末）
//!
//! 使用场景
//!   画 2D 图形、摆放位置。坐标系的知识 2D / 3D 通用，027 讲 3D 时会直接复用
//!
//! 注意：Bevy 的 UI 坐标是另一套（左上角原点、Y 向下），两套别混 —— 022 详述

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // ── ① 屏幕正中央 ────────────────────────────────────────────
    // Bevy 用的是**右手系、Y 轴朝上**的世界坐标系，2D 和 3D 共用同一套：
    //
    //     +X 向右      +Y 向上      +Z 指向你（从屏幕里出来）
    //
    // 2D 里原点默认在窗口正中央。
    // 三根轴的正方向可以用右手记住：拇指 = X、食指 = Y、中指 = Z。
    //
    // ⚠️ 同样面对"屏幕"，Bevy 却有**两套方向相反**的坐标，别搞混：
    //   · 世界坐标 —— 精灵 / 相机 / `Transform`：原点在中心，Y 向上  ← 本讲
    //   · UI 坐标  —— `Node` / `Text` / 鼠标位置：原点在左上角，Y 向下 ← 022 讲
    // 从别的引擎或前端过来的人，最容易在这里翻车。
    commands.spawn((
        Sprite::from_color(Color::srgb(0.91, 0.30, 0.31), Vec2::splat(150.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // ── ② 左上角：缩小到 55%，并旋转 45° ────────────────────────
    // z 决定谁在前：z 越大越靠前（越靠近观察者，因为 +Z 指向你）。
    commands.spawn((
        Sprite::from_color(Color::srgb(0.36, 0.78, 0.42), Vec2::splat(150.0)),
        Transform::from_xyz(-280.0, 160.0, 1.0)
            .with_scale(Vec3::splat(0.55))
            // 2D 里的"旋转"是绕 z 轴转 —— 也就是绕垂直于屏幕的那根轴。
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
    ));

    // ── ③ 右下角：横向拉长压扁 ──────────────────────────────────
    // scale 是 Vec3：2D 里只用得上 x、y，z 保持 1.0 就行。
    commands.spawn((
        Sprite::from_color(Color::srgb(0.35, 0.58, 0.93), Vec2::splat(150.0)),
        Transform::from_xyz(280.0, -160.0, 2.0).with_scale(Vec3::new(1.8, 0.45, 1.0)),
    ));
}

// ─────────────────────────────────────────────────────────────────────
// 坐标系与手性（handedness）
//
// ┌ 各引擎/软件的坐标轴朝向对照图：
// │   https://topkg.github.io/bevy-cheatbook/img/handedness.png
// └ 出处：Unofficial Bevy Cheat Book · Coordinate System
//     https://topkg.github.io/bevy-cheatbook/fundamentals/coords.html
//     （该图经修改后使用，原图作者 @FreyaHolmer）
//
// Bevy 是**右手系（right-handed）+ Y-up**：
//   · +X 向右，+Y 向上，+Z 由屏幕指向观察者（朝你）
//   · 前进方向是 **-Z**（相机默认朝 -Z 看，这也正是"z 越大越靠近你"的原因）
//   · 右手定则：拇指 = X、食指 = Y、中指 = Z，三指两两垂直
//
// 和别的引擎对照（对照图里一眼就能看出差别）：
//   · 与 **Godot、Maya、OpenGL** 一致
//   · 与 **Unity 不同：Z 轴是反的**（Unity 是左手系，+Z 指向屏幕里）
//     所以从 Unity 转过来时，模型/相机的前后关系会和直觉相反
//
// 2D 里的实际含义：
//   · 背景放 z = 0.0，其他精灵用递增的正 z 叠上去 —— 就是本讲的写法
//   · 所以 2D 里 z 不是"深度"，而是**层叠顺序**：越大越靠前
//
// 为什么要 2D/3D 共用一套？因为 `Transform` 是同一个组件，得能同时挂在
// 2D 和 3D 实体上；也让你以后从 2D 转向 3D 时，所有空间直觉都能直接用。
//
// ⚠️ 唯一的例外是 **UI**：UI 用左上角原点、Y 轴向下的屏幕坐标，和网页/CSS 一致
//    （UI 布局天然是"从上往下排"）。022 讲会详细说，这里只要记住"有两套"。
//
// ─────────────────────────────────────────────────────────────────────
// 为什么只写了 Sprite + Transform，精灵就能显示、能移动？
//
// 因为 Bevy 有"必需组件"(required components)：`Sprite` 声明了自己需要
// `Transform`、`Visibility`、`VisibilityClass`、`Anchor`，
// 你 spawn 一个 `Sprite`，引擎会自动把缺的补上。
//
// 所以下面这样写也能跑，精灵会出现在原点：
//
//     commands.spawn(Sprite::from_color(Color::WHITE, Vec2::splat(50.0)));
//
// 这也解释了为什么你从来不需要手写 `Visibility` —— 它一直是引擎替你加的。
// 顺带一提：`Transform` 你不写也会被加上，只是取默认值（原点、无缩放旋转）。
//
// ── 两个容易踩的点 ──
//
// 1. **z 相同时不保证顺序。** sprite 的渲染后端是 `Mesh2d` + `SpriteMaterial`，
//    同一 Z 层级下的绘制顺序取决于内部的排序，引擎对此不做任何承诺，
//    也不保证它换个构建就还是老样子。要控制层叠关系就给不同的 z，
//    别依赖 spawn 先后。
//    本讲三个精灵写的是 0.0 / 1.0 / 2.0，不依赖任何默认顺序。
//
// 2. **旋转的正方向是逆时针。** `Quat::from_rotation_z(+90°)` 会把"向右"转成
//    "向上"，在屏幕上看起来就是逆时针。这不是巧合：右手系里绕 +Z 的正向旋转，
//    按右手定则（拇指指向你、四指弯曲方向为正）本来就该是逆时针；
//    换成左手系，同样的代码会转成顺时针。
//    （实测：`from_rotation_z(FRAC_PI_2) * Vec3::X == Vec3::Y`）
//
// ── 关于画圆 ──
// 本讲的 Sprite 只能画矩形（`from_color` 要的是颜色 + 尺寸）。
// 想画圆/多边形可以走 `Mesh2d` + `ColorMaterial` 那条路，它不需要图片文件，
// 但属于"网格 + 材质"体系，等 027 讲 3D 时一起对比会更清楚。
// 用图片贴图的方式则在 020 讲（需要 assets/ 目录）。
// ─────────────────────────────────────────────────────────────────────
