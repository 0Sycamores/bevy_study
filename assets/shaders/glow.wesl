// 030 讲的自定义材质。
//
// Bevy 的 shader 用 WESL 编写 —— 它是 WGSL 的扩展，提供了模块、导入、
// 条件编译这些能力。写自定义材质时，你只需要关心三件事：
//   ① 把引擎的顶点着色器输出类型 import 进来，作为片元着色器的输入
//   ② 用 `@group` / `@binding` 声明自己的数据
//   ③ 写 `@fragment` 入口

// ── 顶点着色器的输出，也就是片元着色器的输入 ──
//
// 模块名按「crate + 文件在该 crate 里的路径」来写，
// 对应的源文件是 `bevy_pbr/src/render/forward_io.wesl`。
import bevy_pbr::render::forward_io::VertexOutput;

// ── 传给 shader 的数据 ──
//
// 这是**和 Rust 侧的第二份契约**：`@binding(N)` 的 N 必须和 Rust 里
// `#[uniform(N)]` 对上。
//
// ⚠️ group 不能写死数字，要用常量。
//
// 材质 bind group 的编号由引擎按启用的特性动态决定，所以必须让它来填。
// 注意是 **group** 用常量、**binding** 用数字 —— 引擎自己的
// `bevy_pbr/src/render/pbr_bindings.wesl` 就是这么写的：
//
//     @group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> material: StandardMaterial;
@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> base_color: vec4<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var<uniform> params: vec4<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // 法线朝上 -> 亮，朝下 -> 暗。一个不需要光照的"假明暗"。
    let normal = normalize(in.world_normal);
    let facing = clamp(normal.y * 0.5 + 0.5, 0.0, 1.0);

    // 用世界坐标画条纹 —— 这样一眼能看出颜色是**按位置算出来的**，
    // 而不是简单刷了一层纯色。
    let stripe = 0.5 + 0.5 * sin((in.world_position.x + in.world_position.z) * 6.0);

    // `params.x` 由 Rust 每帧更新，驱动"呼吸"。
    let glow = 1.0 + params.x * stripe * 0.8;

    return vec4<f32>(base_color.rgb * (0.35 + 0.65 * facing) * glow, base_color.a);
}
