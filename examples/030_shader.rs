//! 030 · 自定义着色器
//!
//! 运行：`cargo run --example 030_shader`
//!
//! 新增概念
//!   Material trait       自定义材质：告诉引擎"用哪个 WESL 着色器来画"
//!   AsBindGroup          声明"哪些 Rust 数据要传给 shader"（binding 号要跟 shader 对上）
//!   MaterialPlugin::<M>  注册材质类型 —— **不注册等于没写**
//!   ShaderRef            去哪找 shader（路径相对 `assets/`）
//!
//! 使用场景
//!   引擎自带材质不够用时：描边、水波、溶解、全息、卡通渲染
//!
//! 注意：Rust 侧与 WESL 侧是**两份必须手动对齐的契约** —— 类型、binding 号、
//!       结构体布局任何一处对不上，都是**运行时**才炸（画面变黑），编译期查不出来

use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

/// shader 文件相对 `assets/` 的路径。
const SHADER_PATH: &str = "shaders/glow.wesl";

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "030 · 自定义着色器：条纹 + 呼吸".into(),
                    resolution: (960, 640).into(),
                    ..default()
                }),
                ..default()
            }),
            // ★ 把材质类型注册进渲染管线。
            //   少了这一行，`MeshMaterial3d(materials.add(GlowMaterial { .. }))`
            //   照样能编译、能跑，但**画面上什么都不显示** —— 因为渲染器
            //   根本不知道该怎么画这个类型。
            MaterialPlugin::<GlowMaterial>::default(),
        ))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.55, 0.65, 0.85),
            brightness: 220.0,
            ..default()
        })
        .add_systems(Startup, setup)
        // 每帧把新的呼吸值写进材质 —— Rust 侧"喂"数据的典型写法
        .add_systems(Update, pulse_materials)
        .run();
}

/// 自定义材质。
///
/// `#[derive(AsBindGroup)]` 会根据字段上的 `#[uniform(N)]` / `#[texture(N)]`
/// 自动生成"把这些数据打包上传到 GPU"的代码。
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct GlowMaterial {
    // ⚠️ N 必须和 WESL 里的 `@binding(N)` 对上
    #[uniform(0)]
    base_color: Vec4,

    /// `.x` 是呼吸强度，由 `pulse_materials` 每帧更新。
    #[uniform(1)]
    params: Vec4,
}

impl Material for GlowMaterial {
    /// 只覆盖这一个方法就够了 —— `Material` 的其余方法都有合理默认值
    /// （剔除模式、透明混合、阴影投射方式等等）。
    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // 注意是**两个独立的资产集合**：自定义材质和 `StandardMaterial`
    // 是不同的资产类型，各自有自己的 `Assets<T>`
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut glow_materials: ResMut<Assets<GlowMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-4.0, 3.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 自定义材质不参与 PBR，所以严格说不需要光；
    // 但场景里有地面，光能让参照关系更清楚。
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            illuminance: 6_000.0,
            ..default()
        },
        Transform::from_xyz(6.0, 10.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 地面：用引擎自带的 `StandardMaterial`，好和自定义材质做对照
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(11.0, 0.2, 11.0))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: Color::srgb(0.17, 0.19, 0.25),
            ..default()
        })),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));

    // ── 三个用自定义材质的物体 ──
    // `materials.add(GlowMaterial { .. })` 和 `add(StandardMaterial { .. })`
    // 用法完全一样 —— 因为注册进 `MaterialPlugin` 之后，它就是一个正常的材质类型了。
    let shapes: [(Mesh, Color); 3] = [
        (
            Cuboid::new(1.3, 1.3, 1.3).into(),
            Color::srgb(0.95, 0.35, 0.30),
        ),
        (Sphere::new(0.75).into(), Color::srgb(0.30, 0.75, 0.95)),
        (
            Cylinder::new(0.6, 1.5).into(),
            Color::srgb(0.55, 0.90, 0.45),
        ),
    ];
    for (i, (mesh, color)) in shapes.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(glow_materials.add(GlowMaterial {
                base_color: color.to_linear().to_vec4(),
                params: Vec4::ZERO,
            })),
            Transform::from_xyz(-2.8 + i as f32 * 2.8, 0.75, 0.0),
        ));
    }

    println!("── 自定义材质 GlowMaterial 已注册（MaterialPlugin）");
    println!("   shader 来自 {SHADER_PATH}；三个物体用条纹 + 呼吸着色");
}

/// 把时间写进材质的 uniform。
///
/// ⚠️ 注意是**改资产里的数据**（`materials.get_mut/iter_mut`），不是改组件 ——
/// 改完要等渲染侧把新的 uniform 上传，所以效果会有一两帧延迟。
/// 这也是"材质是资产"（027 讲的）在自定义着色器上的体现。
fn pulse_materials(time: Res<Time>, mut materials: ResMut<Assets<GlowMaterial>>) {
    let value = (time.elapsed_secs() * 2.0).sin() * 0.5 + 0.5;
    for (_, material) in materials.iter_mut() {
        material.params.x = value;
    }
}

// ─────────────────────────────────────────────────────────────────────
// 跑起来会看到什么
//
//   · 地面上摆着立方体、球、圆柱，用的都是自定义材质
//   · 它们表面有**斜向条纹**，且条纹的明暗随时间**缓慢呼吸**
//   · 朝上的面偏亮、朝下的面偏暗（那不是光照，是 shader 里用法线算的）
//   · 旁边地面用的是引擎自带材质，一眼能看出两者的差别
//
// 控制台输出：
//
//   ── 自定义材质 GlowMaterial 已注册（MaterialPlugin）
//      shader 来自 shaders/glow.wesl；三个物体用条纹 + 呼吸着色
//
// ─────────────────────────────────────────────────────────────────────
// ★ 两处"不报错但没效果"的坑（本讲最容易踩的）
//
// ① **忘了 `MaterialPlugin::<GlowMaterial>::default()`**
//    材质类型没注册进渲染管线。程序照常跑、日志干干净净，
//    就是**画面上什么都不显示**。
//
// ② **Rust 与 WESL 对不上**
//    binding 号写岔了、类型不匹配、`#import` 路径写错 —— 这些都是
//    **运行时**才报错（naga 编译 shader 失败），表现同样是画面变黑。
//    终端里能看到形如 `Shader validation error` 的日志。
//
// 这两条的根源是同一个：**Rust 侧和 WESL 侧是两份独立的契约，
// 没有编译器替你对齐**。Bevy 尽量在启动时校验，但很多问题只有真正
// 画到屏幕上才暴露。
//
// 所以写自定义着色器的调试顺序建议是：
//   1. 先看终端有没有 shader 编译错误（有就直接改 WESL）
//   2. 没有的话，确认 `MaterialPlugin` 装了没
//   3. 还不行，把 fragment 改成 `return vec4(1.0, 0.0, 1.0, 1.0)`
//      （纯品红）—— 能看见就说明管线通了，问题在计算逻辑里
//
// ─────────────────────────────────────────────────────────────────────
// ★ 材质 bind group 的编号**不能写死**（本讲最大的坑）
//
// 这是本轮踩得最狠的一处。初版照老写法写了 `@group(2)`，编译一切正常，
// **一跑就炸**：
//
//     ERROR bevy_render::error_handler: Caught rendering error: Validation Error
//       Shader global ResourceBinding { group: 2, binding: 0 } is not available
//       in the pipeline layout
//         Storage class Storage { .. } doesn't match the shader Uniform
//     ERROR ... Quitting the application due to Validation RenderError
//
// 报错的意思是"group 2 的 binding 0 在管线里是 storage buffer，不是 uniform"——
// 也就是**撞到别的组上了**。
//
// 原因是 0.19 里材质 bind group 的**编号不是固定的**：引擎会根据启用的特性
// （bindless 之类）动态决定，再通过预处理器变量注入。看引擎自己的
// `pbr_bindings.wesl` 就能发现它一个数字都没写死：
//
//     @group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> material: StandardMaterial;
//
// 所以正确写法是**让引擎来填这个数**：
//
//     @group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> base_color: vec4<f32>;
//
// 记住这条：**shader 里凡是和管线布局有关的编号，优先找引擎的常量**，
// 别照抄示例里的字面量 —— 那些数字在版本之间会变。
//
// ─────────────────────────────────────────────────────────────────────
// group 的分工（概念上的划分）
//
//   `@group(0)`  视图级：相机矩阵、光照、环境贴图（引擎管）
//   `@group(1)`  网格级：模型矩阵、骨骼、形变（引擎管）
//   材质级       **你自己声明的数据** —— 编号用 `constants::MATERIAL_BIND_GROUP`
//
// 需要更多材质数据时，`AsBindGroup` 支持的类型不止 uniform：
//
//   #[uniform(0)]             普通 uniform（本讲用的）
//   #[texture(1)]             贴图
//   #[sampler(2)]             采样器（配 `#[texture]` 用）
//   #[storage(3, read_only)]  存储缓冲（大量数据，比如每个实例一份参数）
//
// 每个字段占一个 binding 号，**编号不要跳着写** —— 跳号本身合法，
// 但很容易和 WESL 对不上，是上面②那类 bug 的高发区。
//
// 两个 uniform 都用 `vec4<f32>` 而不是把 `f32` 单独绑一个 binding：
// uniform 缓冲有 16 字节对齐要求，塞进 vec4 最省心，不用算 padding。
// （本讲只用 `params.x`，另外三个分量留着备用。）
//
// ─────────────────────────────────────────────────────────────────────
// ✅ 热重载（本项目已开 `dev`，所以可用）
//
// 写 shader 最爽的一点是"改 WESL 存盘、画面立刻更新，不用重编译 Rust"。
// 这需要 `file_watcher` feature —— 它属于 `dev` 特性组（`default` 里没有），
// 而本项目的 Cargo.toml 已经开了 `dev`，所以它是生效的：
//
//     bevy = { version = "0.20", features = ["dev"] }
//
// 跑起来之后改 `assets/shaders/glow.wesl` 存盘，画面几帧内就会变 ——
// 调 shader 效率高很多（详见 020 讲的说明）。
//
// 另一种做法是 `embedded_asset!(app, "glow.wesl")`：把 WESL **编进二进制**，
// 不依赖 `assets/` 目录、也不会因为路径找不到而失败，代价是彻底没有热重载
// （改一次就得重编译）。发布时常用这种。
//
// ─────────────────────────────────────────────────────────────────────
// shader 文件的归属
//
// `assets/shaders/glow.wesl` 和 `assets/` 下别的文件性质不同：
//
//   `assets/textures/*.png`、`assets/models/*.gltf`   由 `tools/make_assets.py` **生成**
//   `assets/shaders/glow.wesl`                        是**手写的源码**
//
// 所以跑素材脚本不会覆盖它，它应当随源码一起进版本库。
// 把它放在 `assets/` 下是因为 `ShaderRef::path(..)` 的路径基准就是 `assets/`。
//
// ─────────────────────────────────────────────────────────────────────
// 想要"在引擎光照基础上做效果"怎么办
//
// 本讲故意**不接** PBR —— 从零算颜色能把契约讲清楚。真实项目里更多是：
//
//   #import bevy_pbr::pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing}
//   #import bevy_pbr::mesh_view_bindings::view
//   #import bevy_pbr::mesh_bindings::mesh
//
// 然后把自定义材质的字段填进 `StandardMaterial`（用
// `#[uniform(0)] standard_material: StandardMaterial` 直接内嵌），
// 拿光照结果再做描边、发光等后处理。
//
// 上游 `examples/shader/` 下有一系列递进例子（`shader_material`、
// `shader_material_glsl`、`extended_material`、`array_texture`…），
// 本讲是入门那一档，往下走从那里接。
// ─────────────────────────────────────────────────────────────────────
