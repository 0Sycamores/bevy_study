//! 003 · 第一个精灵：实体、组件、坐标
//!
//! 运行：`cargo run --example 003_sprite`
//!
//! 本讲是**第一次出现 `Transform`**，所以顺带把 Bevy 的坐标系一次讲清楚。
//! 坐标系不是细节——`Transform` 的每个字段（translation / rotation / scale）
//! 都要靠它才能理解，而且 2D 和 3D 用的是**同一套**，现在搞明白，027 讲 3D 时能直接搬。
//! 完整说明见文件末尾「坐标系与手性」，含各引擎朝向对照图。

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
// 2D 和 3D 实体上；也让你以后把 2D 项目升级到 3D 时，所有空间直觉都能直接用。
//
// ⚠️ 唯一的例外是 **UI**：UI 用左上角原点、Y 轴向下的屏幕坐标，和网页/CSS 一致
//    （UI 布局天然是"从上往下排"）。022 讲会详细说，这里只要记住"有两套"。
//
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
// 4. ★ 验证手性：把 ② 的旋转改成 `Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)`
//    （正 90°）和 `-FRAC_PI_2`（负 90°）。正角度是**逆时针**转。
//    这不是巧合：右手系里绕 +Z 的正向旋转，按右手定则（拇指指向你、四指弯曲方向为正）
//    看起来就是逆时针。换成左手系，同样的代码会转成顺时针。
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
