//! 014 · 父子层级
//!
//! 运行：`cargo run --example 014_hierarchy`
//!
//! 新增概念
//!   ChildOf        子实体身上的组件，指向父实体
//!   Children       父实体身上的组件，列出所有子实体
//!   children![..]  宏：生成父实体时内联声明它的子实体
//!   变换继承       父的平移/旋转/缩放会自动作用到整棵子树
//!
//! 使用场景
//!   搭结构：太阳系、角色与手持武器、UI 面板与子控件
//!   一起动、一起消失：转父节点整组跟着转，销毁父节点子树全没了
//!
//! 注意：`Transform` 是你写的**局部**变换，实际世界位置在只读的 `GlobalTransform` 里 ——
//!       别去手改 `GlobalTransform`，它每帧由引擎算出来

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "014 · 父子层级：只转父节点，整棵树跟着动".into(),
                resolution: (960, 640).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (spin, print_tree))
        .run();
}

#[derive(Component)]
struct Label(&'static str);

/// 每秒转多少弧度。挂在谁身上就转谁 —— 子实体会跟着转，那是继承的效果。
#[derive(Component)]
struct Spin(f32);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // ── 建层级的方式一：`children![..]` 宏，生成父实体时把子树一起写出来 ──
    let sun = commands
        .spawn((
            Label("太阳"),
            Spin(0.4),
            Sprite::from_color(Color::srgb(0.98, 0.78, 0.25), Vec2::splat(90.0)),
            Transform::default(),
            children![(
                Label("行星"),
                Spin(1.2),
                Sprite::from_color(Color::srgb(0.35, 0.62, 0.95), Vec2::splat(40.0)),
                // 子实体的 translation 是**相对父实体**的偏移
                Transform::from_xyz(220.0, 0.0, 0.0),
                children![(
                    Label("卫星"),
                    Sprite::from_color(Color::srgb(0.80, 0.80, 0.85), Vec2::splat(16.0)),
                    Transform::from_xyz(80.0, 0.0, 0.0),
                )],
            )],
        ))
        .id();

    // ── 建层级的方式二：各自生成，之后再挂上去 ──
    let comet = commands
        .spawn((
            Label("彗星"),
            Sprite::from_color(Color::srgb(0.36, 0.82, 0.66), Vec2::splat(22.0)),
            Transform::from_xyz(0.0, 260.0, 0.0),
        ))
        .id();
    commands.entity(sun).add_child(comet);

    println!("── 只转父节点，看子实体会不会跟着动 ──");
    println!("   太阳自转 → 行星、彗星跟着绕；行星自转 → 卫星再跟着绕");
}

/// 遍历所有带 `Spin` 的实体，各自转各自的局部 `Transform`。
///
/// 关键点：这里**没有**任何"让子实体跟着动"的代码。
/// 子实体的世界位置由引擎每帧从父到子累加算出（写进 `GlobalTransform`），
/// 所以只转父节点，整棵子树自然就动起来了。
fn spin(time: Res<Time>, mut spinners: Query<(&Spin, &mut Transform)>) {
    for (spin, mut transform) in &mut spinners {
        transform.rotate_z(spin.0 * time.delta_secs());
    }
}

/// 把实际的树结构打印一次，让"组件层面的父子关系"看得见。
fn print_tree(
    labels: Query<&Label>,
    children: Query<&Children>,
    roots: Query<Entity, (With<Label>, Without<ChildOf>)>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    *done = true;

    println!();
    println!("── 实体树（靠 Children / ChildOf 读出来）──");
    // `Without<ChildOf>` 就是"没有父实体的那些" —— 也就是根节点。
    for root in &roots {
        print_branch(root, 1, &labels, &children);
    }
}

fn print_branch(entity: Entity, depth: usize, labels: &Query<&Label>, children: &Query<&Children>) {
    let name = labels.get(entity).map(|label| label.0).unwrap_or("?");
    println!("   {}{}", "    ".repeat(depth), name);
    if let Ok(kids) = children.get(entity) {
        for child in kids {
            print_branch(*child, depth + 1, labels, children);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// 跑起来会看到什么
//
//   ── 只转父节点，看子实体会不会跟着动 ──
//      太阳自转 → 行星、彗星跟着绕；行星自转 → 卫星再跟着绕
//
//   ── 实体树（靠 Children / ChildOf 读出来）──
//          太阳
//              行星
//                  卫星
//              彗星
//
// 画面上：黄色太阳在中心，蓝色行星绕着它转、自己也在转，
// 灰色小卫星跟着行星转，绿色彗星跟着太阳转 —— **没有任何一行代码在移动子实体**。
//
// 这就验证了变换继承：`Transform` 是**局部**的，最终世界变换由引擎逐级累加。
//
// ─────────────────────────────────────────────────────────────────────
// 两个组件，一体两面
//
//   `ChildOf`   子实体身上一个，指向父实体（**多对一**）
//   `Children`  父实体身上一个，列出所有子实体（**一对多**）
//
// 引擎会在你增删层级时自动同步这两个组件，你不用手动维护。
// 想找"某个实体的父"就查 `ChildOf`，想找"它的孩子"就查 `Children`：
//
//     Query<&ChildOf>                  谁是我的父？
//     Query<&Children>                 我的孩子有谁？
//     Query<Entity, Without<ChildOf>>  哪些是根节点？（本讲找根就是这么写的）
//
// 用 `Or<(With<A>, With<B>)>` 那套过滤（007 讲的）同样适用于这两个组件。
//
// ─────────────────────────────────────────────────────────────────────
// 建层级的三种写法
//
//   1. `children![..]`         生成时内联声明，层级一眼看全 —— **最常用**
//   2. `.with_children(|p| ..)` 闭包里生成，适合子实体需要动态计算的情况
//   3. `.add_child(id)`        先各自生成、之后挂上，适合"运行中途换爹"
//
// 对应的移除写法：`commands.entity(child).remove::<ChildOf>()`（脱离父）、
// `commands.entity(parent).remove_children(&[child])`（移除指定孩子）。
//
// ─────────────────────────────────────────────────────────────────────
// 三个容易踩的点
//
// 1. **`Transform` 是局部的，`GlobalTransform` 才是世界坐标。**
//    子实体的 `Transform.translation` 是"相对父"的偏移，不是屏幕位置。
//    想知道真实位置就读 `GlobalTransform` —— 它是**只读**的，每帧由引擎算，别手改。
// 2. **销毁父实体会连带销毁整棵子树。** `despawn()` 一个节点，它的子孙全没。
//    所以别随手 despawn 一个中间节点，除非你确实要整支都清掉。
// 3. **父的缩放也会继承**，而且会**逐级相乘**。父放大 2 倍、子再放大 2 倍，
//    孙辈看起来就是 4 倍。搭层级时scale容易被无意识地叠乘，注意。
//
// ─────────────────────────────────────────────────────────────────────
// 和 013 的联系：事件冒泡
//
// `EntityEvent` 能沿 `ChildOf` 这条链**向上冒泡** —— 子实体上触发的事件，
// 它的父、祖父……也能观察到。这正是靠本讲的层级关系实现的。
// 想让某个事件冒泡，在触发时指定传播方式即可（013 末尾提过）。
// ─────────────────────────────────────────────────────────────────────
