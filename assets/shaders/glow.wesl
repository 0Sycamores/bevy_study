// 030 讲的自定义材质（Bevy 0.20 / WESL）。
//
// 0.20 起 Bevy 用 **WESL** 取代了自研的 "Bevy Extended WGSL" 方言：
//   · 文件扩展名 .wgsl → .wesl
//   · `#import a::b`      →  `import a::b;`（结尾分号，且必须放在文件最前面）
//   · `#ifdef X`          →  `@if(X)`
//   · `#{NAME}`           →  `constants::NAME`
//   · `#define_import_path` 没有了；模块名改为「crate + 文件路径」
//
// ⚠️ 纯 WGSL（不含任何预处理器指令）仍可直接用 —— 本文件曾经是那种半方言写法，
//    升级到 0.20 后**运行时**直接报：
//      Shader '' parsing error: ... found "#"
//    所以必须改写。注解见文末的迁移说明。

// ── 顶点着色器的输出，也就是片元着色器的输入 ──
//
// 注意模块路径变了：旧的是 `bevy_pbr::forward_io`，
// 0.20 按「crate + 文件在 crate 里的路径」命名，所以是 `bevy_pbr::render::forward_io`
// （对应 `bevy_pbr/src/render/forward_io.wesl`）。
import bevy_pbr::render::forward_io::VertexOutput;

// ── 传给 shader 的数据 ──
//
// 这是**和 Rust 侧的第二份契约**：`@binding(N)` 的 N 必须和 Rust 里
// `#[uniform(N)]` 对上。
//
// ⚠️ group 不能写死数字，要用常量。
//
// 注意是 **group** 用常量、**binding** 用数字 —— 官方迁移指南里那句
// `@group(2) @binding(constants::MATERIAL_BIND_GROUP)` 是写反了的；
// 引擎自己的 `bevy_pbr/src/render/pbr_bindings.wesl` 写的是：
//
//     @group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> material: StandardMaterial;
//
// 材质 bind group 的编号由引擎按启用的特性动态决定，所以必须让它来填。
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
