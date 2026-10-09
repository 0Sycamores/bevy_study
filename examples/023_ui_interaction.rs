//! 023 · UI 交互
//!
//! 运行：`cargo run --example 023_ui_interaction`
//!
//! 新增概念
//!   ui_widgets::Button        无外观的按钮控件，`DefaultPlugins` 里已带 `ButtonPlugin`
//!   ui::Pressed               标记组件：按住期间存在（`Added<Pressed>` 抓"刚按下"那帧）
//!   picking::hover::Hovered   标记组件：是否悬停（轮询用，要手动挂上）
//!   On<PointerOver> 等        扁平化的指针事件，一个事件挂一个观察者（013 讲的）
//!   ui_widgets::Activate      按钮"松开完成点击"时发出（键盘回车/空格也会发）
//!
//! 使用场景
//!   开始/退出按钮、设置面板、任何要点一下的东西
//!
//! 注意：轮询适合"要看持续状态"，观察者适合"发生才算数"，本讲两个按钮并排对照

use bevy::{
    // 这几个都不在 prelude 里（prelude 里的 `Button` / `Interaction` 是 0.20 要淘汰的旧 API）。
    picking::hover::Hovered,
    prelude::*,
    ui::Pressed,
    ui_widgets::{Activate, Button},
};

fn main() {
    App::new()
        // `DefaultPlugins` 里已经有 `UiWidgetsPlugins`（按钮行为）和拾取插件，
        // 所以 `Button` / `Hovered` / `Activate` 开箱即用，不用手动加插件。
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

    // 两个按钮都用同一套东西：
    //   Button            0.20 的"无外观"按钮控件，行为（按下/点击）由 ButtonPlugin 管
    //   Hovered           悬停状态 —— 0.20 要自己挂，拾取后端会更新它
    //   Pickable::IGNORE  挂在下面的文字上，让文字不参与拾取，事件才会落在按钮本体上
    //
    // ── 按钮 A：靠轮询 `Hovered` + `Pressed` 驱动 ──
    let poll_button = commands
        .spawn((
            Button,
            PollButton,
            Hovered::default(),
            button_node.clone(),
            BackgroundColor(NORMAL),
            children![(
                Text::new("Poll (click me)"),
                Pickable::IGNORE,
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
    // 外观反馈用了整整四个观察者（移入/移出/按下/松开），再加一个数点击的；
    // 而按钮 A 只用了一个轮询系统就搞定 —— 这正是两种风格的典型差异。
    let observer_button = commands
        .spawn((
            Button,
            Hovered::default(),
            button_node,
            BackgroundColor(NORMAL),
            children![(
                Text::new("Observe (click me)"),
                Pickable::IGNORE,
                button_font,
                TextColor(Color::srgb(0.92, 0.92, 0.96)),
            )],
        ))
        .observe(on_over)
        .observe(on_out)
        .observe(on_press)
        .observe(on_release)
        .observe(on_activate)
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

    println!("── 左：轮询 `Hovered` + `Pressed`；右：观察者 `On<PointerOver>` / `On<Activate>`");
    println!("   两个按钮外观规则一样，但实现路子完全不同 —— 看文末对比");
}

/// 写法①：**轮询**。每帧查一遍"按钮现在是什么状态"，一个系统管完外观三态 + 计数。
///
/// 0.20 里没有"三种状态的枚举"了：悬停看 `Hovered`（一个 `bool`），
/// 按住看 `Pressed` 这个**标记组件在不在** —— 三态是这两个组件凑出来的。
/// 好在它们都由引擎驱动，我们只要读，不用自己维护。
fn poll_button(
    mut buttons: Query<(&Hovered, Has<Pressed>, &mut BackgroundColor), With<PollButton>>,
    // 计数依旧靠变更检测：`Pressed` 刚被加上的那一帧（015 讲的 `Added`）
    just_pressed: Query<(), (With<PollButton>, Added<Pressed>)>,
    mut counters: ResMut<Counters>,
) {
    for (hovered, pressed, mut color) in &mut buttons {
        // 按下必然同时悬停；指针移开就回到 NORMAL（和右边观察者的行为对齐）
        *color = match (hovered.get(), pressed) {
            (true, true) => PRESSED.into(),
            (true, false) => HOVERED.into(),
            (false, _) => NORMAL.into(),
        };
    }

    if !just_pressed.is_empty() {
        counters.polled += 1;
        println!("   [轮询]   按下 → 累计 {} 次", counters.polled);
    }
}

// ── 写法②：观察者。每个都只管一件小事 ────────────────────────────────
//
// 观察者拿到的 `On<PointerOver>` 里有 `entity`（被指到的实体）和 `pointer`
// （指针信息），所以可以直接改**那一个**按钮的颜色，不需要 `With<..>` 去筛。

fn on_over(over: On<PointerOver>, mut colors: Query<&mut BackgroundColor>) {
    if let Ok(mut color) = colors.get_mut(over.entity) {
        *color = HOVERED.into();
    }
}

fn on_out(out: On<PointerOut>, mut colors: Query<&mut BackgroundColor>) {
    if let Ok(mut color) = colors.get_mut(out.entity) {
        *color = NORMAL.into();
    }
}

fn on_press(press: On<PointerPress>, mut colors: Query<&mut BackgroundColor>) {
    if let Ok(mut color) = colors.get_mut(press.entity) {
        *color = PRESSED.into();
    }
    // `Pointer` 在 0.20 不再是泛型，而是挂在事件上的字段；位置是 `Vec2`
    println!("   [观察者] 按下 @ {:?}", press.pointer.position);
}

fn on_release(release: On<PointerRelease>, mut colors: Query<&mut BackgroundColor>) {
    if let Ok(mut color) = colors.get_mut(release.entity) {
        *color = HOVERED.into();
    }
}

/// `Button` 在"松开且仍指着自己"时发出 `Activate`，比 `PointerPress` 更接近真正的"点击"。
fn on_activate(activate: On<Activate>, mut counters: ResMut<Counters>) {
    counters.observed += 1;
    println!(
        "   [观察者] 点到实体 {:?} → 累计 {} 次",
        activate.entity, counters.observed
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
//   · 按住 → 底色变绿；松开（仍指着按钮）→ 回到"变亮"
//   · 左按钮按下时打印：`[轮询]   按下 → 累计 N 次`
//   · 右按钮按下时打印：`[观察者] 按下 @ Vec2(.., ..)`
//   · 右按钮完成一次点击打印：`[观察者] 点到实体 7v0 → 累计 N 次`
//   · 每次计数变化，下面那行状态文字同步刷新
//
// 两个按钮外观规则一致（都按"悬停 / 按住"上色），但实现完全不同。
//
// ⚠️ 悬停和点击要**动鼠标**才看得到，机器验证不了。本项目只自动验证了
//    "能编译、能启动、5 秒内不报错"，其余靠手动操作确认。
//
// ─────────────────────────────────────────────────────────────────────
// 轮询 vs 观察者：怎么选
//
//   `Hovered` + `Pressed` 轮询                `On<Pointer..>` / `On<Activate>` 观察者
//   --------------------------------------------------------------------------
//   **1 个系统**管完外观三态 + 计数            外观三态要 **4 个观察者**，再加 1 个数点击的
//   每帧遍历所有按钮（计数那份用 `Added` 过滤）  只在该实体出事时才跑
//   拿得到 `Hovered` / `Pressed` 的**持续状态**  只有"事件"，没有持续状态
//   语义是"按下"（不含松开）                    语义是"点击"（`Activate` 在松开时发）
//
// 经验上：
//   · **按钮外观**（悬停/按下的颜色）用轮询 —— 它描述的是"当前什么状态"
//   · **点一下触发动作**（开始游戏、开关面板）用观察者 —— 语义更准
//
// 两者混用完全没问题，本讲本来也就是混用的。
//
// ─────────────────────────────────────────────────────────────────────
// ⚠️ 别拿 `Pressed` 当"点击"
//
// `Pressed` 是**按住期间一直存在**的标记组件（和 011 讲的 `pressed` 一模一样）。
// 直接拿它做"点一下加一次分"，按住不放就会每帧加一次。
//
// 本讲用 `Added<Pressed>` 把"不是刚按下"的帧过滤掉了，所以一个按下动作只记一次 ——
// 但它记的仍是**"按下的瞬间"**，不是"松开且仍指着这个按钮"。
// 要严格的点击语义就用 `Activate`（本讲右边用的就是它）。
//
// ─────────────────────────────────────────────────────────────────────
// 指针事件家族
//
// **UI 与精灵的指针拾取默认开启**（3D 网格拾取要手动加 `MeshPickingPlugin`，029 会讲）。
// 0.20 把它们都"扁平化"了 —— 没有 `Pointer<Over>` 这种泛型写法，事件本身就是类型：
//
//   `PointerOver` / `PointerOut`                  移入 / 移出
//   `PointerPress` / `PointerRelease`             按下 / 松开
//   `PointerClick`                                按下+松开落在**同一个实体**上
//   `PointerDragStart` / `PointerDrag` / `PointerDragEnd`              拖拽
//   `PointerDragEnter` / `PointerDragOver` / `PointerDragDrop` / `PointerDragLeave`  拖放
//
// 每个事件里都有 `entity`（被指到的实体）和 `pointer`（指针信息：
// `pointer.id` 是哪个指针、`pointer.position` 是指针位置 `Vec2`），
// 所以"点到哪个格子"这类问题不用自己做射线。
//
// ⚠️ 事件会**沿层级冒泡**，目标可能不是按钮本体；本讲给按钮上的文字挂了
// `Pickable::IGNORE`，让拾取稳定命中按钮，`entity` 也就不用再往上找。
//
// ─────────────────────────────────────────────────────────────────────
// 关于 `Button` 组件
//
// 0.20 的 `ui_widgets::Button` 是**无外观的 headless 控件**：只给行为，长相全靠自己拼。
// 配套的 `ButtonPlugin`（`DefaultPlugins` 里已带）替我们维护：
//   · `PointerPress` → 给实体插入 `Pressed`；`PointerRelease` → 摘掉它
//   · 松开且仍指着自己 → 发出 `Activate`（键盘回车/空格，在聚焦时也发）
//   · 还要求一个 `AccessibilityNode`，无障碍工具能认出这是按钮
//
// 另一处变化：悬停状态要自己挂 `Hovered` 组件（`Hovered::default()`），
// 拾取后端在指针进出时把它改成 `true`/`false` —— 这就是"能用变更检测轮询"的那份状态。
//
// 旧的 `bevy_ui::Interaction`（枚举）在 0.20 已标 `deprecated`，prelude 里还留着但别再用：
// 三态换成了标记组件（`Pressed`）加悬停组件（`Hovered`），本讲就是这么写的。
// ─────────────────────────────────────────────────────────────────────
