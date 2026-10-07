# bevy_study

用递进的小例子学习 **Bevy 0.19.1** 的 ECS 核心。
每个例子都是一个可以单独运行的程序，建议按编号顺序看。

- 引擎版本：`bevy = "0.19.1"`
- Rust edition：`2024`（实测工具链 `rustc 1.99.0`）
- 所有例子在 `examples/` 下，每个都能单独运行；`src/main.rs` 只是一个指针，运行它会提示去看 examples

> ⚠️ **课程正在重排。** 新版规划见 [`CURRICULUM.md`](CURRICULUM.md)：33 讲 / 7 个阶段。
> **当前进度：阶段一（001–004）已按新规划落地。**
> `examples/` 里的 **005–010 仍是旧版**，编号与内容都还没有重排，`CURRICULUM.md` 第 5 节记录了它们的去向。
> 也就是说：**001–004 看本文档下面的新说明，005–010 暂时按旧说明看**，两者编号体系不同。

---

## 运行方式

```bash
# 运行某一个例子
cargo run --example 001_app
cargo run --example 004_schedule

# 开日志跑，方便观察系统执行情况
RUST_LOG=info cargo run --example 008_message

# 只做类型检查，不改动窗口
cargo check --examples
```

首次编译 Bevy 比较慢（需要编译渲染后端），之后的增量编译很快。

> Windows / PowerShell 下设置日志用：`$env:RUST_LOG="info"; cargo run --example 008_message`

---

## 怎么读这些例子

每个例子都按「先跑起来看现象，再看代码」的顺序组织：现象写在**观察点**里，原理和踩坑写在**要点**里。
例子里的参数（速度、颜色、过滤条件）都可以直接改，行为会随之变化；把 `pressed` 改成 `just_pressed`、把 `With` 去掉，也能立刻看到 Rust 编译器和 Bevy 的报错。

---

## 课程表

| # | 文件 | 主题 | 状态 |
|---|------|------|------|
| 001 | [001_app.rs](examples/001_app.rs) | 最小 App：为什么它只跑一帧就退出 | ✅ 新规划 · 阶段一 |
| 002 | [002_window.rs](examples/002_window.rs) | 窗口与相机：窗口 ≠ 画面 | ✅ 新规划 · 阶段一 |
| 003 | [003_sprite.rs](examples/003_sprite.rs) | 精灵、坐标系与手性、层叠、必需组件 | ✅ 新规划 · 阶段一 |
| 004 | [004_schedule.rs](examples/004_schedule.rs) | **系统执行顺序与顺序歧义检测** | ✅ 新规划 · 阶段一 |
| 005 | [005_query.rs](examples/005_query.rs) | 查询过滤 | 🕗 旧版，待重排 → 新 006/007 |
| 006 | [006_input.rs](examples/006_input.rs) | 键盘输入 | 🕗 旧版，待重排 → 新 011 |
| 007 | [007_resource.rs](examples/007_resource.rs) | 全局资源 | 🕗 旧版，待重排 → 新 009 |
| 008 | [008_message.rs](examples/008_message.rs) | 消息与观察者 | 🕗 旧版，待重排 → 新 012/013 |
| 009 | [009_plugin.rs](examples/009_plugin.rs) | 插件与插件组 | 🕗 旧版，待重排 → 新 016 |
| 010 | [010_game.rs](examples/010_game.rs) | 综合小游戏 | 🕗 旧版，待重排 → 新 031 |

---

## 逐个说明 · 阶段一（新规划）

### 001_app.rs —— 最小的 App，以及一个反直觉的事实

```rust
App::new()
    .add_systems(Startup, say_hello)
    .add_systems(Update, tick)
    .run();
```

- **观察点**：程序打印三行就退出了，**没有窗口**：

  ```text
  [Startup] 整个 App 只跑这一次
  [Update] 第 1 帧
  —— .run() 已返回，进程结束 ——
  ```

  `Update` 号称「每帧跑一次」，可这里只跑了一次——**第 1 帧也是最后一帧**。

- **要点**：
  - 一个 Bevy 程序只有三件套：`App`（容器）、系统（普通函数，参数即依赖）、`Schedule`（挂在哪个时间点跑）。
  - `App::new()` 其实就是 `App::default()`，而它内部调用了 `App::empty()`，那里把 runner 设成了 `run_once`（`bevy_app-0.19.1/src/app.rs:152`）。`run_once` 的实现只调用**一次** `app.update()`。
  - 加上 `DefaultPlugins` 后，winit 会 `set_runner` 把 runner 换成事件循环，程序才会一直运行到关窗口。
  - `Local<T>` 是**系统私有状态**：只属于这一个系统，跨帧保持。
- **对照**：把 `.add_plugins(DefaultPlugins)` 的注释去掉，程序会弹窗并永远运行，末尾那行 `println!` 再也不会执行。
- **坑**：这里必须用 `println!` 而不是 `info!`。没有 `DefaultPlugins` 就没有 `LogPlugin`，没人安装 tracing 订阅者，`info!` 会被**静默丢弃**，终端上什么都看不到。

---

### 002_window.rs —— 窗口与相机

- **观察点**：弹出一个 960×640、标题为「002 · 窗口与相机」的窗口，背景是偏蓝的深色。
- **要点**：
  - `DefaultPlugins` 是引擎的功能包（窗口、渲染、输入、时间、日志……），同时把 runner 换成事件循环。
  - `DefaultPlugins.set(WindowPlugin { .. })` 用来覆盖其中某一个插件：这里设置了标题和初始分辨率。
  - `ClearColor` 是一个 **Resource**，存「默认清屏色」。
- **★ 本讲的关键结论**：相机决定的是「往窗口里渲染什么」，而不是「窗口存不存在」。
  去掉 `commands.spawn(Camera2d)` 之后，程序照常启动、窗口照常弹出、终端**不报任何错**，但窗口里什么都没有——**连 `ClearColor` 都不生效**。
  因为清屏是**相机**的渲染流程干的：`ClearColor` 只是个默认值，真正去读它、执行清屏的是相机，没有相机就没有渲染流程。
  → 所以 **窗口 ≠ 画面**。遇到「窗口是黑的但不报错」，第一反应就查相机。
- **扩展**（文件末尾有完整写法）：`WindowMode::BorderlessFullscreen(..)` 全屏；
  `Camera { clear_color: ClearColorConfig::Custom(..) }` 给单台相机换清屏色。
  注意区分 `ClearColor`（全局 Resource）与 `ClearColorConfig`（相机身上的字段）。

---

### 003_sprite.rs —— 精灵、坐标系与手性、层叠

- **观察点**：三个方块——中间红色（原始大小）、左上绿色（缩到 55% 并旋转 45°）、右下蓝色（横向拉长压扁）。
- **要点**：
  - **坐标系（本讲第一次出现 `Transform`，所以在这里讲透）**：Bevy 是**右手系 + Y 轴朝上**，2D 和 3D 共用同一套——
    **+X 向右、+Y 向上、+Z 指向你**（从屏幕里出来），2D 原点在窗口正中央。
    三根轴的方向可以用右手记：拇指 = X、食指 = Y、中指 = Z。
    与 Godot / Maya / OpenGL 一致；**与 Unity 不同，Z 轴是反的**。
    📊 [各引擎坐标轴朝向对照图](https://topkg.github.io/bevy-cheatbook/img/handedness.png)（出自 [Unofficial Bevy Cheat Book · Coordinate System](https://topkg.github.io/bevy-cheatbook/fundamentals/coords.html)）
  - ⚠️ **Bevy 有两套方向相反的「屏幕坐标」**：世界坐标（精灵 / 相机 / `Transform`）是中心原点、Y 向上；
    而 UI 坐标（`Node` / `Text` / 鼠标位置）是左上角原点、Y 向下（和网页一致）。022 讲 UI 时会再强调。
  - `z` 决定谁在前：z 越大越靠前。2D 里 z 不是「深度」，而是**层叠顺序**。
  - `scale` 是 `Vec3`，2D 只用 x、y，z 保持 1.0。
  - **必需组件**：只写 `Sprite` 也能跑，因为 `Sprite` 声明了自己需要 `Transform` 和 `Visibility`，引擎会自动补齐。这也解释了为什么你从来不用手写 `Visibility`。
  - **旋转的正方向是逆时针**：`Quat::from_rotation_z(+90°)` 把「向右」转成「向上」。
    这不是巧合，而是右手系的必然结果——按右手定则（拇指指向你、四指弯曲方向为正），绕 +Z 的正向旋转看起来就是逆时针；换成左手系，同样的代码会转成顺时针。（实测 `rot_z(+90°) · +X = +Y`）
  - **z 相同时不保证顺序**：两个精灵 z 相同时谁在前面，取决于内部排序，引擎对此不做任何承诺。要控制层叠就给不同的 z，别依赖 spawn 先后。
- **关于画圆**：本讲的 `Sprite` 只能画矩形。想画圆/多边形要走 `Mesh2d` + `ColorMaterial`（不需要图片文件），等 027 讲 3D 时对比更清楚；用贴图则在 020。

---

### 004_schedule.rs —— 系统执行顺序（本阶段最重要的一讲）

**观察点**：跑起来会看到一条 WARN，加上一份**明显算错的结算**：

```text
[new_wave]     第 1 波敌人来袭
[check_death]  血量 5 > 0，还活着              ← 先判死，看到的还是旧血量
[apply_damage] 受到 10 点伤害，剩余血量 -5      ← 后扣血，扣完其实已经死了
[report]       —— 本帧结算：血量 -5，得分 0 ——  ← 血量 -5，却一分没加

WARN bevy_ecs::schedule::schedule: Update schedule built successfully, however:
  1 pairs of systems with conflicting data access have indeterminate execution order.
  Consider adding `before`, `after`, or `ambiguous_with` relationships ...
   -- <Enable the debug feature to see the name> (in set Battle) and ... (in set Battle)
```

- **三句话版本**：
  1. 元组里并列的系统默认**并行**。互不冲突时，谁先谁后观察不到差别。
  2. 一旦两个系统访问同一份数据（一读一写、或都写），引擎会自动把它们**串行**，但**先后顺序仍然不做保证**——这叫**顺序歧义**。
  3. 需要确定顺序必须**显式声明**：`.chain()` / `.before()` / `.after()` / `SystemSet`。
- **要点**：
  - 用 `edit_schedule(Update, |s| s.set_build_settings(ScheduleBuildSettings { ambiguity_detection: LogLevel::Warn, .. }))` **主动打开歧义检测**，这是排查这类 bug 最有效的一招。
  - `#[derive(SystemSet)]` 给一组系统起名，`report.after(Battle)` 就能对**整组**声明顺序；以后往组里加系统不用改外面的声明。
  - `.before()` / `.after()` 是**传递**的：A before B、B before C ⇒ A before C。
  - 只有 `LogPlugin` 而没有 `DefaultPlugins` 时，App 仍然只跑一帧就退出（呼应 001），正好适合看这种一次性报告。
- **⚠️ 最重要的认知**：**这个错误顺序是稳定的。** 实测连跑 5 次，`check_death` 每次都排在 `apply_damage` 前面。
  Bevy 的调度在同一构建里通常表现一致，所以这类 bug **不会随机复现**——要么一直对、要么一直错，错的时候看起来还挺像「故意这么设计的」。
  危险在于它**没有任何保证**：加一个系统、调一次注册顺序，就可能翻过来。
  所以判断标准不是「我现在跑着是对的」，而是「我有没有显式声明顺序」。
- **两种修法（都已实测）**：
  - 改法 ①：`check_death.in_set(Battle).after(apply_damage)` → 得分变 `100`，WARN 消失。
  - 改法 ②：`(new_wave, apply_damage, check_death).chain().in_set(Battle)` → 效果相同。
- **看不到系统名？** Bevy 默认把名字藏起来了（那几处 `<Enable the debug feature ...>`）。在 `Cargo.toml` 里打开即可：
  ```toml
  bevy = { version = "0.19.1", features = ["debug"] }
  ```

---

## 逐个说明 · 旧版（005–010，待重排）

> 下面这些例子**仍然是旧编号下的内容**，尚未按 `CURRICULUM.md` 重写。它们的说明与代码是对应的、可以直接跑。

### 005_query.rs —— 用过滤器挑出想要的实体

- **观察点**：深色方块（Player）向右移动，红色方块（Enemy）不动；控制台每帧打印敌人坐标。
- **要点**：
  - `Query<&mut Transform, With<Player>>`：第一个参数是「取什么数据」，第二个是「过滤条件」。
  - `With<T>` = 只匹配拥有 `T` 的实体（不读取 `T` 本身）。对应的还有 `Without<T>`。
  - 同样的 `Transform` 组件，靠过滤器区分出了两种角色。

---

### 006_input.rs —— 键盘输入

- **观察点**：WASD 移动小方块；按空格打印一次 `Space Pressed`。
- **要点**：
  - `Res<ButtonInput<KeyCode>>` 读取键盘状态。
  - `pressed(..)` = 按住期间**每帧**为真；`just_pressed(..)` = 只在**按下的那一帧**为真。
    这是最容易搞混的一点：用 `pressed` 做「按一次触发一次」的事情会变成每帧触发。
  - 这里生成玩家时**没写 `Transform`**，但 `move_player` 照样能查到它——因为 `Sprite` 自动补齐了 `Transform`（required components）。

---

### 007_resource.rs —— 资源：全局唯一的数据

- **观察点**：控制台每帧打印分数，数值每帧 +100，倍率每帧 +0.1。
- **要点**：
  - `#[derive(Resource)]` + `init_resource::<Score>()` 注册全局资源。
  - `Res<Score>` 只读，`ResMut<Score>` 可写，两者**不能同时存在于一个系统**。
  - 组件描述「有哪些实体」，资源描述「整个游戏共享的状态」（分数、配置、计时器……）。
- **注意**：`add_score`（写）和 `show_score`（读）都访问 `Score`，所以调度器会把它们**串行**执行，但**谁先谁后是不确定的**——参见 004。

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

  - `.chain()` 表示「按顺序执行」。不写的话，Bevy 只在**数据冲突**时才强制串行，顺序是它自己挑的（详见 004）。
  - `shoot` 写消息、`spawn_bullet` 读消息：**写入者必须在前**，否则子弹要等下一帧才生成。
  - `check_collision` 改分、`update_score_display` 读分：顺序反了 UI 就会慢一帧。
  - 射速用 `FireCooldown(Timer)` 控制。如果用裸 `keyboard.pressed(Space)` 不加冷却，按住就是**每秒 60 发**。
  - `cleanup_bullets` 飞出屏幕就 `despawn`，避免实体无限增长。

---

## 常见坑速查

| 现象 | 原因 |
|------|------|
| 控制台疯狂刷屏 | 在系统里无条件 `println!`。系统每帧都跑，用 `is_changed()` / `Timer` / `on_timer` 节流 |
| 「按一次触发多次」 | 用了 `pressed`（按住每帧为真），应该用 `just_pressed` |
| 结果算错但不报错、且稳定复现 | **顺序歧义**：两个系统抢同一份数据却没声明先后。开 `ambiguity_detection` 查（见 004） |
| 某个系统完全没执行 | 系统参数获取失败（如资源没 `init_resource`）时，Bevy 会**静默跳过**该系统，不报错 |
| 子弹/事件晚一帧生效 | 消息是双缓冲的，读取者必须排在写入者之后（`.chain()` / `.after()`） |
| 编译报查询冲突 | 两个查询访问同一组件的读/写。用 `With` + `Without` 让编译器证明它们不相交 |
| `commands.spawn` 之后立刻查不到 | 命令是延迟执行的，要到本帧末的同步点才生效 |
| 移动速度随帧率变化 | 位移没乘 `time.delta_secs()` |
| 什么都看不见 | 场景里没有相机（2D 需要 `Camera2d`）。窗口在、程序不报错、但画面空白 |
| `info!` 什么都不打印 | 没装 `DefaultPlugins`（或 `LogPlugin`），没有 tracing 订阅者 |

---

## 后续规划

完整课程规划见 [`CURRICULUM.md`](CURRICULUM.md)。七个阶段：

| 阶段 | 讲次 | 状态 |
|------|------|------|
| 一 · 起步（App / 窗口 / 精灵 / 调度） | 001–004 | ✅ 已完成 |
| 二 · ECS 核心（组件 → 消息） | 005–012 | ⏳ 待做 |
| 三 · 事件与关系（观察者 / 层级 / 变更检测） | 013–015 | ⏳ |
| 四 · 组织与状态（插件 / 模块 / 状态机） | 016–019 | ⏳ |
| 五 · 2D 表现层（资产 / UI / 音频 / 相机 / Gizmos） | 020–026 | ⏳ |
| 六 · 3D 与渲染（3D / glTF / 拾取 / 着色器） | 027–030 | ⏳ |
| 七 · 工程质量（综合 / 测试 / 剖析发布） | 031–033 | ⏳ |

旧 10 讲的内容在新规划中**全部保留**，对应关系见 `CURRICULUM.md` 第 5 节。
