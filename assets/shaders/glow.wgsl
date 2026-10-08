// 030 讲的自定义材质。
//
// 这个 shader **不参与 PBR 光照** —— 它自己算颜色，用来把"Rust 数据怎么进 shader"
// 这件事讲清楚。真实项目里更常见的是先 `#import bevy_pbr::pbr_functions` 拿到
// Bevy 的光照结果，再在此基础上做自己的效果。

// `VertexOutput` 是引擎的**顶点着色器输出**，也就是片元着色器的输入。
// 用它当参数类型，才能和 Bevy 自带的网格管线对接上。
//
// ⚠️ 这个 `#import` 路径要写全：`bevy_pbr::forward_io::VertexOutput`。
//    写错的话是**运行时**报错（naga 编译失败，画面一片黑），不是编译期。
#import bevy_pbr::forward_io::VertexOutput

// ── 传给 shader 的数据 ──
//
// 这里是**和 Rust 侧的第二份契约**：`@binding(N)` 里的 N 必须和 Rust 里
// `#[uniform(N)]` 对上。
//
// ⚠️⚠️ 但 **group 不能写死数字**。
//
// 老教程（以及不少 AI 生成的代码）会写 `@group(2)` —— 在 0.19 上这会
// **运行时炸掉**，报的是这么一句很绕的错：
//
//     Shader global ResourceBinding { group: 2, binding: 0 } is not available
//     in the pipeline layout
//       Storage class Storage { .. } doesn't match the shader Uniform
//
// 因为 0.19 里材质 bind group 的**编号不是固定的** —— 引擎会根据启用的特性
// （比如 bindless）动态决定，然后通过 `#{MATERIAL_BIND_GROUP}` 注入。
// 写死 2 就会撞到别的组上（本讲实测撞到了一个 storage buffer）。
//
// 所以正确写法是让引擎来填这个数：
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> base_color: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> params: vec4<f32>;

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
