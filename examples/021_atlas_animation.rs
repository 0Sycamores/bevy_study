//! 021 · 图集与帧动画
//!
//! 运行：`cargo run --example 021_atlas_animation`
//!
//! 新增概念
//!   TextureAtlasLayout           一张大图怎么切成小格（列数 × 行数 + 每格尺寸）
//!   TextureAtlas { layout, index }   某个精灵"正在用第几格"
//!   Assets<TextureAtlasLayout>   布局也是资产，需要 `add` 进仓库
//!   Sprite::from_atlas_image(..) 用图集创建精灵
//!   ImagePlugin::default_nearest()  像素图必须用最近邻采样，否则会糊
//!
//! 使用场景
//!   逐帧动画（走路、爆炸、特效）；把几十张小图打成一张大图省内存与绘制调用
//!
//! 注意：**布局是可共享的**（`Handle` 克隆即可），而"当前第几帧"是**每个精灵各一份** ——
//!       这正是下面三个精灵能共用一张图却各演各的原因

use bevy::prelude::*;

fn main() {
    App::new()
        // ⚠️ 像素图一定要用最近邻采样，否则放大后边缘会糊成一团。
        // 本项目 tools/make_assets.py 生成的就是硬边像素风，正好演示这一点。
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "021 · 图集动画：甲 6fps / 乙 12fps / 丙 定格第 4 帧".into(),
                        resolution: (960, 640).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, animate)
        .run();
}

/// 一帧动画需要的信息：第几帧到第几帧、多久换一帧。
#[derive(Component)]
struct SpriteAnimation {
    name: &'static str,
    frames: usize,
    timer: Timer,
    loops: u32,
}

/// 雪碧图的规格：6 帧，每帧 48×48，横向排成一行。
const FRAME_SIZE: u32 = 48;
const FRAME_COUNT: usize = 6;

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2d);

    // 整张雪碧图是 288×48 —— 就是脚本里 `make_runner_sheet()` 画出来的那条。
    let texture: Handle<Image> = assets.load("textures/runner.png");

    // ★ 把"怎么切"描述出来，存进布局仓库，拿回一个句柄。
    //   参数：每格尺寸、列数、行数、格间内边距、整体偏移。
    //   本讲生成的图没有内边距和偏移，所以后两个传 None。
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(FRAME_SIZE),
        FRAME_COUNT as u32,
        1,
        None,
        None,
    ));

    println!(
        "── textures/runner.png 是 288x48，按 {FRAME_SIZE}x{FRAME_SIZE} 切成 {FRAME_COUNT} 帧"
    );

    // 两个会动的：贴图和布局都相同，只是帧率不同。
    // 注意 `texture.clone()` / `layout.clone()` 只是**复制句柄**（加引用计数），
    // 并不会把图片复制一份 —— 三个精灵共用同一份贴图数据。
    for (name, fps, x) in [("甲 6fps", 6.0, -240.0), ("乙 12fps", 12.0, 0.0)] {
        commands.spawn((
            Sprite::from_atlas_image(
                texture.clone(),
                TextureAtlas {
                    layout: layout.clone(),
                    index: 0, // 从第 0 帧开始
                },
            ),
            Transform::from_xyz(x, 0.0, 0.0).with_scale(Vec3::splat(3.0)),
            SpriteAnimation {
                name,
                frames: FRAME_COUNT,
                // 帧间隔 = 1 / 帧率（010 讲的 Timer，这里是"每个实体各自节奏"的典型用法）
                timer: Timer::from_seconds(1.0 / fps, TimerMode::Repeating),
                loops: 0,
            },
        ));
    }

    // 第三个：**同一个布局**，但 index 固定在第 3 帧 ——
    // 它不动，说明"当前第几帧"是每个精灵自己的状态，不共享。
    commands.spawn((
        Sprite::from_atlas_image(texture, TextureAtlas { layout, index: 3 }),
        Transform::from_xyz(240.0, 0.0, 0.0).with_scale(Vec3::splat(3.0)),
    ));
}

/// 每帧推进计时器，到点就把 `index` 往后挪一格。
fn animate(time: Res<Time>, mut sprites: Query<(&mut SpriteAnimation, &mut Sprite)>) {
    for (mut animation, mut sprite) in &mut sprites {
        if !animation.timer.tick(time.delta()).just_finished() {
            continue;
        }

        // 图集信息挂在 `Sprite` 上：`sprite.texture_atlas` 是 `Option<TextureAtlas>`。
        let Some(atlas) = &mut sprite.texture_atlas else {
            continue;
        };

        atlas.index = (atlas.index + 1) % animation.frames;
        if atlas.index == 0 {
            animation.loops += 1;
            println!(
                "   {} 跑完第 {} 圈（{} 帧）",
                animation.name, animation.loops, animation.frames
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// 实测输出
//
//   ── textures/runner.png 是 288x48，按 48x48 切成 6 帧
//      乙 12fps 跑完第 1 圈（6 帧）
//      甲 6fps 跑完第 1 圈（6 帧）
//      乙 12fps 跑完第 2 圈（6 帧）
//      乙 12fps 跑完第 3 圈（6 帧）
//      甲 6fps 跑完第 2 圈（6 帧）
//      乙 12fps 跑完第 4 圈（6 帧）
//      ...（乙 每秒两圈、甲 每秒一圈；两者的先后顺序随时序浮动，
//           但圈数的比例始终是 2:1）
//
// 画面上：三个小人并排。左边的甲慢悠悠地跑，中间的乙跑得快一倍，
// 右边的丙**定在第 4 帧不动**（注意它是第 4 帧，因为 index 从 0 数起）。
//
// ─────────────────────────────────────────────────────────────────────
// 图集到底解决了什么
//
// 不用图集的话，6 帧动画就是 6 个 PNG 文件、6 次加载、6 个 `Handle<Image>`，
// 而引擎还得为每张图单独安排绘制。
//
// 用图集则变成：**1 个文件、1 次加载、1 个句柄**，切法写在一个 `TextureAtlasLayout` 里。
// 精灵身上只多了一个"用第几格"的数字。
//
//   `TextureAtlasLayout`   怎么切 —— 可共享，通常一个动画一份
//   `TextureAtlas.index`   用第几格 —— **每个精灵各一份**
//
// 这个区分很重要，它意味着：
//   · 三个精灵共用一份贴图和一份布局，内存里只有一份 288×48 的图
//   · 但它们各自可以停在不同帧（丙就是定格的）
//   · 想加第四个角色，只要换个贴图句柄，布局逻辑照搬
//
// ─────────────────────────────────────────────────────────────────────
// `default_nearest()` 为什么必须加
//
// 默认的采样方式是线性插值，适合照片，但会让像素图**糊**：
// 放大 3 倍时，本应锐利的方块边缘会变成渐变色带。
// `ImagePlugin::default_nearest()` 换成最近邻采样，边缘就保持硬朗。
//
// 判断标准很简单：**素材是像素画就加它，是照片/矢量图就别加。**
//
// ─────────────────────────────────────────────────────────────────────
// 从图集到"动画系统"
//
// 本讲的动画逻辑只有几行：计时器到点 → `index + 1` → 到末尾回 0。
// 真实项目里通常还会加上：
//   · 不同动作不同区间（站立 0..1 帧、走 4..7 帧、攻击 8..11 帧）——
//     也就是给 `SpriteAnimation` 加 `first` / `last` 两个字段
//   · 由状态驱动切换动作 —— 那是 018 的 `States` 配合
//   · `TimerMode::Once` + 停在最后一帧，做"播完就定格"的一次性动画
//
// 顺带一提：`Sprite` 上的 `texture_atlas` 是 `Option`，
// 所以普通精灵（`Sprite::from_image`）和帧动画精灵是同一个组件类型 ——
// 上面那个 `let Some(atlas) = .. else { continue }` 就是在处理"这个精灵没有图集"的情况。
// ─────────────────────────────────────────────────────────────────────
