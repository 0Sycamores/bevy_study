//! 028 · 加载 glTF 模型
//!
//! 运行：`cargo run --example 028_3d_gltf`
//!
//! 新增概念
//!   WorldAssetRoot(handle)    场景组件：加载完成后会**实例化出一棵实体树**
//!   GltfAssetLabel::Scene(0)  指定 glTF 里的第几个场景（也可以直接写 `"x.gltf#Scene0"`）
//!   On<WorldInstanceReady>    实例化完成的事件 —— **之后**才查得到里面的实体
//!   Children::iter_descendants  在实例化出来的层级里递归遍历
//!
//! 使用场景
//!   用美术做好的模型（.gltf / .glb），代替 027 那样用代码拼几何体
//!
//! 注意：`WorldAssetRoot` **不是一个实体**，它是一次"实例化请求"。
//!       加载是异步的（020 讲的），所以紧接着 `Query` 只会得到 0 个结果

use bevy::prelude::*;
// `WorldAssetRoot` 在 prelude 里，但**就绪事件不在**，要单独引。
use bevy::world_serialization::WorldInstanceReady;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "028 · glTF：一棵被实例化出来的实体树".into(),
                resolution: (960, 640).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.55, 0.65, 0.85),
            brightness: 220.0,
            ..default()
        })
        .init_resource::<Frames>()
        .add_systems(Startup, setup)
        // 前几帧数一数"场景里有几个网格"，用来看清异步的过程
        .add_systems(Update, count_meshes_early)
        .run();
}

#[derive(Resource, Default)]
struct Frames(u32);

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-4.5, 4.0, 9.0).looking_at(Vec3::new(0.0, 0.8, 0.0), Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            illuminance: 6_000.0,
            ..default()
        },
        Transform::from_xyz(6.0, 10.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // ── 关键的一行 ──
    // `load` 立刻返回句柄；真正"把场景里的节点变成实体"是**之后**的事。
    let scene = assets.load(GltfAssetLabel::Scene(0).from_asset("models/pyramid.gltf"));
    println!("── assets.load(\"models/pyramid.gltf#Scene0\") 已返回句柄");
    println!("   此刻场景里还没有任何实体 —— 实例化是异步的");

    commands
        .spawn(WorldAssetRoot(scene))
        // 场景就绪时触发。**这是唯一能安全操作场景内容的时机** ——
        // 早于它的话，那些实体压根还不存在（013 讲的观察者，用在资产上）
        .observe(on_scene_ready);
}

/// 头几帧数一下有几个带 `Mesh3d` 的实体 —— 用来看清"什么时候才有东西"。
fn count_meshes_early(mut frames: ResMut<Frames>, meshes: Query<&Mesh3d>, mut done: Local<bool>) {
    frames.0 += 1;
    if *done || frames.0 > 6 {
        return;
    }
    let count = meshes.iter().count();
    println!("   第 {} 帧：场景里带 Mesh3d 的实体 {} 个", frames.0, count);
    if count > 0 {
        *done = true;
    }
}

/// 场景实例化完成后才会跑 —— 这时才谈得上"遍历场景里的实体"。
fn on_scene_ready(
    ready: On<WorldInstanceReady>,
    children: Query<&Children>,
    names: Query<&Name>,
    handles: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!(
        "   [WorldInstanceReady] 实例化完成，根实体 {:?}",
        ready.entity
    );
    println!("   实例化出的实体树：");

    let mut total = 0;
    let mut pyramid_b = None;
    for descendant in children.iter_descendants(ready.entity) {
        total += 1;
        // glTF 里的节点名会被加载成 `Name` 组件 —— 这就是"文件里的名字"落到世界里
        let name = names.get(descendant).map_or("<无名>", Name::as_str);
        let has_mesh = if handles.contains(descendant) {
            "  [有网格]"
        } else {
            ""
        };
        println!("      ├─ {descendant:?}  {name}{has_mesh}");

        if name == "PyramidB" {
            pyramid_b = Some(descendant);
        }
    }
    println!("   共 {total} 个后代实体");

    // ── 改实例化出来的材质 ──
    //
    // ⚠️ 注意不能直接找 `Name == "PyramidB"` 的实体去改材质：
    //    glTF 的**节点**和它的**网格**是两个实体 —— `PyramidB` 那个节点上
    //    没有 `Mesh3d`，真正的网格是它的**子节点**（名字形如
    //    `Pyramid.PyramidMaterial`，由加载器按"网格名.材质名"拼出来）。
    //
    //    所以正确姿势是：先按名字找到节点，再往下找带材质的后代。
    //    如果只查第一层，代码不会报错，只是**什么都不发生**。
    if let Some(node) = pyramid_b {
        let node_and_descendants = std::iter::once(node).chain(children.iter_descendants(node));
        for entity in node_and_descendants {
            if let Ok(handle) = handles.get(entity)
                && let Some(mut material) = materials.get_mut(&handle.0)
            {
                material.base_color = Color::srgb(0.25, 0.65, 0.95);
                let name = names.get(entity).map_or("<无名>", Name::as_str);
                println!("   把 {name} 改成蓝色（文件里原本是红铜色）");
                break;
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// 实测输出
//
//   ── assets.load("models/pyramid.gltf#Scene0") 已返回句柄
//      此刻场景里还没有任何实体 —— 实例化是异步的
//      第 1 帧：场景里带 Mesh3d 的实体 0 个
//      [WorldInstanceReady] 实例化完成，根实体 375v0
//      实例化出的实体树：
//         ├─ 382v0  PyramidPair
//         ├─ 377v0  Root
//         ├─ 378v0  PyramidA
//         ├─ 379v0  PyramidB
//         ├─ 380v0  Pyramid.PyramidMaterial  [有网格]
//         ├─ 381v0  Pyramid.PyramidMaterial  [有网格]
//      共 6 个后代实体
//      把 Pyramid.PyramidMaterial 改成蓝色（文件里原本是红铜色）
//      第 2 帧：场景里带 Mesh3d 的实体 2 个
//
// 文件里只有 **3 个节点**，实例化出来却是 **6 个实体**。多出来的三个是：
//
//   `PyramidPair`                  glTF 的 scene 本身也占一个实体
//   `Pyramid.PyramidMaterial` ×2   每个**网格图元**又是一个独立实体
//
// （实体编号每次运行都不同；帧数取决于磁盘速度，本机是 2 帧左右。）
//
// ─────────────────────────────────────────────────────────────────────
// ★ 节点 ≠ 网格实体（这条最反直觉）
//
// 从上面对照可以看出：**`PyramidA` / `PyramidB` 那两个节点上并没有网格。**
// 真正的网格是它们的**子实体**，名字由加载器按"网格名.材质名"拼出来。
//
// 这解释了一个很容易踩的坑：想"按节点名找到那个模型再改材质"，
// 直接查 `Name == "PyramidB"` 会**什么都查不到** —— 那个实体上压根没有 `Mesh3d`。
// 代码不会报错，只是静默地什么都不发生（本讲第一版就是这样，跑起来才发现）。
//
// 正确姿势是**往下再走一层**：
//
//     let node = <按名字找到的节点实体>;
//     once(node).chain(children.iter_descendants(node))   // 自己 + 后代
//         .find(|e| handles.contains(*e))
//
// 为什么会拆成两层？因为 glTF 的一个节点可以引用一个含**多个图元**的网格，
// 每个图元（不同材质各一份）都得是独立实体才能各自挂材质。
// 本讲的文件只有一个图元，所以是 1:1，但结构上仍然是两层。
//
// 记住这条判断：**"我明明找到了那个实体，改它却没反应"** ——
// 先看看网格是不是在它的子实体上。
//
// ─────────────────────────────────────────────────────────────────────
// ★ glTF 场景不是"一个实体"，而是一棵树
//
// 这是本讲最要紧的一点。`models/pyramid.gltf` 里写的是三个节点：
//
//     Root
//     ├── PyramidA   (引用 mesh 0)
//     └── PyramidB   (引用同一个 mesh 0，缩小到 0.6)
//
// 而 `commands.spawn(WorldAssetRoot(..))` 只生成了**一个**实体 ——
// 那个实体是"根"，上面挂着的场景会在加载完成后**作为它的后代**展开。
//
// 所以想碰场景里的东西，必须走层级：
//
//     children.iter_descendants(root)     // 递归遍历所有后代
//
// 直接 `Query<&Mesh3d>` 当然也能查到，但你不知道哪个是哪个 ——
// 而沿着 `Children` 走，你能看清结构。`Name` 组件就是从 glTF 的节点名来的，
// 是最好的路标。
//
// ─────────────────────────────────────────────────────────────────────
// ★ 为什么不能立刻查
//
// 因为加载是**异步**的（020 讲的那件事，在场景上再次出现）：
//
//   `load(..)` 立刻返回句柄  →  spawn 一个"占位"实体
//   ↓ （后台线程读文件、解析 glTF）
//   实例化完成，产生后代实体
//   ↓ 触发 `WorldInstanceReady`
//
// 在 `Startup` 里 spawn 完之后紧接着 `Query<&Name>`，结果必然是空的 ——
// **而且不会报错**，只是什么都没查到。这是最难查的一类问题：
// 代码看着没问题，就是没效果。
//
// 正确姿势就是本讲的做法：**把"操作场景内容"的代码放进 `WorldInstanceReady` 的观察者里**。
// 观察者收到的事件里带 `entity`（也就是那个根实体），从它往下遍历即可。
//
// ─────────────────────────────────────────────────────────────────────
// 路径的两种写法
//
// 本讲用的显式标签写法：
//
//     asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/pyramid.gltf"))
//
// 也可以直接写路径片段（等价，更短）：
//
//     asset_server.load("models/pyramid.gltf#Scene0")
//
// `Scene(0)` 里的数字是"文件里的第几个场景"。一个 glTF 可以含多个场景，
// 而更多时候你想要的其实是**某个具体节点**，那就用别的标签：
//
//     GltfAssetLabel::Mesh(0)          只要第 0 个网格
//     GltfAssetLabel::Node(2)          只要第 2 个节点
//     GltfAssetLabel::Primitive { .. } 只要某个图元
//     GltfAssetLabel::Animation(0)     只要第 0 段动画
//
// 完整用法见上游 `examples/gltf/query_gltf_primitives.rs`（按图元拆分模型）。
//
// ─────────────────────────────────────────────────────────────────────
// 关于 glTF 动画
//
// glTF 文件里可以带骨骼动画 / 节点动画，Bevy 会把它实例化成
// `AnimationPlayer` 组件（挂在场景内某个实体上）。播放的钩子同样在就绪之后：
//
//     fn play_when_ready(ready: On<WorldInstanceReady>, children: Query<&Children>, ...) {
//         for child in children.iter_descendants(ready.entity) {
//             if let Ok(mut player) = players.get_mut(child) {
//                 player.play(animation_index).repeat();
//             }
//         }
//     }
//
// 注意 0.19 里播放动画还需要 `AnimationGraphHandle`（一套动画图），
// 有专门的 `examples/animation/` 系列讲这件事 —— 本讲没有现场演示，
// 因为它的主体是**动画系统**而不是"加载 glTF"，按"一讲一个概念"的原则留给后续。
//
// 本讲用的 `models/pyramid.gltf` 里也**没有**动画：它只包含网格、材质与节点树，
// 正好把注意力集中在"场景 = 实体树"这一件事上。
//
// ─────────────────────────────────────────────────────────────────────
// 资产从哪来
//
// `assets/models/pyramid.gltf` 是本项目**手写生成**的，跑：
//
//     python tools/make_assets.py
//
// glTF 说到底就是个 JSON，二进制数据用 base64 内嵌在 `buffers[0].uri` 里。
// 所以"手写一个模型"完全可行 —— 脚本里 `make_pyramid_gltf()` 就是干这个的，
// 顺带还能看清顶点、法线、绕序这些平时被工具藏起来的东西。
//
// 真实项目当然用 Blender 之类导出，但知道文件里有什么，排查问题时很有用
// （比如"模型是黑的"往往是**法线方向反了**，绕序错了会被背面剔除）。
// ─────────────────────────────────────────────────────────────────────
