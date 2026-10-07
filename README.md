# bevy_study

用递进的小例子学习 **Bevy 0.19.1** 的 ECS 核心。
每个例子都是一个可以单独运行的程序，建议按编号顺序看，并**亲手改一改再跑**。

> ⚠️ **课程正在重排。** 新版规划见 [`CURRICULUM.md`](./CURRICULUM.md)：33 讲 / 7 个阶段（起步 → ECS 核心 → 事件与关系 → 组织与状态 → 2D 表现层 → 3D 与渲染 → 工程质量），编号会全部重排。
> **本文档下方描述的仍是当前 `examples/` 里的旧 10 讲（旧编号）**，两者编号不对应，别混着看。旧 10 讲的内容在新规划中全部保留，对应关系见 `CURRICULUM.md` 第 5 节。

- 引擎版本：`bevy = "0.19.1"`
- Rust edition：`2024`（实测工具链 `rustc 1.99.0`）
- 所有例子在 `examples/` 下，每个都能单独运行；`src/main.rs` 只是一个指针，运行它会提示去看 examples

---

## 运行方式

```bash
# 运行某一个例子（编号就是文件名）
cargo run --example 001_hello
cargo run --example 010_game

# 开日志跑，方便观察系统执行情况
RUST_LOG=info cargo run --example 008_message

# 只做类型检查，不改动窗口
cargo check --examples
```

首次编译 Bevy 比较慢（需要编译渲染后端），之后的增量编译很快。

> Windows / PowerShell 下设置日志用：`$env:RUST_LOG="info"; cargo run --example 008_message`

---

## 学习方法

1. **先跑起来看现象**，再看代码。每个例子下面都写了「观察点」。
2. **改参数做实验**：改速度、改颜色、改过滤条件，看行为怎么变。
3. **故意改错**：把 `pressed` 改成 `just_pressed`、把 `With` 去掉，看 Rust 编译器和 Bevy 报什么错——这是理解 ECS 借用规则最快的方式。
4. **做练习**：每个例子末尾的练习题都只用到了当前及之前学过的概念。

---

## 课程表

| # | 文件 | 主题 | 关键 API |
|---|------|------|----------|
| 001 | [001_hello.rs](examples/001_hello.rs) | App / 系统 / 调度标签 | `App::new`、`add_systems(Update, ..)`、`run` |
| 002 | [002_sprite.rs](examples/002_sprite.rs) | 插件、相机、精灵 | `DefaultPlugins`、`Startup`、`Commands`、`Camera2d`、`Sprite` |
| 003 | [003_system.rs](examples/003_system.rs) | 系统参数、时间 | `Res<Time>`、`delta_secs` |
| 004 | [004_component.rs](examples/004_component.rs) | 自定义组件、实体生成 | `#[derive(Component)]`、元组 spawn、`Query<(&A, &mut B)>` |
| 005 | [005_query.rs](examples/005_query.rs) | 查询过滤 | `With<T>` |
| 006 | [006_input.rs](examples/006_input.rs) | 键盘输入 | `ButtonInput<KeyCode>`、`pressed` / `just_pressed` |
| 007 | [007_resource.rs](examples/007_resource.rs) | 全局资源 | `#[derive(Resource)]`、`init_resource`、`Res` / `ResMut` |
| 008 | [008_message.rs](examples/008_message.rs) | 消息与观察者 | `Message` / `MessageWriter` / `MessageReader`、`EntityEvent`、`On<E>`、`observe` |
| 009 | [009_plugin.rs](examples/009_plugin.rs) | 插件与插件组 | `Plugin`、`PluginGroup`、`PluginGroupBuilder`、`Without<T>`、`is_changed` |
| 010 | [010_game.rs](examples/010_game.rs) | 综合小游戏 | 上面全部 + `Timer` 冷却 + `.chain()` 顺序 + UI `Text` |

---

## 逐个说明

### 001_hello.rs —— 最小的 Bevy 程序

```rust
App::new().add_systems(Update, hello_world).run();
```

- **观察点**：程序打印一次 `hello world!` 就退出了，**没有窗口**。
  原因是这里没有加 `DefaultPlugins`，App 用的是默认 runner `run_once`——把调度跑一轮就结束。
  一旦加上 `DefaultPlugins`（见 002），runner 会被 winit 事件循环接管，变成持续运行的窗口程序。
- **要点**：`Update` 是「每帧执行」的调度标签；`App::new()` 已经带了主调度，所以 `Update` 会被执行。

**练习**：再加一个打印系统，观察两条日志的先后顺序（它**不确定**，因为两个系统没有数据冲突，可能被并行执行）；
想固定顺序就用 `.chain()`（见 008 和 010）。

---

### 002_sprite.rs —— 打开窗口，画出第一个方块

- **观察点**：窗口出现，屏幕中央有一个蓝色竖条（高 1000，比窗口还高，所以顶到了上下边缘）。
- **要点**：
  - `DefaultPlugins` 是引擎的功能包（窗口、渲染、输入、时间……）。
  - `Startup` 只在启动时跑一次，用来生成初始实体。
  - `Commands` 是**延迟命令**：`spawn` 只是把命令排队，不会立刻生效。
  - `Camera2d` 是 2D 相机；没有相机什么都看不见。
- **注意**：文件里 `vec2(100.0, 1000.0)` 是调试时留下的尺寸（高 1000），改成 `Vec2::new(100.0, 100.0)` 就是一个正方形。

**练习**：生成 3 个不同颜色、不同位置的方块；把 `Startup` 改成 `Update`，看会发生什么（每帧都生成一个）。

---

### 003_system.rs —— 系统就是一个普通函数

- **观察点**：控制台每帧打印一次两行内容，`Delta time` 在 0.016 左右浮动。
- **要点**：
  - 系统就是函数，**参数即依赖**：写 `Res<Time>` 就自动注入时间资源。
  - `(hello_system, count_system)` 是元组，一次性注册多个系统。
  - `time.delta_secs()` 是上一帧到这一帧的秒数——**所有运动都要乘它**，否则速度快慢会跟着帧率变。
- **注意**：这里没有 `DefaultPlugins` 之外的东西，但确实加了 `DefaultPlugins`，所以窗口会打开（只是没有相机，是黑的）。

**练习**：再加一个系统来打印 `time.elapsed_secs()`；
把两个打印系统改成 `add_systems(Update, print_a).add_systems(Update, print_b)`，观察顺序是否还确定。

---

### 004_component.rs —— 组件：用数据描述实体

- **观察点**：方块从左往右匀速移动（约 20 秒后移出屏幕）。
- **要点**：
  - `#[derive(Component)]` 让一个普通 struct 变成可以挂在实体上的组件。
  - 元组 spawn `(Sprite, Transform, Player, Health)` = 给这个实体一次性挂上 4 个组件。
  - `Query<(&Player, &mut Transform)>` 读 `Player`、写 `Transform`。
  - `Health` 这里挂了但没用到（`#[allow(dead_code)]`）——它演示「实体可以带任意多组件」。
  - `Sprite` 是 **required components** 的典型：挂 `Sprite` 会自动补上 `Transform` 和 `Visibility`。

**练习**：用 `Health` 做一个「血量随时间下降」的系统（需要 `Res<Time>` 和 `Query<&mut Health>`）。

---

### 005_query.rs —— 用过滤器挑出想要的实体

- **观察点**：深色方块（Player）向右移动，红色方块（Enemy）不动；控制台每帧打印敌人坐标。
- **要点**：
  - `Query<&mut Transform, With<Player>>`：第一个参数是「取什么数据」，第二个是「过滤条件」。
  - `With<T>` = 只匹配拥有 `T` 的实体（不读取 `T` 本身）。对应的还有 `Without<T>`。
  - 同样的 `Transform` 组件，靠过滤器区分出了两种角色。

**练习**：给 Enemy 也加一个移动系统；再试试 `Query<&Transform, (With<Enemy>, Without<Player>)>`。

---

### 006_input.rs —— 键盘输入

- **观察点**：WASD 移动小方块；按空格打印一次 `Space Pressed`。
- **要点**：
  - `Res<ButtonInput<KeyCode>>` 读取键盘状态。
  - `pressed(..)` = 按住期间**每帧**为真；`just_pressed(..)` = 只在**按下的那一帧**为真。
    这是最容易搞混的一点：用 `pressed` 做「按一次触发一次」的事情会变成每帧触发。
  - 这里生成玩家时**没写 `Transform`**，但 `move_player` 照样能查到它——因为 `Sprite` 自动补齐了 `Transform`（required components）。

**练习**：加一个「按住 Shift 加速」；
把 `on_space` 里的 `just_pressed` 改成 `pressed`，看看日志是怎么刷屏的——这就是「每帧触发」的坑。

---

### 007_resource.rs —— 资源：全局唯一的数据

- **观察点**：控制台每帧打印分数，数值每帧 +100，倍率每帧 +0.1。
- **要点**：
  - `#[derive(Resource)]` + `init_resource::<Score>()` 注册全局资源。
  - `Res<Score>` 只读，`ResMut<Score>` 可写，两者**不能同时存在于一个系统**。
  - 组件描述「有哪些实体」，资源描述「整个游戏共享的状态」（分数、配置、计时器……）。
- **注意**：`add_score`（写）和 `show_score`（读）都访问 `Score`，所以调度器会把它们**串行**执行，但**谁先谁后是不确定的**——这种「系统顺序」问题在 010 里用 `.chain()` 解决。

**练习**：把 `add_score` 改成每 0.5 秒才加一次分（提示：用 `Res<Time>` 自己累加，或用 `Timer`）。

---

### 008_message.rs —— 消息 vs 观察者

一个例子对比两种事件机制，**都按键盘触发，方便对比**：

| 按键 | 机制 | 特点 |
|------|------|------|
| `空格` | `Message` | 缓冲消息。写入者与读取者无需知道对方，适合「一件事发生，多个系统响应」 |
| `K` | `EntityEvent` + 观察者 | 事件带**目标实体**，只有监听该实体的观察者会收到 |

- **观察点**：按空格 → 先打印 `Sent:`，再打印 `Received: ... total = 100`。
  按 K → 先打印 `Triggered:`，**然后**才是观察者里的 `Entity died!`。
  后者的顺序体现了 `commands.trigger` 是**延迟执行**的（命令在帧末统一生效）。
- **要点**：
  - `add_message::<PlayerDied>()` 注册消息类型。
  - `MessageWriter::write` 写入，`MessageReader::read` 消费。
  - **消息是双缓冲的**：如果读取者排在写入者**之前**，本帧写进去的消息就要等下一帧才读到。
    所以 `main` 里写了 `(death_system, score_system).chain()` 明确顺序。
  - `observe` 挂在实体上，`On<EntityDied>` 里可以像普通系统一样注入 `ResMut<Score>`。

**练习**：再加一个系统也去读 `PlayerDied`，验证「一条消息可以被多个系统消费」；
然后把 `.chain()` 删掉，观察输出是否晚了一帧。

---

### 009_plugin.rs —— 插件：把功能打包

- **观察点**：WASD 控制玩家、方向键控制敌人；**玩家碰到敌人时分数 +10**，敌人镜像跳到对角位置，日志打印 `Score: 10 / 20 / ...`。
- **要点**：
  - `Plugin` 把「一组资源 + 一组系统」打包，`build()` 里注册。
  - `PluginGroup` + `PluginGroupBuilder` 把多个插件再打包成一个（`GamePlugin`）。
  - **插件之间通过共享资源协作**：`Score` 由 `ScorePlugin` 拥有，`EnemyPlugin` 里的 `award_hit` 去写它。
  - `is_changed()` 只在资源**被修改过**的那一帧返回 true，用它做日志就不会每帧刷屏。
  - `Without<Player>` 的作用：`award_hit` 里同时有「读玩家的 Transform」和「写敌人的 Transform」，
    必须让编译器能证明这两个查询**互不相交**，否则会报查询冲突。`With<Player>` + `Without<Player>` 就构成了这个证明。
- **注意**：相机是用裸 `add_systems(Startup, setup_common)` 注册的，没放进插件里——留作对比。

**练习**：把 `setup_common` 也封成一个 `CameraPlugin`；再加一个 `EnemyPlugin` 内部的重生逻辑，让敌人被打中后出现在随机位置。

---

### 010_game.rs —— 综合：一个能玩的小游戏

WASD 移动、按住空格射击、打中敌人 +100 分、分数实时显示在左上角。

- **观察点**：按住空格时子弹是**以固定射速**连续发出的，而不是每帧一颗。
- **要点（这个例子的重点其实是「系统顺序」）**：

```rust
// 输入 -> 写消息 -> 读消息建子弹
(shoot, spawn_bullet).chain(),
// 先移动子弹，再判定碰撞，最后刷新 UI
(move_bullet, check_collision, update_score_display).chain(),
```

  - `.chain()` 表示「按顺序执行」。不写的话，Bevy 只在**数据冲突**时才强制串行，顺序是它自己挑的。
  - `shoot` 写消息、`spawn_bullet` 读消息：**写入者必须在前**，否则子弹要等下一帧才生成。
  - `check_collision` 改分、`update_score_display` 读分：顺序反了 UI 就会慢一帧。
  - 射速用 `FireCooldown(Timer)` 控制。如果用裸 `keyboard.pressed(Space)` 不加冷却，按住就是**每秒 60 发**。
  - `cleanup_bullets` 飞出屏幕就 `despawn`，避免实体无限增长。

**练习**：
1. 把冷却从 0.15 秒改成 0.05 秒，感受射速变化。
2. 让敌人被消灭后在随机位置重生，做成可以一直玩的循环。
3. 加一个「敌人也会向玩家移动」的系统。
4. 用 `States` 加一个「开始 / 游戏中 / 结束」的界面切换（这是当前课程的空白，见下）。

---

## 常见坑速查

| 现象 | 原因 |
|------|------|
| 控制台疯狂刷屏 | 在系统里无条件 `println!`。系统每帧都跑，用 `is_changed()` / `Timer` / `on_timer` 节流 |
| 「按一次触发多次」 | 用了 `pressed`（按住每帧为真），应该用 `just_pressed` |
| 子弹/事件晚一帧生效 | 消息是双缓冲的，读取者必须排在写入者之后（`.chain()` / `.after()`） |
| 编译报查询冲突 | 两个查询访问同一组件的读/写。用 `With` + `Without` 让编译器证明它们不相交 |
| `commands.spawn` 之后立刻查不到 | 命令是延迟执行的，要到本帧末的同步点才生效 |
| 移动速度随帧率变化 | 位移没乘 `time.delta_secs()` |
| 什么都看不见 | 场景里没有相机（2D 需要 `Camera2d`） |

---

## 当前课程的缺口（待补）

这 10 个例子覆盖了 ECS 的核心四件套（组件 / 查询 / 资源 / 消息）与插件，但下面这些 Bevy 的主要部分**还没有例子**，是后续扩展方向：

- **States**：`States` / `OnEnter` / `OnExit`，做菜单、暂停、胜负切换
- **定时逻辑**：`Timer` 只在 010 里被当作开火冷却用过，没有专门讲解计时器的几种写法
- **系统调度进阶**：`SystemSet`、`.before()` / `.after()`、`run_if`、`on_timer`、`FixedUpdate`
- **资产系统**：`AssetServer` 加载图片 / 音频 / 字体，`Handle<T>` 的用法
- **表现层**：贴图与精灵图集、动画、UI 布局、`Gizmos` 调试绘制
- **3D**：目前全是 2D
- **工程化**：多文件模块拆分、`Result` 返回式系统与错误处理、ECS 单元测试（`app.update()` 驱动断言）
- **调试工具**：`RUST_LOG` 分级、diagnostics overlay、`bevy_dev_tools` 检查器
