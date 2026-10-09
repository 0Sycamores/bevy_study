//! 027 · 3D 基础
//!
//! 运行：`cargo run --example 027_3d_basic`
//!
//! 新增概念
//!   Camera3d                  3D 相机，默认朝 **-Z** 方向看
//!   Mesh3d(handle)            网格：物体的"形状"
//!   MeshMaterial3d(handle)    材质：物体的"表面"
//!   Assets<Mesh>::add(..)     把几何图元（Cuboid / Sphere / Cylinder）变成网格资产
//!   PointLight / DirectionalLight   光源；`shadow_maps_enabled` 打开阴影
//!   GlobalAmbientLight        环境光（**资源**，见文末）
//!
//! 使用场景
//!   任何 3D 场景都是这四样凑出来的：网格 + 材质 + 光 + 相机
//!
//! 注意：2D 和 3D 用的是**同一个 `Transform` 组件**，但 z 的含义完全不同 ——
//!       2D 里 z 是层序（谁盖住谁），3D 里 z 是**纵深**（离相机多远）

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "027 · 3D 基础：一个转动的立方体 + 球 + 圆柱".into(),
                resolution: (960, 640).into(),
                ..default()
            }),
            ..default()
        }))
        // 环境光用一个资源设。它决定了"背光面有多黑"——不设的话暗部会死黑。
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.55, 0.65, 0.85),
            brightness: 220.0,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, spin)
        .run();
}

/// 只给立方体加这个标记，让它自己转 —— 用来展示 3D 旋转。
#[derive(Component)]
struct Spin;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ── ① 地面：一块很扁的立方体 ──
    // 3D 里没有"2D 背景色"那种概念，场景需要有一块东西接着光，
    // 不然物体看起来像浮在虚空里。
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(9.0, 0.2, 9.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.17, 0.19, 0.25),
            ..default()
        })),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));

    // ── ② 三个几何图元 ──
    // `meshes.add(..)` 接受实现了 `Meshable` 的图元，直接换算成网格资产。
    // 这与 2D 的 `Sprite::from_color(..)` 不同：3D 的形状必须来自**网格**。
    commands.spawn((
        Spin,
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(-1.9, 0.5, 0.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.6))),
        MeshMaterial3d(materials.add(Color::srgb_u8(240, 152, 60))),
        Transform::from_xyz(0.0, 0.6, 0.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.4, 1.2))),
        MeshMaterial3d(materials.add(Color::srgb_u8(120, 220, 140))),
        Transform::from_xyz(1.9, 0.6, 0.0),
    ));

    // ── ③ 光 ──
    // 主光：平行光。它没有"位置"只有"方向"，所以靠 `Transform` 的朝向决定照向哪边。
    commands.spawn((
        DirectionalLight {
            // ⚠️ 0.19 的字段名是 `shadow_maps_enabled`（旧版叫 `shadows_enabled`；0.20 同）
            shadow_maps_enabled: true,
            illuminance: 6_000.0,
            ..default()
        },
        Transform::from_xyz(6.0, 10.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 补光：点光源。有位置、有衰减，用来把暗部提亮一点。
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.85, 0.7),
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-5.0, 4.0, 5.0),
    ));

    // ── ④ 相机 ──
    // `looking_at(目标, 上方向)` 是最省事的摆法：告诉它看哪儿、头顶朝哪。
    // 3D 相机的"前方"是 **-Z**（和 2D 正交相机一致，但 2D 里感受不到）。
    let camera_transform =
        Transform::from_xyz(-4.5, 4.0, 9.0).looking_at(Vec3::new(0.0, 0.6, 0.0), Vec3::Y);
    commands.spawn((Camera3d::default(), camera_transform));

    println!("── 3D 场景：地面 + 立方体(会转) + 球 + 圆柱");
    println!("   相机位置 {:?}", camera_transform.translation);
    println!(
        "   相机朝向 {:?}   ← 注意 z 是负的：3D 相机默认朝 -Z 看",
        camera_transform.forward()
    );
    println!("   平行光(白) + 点光源(暖) 都开了阴影，环境光资源负责提亮暗部");
}

/// 让立方体绕 Y 轴自转 —— 转起来才看得出这是个 3D 物体。
///
/// `Transform::rotate_y(角度)` 是绕**世界 Y 轴**转；想绕自己的局部轴转用
/// `rotate_local_y(..)`。刚体旋转用四元数（`Quat`）内部表示，不用欧拉角，
/// 所以不会出现"万向锁"那类问题。
fn spin(time: Res<Time>, mut spinners: Query<&mut Transform, With<Spin>>) {
    for mut transform in &mut spinners {
        transform.rotate_y(0.9 * time.delta_secs());
    }
}

// ─────────────────────────────────────────────────────────────────────
// 跑起来会看到什么
//
//   · 一块深灰地面，上面摆着蓝色立方体、橙色球、绿色圆柱
//   · 立方体绕竖轴慢慢自转
//   · 两个光源都在物体下方投出**阴影**，物体之间也互相投影
//   · 暗部不是死黑，而是带一点冷色 —— 那是环境光
//
// 控制台输出：
//
//   ── 3D 场景：地面 + 立方体(会转) + 球 + 圆柱
//      相机位置 Vec3(-4.5, 4.0, 9.0)
//      相机朝向 Dir3(Vec3(0.42368072, -0.32011428, -0.84736145))   ← 注意 z 是负的：3D 相机默认朝 -Z 看
//      平行光(白) + 点光源(暖) 都开了阴影，环境光资源负责提亮暗部
//
// 那个朝向值得自己算一遍：`looking_at` 得到的方向就是
// **目标 − 位置** = `(0, 0.6, 0) − (-4.5, 4.0, 9.0)` = `(4.5, -3.4, -9.0)`，
// 归一化后正好是 `(0.42, -0.32, -0.85)`。
//
// 注意 x 分量是**正**的：相机站在 x = -4.5，要看向原点的物体，
// 自然得朝 +X 方向偏。三个分量分别说明"往右看、往下看、往里看"。
//
// ─────────────────────────────────────────────────────────────────────
// ★ 同一个 Transform，两套坐标含义
//
// 2D（003 讲的精灵）和 3D（本讲）用的是**同一个 `Transform` 组件**，
// 但在两个世界里的意义不一样：
//
//             2D 精灵                3D 物体
//   x        左右                   左右
//   y        上下                   上下
//   z        **层序**               **纵深**
//            越大越靠前、越晚被盖     越大越靠近相机
//
// 所以 `Transform::from_xyz(0.0, 0.0, 5.0)`：
//   · 在 2D 里是"把图片提到最上层"
//   · 在 3D 里是"往相机方向挪 5 个单位"（也就是**变大**，因为离得近了）
//
// 这也是为什么 2D 那一讲的 `Camera2d` 从来不用管 z：正交投影下 z 只决定层序。
// 换成 `Camera3d` 之后，z 立刻变成透视深度 —— **同一个组件，两个世界**。
//
// 顺带一提 `Vec3::Y` 那个"上方向"：3D 里必须有它才能唯一确定相机的姿态。
// 只给"看哪里"是不够的 —— 相机还可以绕视线轴自转，`up` 就是用来锁死这一点的。
//
// ─────────────────────────────────────────────────────────────────────
// 3D 场景的四件套
//
// 任何 3D 物体都是"网格 + 材质 + 变换"三件套，再加上光与相机：
//
//   `Mesh3d(handle)`            形状（`Assets<Mesh>` 里的资产）
//   `MeshMaterial3d(handle)`    表面（`Assets<StandardMaterial>` 里的资产）
//   `Transform`                 位置 / 旋转 / 缩放
//   —— 以上三个凑齐，物体就会出现在场景里。
//
// 注意两个组件都是**句柄**，不是数据本体：
//   · 多个物体可以 clone 同一个句柄 → 共用一份网格 / 材质（省内存、省上传）
//   · 改材质要 `materials.get_mut(&handle)`，会立刻影响所有用它的物体
//   · 这也自然接上了 020 讲的资产体系：网格和材质就是"内存里造出来的资产"
//
// ⚠️ `Mesh3d` / `MeshMaterial3d` 是**必需组件**（required components）：
//    只写 `Mesh3d` 不写材质，物体也画不出来（但不会报错）。
//    这跟 003 讲的 `Sprite` 必须配 `Transform` 是同一类陷阱 ——
//    **缺必需组件不报错，只是什么都看不见**。
//
// ─────────────────────────────────────────────────────────────────────
// 光：三种光源 + 环境光
//
//   `DirectionalLight`  平行光：模拟太阳。**没有位置**，只有方向，
//                       所以靠 `Transform` 的朝向决定照向哪边；强度用 `illuminance`
//   `PointLight`        点光源：灯泡。有位置、有距离衰减，亮度用 `intensity`
//   `SpotLight`         聚光灯：有锥角、有方向
//
// 三个都有 `shadow_maps_enabled`（0.19 的字段名，旧版是 `shadows_enabled`；0.20 同）——
// 打开后该光源就会投影。
//
// 环境光要注意 0.19 的一个改动：
//
//   `GlobalAmbientLight` 是**资源**，全局默认环境光 → `insert_resource(..)`
//   `AmbientLight` 现在是**组件**，挂在某台相机上**覆盖**全局值
//
// 所以"整个场景亮一点"用资源；"这台相机的画面亮一点"用组件。
// 环境光不产生阴影，作用只是"别让暗部死黑"。
//
// ─────────────────────────────────────────────────────────────────────
// 2D 到 3D 的迁移清单
//
//   `Camera2d`          → `Camera3d`
//   `Sprite`            → `Mesh3d` + `MeshMaterial3d`
//   `Sprite::from_image`→ `Assets<Mesh>` + `Assets<StandardMaterial>`
//   （不需要光）          → **必须有光**，否则全黑
//   （不需要相机朝向）     → 相机要用 `looking_at(..)` 摆姿态
//
// 最后一条最容易被忽略：2D 里相机放在哪基本无所谓（正交投影没有透视），
// 3D 里相机**必须真的摆对位置和朝向**，否则要么看不到东西，要么一片黑。
// ─────────────────────────────────────────────────────────────────────
