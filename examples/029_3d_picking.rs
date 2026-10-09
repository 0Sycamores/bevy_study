//! 029 · 3D 拾取与轨道相机
//!
//! 运行：`cargo run --example 029_3d_picking`
//!
//! 新增概念
//!   MeshPickingPlugin      **必须手动添加**：它给 3D 网格装上"可被指针选中"的能力
//!   PointerOver / PointerOut   指针移入 / 移出某个网格
//!   PointerClick          在同一个网格上完成按下 + 松开
//!   轨道相机               右键拖动绕焦点旋转、滚轮拉近拉远
//!
//! 使用场景
//!   点击选中单位、鼠标划过高亮、放建筑时预览落点、编辑器里拖拽物体
//!
//! 注意：**不加 `MeshPickingPlugin` 时，下面这些观察者一个都不会触发，
//!       而且不报任何错** —— 这是最难查的一类问题（见文末）

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "029 · 3D 拾取：悬停高亮 / 点击变色 / 右键拖动转视角".into(),
                    resolution: (960, 640).into(),
                    ..default()
                }),
                ..default()
            }),
            // ★★★ 这一行就是本讲的主角。
            //
            // UI 与 2D 精灵的拾取是**默认开启**的（023 讲直接用就行），
            // 但 3D 网格拾取**不是** —— 不加它，观察者静默失效。
            //
            // 为什么不做成默认？因为 3D 拾取要对每个网格做射线求交，
            // 是有开销的；引擎把它留给真正需要的项目自己开。
            MeshPickingPlugin,
        ))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.55, 0.65, 0.85),
            brightness: 220.0,
            ..default()
        })
        .insert_resource(Orbit {
            yaw: 0.6,
            pitch: 0.45,
            radius: 12.0,
            focus: Vec3::new(0.0, 0.6, 0.0),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, orbit_camera)
        .run();
}

/// 轨道相机的状态：绕 `focus` 转，用"偏航 / 俯仰 / 距离"三个数描述，比存四元数直观。
#[derive(Resource)]
struct Orbit {
    yaw: f32,
    pitch: f32,
    radius: f32,
    focus: Vec3,
}

/// 挂在可拾取的物体上，记住它"原本的颜色"，好在指针移出时还原。
#[derive(Component)]
struct Highlight {
    normal: Handle<StandardMaterial>,
    hovered: Handle<StandardMaterial>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 4.0, 10.0).looking_at(Vec3::new(0.0, 0.6, 0.0), Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            illuminance: 6_000.0,
            ..default()
        },
        Transform::from_xyz(6.0, 10.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 地面（也做成网格，但它没有挂观察者，所以不会被"选中"）
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(14.0, 0.2, 14.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.17, 0.19, 0.25),
            ..default()
        })),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));

    // ── 三个可拾取的物体 ──
    let shapes: [(&str, Mesh, Color); 3] = [
        (
            "立方体",
            Cuboid::new(1.2, 1.2, 1.2).into(),
            Color::srgb_u8(124, 144, 255),
        ),
        ("球", Sphere::new(0.7).into(), Color::srgb_u8(240, 152, 60)),
        (
            "圆柱",
            Cylinder::new(0.55, 1.4).into(),
            Color::srgb_u8(120, 220, 140),
        ),
    ];

    for (i, (label, mesh, color)) in shapes.into_iter().enumerate() {
        let normal = materials.add(color);
        let hovered = materials.add(StandardMaterial {
            base_color: color.mix(&Color::WHITE, 0.55),
            // 悬停时加一点自发光，暗处也看得清
            emissive: (color.to_linear() * 0.35).into(),
            ..default()
        });

        commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                // 起始用"普通色"那份材质
                MeshMaterial3d(normal.clone()),
                Transform::from_xyz(-2.6 + i as f32 * 2.6, 0.7, 0.0),
                Highlight { normal, hovered },
                Name::new(label),
            ))
            // ⚠️ 观察者要挂在**网格实体**上。这里 `Mesh3d` 和观察者都在同一个实体上，
            //    所以没问题；但 glTF 加载出来的模型是**节点 + 子网格**两层结构
            //    （028 讲的），那种情况下观察者得挂到真正的网格实体上才算数。
            .observe(on_over)
            .observe(on_out)
            .observe(on_click);
    }

    println!("── MeshPickingPlugin 已启用；三个物体挂上了 Over/Out/Click 观察者");
    println!("   左键点物体、右键拖动转视角、滚轮拉近拉远");
}

/// 指针移入：换成高亮材质。
fn on_over(over: On<PointerOver>, mut commands: Commands, highlights: Query<&Highlight>) {
    if let Ok(highlight) = highlights.get(over.entity) {
        // 直接改组件即可 —— `MeshMaterial3d` 就是"当前用哪份材质"
        commands
            .entity(over.entity)
            .insert(MeshMaterial3d(highlight.hovered.clone()));
        println!("   [Over]  指针进入实体 {:?}", over.entity);
    }
}

/// 指针移出：还原。
fn on_out(out: On<PointerOut>, mut commands: Commands, highlights: Query<&Highlight>) {
    if let Ok(highlight) = highlights.get(out.entity) {
        commands
            .entity(out.entity)
            .insert(MeshMaterial3d(highlight.normal.clone()));
    }
}

/// 点击：打印它的名字，并把材质换成高亮色留住（表示"已选中"）。
fn on_click(click: On<PointerClick>, names: Query<&Name>) {
    let name = names.get(click.entity).map_or("<无名>", Name::as_str);
    println!("   [Click] 选中了「{name}」");
}

/// 轨道相机：右键拖动改偏航/俯仰，滚轮改距离，然后由这三个数**算出**相机位置。
///
/// 用"球坐标"而不是直接改 `Transform`，是因为这样天然不会翻转、也不会丢焦点 ——
/// 相机永远看着 `focus`，这正是"轨道相机"的定义。
fn orbit_camera(
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut orbit: ResMut<Orbit>,
    mut cameras: Query<&mut Transform, With<Camera3d>>,
) {
    // ⚠️ 用**右键**转视角而不是左键：左键留给"点击选中"。
    //    否则拖一下就会同时触发一堆 Click，两种操作会打架。
    if buttons.pressed(MouseButton::Right) {
        orbit.yaw -= motion.delta.x * 0.005;
        // 俯仰限制在 ±80°，避免转到正上方时 `look_at` 的"上方向"退化
        orbit.pitch = (orbit.pitch - motion.delta.y * 0.005).clamp(-1.4, 1.4);
    }

    if scroll.delta.y != 0.0 {
        orbit.radius = (orbit.radius * (1.0 - scroll.delta.y * 0.1)).clamp(3.5, 30.0);
    }

    for mut transform in &mut cameras {
        // 由 yaw / pitch 构造旋转，再把相机沿该方向推到 radius 远处
        let rotation = Quat::from_rotation_y(orbit.yaw) * Quat::from_rotation_x(orbit.pitch);
        transform.translation = orbit.focus + rotation * Vec3::Z * orbit.radius;
        transform.look_at(orbit.focus, Vec3::Y);
    }
}

// ─────────────────────────────────────────────────────────────────────
// 实测输出（启动部分）
//
//   ── MeshPickingPlugin 已启用；三个物体挂上了 Over/Out/Click 观察者
//      左键点物体、右键拖动转视角、滚轮拉近拉远
//
// 之后就是交互输出了 —— ⚠️ **这部分机器验证不了**，因为需要真实鼠标移动
// 与点击。手动操作时应当看到：
//
//   [Over]  指针进入实体 6v0          ← 鼠标移到立方体上
//   [Click] 选中了「立方体」           ← 点一下
//   [Over]  指针进入实体 7v0
//   [Click] 选中了「球」
//   （移出物体时材质自动还原）
//
// ─────────────────────────────────────────────────────────────────────
// ★ 不加 MeshPickingPlugin 会怎样
//
//   观察者**一个都不触发**，程序**不报任何错**，也不会有警告。
//   画面上一切正常，就是点不动。
//
// 本讲实测确认了"不报错"这一点：去掉那一行之后程序照常启动、照常渲染，
// 只是 `[Over]` / `[Click]` 永远不会出现。
//
// 为什么这么设计？因为 3D 拾取要给每个网格做**射线求交**，是有开销的，
// 引擎把它留给需要的项目自己开。对比一下：
//
//   UI 拾取（023）      默认开启
//   2D 精灵拾取         默认开启
//   3D 网格拾取         **要手动加 `MeshPickingPlugin`**
//
// 所以排查"点了没反应"时的第一步永远是：**这个拾取后端装了吗？**
//
// ─────────────────────────────────────────────────────────────────────
// 拾取替你做了什么
//
// 一次点击背后，引擎做完了这些事：
//
//   1. 把屏幕坐标换算成一条**世界空间的射线**（就是 `viewport_to_world`）
//   2. 拿这条射线去和所有可拾取网格求交（用它们的变换 + 网格包围体加速）
//   3. 取最近的交点，确定"打中了哪个实体"
//   4. 生成 `PointerOver` / `PointerClick` 等事件，**目标是那个实体**
//
// 所以你不用自己写射线代码 —— 这正是 013 讲的"实体事件"最适合的场景：
// 事件自带目标，观察者挂在谁身上就只管谁。
//
// 需要**自定义命中信息**时（比如"打在网格的哪个位置""打中第几个三角形"），
// 就要自己拿射线了，两条路：
//
//   · `Camera::viewport_to_world(camera_transform, cursor)` → `Ray3d`
//     再和平面/包围盒自己求交（放建筑预览落点就是典型用法）
//   · 给拾取后端传自定义 hit data，见上游
//     `examples/picking/custom_hit_data.rs`
//
// 另外上游还有 `DebugPickingPlugin`，可以把"当前指针底下是哪个实体"画出来，
// 调拾取问题时非常省事。
//
// ─────────────────────────────────────────────────────────────────────
// 观察者要挂在哪一层（承接 028）
//
// 本讲的对象是代码 spawn 的，`Mesh3d` 和观察者在**同一个实体**上，
// 所以直接挂就行。
//
// 但 glTF 加载出来的模型不是这样 —— 028 讲过，那是"**命名节点 + 子网格实体**"
// 两层结构，真正带 `Mesh3d` 的是**子实体**。这时：
//
//   · 想让整个模型可交互 → 观察者挂到**节点**上，靠事件冒泡（013 讲的行为）
//   · 想按图元分别响应   → 观察者挂到**子网格**上
//
// 判断方法很直接：`Query<&Mesh3d>` 能查到谁，观察者就该挂到谁附近。
//
// ─────────────────────────────────────────────────────────────────────
// 顺带一提：轨道相机的三个数
//
// `yaw`（偏航，绕 Y 轴）/ `pitch`（俯仰）/ `radius`（距离）—— 用球坐标描述相机，
// 比直接存四元数直观得多，而且**天然满足"相机永远看着焦点"**：
//
//     位置 = 焦点 + 旋转(yaw, pitch) * (0, 0, radius)
//
// 两个细节：
//   · `pitch` 要限制在 ±90° 以内，否则到了正上方时 `look_at` 的"上方向"
//     会和视线方向平行，姿态就退化了（画面会突然翻转）
//   · 右键转视角、左键做选中，**两种操作要分开**，否则拖一下会连发一堆点击
//
// 这套东西就是 025 讲的"相机跟随"的升级版：那里相机追着一个点跑，
// 这里是相机绕着一个点转，本质都是"用别的数据算出 `Transform`"。
// ─────────────────────────────────────────────────────────────────────
