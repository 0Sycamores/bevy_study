//! 003 · 第一个精灵：实体、组件、坐标
//!
//! 运行：`cargo run --example 003_sprite`

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
    // Bevy 2D 的坐标原点在**窗口中心**，x 向右、y 向上。
    // 和大多数 2D 引擎"左上角为原点、y 向下"正好相反，先记住这一条。
    commands.spawn((
        Sprite::from_color(Color::srgb(0.91, 0.30, 0.31), Vec2::splat(150.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // ── ② 左上角：缩小到 55%，并旋转 45° ────────────────────────
    // z 决定谁在前：z 越大越靠前（越靠近观察者）。
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
// 为什么只写了 Sprite + Transform，精灵就能显示、能移动？
//
// 因为 Bevy 有"必需组件"(required components)：`Sprite` 声明了自己需要
// `Transform` 和 `Visibility`，你 spawn 一个 `Sprite`，引擎会自动把缺的补上。
//
// 所以下面这样写也能跑，精灵会出现在原点：
//
//     commands.spawn(Sprite::from_color(Color::WHITE, Vec2::splat(50.0)));
//
// 这也解释了为什么你从来不需要手写 `Visibility` —— 它一直是引擎替你加的。
// 顺带一提：`Transform` 你不写也会被加上，只是取默认值（原点、无缩放旋转）。
//
// ── 观察实验 ──
// 1. 把 ② 的 z 从 1.0 改成 -1.0，它会跑到 ① 后面去。
// 2. 把 ② 的 y 从 160.0 改成 -160.0，验证 y 轴确实是**向上**为正。
// 3. 两个精灵 z 相同时，谁在前面**没有保证**（内部排序对相等 z 不承诺顺序）。
//    要控制层叠关系就老老实实给不同的 z，别依赖 spawn 先后。
//
// ── 练习 ──
// 用若干方块拼一个"雪人"：下面一个大方块当身体、上面一个小方块当头、
// 再加一个横向细方块当帽檐。要求三层 z 依次递增，保证层叠正确。
//
// ── 关于画圆 ──
// 本讲的 Sprite 只能画矩形（`from_color` 要的是颜色 + 尺寸）。
// 想画圆/多边形可以走 `Mesh2d` + `ColorMaterial` 那条路，它不需要图片文件，
// 但属于"网格 + 材质"体系，等 027 讲 3D 时一起对比会更清楚。
// 用图片贴图的方式则在 020 讲（需要 assets/ 目录）。
// ─────────────────────────────────────────────────────────────────────
