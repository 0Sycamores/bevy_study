# bevy_study

用递进的小例子学习 **Bevy 0.20** 的 ECS 核心。
每个例子都是一个可以单独运行的程序，建议按编号顺序看。

- 引擎版本：`bevy = { version = "0.20", features = ["dev"] }`（`dev` 含 `debug` / `file_watcher` / `bevy_dev_tools`）
- Rust edition：`2024`（实测工具链 `rustc 1.99.0`）
- 所有例子在 `examples/` 下，每个都能单独运行；`src/main.rs` 只是一个指针，运行它会提示去看 examples

> ⚠️ **课程仍在推进中。** 完整规划见 [`CURRICULUM.md`](CURRICULUM.md)：33 讲 / 7 个阶段。
> **当前进度：阶段一~六已完成（001–030）。**

---

## 运行方式

```bash
# 运行某一个例子
cargo run --example 005_component
cargo run --example 008_commands

# 开日志跑，方便观察系统执行情况
RUST_LOG=info cargo run --example 005_component

# 只做类型检查，不改动窗口
cargo check --examples
```

首次编译 Bevy 比较慢（需要编译渲染后端），之后的增量编译很快。

> 跑某个例子时如果要在**别处**找 `assets/`，用环境变量 `BEVY_ASSET_ROOT` 指定（见 [020_asset.rs](examples/020_asset.rs)）。

---

## 怎么读这些例子

每个例子都按「先跑起来看现象，再看代码」的顺序组织：现象写在**观察点**里，原理和踩坑写在**要点**里。

打开任意一个例子文件，结构都是固定的：

- **顶部注释**：`新增概念` / `使用场景` / `注意` 三段 —— 一眼看清这讲教什么、什么时候用得上
- **中间代码**：关键处都有注释，踩坑点就地标出
- **文末注释**：实测输出、原理展开、API 速查表 —— 需要深入时再看
例子里的参数（速度、颜色、过滤条件）都可以直接改，行为会随之变化；把 `pressed` 改成 `just_pressed`、把 `With` 去掉，也能立刻看到 Bevy 的报错。

---

## 课程表（新规划）

| # | 文件 | 主题 |
|---|------|------|
| 001 | [001_app.rs](examples/001_app.rs) | 最小 App：为什么它只跑一帧就退出 |
| 002 | [002_window.rs](examples/002_window.rs) | 窗口与相机：窗口 ≠ 画面 |
| 003 | [003_sprite.rs](examples/003_sprite.rs) | 精灵、坐标系与手性、层叠、必需组件 |
| 004 | [004_schedule.rs](examples/004_schedule.rs) | **系统执行顺序与顺序歧义检测** |
| 005 | [005_component.rs](examples/005_component.rs) | 组件：挂在实体上的数据 |
| 006 | [006_query.rs](examples/006_query.rs) | 查询的几种写法 + 取不到数据会怎样 |
| 007 | [007_query_filter.rs](examples/007_query_filter.rs) | 过滤（`With`/`Without`/`Or`/`Has`）与借用冲突 |
| 008 | [008_commands.rs](examples/008_commands.rs) | 命令的延迟执行与同步点 |
| 009 | [009_resource.rs](examples/009_resource.rs) | 资源：全局一份的数据；缺资源会 panic |
| 010 | [010_time.rs](examples/010_time.rs) | 时间与计时器：帧率无关 + `Timer` 三种写法 |
| 011 | [011_input.rs](examples/011_input.rs) | 键盘鼠标输入；`pressed` 的 60 倍陷阱 |
| 012 | [012_message.rs](examples/012_message.rs) | 消息：只活两帧，顺序要自己声明 |
| 013 | [013_observer.rs](examples/013_observer.rs) | 观察者与实体事件：精准投递、链式反应 |
| 014 | [014_hierarchy.rs](examples/014_hierarchy.rs) | 父子层级：变换继承，只转父节点整棵树跟着动 |
| 015 | [015_change_detection.rs](examples/015_change_detection.rs) | 变更检测：只对"变过的数据"干活 |
| 016 | [016_plugin.rs](examples/016_plugin.rs) | 插件：把功能打包，用 `SystemSet` 声明跨插件顺序 |
| 017 | [017_module/](examples/017_module/) | 多文件工程结构（**目录式 example**） |
| 018 | [018_states.rs](examples/018_states.rs) | 状态机：`OnEnter` / `OnExit` / `in_state` |
| 019 | [019_state_advanced.rs](examples/019_state_advanced.rs) | 状态进阶：子状态、`DespawnOnExit` 自动清理 |
| 020 | [020_asset.rs](examples/020_asset.rs) | 资产加载：`load` 是异步的，先占位后替换 |
| 021 | [021_atlas_animation.rs](examples/021_atlas_animation.rs) | 图集与帧动画：一张图切 6 帧 |
| 022 | [022_ui.rs](examples/022_ui.rs) | UI 布局：flex 排版，坐标原点在左上角 |
| 023 | [023_ui_interaction.rs](examples/023_ui_interaction.rs) | UI 交互：轮询 vs 观察者对写 |
| 024 | [024_audio.rs](examples/024_audio.rs) | 音频：`Pitch` 程序合成，`AudioSink` 运行中控制 |
| 025 | [025_camera.rs](examples/025_camera.rs) | 相机：`Viewport` 分屏、正交缩放 |
| 026 | [026_gizmos.rs](examples/026_gizmos.rs) | Gizmos：把看不见的半径与速度画出来 |
| 027 | [027_3d_basic.rs](examples/027_3d_basic.rs) | 3D 基础：`Camera3d` + 网格 + 材质 + 光 |
| 028 | [028_3d_gltf.rs](examples/028_3d_gltf.rs) | 加载 glTF：场景是一棵实例化出来的实体树 |
| 029 | [029_3d_picking.rs](examples/029_3d_picking.rs) | 3D 拾取：`MeshPickingPlugin` 必须手动加 |
| 030 | [030_shader.rs](examples/030_shader.rs) | 自定义 WGSL 材质：uniform 传参 |

---

## 逐个说明 · 阶段一

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
  - `App::new()` 其实就是 `App::default()`，而它内部调用了 `App::empty()`，那里把 runner 设成了 `run_once`（`bevy_app-0.20.0/src/app.rs:192`）。`run_once` 的实现只调用**一次** `app.update()`。
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

### 004_schedule.rs —— 系统执行顺序（阶段一最重要的一讲）

**观察点**：跑起来会看到一条 WARN，加上一份**明显算错的结算**：

```text
[new_wave]     第 1 波敌人来袭
[check_death]  血量 5 > 0，还活着              ← 先判死，看到的还是旧血量
[apply_damage] 受到 10 点伤害，剩余血量 -5      ← 后扣血，扣完其实已经死了
[report]       —— 本帧结算：血量 -5，得分 0 ——  ← 血量 -5，却一分没加

WARN bevy_ecs::schedule::schedule: Update schedule built successfully, however:
  1 pairs of systems with conflicting data access have indeterminate execution order.
  Consider adding `before`, `after`, or `ambiguous_with` relationships ...
   -- check_death (in set Battle) and apply_damage (in set Battle)
      conflict on: ["004_schedule::Health"]
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
- **为什么警告里直接写了系统名？** 靠 `Cargo.toml` 里开着的 `dev`（含 `debug`）：
  ```toml
  bevy = { version = "0.20", features = ["dev"] }
  ```
  没开 `debug` 时，那几处名字会退化成 `<Enable the debug feature to see the name>` —— 警告照样发，但不告诉你是谁在冲突。

---

## 逐个说明 · 阶段二

### 005_component.rs —— 组件：挂在实体上的数据

- **观察点**：蓝色、绿色两个方块持续向右移动且**速度不同**；红色方块**原地不动**，颜色由红逐渐变暗、再回满、循环。
- **要点**：
  - **两种组件**：**标记组件**（无字段，只当标签，如 `Player` / `Enemy`）和**数据组件**（每个实体各存一份，如 `Speed(f32)` / `Health(f32)`）。
  - **实体 = 一组组件**。`spawn` 时给什么组件，这个实体就「是什么」。
  - **查询按组件组合筛选**：`Query<(&Speed, &mut Transform), With<Player>>` —— 元组是要读写的**数据**，第二个参数是**过滤器**。
  - 红方块不动就是核心证据：它的 `Transform` 组件明明也在，但没有 `Player` 标记就**不会被命中**。查询匹配的是「组件组合」，不是「某一个组件」。
  - 一个实体上同一种组件**只能有一份**；`derive(Component)` 的类型必须 `Send + Sync + 'static`。
  - 查询命中 0 个完全正常：系统照常运行、不会被跳过（对比 006 的 `Single`）。
  - 这里的 `Health` 是**真正被使用**的：跑起来能看到颜色随血量变化，而不是挂上去就没人管。
- **布局**：窗口 960×640；方块的坐标和 `Speed` 值都写在 `setup` 里，可直接改。

---

### 006_query.rs —— 查询的几种写法 + 取不到数据会怎样

- **观察点**：控制台依次打印遍历结果、`single()` 命中、`Single` 参数命中、`Populated` 计数。
  **注意「写法⑤」一行输出都没有** —— 它被静默跳过了。
- **要点**：
  - **取数方式对照**：`iter()` / `iter_mut()` / `par_iter_mut()` / `single()`（返回 `Result`）/ `get(entity)` / `iter_many()`，
    以及三个「取不到就不跑系统」的参数：`Single` / `Option<Single>` / `Populated`。
  - **取不到数据时 Bevy 的三种反应（实测，见 CURRICULUM 4.5 表）**：
    1. `Query` 命中 0 个 → 完全正常，系统照常运行，`iter().count()` 得 0；
    2. `Single` / `Option<Single>` / `Populated` 条件不满足 → **静默跳过整个系统**（不打印、不报错、不 panic）；
    3. 缺 `Res<T>`（资源压根没注册）→ **直接 panic**，报 `Resource does not exist`，并提示用 `Option<Res<T>>` 或 `If<Res<T>>`。
    一句话：**实体查询不匹配只会空手而归，资源缺失会当场炸掉。**
  - ⚠️ **遍历顺序不是生成顺序**。Bevy 按**原型(archetype)**分组存储：组件组合相同的实体放在一起，遍历是**逐组**进行的，组间先后与生成时间无关。
    所以生成顺序是「玩家A、玩家B、敌人」，打印出来敌人却排在最前面。**永远不要依赖 `Query` 的遍历顺序**——需要固定次序就收集成 `Vec` 再显式排序。
    （「按原型存储」正是 Bevy 查询快的原因：同一原型的组件在内存里连续排列，遍历时缓存友好。顺序不可依赖是它付出的代价。）

---

### 007_query_filter.rs —— 过滤与借用冲突

- **观察点**：控制台打印敌人镜像前后的坐标、`Or` 命中的 3 个战斗单位、`Has<Health>` 的逐个探测（石柱为 `false`）、`&Health` 的实际数值、`Without` 单独挑出的石柱。
- **要点**：
  - **过滤器分工**：`With<T>`（必须有 T，但**不读**数据）/ `Without<T>`（必须没有 T）/ `Or<(..)>`（任一满足即可）。
    它们写在查询的**第二个参数位置**。
  - **`Has<T>` 写在元组里**，返回 `bool`，用来顺便问一句「有没有」，且不关心具体数值。
    如果只是「有才处理」，直接写 `&Health` 就够了——没有该组件的实体根本不会命中。
  - **借用冲突**：同一系统里两个查询都碰 `Transform`（一个读、一个写）时会冲突。
  - ⚠️ **这个冲突是运行时 panic（错误码 `B0001`），不是编译错误** —— `cargo check` 照样通过，一运行才炸：
    ```text
    error[B0001]: Query<...> in system ... accesses component(s) ... in a way
    that conflicts with a previous system parameter.
    Consider using `Without<T>` to create disjoint Queries or merging
    conflicting Queries into a `ParamSet`.
    ```
    报错信息自己给了两种修法：① `Without<T>` 造互斥查询（本讲用的）；② `ParamSet` 把冲突查询合并，同一时刻只借出一种。
  - **为什么 Bevy 不认「数学上的不相交」**：一个实体可以同时拥有 `Player` 和 `Enemy`（比如被策反的敌人），所以 `With<Player>` 与 `With<Enemy>` 本身并不互斥。只有显式写 `Without<Player>`，它才敢认定两者不相交。
  - `Without` 在本讲出现两次：一次是**为了让代码能跑**（消除冲突），一次是**纯粹当过滤条件**（挑出石柱）。

---

### 008_commands.rs —— 命令的延迟执行

- **观察点**：六步流水线依次打印。核心是第一步里的 `0 → 0` —— 排队了 3 个 `spawn`，立刻再查仍然是 0 个。
- **要点**：
  - `Commands` 内部是一份**命令队列**：`spawn` / `insert` / `remove` / `despawn` 全都只是写进队列，到**同步点**才被逐条执行。
  - **同步点有两个来源**：
    1. 一个 Schedule 跑完时（整个 `Update` 结束时、`Startup` 结束时……）；
    2. **排序**把「带延迟参数的系统」排在「会读相关数据的系统」之前时，自动插入一个 `ApplyDeferred`。
       这由 `ScheduleBuildSettings::auto_insert_apply_deferred` 控制，**默认就是 `true`**。
  - **实测对比**：把 `.chain()` 去掉（写成 `(spawn_enemies, count_enemies)`），`count_enemies` 会打印 **0 个** —— 不声明顺序，命令什么时候生效就不确定，又是 004 讲的顺序问题。
  - **控制同步点三法**：`.chain()`（排序 + 自动插同步点）、`.chain_ignore_deferred()`（只排序、**不**插）、显式写 `ApplyDeferred`：
    `(spawn_enemies, ApplyDeferred, count_enemies).chain()`。
  - **常用操作**：`commands.spawn(..)` / `commands.entity(e).insert(..)` / `.remove::<T>()` / `.despawn()` / `commands.get_entity(e)`（返回 `Option`，实体已不存在时是 `None` 而不是 panic）。
  - **回看 005**：`Startup` 里 spawn 的东西 `Update` 能查到，正是因为 Startup 这个 Schedule 跑完时产生了一次同步点。这不是特例，正是上面第 1 条规则在起作用。

---

### 009_resource.rs —— 资源：全局一份的数据

- **观察点**：控制台依次打印加分、乘倍率、最终得分；`report_optional` 报告一个**没注册**的资源；`skip_if_missing` 一行输出都没有。
- **要点**：
  - **组件 vs 资源**的判据就一句：这个数据是「每个实体各有一份」（组件），还是「全局只有一份」（资源）。敌人各自的血量是组件，玩家总分是资源。
  - `init_resource::<T>()` 要求 `T: Default`；`insert_resource(v)` 自己给值。
  - **实测差异**：`init_resource` 在资源**已存在时不会覆盖**（先 `insert(42)` 再 `init` 仍是 42）；`insert_resource` 会覆盖。所以「重置一个资源」要用后者。
  - ⚠️ **缺 `Res<T>` 会直接 panic**，不是静默跳过。三种应对：`Option<Res<T>>`（自己处理 `None`）、`run_if(resource_exists::<T>)`（不存在就整系统跳过）、`If<Res<T>>`。
  - 对比 006：`Query<&T>` 命中 0 个**永远**不会 panic。**资源缺失会炸，实体查询不会** —— 这个不对称要记牢。
  - `gain_score` 和 `award_bonus` 都要 `ResMut<Score>`，会串行但顺序不保证（004 讲的），所以显式写了 `.after()`。

---

### 010_time.rs —— 时间与计时器

- **观察点**：分两段。第一段逐帧打印"时间现场"（5 帧，输出稳定可复核）；第二段演示暂停与倍速：

  ```text
  ── 第 3 帧
       【写法① 手动累加】触发（结转 0.05s）
       【写法② Timer 资源】触发
       【写法③ Timer 组件】组件A 触发
       Time      delta 0.10s   elapsed 0.30s
       Repeating elapsed 0.05/0.25s   remaining 0.20s   fraction  20%   finished true
       Once      elapsed 0.30/0.35s   remaining 0.05s   fraction  86%   finished false
       Stopwatch elapsed 0.20s

  ── 正常      Time.delta  100.3ms   Real.delta  100.3ms   Virtual.delta  100.3ms   paused=false  speed=1
  ── 暂停      Time.delta    0.0ms   Real.delta  100.3ms   Virtual.delta    0.0ms   paused=true   speed=1
  ── 3 倍速    Time.delta  302.0ms   Real.delta  100.7ms   Virtual.delta  302.0ms   paused=false  speed=3
  ```
- **要点**：
  - **`Timer` 的三种驱动写法**在同一讲里对照，第 3 帧三种**同时触发**（说明等价），选哪种看用途：
    手动累加（最灵活、也最容易写错，必须**结转**超出部分）/ `Timer` 资源（全局唯一的定时）/ `Timer` 组件（每个实体各自节奏）。
  - `Timer` 的状态全都能读：`elapsed` / `remaining` / `fraction` / `duration` / `is_finished`；另有 `pause` / `unpause` / `reset` / `set_duration`。
  - ⚠️ **`is_finished()` 对 `Once` 到点后每帧都为真**（第 4 帧起一直是 `true`），拿它做「触发一次」会变成每帧触发，要「刚刚到点」必须用 `just_finished()`。
    另外注意 Repeating 在**完成的那一帧** `finished` 也是 `true`，但 `elapsed` 已经绕回（看第 3、5 帧）。
  - **`Res<Time>` 拿到的就是虚拟时钟**：暂停时 `Time = Virtual = 0` 而 **`Real` 照走**；倍速则 `Time = Virtual ≈ 3×Real`。做暂停菜单改 `Time<Virtual>` 就够了。
  - `Stopwatch` 只计时、永远不 finished：`pause()` 冻结、`unpause()` 从原处继续、`reset()` 归零。用它做冷却显示、计时赛、性能统计。
  - 本讲**只讲时间本身**，不碰移动、射击之类的游戏逻辑。第一段**不装** `TimePlugin`（这样 `advance_by` 才能精确控时），第二段**必须装**（`Time<Virtual>` 由它提供）—— 所以分成两个 App 跑，原因写在例子注释里。

---

### 011_input.rs —— 键盘与鼠标输入

- **操作**：移动鼠标看方块跟随；按住左键变色；按空格观察计数差别；按 `?` 或 `+` 体验 `Key` 与 `KeyCode`。
- **观察点**：每秒一行汇总。按住空格不放时：

  ```text
  【每秒】空格 just_pressed 1 次 / pressed 60 次   鼠标移动 (+12.0, -3.0)   滚轮 +0.0   光标 屏幕 (480, 320) → 世界 (0, 0)
  【每秒】空格 just_pressed 0 次 / pressed 61 次   ...
  【每秒】空格 just_pressed 0 次 / pressed 59 次   ...
  ```
- **要点**：
  - 输入来自**五个资源、两类形态**：
    `ButtonInput<KeyCode>`（按物理位置）/ `ButtonInput<Key>`（按实际字符）/ `ButtonInput<MouseButton>` / `ButtonInput<GamepadButton>`，
    以及 `AccumulatedMouseMotion` / `AccumulatedMouseScroll`。
  - **离散 vs 连续**：按键用 `ButtonInput` 查「这一刻的状态」；鼠标移动和滚轮用 `Accumulated*` 拿「这一帧的增量」，得自己累加。
  - **按下的三种语义**（本讲核心）：`pressed` 按住期间每帧为真 / `just_pressed` 只在按下那一帧 / `just_released` 只在松开那一帧。
  - 用固定帧长精确复现了一遍（按住 60 帧 × 每帧 1/60 秒）：`just_pressed` 触发 **1** 次、`pressed` 触发 **60** 次 —— 那个 **60 就是帧率**。
    用 `pressed` 做「按一次做一件事」，等于让帧率决定游戏平衡：不崩溃、不报警，只是"数值不对"，属于 004 讲的同一类隐性错误。
  - `KeyCode` vs `Key`：`KeyCode::KeyW` 指"W 那个位置"（**移动键绑定用它**，手感才一致）；
    `Key::Character("?".into())` 指"打出问号"（**符号快捷键用它**，问号在美式/德语键盘上位置不同，但字符不变）。
  - 光标位置 `window.cursor_position()` 给的是**屏幕坐标**（左上角原点、Y 向下），再经 `Camera::viewport_to_world_2d` 换算成世界坐标 —— 这正是 003 讲过的两套坐标**正面相遇**的地方。
  - `Time` / `Timer` 在本讲只是**配角**（把刷屏的输入压成每秒一行），细节在 010；相机本身的玩法（跟随、缩放、分屏）在 025。

---

### 012_message.rs —— 消息

- **观察点**：手动逐帧推进 4 帧，每条输出都标出帧号。同帧、同类型的两个读者，读到的却不一样：

  ```text
  ═══ 第 2 帧 ═══
     [晚一帧读者] 读到 [1]     ← 上一帧那条
     [写] Heartbeat(2)
     [有序读者]   读到 [2]     ← 本帧那条
  ```
- **要点**：
  - ⚠️ **Bevy 不会自动把 `MessageWriter` 排在 `MessageReader` 之前。** 不声明顺序，读者就可能排在写者前面，表现就是消息**晚一帧**生效。必须 `.chain()` / `.after()` 明确顺序。
  - **消息只活两帧**（双缓冲）：写在第 N 帧 → 第 N、N+1 帧可读 → 第 N+2 帧消失。没人读就自己丢弃，不会无限堆积。
  - 所以消息适合**一次性通知**（受伤、死亡、得分、播音效），不适合长期状态（当前血量、是否暂停）—— 那些该用组件或资源。
  - 三种角色：`MessageWriter`（只写）/ `MessageReader`（只读，**必须声明 `mut`**，它要跨帧记住读到哪儿）/ `MessageMutator`（同类型既读又写 —— 因为同类型的 Reader + Writer 会冲突）。
  - **每个读者各自独立记录进度**，一条消息可以被任意多个系统消费。
  - 消息是**全局广播**（谁都能读）；013 的观察者则是**精准投递到某个实体**。

---

### 013_observer.rs —— 观察者与实体事件

- **观察点**：两帧的完整输出，四个结论都能直接读出来：

  ```text
  ═══ 第 1 帧 ═══
     [生命周期] 有实体被加上了 Health（10v0）        ← 自动触发，没写 trigger
     [触发] 对 敌人A 造成 6 点伤害
     [全局观察者] 敌人A 被击中，伤害 6
     [全局观察者] 敌人A 剩余血量 4
     [专属观察者] 这是敌人A 自己的观察者收到了 Hit     ← 只有 A 有专属观察者
     [全局观察者] 敌人B 被击中，伤害 6                ← B 这次没有专属观察者
  ═══ 第 2 帧 ═══
     [全局观察者] 敌人A 剩余血量 -2
     [观察者] 敌人A 阵亡，销毁实体                    ← 血量归零后链式触发
  ```
- **要点**：
  - **四种投递方式**：全局观察者（`add_observer`）、实体专属观察者（`spawn(..).observe(..)`）、**生命周期事件**（`On<Add, T>` / `On<Remove, T>`，组件增删时自动触发，不用手写 `trigger`）、以及观察者里再 `trigger` 形成的**链式反应**。
  - `EntityEvent` 存在的意义就是"点名"——事件带着目标 `entity`，才能精准投递。不带目标的纯广播事件用 `#[derive(Event)]`。
  - **精准投递的验证**：三个敌人都被全局观察者看到，但 `[专属观察者]` 只在**敌人A** 被击中时出现；**敌人C 全程没出现**（压根没对它 trigger）。
  - **链式反应**：`击中 → 掉血 → 阵亡 → 销毁` 整条链自动跑完，每段的双方互不认识，只靠事件串起来。
  - ⚠️ `commands.trigger` 和别的命令一样是**延迟**的（008 讲的），观察者要到**同步点**才跑 —— 输出里 `[触发]` 那两行总是排在观察者输出之前。
  - ⚠️ 观察者之间的执行顺序**不做保证**，别写依赖顺序的逻辑（又是 004 那类问题）。
  - 观察者**就是系统**：可以用 `Query` / `Res` / `Commands`，也能带 `run_if` 运行条件。

---

### 014_hierarchy.rs —— 父子层级

- **观察点**：窗口里黄色太阳在中心，蓝色行星绕它转且自转，灰色小卫星跟着行星转，绿色彗星跟着太阳转 —— **没有任何一行代码在移动子实体**。控制台还会打印一次树：

  ```text
  ── 实体树（靠 Children / ChildOf 读出来）──
       太阳
           行星
               卫星
           彗星
  ```
- **要点**：
  - **两个组件一体两面**：`ChildOf`（子实体身上，指向父）与 `Children`（父实体身上，列出所有子）。引擎在你增删层级时自动同步，不用手动维护。
  - **`Transform` 是局部的**：子实体的 `translation` 是"相对父"的偏移，不是屏幕位置。真实世界坐标在**只读**的 `GlobalTransform` 里，每帧由引擎逐级累加算出 —— 别手改它。
  - 所以"只转父节点、整棵子树跟着动"是自动发生的，这就是**变换继承**。
  - **建层级三种写法**：`children![..]` 宏（最常用）/ `.with_children(..)` / `.add_child(id)`（先各自生成、之后挂上，适合中途换爹）。本讲前两种都用了。
  - 找根节点的小技巧：`Query<Entity, (With<Label>, Without<ChildOf>)>` —— 没有父的就是根。
  - ⚠️ **销毁父实体会连带销毁整棵子树**，所以别随手 despawn 中间节点。
  - ⚠️ **父的缩放会逐级相乘**：父 2×、子 2×，孙辈看起来就是 4×。搭层级时容易无意识叠乘。
  - 013 末尾提过的**事件冒泡**，就是沿这条 `ChildOf` 链向上传播的。

---

### 015_change_detection.rs —— 变更检测

- **观察点**：盯着**敌人B** 那一行走势：

  ```text
  ═══ 第 1 帧 ═══   [Changed] 敌人A 值=100     [Changed] 敌人B 值=100
  ═══ 第 2 帧 ═══   [Changed] 敌人A 值=90      [Changed] 敌人B 值=100   ← A 真扣了血
  ═══ 第 3 帧 ═══                              [Changed] 敌人B 值=100   ← 只有 B 在"谎报"
  ═══ 第 4 帧 ═══   [Changed] 敌人A 值=80      [Changed] 敌人B 值=100
  ═══ 第 5 帧 ═══   （什么都没有）
  ```
- **要点**：
  - ⚠️ **`Changed<T>` 说的是「这一帧有人可变访问过 T」，不是「值真的不同」。**
    敌人B 的值从头到尾都是 100、一次都没变，却**每帧都被 `Changed` 命中**；而敌人A 只在真正扣血的那两帧出现。
  - 原因是 `rewrite_b_unchanged` 里那句 `health.0 = health.0.clamp(0, 100)` —— 写进去的值一样，但已经发生了可变借用（`DerefMut`）。现实里"无脑 clamp / 无脑重新赋值"很常见，于是血条被每帧重画。
  - 想表达"值真的不同才处理"，写组件时用 **`set_if_neq`**（需要组件 derive `PartialEq`）。
  - **四种写法**：`Changed<T>` / `Added<T>` 是**过滤器**（会筛掉实体）；`Ref<T>` 与 `Mut<T>` 不改筛选，但能问 `is_added()` / `is_changed()` / `last_changed()`；资源用 `Res::is_changed()`；移除用 `RemovedComponents<T>`（和消息同源的缓冲，多个系统都能读到同一次移除）。
  - **`Added` 是 `Changed` 的子集**：组件刚加上那一帧两者都命中（输出第 1 帧可见）。
  - **变更检测按系统各记各的进度**：同一次变更不会被谁"读掉"，所以多个系统都能看到。
  - 只在**活够重**时才划算：重建 UI 布局、重算寻路值得；`x += 1` 这种本来就极便宜，过滤器开销反而占比更大。

---

### 016_plugin.rs —— 插件

- **观察点**：三帧输出，分数只在敌人结算那一帧更新：

  ```text
  ═══ 第 1 帧 ═══
     [分数插件] Score = 0
  ═══ 第 2 帧 ═══
  ═══ 第 3 帧 ═══
     [敌人插件] 小兵甲 抵达终点，+10 分
     [敌人插件] 小兵乙 抵达终点，+10 分
     [分数插件] Score = 20
  ```
- **要点**：
  - `Plugin::build` 里能做的，就是你在 `main` 里能做的一切 —— **插件不是新概念，只是把一段配置搬了个家**（从"堆在 main 里"变成"归属于某个功能模块"）。
  - **`SystemSet` 是插件的必要配套**：插件之间同样要声明顺序。本讲那句 `GameSet::Score.after(GameSet::Enemy)` 是集中声明的跨插件约束；**不写的话，那一帧加的 `Score = 20` 永远不会被打印出来**（实测确实会丢）。
  - 插件之间靠**共享资源 / 消息 / 事件**协作，不是互相调用 —— 代价是"谁拥有 `Score`"成了需要约定的问题，所以要在注释里写清归属。
  - `PluginGroup` + `PluginGroupBuilder` 把多个插件打包；`DefaultPlugins.set(..)` / `.disable::<T>()` / `.add_before::<T>(..)` 用来定制引擎自带的插件组。
  - 拆插件的判断标准：**如果你要为这组东西起个名词（"敌人"、"分数"、"音频"），它多半就该是个插件。**
  - 第 1 帧那行 `Score = 0` 是"资源刚创建也算变过"（015 讲的 `Added` 是 `Changed` 的子集）。
  - 一句话概括 `main`：**只剩一张插件清单。**

---

### 017_module —— 多文件工程结构

- **形式**：本讲是**目录式 example** —— `examples/017_module/` 下有 `main.rs` + `common.rs` + `score.rs` + `enemy.rs`。
  Cargo 会自动把 `examples/<名字>/main.rs` 识别成名为 `<名字>` 的 example，而**子模块文件不会**被当成独立 example。
- **观察点**：输出与 016 **逐字相同**。这就是本讲要的效果：

  ```text
  ═══ 第 3 帧 ═══
     [敌人插件] 小兵甲 抵达终点，+10 分
     [敌人插件] 小兵乙 抵达终点，+10 分
     [分数插件] Score = 20
  ```
- **要点**：
  - `main.rs` 从"两百行实现"变成"三十行声明"——只做两件事：**装插件** + 声明**跨插件**的顺序约束。
  - 三条值得照抄的约定：
    1. **一个功能一个文件**，文件里就三样：专属资源 / 专属组件 / 那个插件。
    2. **只 `pub` 出插件**，系统函数保持私有（本讲里 `report_score`、`check_arrival` 都没有 `pub`）。既是封装，也让"谁会用到这个函数"有唯一答案。
    3. **共享的东西才进 `common`**，否则它会退化成新的"巨型 main"。
  - ⚠️ **`mod` 与 `Plugin` 不是一回事**：`mod` 是 Rust 的模块系统（代码放哪、谁能看见），`Plugin` 是 Bevy 的组织方式（什么被注册进 App）。两者常一一对应，但**没有绑定关系**。
  - ⚠️ `examples/` 下每个 example 都是**独立 crate**，所以 `use crate::common::..` 指的是**本 example 的** crate 根，与顶层 `src/` 无关。

---

### 018_states.rs —— 状态机

- **观察点**：状态每 1.5 秒自动推进一圈，控制台打印进入/离开：

  ```text
  [OnEnter] Menu     背景变深蓝          ← 启动时也会跑一次
  ── 请求切换到 Playing
     [OnExit ] Menu
     [OnEnter] Playing  背景变深绿，方块开始旋转
  ── 请求切换到 GameOver
     [OnExit ] Playing
     [OnEnter] GameOver 背景变暗红，方块停转
  ```
- **要点**：
  - ⚠️ **`set()` 是请求，不是执行**：切换发生在**帧末**的 `StateTransition` 调度里。所以同一帧内 `State<S>` 还是旧值，且 `OnExit(旧)` 一定排在 `OnEnter(新)` 之前。连调两次 `set()` 只有最后一次算数。
  - ⚠️ **`OnEnter` / `OnExit` 只跑一次**（切换那一帧），不是"该状态下每帧"。每帧逻辑要用 `Update` + `in_state(..)`；用错的典型症状是"我的初始化只跑了一次"或"我的每帧逻辑压根没跑"。
  - **`init_state` 注册完也会触发一次 `OnEnter`**，哪怕什么都没切过去 —— 初始状态的准备工作可以放心放进去。
  - **状态是资源，不是组件**：全 App 一份 `State<AppState>`，描述的是"世界整体的阶段"；实体的状态该用组件（配合 015 的 `Changed`）。
  - **状态 vs 布尔标志**：`States` 自带进出调度、现成的运行条件、以及引擎主动做好的变化检测。只要这个"阶段"**进出时有事要做**，就该用它。
  - 画面：背景色随状态变化，黄色方块**只在 `Playing` 那 1.5 秒里转** —— 那就是 `in_state` 生效的肉眼证据。

---

### 019_state_advanced.rs —— 状态进阶

- **观察点**：6 帧输出，三列走势把两件事一起讲清：

  ```text
  ═══ 第 1 帧 ═══   AppState=Menu     IsPaused=不存在   Progress=0  场景 ["菜单背景板"]
  ═══ 第 2 帧 ═══   AppState=Menu     IsPaused=不存在   Progress=0  场景 ["菜单背景板"]
  ═══ 第 3 帧 ═══   AppState=Playing  IsPaused=Running  Progress=1  场景 [背景板+道具甲乙丙]
  ═══ 第 4 帧 ═══   AppState=Playing  IsPaused=Paused   Progress=1  场景 [同上]
  ═══ 第 5 帧 ═══   AppState=Playing  IsPaused=Running  Progress=2  场景 [同上]
  ═══ 第 6 帧 ═══   AppState=Menu     IsPaused=不存在   Progress=2  场景 ["菜单背景板"]
  ```
- **要点**：
  - **`SubStates` 随父状态一起出现和消失**：Menu 阶段 `IsPaused` 显示 `不存在` —— 不是"值为空"，而是**整个状态没被创建**。它表达的正是"菜单里根本没有'是否暂停'这个概念"。
    ⚠️ 所以读子状态必须用 `Option<Res<State<子状态>>>`；直接写 `Res<State<..>>` 在父状态不满足时会 panic。
  - **暂停时逻辑真的停**：`Progress` 在 `Paused` 那一帧**原地不动**（被 `in_state(IsPaused::Running)` 挡住了）。`in_state` 用在子状态上和用在父状态上一样好使。
  - **`DespawnOnExit(Playing)` 自动清理**：第 6 帧离开 `Playing` 时 3 个道具**自动消失，全程没人写过清理代码**。它把生命周期写在实体自己身上，不用维护一个"哪些实体该清"的集中查询（漏一个就泄漏一个）。
    兄弟：`DespawnOnEnter(S)`（进入时销毁，适合清上轮残留）、`DespawnWhen::new(|transition| ..)`（自定义判断）。重复挂也不会报错。
  - **`ComputedStates`** 由别的状态**推导**（`type SourceStates` + `fn compute`），没有自己的 `NextState`、不能直接切。适合"好几个状态下跑同一批逻辑"。本讲只在文末介绍，没现场演示。
  - ⚠️ **时序**：第 2 帧请求切到 `Playing`，状态到第 3 帧才变；第 5 帧请求切回 `Menu`，第 6 帧才生效。原因是切换在帧末发生，且 `OnEnter` 里的 `commands.spawn` 还要再等一个同步点 —— 所以本讲用了 6 帧而不是 5 帧，否则看不到自动清理。
  - ⚠️ **状态机需要 `StatesPlugin`**：`DefaultPlugins` 自带它，而本讲只装了 `LogPlugin`，所以显式加上了 —— 忘了会在 `init_state` 处直接 panic。

---

### 020_asset.rs —— 资产加载

- **观察点**：三个精灵刻意做对照 —— 左=拿到句柄直接用、右=先占位后替换、下=加载一个**不存在**的路径：

  ```text
  ── assets.load("textures/logo.png") 已返回句柄（内容此刻还没到）
     第 1 帧  LoadState = Some(Loaded)
     第 1 帧：内容到位，右边从占位方块换成真图
     [AssetEvent] Added    —— 图片加载完成
  ```
- **要点**：
  - **`load` 是异步的，返回的只是"取货凭证"（`Handle`）**。判断内容到了没有：`Assets::get(handle)` 为 `None` 就是还没到；`AssetServer::is_loaded_with_dependencies(..)` 连依赖一起算；`get_load_state(..)` 想知道卡在哪一步。
  - ⚠️ **`LoadState::Failed` 不会 panic**，只是永远不 `Loaded` —— 最典型的表现是"东西一直不出现，程序也不报错"。本讲第三个精灵就是这种情况（终端只会有一行 ERROR）。
  - ⚠️ **本机实测第 1 帧就 `Loaded`**（1.7 KB 小图 + SSD，读盘比第一帧还快），所以"Loading"那段没被观察到。但这**不是省掉判断的理由** —— 换大模型/机械盘/Web 端，等待就是几帧甚至几秒。
  - ⚠️ **`assets/` 的根按顺序确定**：`BEVY_ASSET_ROOT` → `CARGO_MANIFEST_DIR`（`cargo run` 自动设置）→ **可执行文件所在目录**。所以直接双击 exe 跑会去 `target/debug/examples/assets/` 找，全找不到。
  - ⚠️ **热重载要 `file_watcher` feature，它不在默认里**（默认只有 `2d/3d/ui/audio`），而在 `dev` 组：`dev = ["debug", "bevy_dev_tools", "file_watcher"]`。本项目没开，实测改 PNG 等 10 秒也没有 `Modified`。

---

### 021_atlas_animation.rs —— 图集与帧动画

- **观察点**：三个小人并排 —— 甲 6fps、乙 12fps、丙定在第 4 帧不动。控制台打印跑圈：

  ```text
  ── textures/runner.png 是 288x48，按 48x48 切成 6 帧
     乙 12fps 跑完第 1 圈（6 帧）
     甲 6fps 跑完第 1 圈（6 帧）
  ```
- **要点**：
  - **`TextureAtlasLayout` 管"怎么切"，`TextureAtlas.index` 管"用第几格"**。前者可共享（克隆句柄即可），后者**每个精灵各一份** —— 所以三个精灵共用一张图和一份布局，却各演各的。
  - `TextureAtlasLayout::from_grid(每格尺寸, 列数, 行数, padding, offset)` 存进 `Assets<TextureAtlasLayout>`。
  - 动画逻辑只有几行：`Timer` 到点 → `index + 1` → 到末尾回 0（复用 010 的 `Timer`）。
  - ⚠️ **像素图必须加 `ImagePlugin::default_nearest()`**，否则线性插值会把硬边糊成渐变。
  - `Sprite::from_atlas_image(..)` 建出来的精灵，图集信息挂在 `sprite.texture_atlas`（是 `Option`）—— 所以帧动画精灵和普通精灵是同一个组件类型。

---

### 022_ui.rs —— UI 布局

- **观察点**：左上角一张图片、右上角文字「HP xx」、底部中间一条血条（每 0.4 秒变化一次）。
- **要点**：
  - ★ **UI 是另一套坐标系**：原点在**左上角**、Y 轴**向下**、单位是**逻辑像素**。跟 003 的精灵世界坐标（中心原点、Y 向上）正好相反。
  - UI 是**排版**不是摆坐标：`Node` 用 flexbox（`flex_direction` / `justify_content` / `align_items` / `padding` / `row_gap`），本讲的血条就是用 `Column + FlexEnd + Center` 推到"底部居中"，**没有一个坐标是手算的**。
  - `position_type: Absolute` 可以让元素脱离排版流、回到"按 `top/left/right/bottom` 钉死"（本讲的图片与文字就是）。
  - ⚠️ **中文字体**：Bevy 默认字体是 `FiraMono-subset.ttf`（拉丁字母），**不含汉字字形**。直接 `Text::new("血量")` 会刷 `ICU4X data error: No segmentation model for complex script` 而且画面上出不来字。必须自带中文字体（系统字体或放进 `assets/fonts/`）。
    注意区分：**`println!` 的字由终端画，`Text` 的字由 Bevy 画** —— 前者什么语言都行。
  - `ZIndex` 管 UI 内部层叠，和精灵的 `Transform.z` 是两套体系；UI 本身始终渲染在世界之上。
  - 别每帧无脑刷新 UI —— 用 015 的 `is_changed()`（本讲的 `update_hud` 就是这么写的）。

---

### 023_ui_interaction.rs —— UI 交互

- **观察点**：两个按钮外观与行为一致，但实现路子完全不同 —— 左边轮询、右边观察者。控制台分别打印：

  ```text
  [轮询]   按下 → 累计 1 次
  [观察者] 点到实体 7v0 → 累计 1 次
  ```
- **要点**：

  | | `Changed<Interaction>` 轮询 | `On<Pointer<..>>` 观察者 |
  |---|---|---|
  | 系统数量 | **1 个**就管完外观三态 + 计数 | 外观三态要 **4~5 个**观察者 |
  | 开销 | 每帧遍历所有按钮（有 `Changed` 过滤） | 只在该实体出事时跑 |
  | 能拿到持续状态 | ✅ `Hovered` / `Pressed` | ❌ 只有事件 |
  | 语义 | "按下"（不含松开） | "点击"（按下+松开同实体） |

  经验：**外观用轮询、动作触发用观察者**，混用没问题。
  - ⚠️ **别拿 `Interaction::Pressed` 当"点击"** —— 它按住期间**持续为真**（和 011 的 `pressed` 一样）。本讲用 `Changed<Interaction>` 过滤掉了重复帧。
  - ⚠️ `observe(..)` **不是 bundle**，不能写进 `children![..]` —— 它是 `EntityCommands` 的方法，要先 `spawn` 再挂（本讲的观察者按钮就是这么建的）。
  - **UI 与精灵的指针拾取默认开启**；3D 网格拾取才需要手动加 `MeshPickingPlugin`（029 讲）。

---

### 024_audio.rs —— 音频

- **观察点**：一进画面就有 0.5 秒一拍的琶音；空格加一声音效；M 立刻静音/恢复，并打印改了几个正在播放的声音。
- **要点**：
  - **两个组件分工**：`PlaybackSettings` 是**开播参数**（开始后改它无效）；`AudioSink` 是**播放句柄**（运行中调音量/暂停/停止靠它）。本讲的静音开关两样都用 —— 只改资源，新音符会漏出来；只改 sink，下一个音符又用旧音量。
  - `PlaybackMode`：`Once` / `Loop`（BGM）/ `Despawn`（音效，播完自动销毁实体）/ `Remove`。
  - ⚠️ `Once` 播完后 **`AudioPlayer` 不能复用**，要重播得摘掉组件再加回去。所以短音效的标准做法是**每次 spawn 一个临时实体**（本讲就是），天然支持多声重叠。
  - ⚠️ **默认只支持 OGG，不支持 WAV**：`audio = ["bevy_audio", "vorbis"]`。拿 `.wav` 去播会在解码处**直接 panic**（`audio_source.rs:101`）。要 WAV 得开 `wav` feature（引入 `hound`），全套格式用 `audio-all-formats`。
  - 本讲因此改用 **`Pitch` 程序合成音**（给频率 + 时长就有声音，不需要音频文件、不需要额外 feature）。注意两种构造方式的区别：文件走 `AudioPlayer::new(handle)`，`Pitch` 走 `AudioPlayer(handle)`。

---

### 025_camera.rs —— 相机

- **观察点**：窗口左右两半**显示同一份世界** —— 左相机 `scale=1.0`，右相机 `scale=2.0`（视野宽两倍，东西看起来小一半）。滚轮调整左相机，左上角文字实时显示 scale。
- **要点**：
  - ★ **`scale` 是"视野"的倍数，不是"物体"的倍数**：`scale` 越大 → 视野越广 → 东西越**小**。直观理解是"相机往后退了几倍"，所以想放大画面要**减小** scale。
  - **`Viewport { physical_position, physical_size }`** 把相机输出限制在屏幕一块矩形里 —— 分屏就靠它。注意单位是**物理像素**，写死 `480` 只在 960 宽窗口下正确，真实项目该按 `window.physical_width()` 算。
  - 每台相机都要有 `Camera2d`（或 `Camera3d`），`Camera` 组件只是配置；多相机同屏用 `Camera.order` 决定绘制顺序。
  - **跟随的两种做法**：① 把相机挂成目标的**子实体**（014 的层级，一行搞定但生硬）；② 每帧**插值逼近**（有跟随感，但要自己处理边界）。多数游戏要 ②。
  - 世界坐标 ⇄ 屏幕坐标靠 `viewport_to_world_2d` / `world_to_viewport`（011 讲光标时用过前者）。

---

### 026_gizmos.rs —— Gizmos 调试绘制

- **观察点**：三个蓝方块漂移，每个外面套一个彩色圈（索敌半径）、伸出一支黄色箭头（速度）；外框是世界边界、中心十字是原点。
- **要点**：
  - ★ **Gizmos 只活一帧** —— 图形在下一帧开始时会全部清空，所以**必须每帧重画**。这跟精灵正好相反：精灵 spawn 一次就一直在，想让它消失要 `despawn`；Gizmos 是"想让它消失就这一帧别画"。
  - 因此"开关调试显示"只需要一个 `if`，不需要管理实体。
  - 常用方法：`line_2d` / `circle_2d` / `rect_2d` / `arrow_2d` / `linestrip_2d`（3D 版把 `_2d` 换成 `_3d`）。
  - `GizmoConfigStore` 管全局参数：本讲把 `config.line.width` 从默认的 1 改成 3（高分屏上 1 像素基本看不见）；还有 `config.enabled`（整体开关）、`config.depth_bias`（与场景重叠）等。
  - ⚠️ 别拿 Gizmos 做正式美术 —— 它是调试工具，线宽/颜色/层级都不适合最终画面。
  - `TransformGizmo` 那种"拖拽手柄"是编辑工具，跟本讲的调试绘制不是一回事。

---
### 027_3d_basic.rs —— 3D 基础

- **观察点**：一块地面上摆着蓝色立方体（自转）、橙色球、绿色圆柱，两个光源投出阴影。

  ```text
  ── 3D 场景：地面 + 立方体(会转) + 球 + 圆柱
     相机位置 Vec3(-4.5, 4.0, 9.0)
     相机朝向 Dir3(Vec3(0.42368072, -0.32011428, -0.84736145))   ← 注意 z 是负的
  ```
- **要点**：
  - ★ **2D 和 3D 用的是同一个 `Transform`，但 z 的含义完全不同**：2D 里 z 是**层序**（谁盖住谁），3D 里 z 是**纵深**（离相机多远）。所以 `from_xyz(0,0,5)` 在 2D 是"提到最上层"，在 3D 是"朝相机挪 5 个单位"。
  - 3D 物体三件套：`Mesh3d`（形状）+ `MeshMaterial3d`（表面）+ `Transform`，两者都是**句柄** → 多个物体可共用一份网格/材质。
  - ⚠️ 缺必需组件**不报错，只是看不见**（和 003 的"窗口 ≠ 画面"同类）。
  - 相机姿态用 `looking_at(目标, 上方向)`；那个 `up` 不能省 —— 只给"看哪里"确定不了姿态（相机还能绕视线自转）。
  - ⚠️ 两处容易写错的名字：环境光资源是 **`GlobalAmbientLight`**（`AmbientLight` 是挂相机上覆盖用的**组件**）；光源开阴影的字段是 **`shadow_maps_enabled`**（写成 `shadows_enabled` 编译不过）。
  - 相机朝向那个数可以自己算：`looking_at` 的方向就是**目标 − 位置**归一化。相机在 x=−4.5 看原点，所以 x 分量是**正**的。

---

### 028_3d_gltf.rs —— 加载 glTF 模型

- **观察点**：文件里只有 3 个节点，实例化出来是 6 个实体：

  ```text
  ── assets.load("models/pyramid.gltf#Scene0") 已返回句柄
     第 1 帧：场景里带 Mesh3d 的实体 0 个
     [WorldInstanceReady] 实例化完成，根实体 375v0
        ├─ 382v0  PyramidPair
        ├─ 377v0  Root
        ├─ 378v0  PyramidA
        ├─ 379v0  PyramidB
        ├─ 380v0  Pyramid.PyramidMaterial  [有网格]
        ├─ 381v0  Pyramid.PyramidMaterial  [有网格]
  ```
- **要点**：
  - `WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("x.gltf")))`；也可以直接写 `"x.gltf#Scene0"`。
  - ★ **场景不是一个实体，而是一次实例化请求**：spawn 出来的那个是"根"，内容作为它的**后代**展开。操作场景要走 `Children::iter_descendants(root)`。
  - ★ **不能立刻查**：加载是异步的（020），在 `Startup` 里 spawn 完紧接着 `Query` 必然是空的，**而且不报错**。正确时机是 `On<WorldInstanceReady>` 观察者（注意这个事件**不在 prelude**，要 `use bevy::world_serialization::WorldInstanceReady`）。
  - ★ **节点 ≠ 网格实体**：命名节点上**没有** `Mesh3d`，真正的网格是它的**子实体**（名字按"网格名.材质名"拼）。所以"按节点名找到实体再改材质"会**静默失败** —— 得先找节点、再往下走一层。多出来的三个实体是：glTF 的 scene 本身 + 每个网格图元各一个。
  - 资产 `assets/models/pyramid.gltf` 是 `tools/make_assets.py` **手写生成**的：glTF 就是 JSON，二进制用 base64 内嵌。手写一遍能看清顶点、法线、绕序这些平时被工具藏起来的东西。

---

### 029_3d_picking.rs —— 3D 拾取与轨道相机

- **观察点**：启动输出两行；之后需要真实鼠标操作（**机器验证不了**）：

  ```text
  ── MeshPickingPlugin 已启用；三个物体挂上了 Over/Out/Click 观察者
     左键点物体、右键拖动转视角、滚轮拉近拉远
  ```
- **要点**：
  - ★ **`MeshPickingPlugin` 必须手动加**。UI 拾取（023）与 2D 精灵拾取是**默认开启**的，3D 网格拾取不是 —— 不加的话观察者**一个都不触发，且零警告零错误**。
    本条已实测确认：临时注释掉那一行后，程序照常启动，stderr 完全为空。所以排查"点了没反应"的第一步永远是：**拾取后端装了吗？**
  - 拾取背后是引擎替你做的射线求交（屏幕坐标 → 世界射线 → 与网格求交 → 取最近 → 生成带目标的实体事件）。要自定义命中信息才需要自己拿 `Camera::viewport_to_world`。
  - **观察者挂哪一层**（承接 028）：代码 spawn 的物体 `Mesh3d` 与观察者同实体即可；glTF 那种"节点 + 子网格"两层结构要按需选一层。
  - 轨道相机用**球坐标**（yaw/pitch/radius）描述，天然满足"永远看着焦点"；`pitch` 要限制在 ±90° 内，否则 `look_at` 的上方向会退化、画面翻转。
  - **左键选中、右键转视角**：两种操作要分开，否则拖一下会连发一堆 `Click`。

---

### 030_shader.rs —— 自定义着色器

- **观察点**：三个物体表面有斜向条纹并缓慢呼吸（`params.x` 由 Rust 每帧更新），旁边地面用自带材质作对照。

  ```text
  ── 自定义材质 GlowMaterial 已注册（MaterialPlugin）
     shader 来自 shaders/glow.wgsl；三个物体用条纹 + 呼吸着色
  ```
- **要点**：
  - Rust 侧：`#[derive(Asset, TypePath, AsBindGroup)]` + `#[uniform(N)]` 字段 + `impl Material { fn fragment_shader() -> ShaderRef }`，注册用 `MaterialPlugin::<M>::default()`。
  - ★ **`@group` 的编号不能写死** —— 这是本讲最大的坑。写成 `@group(2)` 编译正常、**一跑就炸**：

    ```text
    Validation Error: Shader global ResourceBinding { group: 2, binding: 0 }
    is not available in the pipeline layout
      Storage class Storage { .. } doesn't match the shader Uniform
    Quitting the application due to Validation RenderError
    ```

    材质 bind group 的编号由引擎按启用的特性**动态决定**并注入，所以不能写死 —— 要用引擎给的常量：`@group(constants::MATERIAL_BIND_GROUP)`（见 `bevy_pbr/src/render/pbr_bindings.wesl`）。**凡和管线布局有关的编号，优先找引擎的常量，别抄字面量。**
  - ⚠️ **忘了 `MaterialPlugin`** → 编译能过、程序能跑、日志干净，就是**画面上什么都没有**。
  - Rust 与 WGSL 是**两份要手动对齐的契约**（类型、binding 号、`#import` 路径），对不上大多是**运行时**才暴露。调试顺序：先看终端有没有 shader 校验错误 → 再确认插件装了没 → 最后把 fragment 改成返回纯品红，能看见就说明管线通了。
  - ⚠️ **热重载在本项目不可用**（`file_watcher` 在 `dev` 组里、不在默认，且开启要下载依赖），所以改 shader 必须重新 `cargo run`。另一种做法是 `embedded_asset!` 把 WGSL 编进二进制（发布常用，代价是彻底没有热重载）。
  - `assets/shaders/glow.wgsl` 是**手写源码**（不像 `textures/`、`models/` 是脚本生成的），放在 `assets/` 下是因为 `ShaderRef::path(..)` 的基准就是它。

---
## 常见坑速查

| 现象 | 原因 |
|------|------|
| 控制台疯狂刷屏 | 在系统里无条件 `println!`。系统每帧都跑，用 `is_changed()` / `Timer` / `on_timer` 节流 |
| 「按一次触发多次」 | 用了 `pressed`（按住每帧为真），应该用 `just_pressed`。实测按住一秒：`just_pressed` 1 次、`pressed` 60 次 = 帧率（见 011） |
| 结果算错但不报错、且稳定复现 | **顺序歧义**：两个系统抢同一份数据却没声明先后。开 `ambiguity_detection` 查（见 004） |
| 某个系统完全没执行，且毫无报错 | `Single` / `Option<Single>` / `Populated` 的条件不满足时，Bevy 会**静默跳过**整个系统（见 006） |
| 一运行就 panic：`Resource does not exist` | 缺 `Res<T>` / `ResMut<T>`，资源没注册。用 `Option<Res<T>>` 自己处理，或用 `If<Res<T>>` 让系统跳过（见 009） |
| 一运行就 panic：`error[B0001]` | 同一系统里两个查询访问同一组件的读/写。这是**运行时**检查，编译能过。用 `Without<T>` 造互斥查询，或 `ParamSet`（见 007） |
| 遍历顺序和生成顺序对不上 | Bevy 按**原型**分组存储，遍历逐组进行，组间顺序与生成时间无关。别依赖遍历顺序（见 006） |
| `commands.spawn` 之后立刻查不到 | 命令是延迟执行的，要到同步点才生效（见 008） |
| 消息晚一帧才被读到 | **Bevy 不会自动把写者排在读者之前**，必须自己 `.chain()` / `.after()`；且消息双缓冲只活两帧（见 012） |
| 定时器每帧都触发 | `Once` 计时器到点后 `is_finished()` **每帧都为真**，应该用 `just_finished()`（见 010） |
| 移动速度随帧率变化 | 位移没乘 `time.delta_secs()`。实测帧长减半后，按帧算的写法会多走 33%（见 010） |
| 什么都看不见 | 场景里没有相机（2D 需要 `Camera2d`）。窗口在、程序不报错、但画面空白 |
| **组件没改却触发了 `Changed`** | 有过可变借用就算"变过"（`DerefMut`），与值是否相同无关。写组件用 `set_if_neq`（见 015） |
| 子实体没跟着动 / 位置算不对 | `Transform` 是**局部**的，世界坐标在只读的 `GlobalTransform`；且父的**缩放会逐级相乘**（见 014） |
| despawn 一个节点后子树也没了 | 销毁父实体会**连带销毁整棵子树**（见 014） |
| 观察者里的改动要等下一帧才生效 | `commands.trigger` 是延迟的，观察者在同步点才跑（见 013） |
| 直接跑 `target/debug/examples/xxx.exe` 时资产全找不到 | `assets/` 的根按 `BEVY_ASSET_ROOT` → `CARGO_MANIFEST_DIR`（`cargo run` 自动有）→ **可执行文件目录** 依次确定；直接跑 exe 会去 exe 旁边找。设 `BEVY_ASSET_ROOT` 或用 `cargo run`（见 020） |
| 资产加载失败但程序不报错 | `LoadState::Failed` **不会 panic**，只是永远没有内容。把 `get_load_state(..)` 打出来看（见 020） |
| 改素材文件后画面不变 | 热重载要 `file_watcher` feature，它在 `dev` 组里、**不在默认 feature**：`features = ["dev"]`（见 020） |
| UI 里的中文不显示 / 刷 ICU4X 警告 | Bevy 默认字体是 `FiraMono-subset`，**不含中文字形**，必须自带中文字体（见 022） |
| 正交相机缩放方向搞反 | `scale` 是**视野**的倍数：`scale` 越大看得越广、东西越**小**（见 025） |
| 播 `.wav` 直接 panic | 默认 `audio` feature 只带 `vorbis`（OGG）。播 WAV 要开 `wav`（依赖 hound），或用 `Pitch` 程序合成（见 024） |
| 3D 物体点了没反应、且不报错 | 3D 网格拾取要手动加 `MeshPickingPlugin`（UI / 2D 精灵拾取才默认开启）（见 029） |
| glTF 里"按节点名找到实体，改它却没反应" | 命名节点上**没有** `Mesh3d`，真正的网格是它的**子实体**（名字形如 `Mesh.Material`），要往下再走一层（见 028） |
| 自定义 shader 运行时校验失败：`group: 2 not available in the pipeline layout` | group 编号**不能写死**：用 `@group(#{MATERIAL_BIND_GROUP})`，引擎会注入（见 030） |
| 自定义材质"什么都看不见"也不报错 | 忘了 `MaterialPlugin::<M>::default()` 注册材质类型（见 030） |
| 环境光设了没效果 | `AmbientLight` 是**组件**（挂相机上覆盖）；全局资源叫 `GlobalAmbientLight`（见 027） |
| 光源开阴影没反应 | 字段名是 `shadow_maps_enabled`，不是旧版的 `shadows_enabled`（见 027） |
| 插件之间改了数据却看不到效果 | 跨插件的顺序也要显式声明：`SystemSet` + `configure_sets`（见 016） |
| `init_state` 处 panic：`StateTransition schedule is missing` | 没装 `StatesPlugin`。`DefaultPlugins` 自带；只用 `LogPlugin` 时要显式补（见 019） |
| 切换状态后当帧读到的还是旧状态 | `set()` 只是**请求**，切换在**帧末**的 `StateTransition` 里（见 018） |
| 初始化"只跑了一次" / 每帧逻辑压根没跑 | `OnEnter` / `OnExit` 只在切换那一帧跑；每帧逻辑要用 `Update` + `in_state(..)`（见 018） |
| 读子状态时 panic | 子状态不在时 `Res<State<子状态>>` 拿不到，得用 `Option<Res<State<..>>>`（见 019） |
| `info!` 什么都不打印 | 没装 `DefaultPlugins`（或 `LogPlugin`），没有 tracing 订阅者 |

---

## 后续规划

完整课程规划见 [`CURRICULUM.md`](CURRICULUM.md)。七个阶段：

| 阶段 | 讲次 | 状态 |
|------|------|------|
| 一 · 起步（App / 窗口 / 精灵 / 调度） | 001–004 | ✅ 已完成 |
| 二 · ECS 核心（组件 / 查询 / 过滤 / 命令 / 资源 / 时间 / 输入 / 消息） | 005–012 | ✅ 已完成 |
| 三 · 事件与关系（观察者 / 层级 / 变更检测） | 013–015 | ✅ 已完成 |
| 四 · 组织与状态（插件 / 模块 / 状态机） | 016–019 | ✅ 已完成 |
| 五 · 2D 表现层（资产 / 动画 / UI / 音频 / 相机 / Gizmos） | 020–026 | ✅ 已完成 |
| 六 · 3D 与渲染（3D / glTF / 拾取 / 着色器） | 027–030 | ⏳ |
| 七 · 工程质量（综合 / 测试 / 剖析发布） | 031–033 | ⏳ |
