//! 023 · UI 交互
//!
//! 运行：`cargo run --example 023_ui_interaction`
//!
//! 新增概念
//!   Interaction               按钮的三种状态：None / Hovered / Pressed
//!   Changed<Interaction>      过滤出"状态刚变过"的按钮（015 讲的变更检测）
//!   On<Pointer<..>>           观察者接收指针事件（013 讲的观察者用在 UI 上）
//!   Button                    UI 按钮的标记组件
//!
//! 使用场景
//!   开始/退出按钮、设置面板、任何要点一下的东西
//!
//! 注意：两种写法**都能用，但要分清场景** —— 轮询适合"要看持续状态"，
//!       观察者适合"发生才算数"。本讲把两个按钮并排对照，两种风格各写一遍

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "023 · UI 交互：左=轮询 / 右=观察者（点一下试试）".into(),
                resolution: (960, 640).into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<Counters>()
        .add_systems(Startup, setup)
        // 按钮 A 走轮询，顺带负责刷新状态文字
        .add_systems(Update, (poll_button, refresh_text))
        .run();
}

#[derive(Resource, Default)]
struct Counters {
    polled: u32,
    observed: u32,
}

#[derive(Component)]
struct PollButton;

#[derive(Component)]
struct StatusText;

const NORMAL: Color = Color::srgb(0.18, 0.19, 0.24);
const HOVERED: Color = Color::srgb(0.28, 0.30, 0.38);
const PRESSED: Color = Color::srgb(0.30, 0.62, 0.38);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // 外层容器：flex 纵向居中（022 讲的排版思路）
    let root = commands
        .spawn(Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: px(24),
            ..default()
        })
        .id();

    let button_node = Node {
        width: px(300),
        height: px(64),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_font = TextFont {
        font_size: FontSize::Px(24.0),
        ..default()
    };

    // ── 按钮 A：靠轮询 `Changed<Interaction>` 驱动 ──
    let poll_button = commands
        .spawn((
            Button,
            PollButton,
            button_node.clone(),
            BackgroundColor(NORMAL),
            children![(
                Text::new("Poll (click me)"),
                button_font.clone(),
                TextColor(Color::srgb(0.92, 0.92, 0.96)),
            )],
        ))
        .id();

    // ── 按钮 B：靠观察者驱动 ──
    //
    // ⚠️ `observe(..)` 不是 bundle，不能写在 `children![..]` 里 ——
    //    它是 `EntityCommands` 上的方法，所以要先把实体 spawn 出来再挂。
    //
    // 外观反馈用了整整四个观察者（移入/移出/按下/松开），
    // 而按钮 A 只用了一个轮询系统就搞定 —— 这正是两种风格的典型差异。
    let observer_button = commands
        .spawn((
            Button,
            button_node,
            BackgroundColor(NORMAL),
            children![(
                Text::new("Observe (click me)"),
                button_font,
                TextColor(Color::srgb(0.92, 0.92, 0.96)),
            )],
        ))
        .observe(on_over)
        .observe(on_out)
        .observe(on_press)
        .observe(on_release)
        .observe(on_click)
        .id();

    let status = commands
        .spawn((
            StatusText,
            Text::new("polled 0 / observed 0"),
            TextFont {
                font_size: FontSize::Px(22.0),
                ..default()
            },
            TextColor(Color::srgb(0.75, 0.78, 0.85)),
        ))
        .id();

    commands.entity(root).add_child(poll_button);
    commands.entity(root).add_child(observer_button);
    commands.entity(root).add_child(status);

    println!("── 左：轮询 `Changed<Interaction>`；右：观察者 `On<Pointer<..>>`");
    println!("   两个按钮外观与行为一致，但实现路子完全不同 —— 看文末对比");
}

/// 写法①：**轮询**。每帧查一遍"哪些按钮的 `Interaction` 刚变过"。
///
/// 一个系统就同时搞定了三种外观反馈，外加"按下"的计数 —— 因为
/// `Interaction` 本来就是"当前处于什么状态"，而外观恰恰关心状态。
fn poll_button(
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<PollButton>),
    >,
    mut counters: ResMut<Counters>,
) {
    for (interaction, mut color) in &mut buttons {
        match interaction {
            Interaction::Pressed => {
                *color = PRESSED.into();
                counters.polled += 1;
                println!("   [轮询]   按下 → 累计 {} 次", counters.polled);
            }
            Interaction::Hovered => {
                *color = HOVERED.into();
            }
            Interaction::None => {
                *color = NORMAL.into();
            }
        }
    }
}

// ── 写法②：观察者。每个都只管一件小事 ────────────────────────────────
//
// 观察者拿到的 `On<Pointer<..>>` 里有 `entity`（被指到的实体），
// 所以可以直接改**那一个**按钮的颜色，不需要 `With<..>` 去筛。

fn on_over(over: On<Pointer<Over>>, mut colors: Query<&mut BackgroundColor>) {
    if let Ok(mut color) = colors.get_mut(over.entity) {
        *color = HOVERED.into();
    }
}

fn on_out(out: On<Pointer<Out>>, mut colors: Query<&mut BackgroundColor>) {
    if let Ok(mut color) = colors.get_mut(out.entity) {
        *color = NORMAL.into();
    }
}

fn on_press(press: On<Pointer<Press>>, mut colors: Query<&mut BackgroundColor>) {
    if let Ok(mut color) = colors.get_mut(press.entity) {
        *color = PRESSED.into();
    }
}

fn on_release(release: On<Pointer<Release>>, mut colors: Query<&mut BackgroundColor>) {
    if let Ok(mut color) = colors.get_mut(release.entity) {
        *color = HOVERED.into();
    }
}

/// `Click` 要求"按下和松开都落在同一个实体上"，比 `Press` 更接近真正的"点击"。
fn on_click(click: On<Pointer<Click>>, mut counters: ResMut<Counters>) {
    counters.observed += 1;
    println!(
        "   [观察者] 点到实体 {:?} → 累计 {} 次",
        click.entity, counters.observed
    );
}

/// 计数变了就刷新状态文字（015 讲的 `is_changed`）。
fn refresh_text(counters: Res<Counters>, mut texts: Query<&mut Text, With<StatusText>>) {
    if !counters.is_changed() {
        return;
    }
    for mut text in &mut texts {
        text.0 = format!(
            "polled {} / observed {}",
            counters.polled, counters.observed
        );
    }
}

// ─────────────────────────────────────────────────────────────────────
// 跑起来会看到什么
//
//   · 鼠标移到任一按钮上 → 底色变亮
//   · 按住 → 底色变绿
//   · 左按钮按下时打印：`[轮询]   按下 → 累计 N 次`
//   · 右按钮点击时打印：`[观察者] 点到实体 7v0 → 累计 N 次`
//   · 每次计数变化，下面那行状态文字同步刷新
//
// 两个按钮外观与行为一致，但实现完全不同 —— 这正是本讲要对照的。
//
// ─────────────────────────────────────────────────────────────────────
// 轮询 vs 观察者：怎么选
//
//   `Changed<Interaction>` 轮询                 `On<Pointer<..>>` 观察者
//   ------------------------------------------------------------------
//   **1 个系统**就能管完外观三态 + 计数        外观三态要 **4~5 个观察者**
//   每帧遍历所有按钮（有 `Changed` 过滤）       只在该实体出事时跑
//   拿得到 Hovered / Pressed 的**持续状态**     只有"事件"，没有持续状态
//   语义是"按下"（不含松开）                    语义是"点击"（按下+松开同实体）
//
// 经验上：
//   · **按钮外观**（悬停/按下的颜色）用轮询 —— 它描述的是"当前什么状态"
//   · **点一下触发动作**（开始游戏、开关面板）用观察者 —— 语义更准，
//     而且不必担心 `Changed` 的时机
//
// 两者混用完全没问题，本讲本来也就是混用的。
//
// ─────────────────────────────────────────────────────────────────────
// ⚠️ 别拿 `Interaction::Pressed` 当"点击"
//
// `Pressed` 是**按住期间持续为真**的（和 011 讲的 `pressed` 一模一样）。
// 直接拿它做"点一下加一次分"，按住不放就会每帧加一次。
//
// 本讲用 `Changed<Interaction>` 把"状态没变"的帧过滤掉了，所以一个按下动作
// 只记一次 —— 但它记的仍是**"按下的瞬间"**，不是"松开且仍指着这个按钮"。
// 要严格的点击语义就用 `Pointer<Click>`。
//
// ─────────────────────────────────────────────────────────────────────
// 指针事件家族
//
// 0.19 里 **UI 与精灵的指针拾取默认开启**（3D 网格拾取要手动加
// `MeshPickingPlugin`，029 会讲）。可用的事件：
//
//   `Pointer<Over>` / `Pointer<Out>`                  移入 / 移出
//   `Pointer<Press>` / `Pointer<Release>`             按下 / 松开
//   `Pointer<Click>`                                  同一点完成按下+松开
//   `Pointer<Drag>` / `DragStart` / `DragEnd`         拖拽
//   `Pointer<DragEnter>` / `DragOver` / `DragLeave` / `Pointer<DragDrop>`  拖放
//
// 每个事件里都有 `entity`（被指到的实体）和指针位置等信息，
// 所以"点到哪个格子"这类问题不用自己做射线。
//
// ─────────────────────────────────────────────────────────────────────
// 关于 `Button` 组件
//
// `Button` 本身**不提供任何行为** —— 它只是让引擎知道"这个节点可以交互"，
// 从而维护它的 `Interaction` 组件。所以：
//   · 想自定义交互控件，`Button` + 自己的标记就够了
//   · 任何带 `Interaction` 的节点都能套用本讲的逻辑
//
// Bevy 还自带一套现成控件（`bevy_ui_widgets`：按钮、滑块、复选框…），
// 带键盘导航与禁用态等无障碍能力。本讲手写是为了看清原理，
// 真实项目里可以直接用那套。
// ─────────────────────────────────────────────────────────────────────
