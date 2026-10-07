# bevy_study

用递进的小例子学习 **Bevy 0.19.1** 的 ECS 核心。
每个例子都是一个可以单独运行的程序，建议按编号顺序看。

- 引擎版本：`bevy = "0.19.1"`
- Rust edition：`2024`（实测工具链 `rustc 1.99.0`）
- 所有例子在 `examples/` 下，每个都能单独运行；`src/main.rs` 只是一个指针，运行它会提示去看 examples

> ⚠️ **课程正在重排。** 新版规划见 [`CURRICULUM.md`](CURRICULUM.md)：33 讲 / 7 个阶段。
> **当前进度：阶段一、二已完成（新编号 001–012）。**
> 重排前的旧例子大部分已被吸收删除，剩下三个存档在 [`legacy_examples/`](legacy_examples/)（**不再参与编译**），
> 只作参考。对应关系见 `CURRICULUM.md` 第 5 节。

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

> Windows / PowerShell 下设置日志用：`$env:RUST_LOG="info"; cargo run --example 008_message`

---

## 怎么读这些例子

每个例子都按「先跑起来看现象，再看代码」的顺序组织：现象写在**观察点**里，原理和踩坑写在**要点**里。
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

## 旧版例子（已移出，存档在 `legacy_examples/`）

这些例子**不再是 cargo 目标**：`cargo run --example 009_plugin` 之类会报「没有这个目标」。
文件放在 [`legacy_examples/`](legacy_examples/)，只作参考，说明见该目录的 `README.md`。

| 文件 | 主题 | 去向 | 状态 |
|------|------|------|------|
| [008_message.rs](legacy_examples/008_message.rs) | 消息与观察者 | → 新 012（已完成）/ 013 | 半吸收 |
| [009_plugin.rs](legacy_examples/009_plugin.rs) | 插件与插件组 | → 新 016 | 待吸收 |
| [010_game.rs](legacy_examples/010_game.rs) | 综合小游戏 | → 新 031 | 待吸收 |

> 已经**吸收完毕并删除**的旧文件：
> `005_query.rs`（→ 新 006/007 的过滤部分）、
> `006_input.rs`（→ 新 011）、
> `007_resource.rs`（→ 新 009）。
> 需要它们时从 git 历史里取，例如 `git show 21b0c91:examples/001_hello.rs`。

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
  - 这里的 `Health` 是**真正被使用**的（跑起来能看到颜色随之变化），不像旧版 004 里挂了却没用。
- **布局**：窗口 960×640；方块的坐标和 `Speed` 值都写在 `setup` 里，可直接改。

---

### 006_query.rs —— 查询的几种写法 + 取不到数据会怎样

- **观察点**：控制台依次打印遍历结果、`single()` 命中、`Single` 参数命中、`Populated` 计数。
  **注意「写法⑤」一行输出都没有** —— 它被静默跳过了。
- **要点**：
  - **取数方式对照**：`iter()` / `iter_mut()` / `par_iter_mut()` / `single()`（返回 `Result`）/ `get(entity)` / `iter_many()`，
    以及三个「取不到就不跑系统」的参数：`Single` / `Option<Single>` / `Populated`。
  - **取不到数据时 Bevy 的三种反应（0.19.1 实测）**：
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

## 逐个说明 · 旧版（待重排）

> 下面这些例子是**重排前的内容**，已移到 [`legacy_examples/`](legacy_examples/)，**不再是 cargo 目标**
> （`cargo run --example 006_input` 会报「没有这个目标」）。说明与代码仍然对应，留作参考；
> 想在当前仓库里跑一下，先 `git mv legacy_examples/006_input.rs examples/`。

### 006_input.rs —— 键盘输入

- **观察点**：WASD 移动小方块；按空格打印一次 `Space Pressed`。
- **要点**：
  - `Res<ButtonInput<KeyCode>>` 读取键盘状态。
  - `pressed(..)` = 按住期间**每帧**为真；`just_pressed(..)` = 只在**按下的那一帧**为真。
    这是最容易搞混的一点：用 `pressed` 做「按一次触发一次」的事情会变成每帧触发。
  - 这里生成玩家时**没写 `Transform`**，但 `move_player` 照样能查到它——因为 `Sprite` 自动补齐了 `Transform`（required components，见 003）。

---

### 007_resource.rs —— 资源：全局唯一的数据

- **观察点**：控制台每帧打印分数，数值每帧 +100，倍率每帧 +0.1。
- **要点**：
  - `#[derive(Resource)]` + `init_resource::<Score>()` 注册全局资源。
  - `Res<Score>` 只读，`ResMut<Score>` 可写，两者**不能同时存在于一个系统**。
  - 组件描述「有哪些实体」（见 005），资源描述「整个游戏共享的状态」（分数、配置、计时器……）。
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
  后者的顺序体现了 `commands.trigger` 是**延迟执行**的（命令在同步点统一生效，见 008）。
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
    必须加上 `Without<Player>` 才能让 Bevy 认定两个查询**互不相交**，否则会触发 `B0001` 冲突。
    ⚠️ 这个检查在**运行时**（详见 007）。
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
| `info!` 什么都不打印 | 没装 `DefaultPlugins`（或 `LogPlugin`），没有 tracing 订阅者 |

---

## 后续规划

完整课程规划见 [`CURRICULUM.md`](CURRICULUM.md)。七个阶段：

| 阶段 | 讲次 | 状态 |
|------|------|------|
| 一 · 起步（App / 窗口 / 精灵 / 调度） | 001–004 | ✅ 已完成 |
| 二 · ECS 核心（组件 / 查询 / 过滤 / 命令 / 资源 / 时间 / 输入 / 消息） | 005–012 | ✅ 已完成 |
| 三 · 事件与关系（观察者 / 层级 / 变更检测） | 013–015 | ⏳ |
| 四 · 组织与状态（插件 / 模块 / 状态机） | 016–019 | ⏳ |
| 五 · 2D 表现层（资产 / UI / 音频 / 相机 / Gizmos） | 020–026 | ⏳ |
| 六 · 3D 与渲染（3D / glTF / 拾取 / 着色器） | 027–030 | ⏳ |
| 七 · 工程质量（综合 / 测试 / 剖析发布） | 031–033 | ⏳ |

旧 10 讲的内容在新规划中**全部保留**，对应关系见 `CURRICULUM.md` 第 5 节。
