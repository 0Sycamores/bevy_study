# bevy_study 课程规划（Bevy 0.19.1）

> 全栈进阶版：33 讲，7 个阶段，从最小 App 到 3D + 着色器 + 性能剖析 + 发布构建。
> 本文是**规划**。当前 `examples/` 里仍是旧的 10 个例子，按本规划实施后会被替换（对应关系见文末）。

---

## 0. 规划原则

1. **一讲一个概念**，每讲一个可独立运行的 `examples/NNN_name.rs`，只依赖编号更小的内容。
2. **不出现前置知识泄漏**。旧版把 `Res<Time>` 放在 003 用（资源到 007 才讲）、`Query` 放在 004 用（查询到 005 才讲），新版按依赖拓扑重排，这类泄漏全部修掉。
3. **每讲必须有"能看见/能验证"的现象**。纯打印的讲次会让位给可观察的行为。
4. **同一件事给出多种写法**（这是本项目的定位）：例如计时器讲 `Timer` 的三种驱动方式、查询讲 `single`/`iter`/`par_iter`。
5. **踩坑即内容**。每讲用注释就地演示一个"写错了会怎样"，而不是只在 README 里描述。
6. **深度锚点**：调度顺序（004）、变更检测（015）、状态机（018/019）、ECS 测试（032）是四个"别人教程常跳过但实战必踩"的深水区，单独成讲。

---

## 1. 阶段总览

| 阶段 | 讲次 | 主题 | 学完能做什么 |
|---|---|---|---|
| 一 · 起步 | 001–004 | App / 窗口 / 渲染 / 调度 | 开窗口画东西，并说清系统执行顺序 |
| 二 · ECS 核心 | 005–012 | 组件 / 查询 / 命令 / 资源 / 时间 / 输入 / 消息 | 用 ECS 写出帧率无关的交互逻辑 |
| 三 · 事件与关系 | 013–015 | 观察者 / 层级 / 变更检测 | 解耦通信、搭实体树、做增量更新 |
| 四 · 组织与状态 | 016–019 | 插件 / 模块 / 状态机 | 把代码拆成可维护的多文件结构 |
| 五 · 2D 表现层 | 020–026 | 资产 / 动画 / UI / 音频 / 相机 / Gizmos | 做出有美术、有界面、有音效的 2D 游戏 |
| 六 · 3D 与渲染 | 027–030 | 3D 基础 / glTF / 拾取 / 着色器 | 加载 3D 模型并写自定义材质 |
| 七 · 工程质量 | 031–033 | 综合 / 测试 / 剖析发布 | 做出并交付一个完整可发布的游戏 |

---

## 2. 逐讲规划

### 阶段一 · 起步（001–004）

#### 001 `001_app.rs` — 最小 App
- **目标**：看清一个 Bevy 程序的三件套：`App`、系统、`Schedule`。
- **核心 API**：`App::new()`、`add_systems(Update, ..)`、`App::run()`、`Startup`。
- **观察点**：不挂 `DefaultPlugins` 时，默认 runner 是 `run_once`（`bevy_app-0.19.1/src/app.rs:152`）——所以只打印一次就退出，**不是**死循环。再把 `DefaultPlugins` 加上，程序会卡住并弹窗，对比同一份代码的两种命运。
- **练习**：改成一个 `Startup` 系统 + 一个 `Update` 系统，解释为什么后者永远只跑一次。
- **吸收旧版**：`001_hello.rs`。

#### 002 `002_window.rs` — 窗口与相机
- **目标**：让画面真的出现，理解"渲染需要相机"。
- **核心 API**：`DefaultPlugins`、`WindowPlugin`/`Window`（标题、分辨率、`WindowMode`）、`ClearColor`、`Camera2d`、`Startup` 系统。
- **观察点**：注释掉 `Camera2d` → 窗口全黑但程序正常，证明"没有相机 = 没有渲染"，而不是崩溃。
- **练习**：改成启动即全屏、背景改成指定颜色。
- **吸收旧版**：从 `002_sprite.rs` 拆出窗口/相机部分。

#### 003 `003_sprite.rs` — 第一个精灵
- **目标**：生成实体、挂组件、被渲染。
- **核心 API**：`commands.spawn((..))`、`Sprite::from_color(..)`、`Transform::from_xyz`、required components（`Sprite` 自动带 `Transform`/`Visibility`）。
- **观察点**：坐标原点在屏幕中心，`y` 向上——和大多数 2D 库相反；spawn 两个重叠精灵看默认 `z` 顺序。
- **练习**：用 `Transform` 的 `scale`/`rotation` 摆出三个不同姿态的方块。
- **吸收旧版**：`002_sprite.rs`（顺便清掉里面残留的 `vec2(100.0, 1000.0)` 和注释掉的死代码）。

#### 004 `004_schedule.rs` — 系统执行顺序 ✅ 已实现
- **目标**：把"系统是并行且顺序不确定"这件事讲透，这是本项目最缺的一课。
- **核心 API**：元组注册（隐式并行）、`.chain()`、`.before()`/`.after()`、`#[derive(SystemSet)]`、`in_set`、`ScheduleBuildSettings { ambiguity_detection }`、`ambiguous_with`。
- **实际采用的观察点**（比原设想更好，因为它不依赖"碰运气复现"）：
  1. **主动打开歧义检测**，让 Bevy 在构建调度时直接 WARN 出"哪两个系统冲突且无序"。
  2. 构造一对真实会算错的系统（`apply_damage` 写血量 / `check_death` 读血量），
     实测输出血量为 -5 却得分 0——**不崩溃、不报错、每个系统单独看都对**。
  3. 用 `.chain()` 或 `.after()` 修掉，实测得分变 100、WARN 消失。
- **⚠️ 实现时发现的重要事实（已写进例子注释）**：错误顺序**在同一构建里是稳定复现的**（连跑 5 次结果一致），
  不是"偶发"。所以教学重点从"偶发错乱"修正为"**无保证**"：
  加一个系统/调一次注册就可能翻过来，判断标准是"有没有声明顺序"，而不是"现在跑着对不对"。
- **另一发现**：Bevy 默认隐藏系统名（警告里显示 `<Enable the debug feature to see the name>`），
  需要 `bevy = { features = ["debug"] }` 才能看到具体是哪两个系统。此 feature **默认未开启**，已在例子与 README 中说明。
- **前置**：002、003。

---

### 阶段二 · ECS 核心（005–012）

#### 005 `005_component.rs` — 组件与查询
- **目标**：自定义组件，读写组件数据。
- **核心 API**：`#[derive(Component)]`、元组 spawn、`Query<(&A, &mut B)>`、`Single<&mut T>`。
- **观察点**：`Query` 只返回**同时拥有**所声明组件的实体；给敌人实体不加 `Player` 标记，它就自动被排除。
- **练习**：加一个 `Health` 组件，写一个只读的"打印全队血量"系统。
- **吸收旧版**：`004_component.rs`（旧版 `Health` 挂了却没用，这里真正用起来）。

#### 006 `006_query.rs` — 查询的几种写法
- **目标**：同一件事的多种查询姿势，对应本项目"练习各种写法"的定位。
- **核心 API**：`query.iter()`、`iter_mut()`、`single()`（返回 `Result`）、`get(entity)`、`par_iter()`、`Query::iter_many`。
- **观察点**：`single()` 在 0.19 返回 `Result`——实体不存在时是 `Err` 而不是 panic，顺势讲"为什么 Bevy 把参数失败设计成跳过系统"。
- **练习**：把同一次遍历分别用 `iter()` 和 `par_iter()` 实现，在几百个实体下对比耗时。
- **前置**：005。

#### 007 `007_query_filter.rs` — 查询过滤与冲突
- **目标**：`With`/`Without` 做筛选，以及"为什么两个查询会冲突"。
- **核心 API**：`With`、`Without`、`Or`、`Has`，以及 `Without` 解开可变借用冲突的经典用法。
- **观察点**：同时写 `Query<&mut Transform, With<Player>>` 和 `Query<&mut Transform, With<Enemy>>` → 编译期报冲突；加上 `Without` 互斥标注后编译通过。这是 Bevy 新手最常撞的墙。
- **练习**：写一个"玩家撞到敌人就镜像敌人位置"的系统（就是旧 009 里 `award_hit` 的雏形）。
- **吸收旧版**：`005_query.rs` 的过滤部分 + `009_plugin.rs` 的 `Without` 用法。
- **前置**：006。

#### 008 `008_commands.rs` — 命令与增删改
- **目标**：理解 `Commands` 是**延迟执行**的，以及同步点在哪。
- **核心 API**：`spawn`/`despawn`/`insert`/`remove`、`entity(id)`、`Commands::get_entity`、`apply_deferred`（隐式同步点）。
- **观察点**：在同一系统里 `spawn` 之后立刻 `query` 查不到——命令还没落地；把 spawn 和 query 拆成两个系统（或加 `.chain()`）就好了。这个"为什么查不到"是新手的第二大坑。
- **练习**：写"按 K 键销毁一个实体并立刻在另一个系统里统计剩余数量"。
- **前置**：005、007。

#### 009 `009_resource.rs` — 资源
- **目标**：全局单例数据。
- **核心 API**：`#[derive(Resource)]`、`init_resource`、`insert_resource`、`Res`/`ResMut`、`resource_exists` 运行条件。
- **观察点**：访问未初始化的资源 → 参数获取失败 → 系统被**静默跳过**（不 panic）。用一条 `info!` 验证"系统根本没跑"。
- **练习**：做一个 `Score` 资源 + `Multiplier` 资源，分别用"每帧加"和"按事件加"两种方式更新。
- **吸收旧版**：`007_resource.rs`。
- **前置**：008。

#### 010 `010_time.rs` — 时间与计时器
- **目标**：帧率无关的逻辑，以及 `Timer` 的多种驱动写法。
- **核心 API**：`Time`（`delta_secs`/`elapsed_secs`）、`Timer`/`TimerMode`/`Stopwatch`、`tick(Duration)`/`is_finished()`/`just_finished()`/`reset()`、`Time<Virtual>`、`Time<Fixed>`。
- **内容安排**（本项目刚删掉的 `main.rs` 计时器内容在这里正式落地）：
  1. 三种计时写法对照：**每帧 if 累加** vs **`Timer` 资源** vs **`Timer` 组件**；讲清各自适用场景。
  2. `TimerMode::Once` 与 `Repeating` 的差异，`just_finished()` 为什么比 `is_finished()` 更适合触发一次性动作。
  3. 用 `Time<Virtual>` 做暂停/倍速。
- **观察点**：用 `delta_secs` 和无 `Time` 的"每帧加固定值"两种写法跑同一个移动系统，在人为降帧时对比位移。
- **练习**：做一个每 2 秒自动变色一次的方块；再改成暂停可停的表。
- **吸收旧版**：`003_system.rs` 的时间部分 + 已从 `src/main.rs` 删除的计时器示例。
- **前置**：009。

#### 011 `011_input.rs` — 键盘与鼠标输入
- **目标**：拿输入驱动逻辑，分清"持续"和"瞬间"。
- **核心 API**：`ButtonInput<KeyCode>`/`ButtonInput<MouseButton>`、`pressed`/`just_pressed`/`just_released`、`AccumulatedMouseMotion`、光标 → 世界坐标。
- **观察点**：经典 bug 现场——用 `pressed` 做"按一次开一枪"，按住会变成每秒 60 发；换成 `just_pressed` + 冷却计时器才对。这正是旧版 `010_game.rs` 踩过的坑，在这里提前讲掉。
- **练习**：用方向键移动方块（`pressed`），空格键切换颜色（`just_pressed`）。
- **吸收旧版**：`006_input.rs`。
- **前置**：010。

#### 012 `012_message.rs` — 消息（0.17 起由 Event 改名）
- **目标**：系统间解耦通信。
- **核心 API**：`#[derive(Message)]`、`app.add_message::<T>()`、`MessageWriter::write`、`MessageReader::read`、`MessageMutator`、`MessageReader` 的双缓冲语义。
- **观察点**：
  1. 消息**会过期**——没被读走就丢，和"事件队列"的直觉不同。
  2. Bevy 自动保证 `MessageWriter` 排在对应 `MessageReader` 之前（上游 `ecs/message.rs:143` 明确说明），所以写读顺序不用手动 `.chain()`。
- **练习**：`PlayerDied` 消息 → 一条链式反应（扣分 → 播音效 → 弹提示）。
- **吸收旧版**：`008_message.rs` 的消息部分。
- **前置**：009、010。

---

### 阶段三 · 事件与关系（013–015）

#### 013 `013_observer.rs` — 观察者与实体事件
- **目标**：比消息更精确的"针对某个实体"的响应式编程。
- **核心 API**：`#[derive(EntityEvent)]`、`.observe(|on: On<E>| ..)`、`commands.trigger(..)`、内置生命周期事件（`Add`/`Insert`/`Remove`）、`Observer` 冒泡传播。
- **观察点**：同一个自定义事件，用**全局消息**广播 vs 用**观察者**只发给目标实体，后者不会误伤别的实体；观察者能拿到被触发实体本身。
- **练习**：给"敌人被击杀"挂观察者，击杀方加分、被击杀方自毁，两者互不认识。
- **吸收旧版**：`008_message.rs` 的 `EntityDied` 部分。
- **前置**：012。

#### 014 `014_hierarchy.rs` — 父子层级
- **目标**：把实体组织成树，理解变换继承。
- **核心 API**：`ChildOf`、`Children`、`children![..]` 宏、`with_children`、`add_child`/`remove_child`、`Transform` 传播。
- **观察点**：父实体旋转/缩放，子实体跟着动（`Transform` 继承）；子实体的 `Transform.translation` 是**相对**父的局部坐标。
- **练习**：搭一个"太阳 → 行星 → 卫星"三层结构，用旋转父节点驱动整棵树公转。
- **前置**：008。

#### 015 `015_change_detection.rs` — 变更检测
- **目标**：只处理变化过的数据，这是 Bevy 性能模型的基石。
- **核心 API**：`Changed<T>`、`Added<T>`、`Res::is_changed()`、`Ref<T>`（`is_added`/`is_changed`/`last_changed`）、`RemovedComponents`。
- **观察点**：不加过滤的每次遍历 vs 加 `Changed<T>` 后只在真正改动时命中；在仅 1/60 帧改动的场景下命中次数差异巨大（用计数器打印验证）。
- **练习**：给血量变化加一个"只在掉血时刷新血条"的系统。
- **前置**：009、013。

---

### 阶段四 · 组织与状态（016–019）

#### 016 `016_plugin.rs` — 插件
- **目标**：把一组系统/资源/事件打包成可复用单元。
- **核心 API**：`impl Plugin`（`build`/`finish`/`cleanup`）、`PluginGroup`、`PluginGroupBuilder`、`DefaultPlugins.set(..)`、`disable::<T>()`。
- **观察点**：把 016 之前所有讲的系统收进自定义插件，`main` 里只剩插件列表；再演示 `DefaultPlugins` 的插件组结构（用 `RUST_LOG` 打印加载了哪些插件）。
- **练习**：把"敌人"相关系统抽成 `EnemyPlugin`，并让它在 `ScorePlugin` 之后初始化。
- **吸收旧版**：`009_plugin.rs`。
- **前置**：004、012。

#### 017 `017_module.rs` — 多文件工程结构（目录式 example）
- **目标**：从"单文件练习"过渡到"能长大的工程"。
- **形式**：本讲是**目录式 example**：`examples/017_module/main.rs` + `player.rs` + `enemy.rs` + `common.rs`。Cargo 会把 `examples/<name>/main.rs` 自动识别为名为 `<name>` 的 example（上游 bevy 仓库即用此模式）。
- **核心 API**：`mod`/`pub use`、`prelude.rs` 汇总导出、`plugin` 与 `module` 的配合。
- **观察点**：同名的 `PlayerPlugin` 分布在各自文件里，`main.rs` 只做组装；对比 016 的单文件版本，改动量集中在哪。
- **练习**：把 016 的代码原地拆成模块，保证 `cargo run --example 017_module` 行为不变。
- **前置**：016。

#### 018 `018_states.rs` — 状态机
- **目标**：菜单 / 游戏中 / 暂停 / 结算 的流程控制。
- **核心 API**：`#[derive(States)]`、`init_state::<T>()`、`OnEnter(S)`/`OnExit(S)`、`in_state(S)` 运行条件、`NextState<S>::set(..)`、`StateTransition` 调度。
- **观察点**：`OnEnter` 只在进入的那一帧跑；`in_state` 的系统在状态外完全不执行（不是"执行了但提前 return"）。
- **练习**：做 `Menu → Playing → GameOver` 三态循环，按空格推进状态，每个状态用不同背景色。
- **前置**：016。

#### 019 `019_state_advanced.rs` — 状态进阶
- **目标**：复杂流程下的状态组合。
- **核心 API**：`SubStates`（依附父状态存在）、`ComputedStates`（由其他状态推导）、`DespawnOnExit(S)`/`DespawnOnEnter(S)`/`DespawnWhen::new(..)`（0.19 的自动清理，旧名 `StateScoped` 已废弃）。
- **观察点**：`DespawnOnExit` 挂上去之后，切状态时实体**自动**销毁，不用手写清理系统——顺手对比"手写 `OnExit` 清理"的写法有多啰嗦。
- **练习**：用 `SubStates` 给 Playing 加 `Normal`/`Boss` 两个子状态，各自刷不同的敌人。
- **前置**：018。

---

### 阶段五 · 2D 表现层（020–026）

> 从本阶段起需要 `assets/` 目录。资产准备方案见第 4 节。

#### 020 `020_asset.rs` — 资产加载
- **目标**：从磁盘加载图片，并正确处理"还没加载完"。
- **核心 API**：`AssetServer::load`、`Handle<T>`、`Assets<T>::add`、`LoadState`/`AssetServer::is_loaded_with_dependencies`、热重载。
- **观察点**：`load` 是**异步**的，句柄立刻返回但图可能还没到；不加加载判断会看到一帧空白/默认贴图。
- **练习**：加载一张图，加载完成前显示占位色块，完成后替换。
- **前置**：008。

#### 021 `021_atlas_animation.rs` — 图集与帧动画
- **目标**：用一张雪碧图做逐帧动画。
- **核心 API**：`TextureAtlasLayout::from_grid(..)`、`Assets<TextureAtlasLayout>`、`TextureAtlas { layout, index }`、`Sprite::from_atlas_image(..)`、用 `Timer` 驱动 `index` 递增。
- **观察点**：`TextureAtlasLayout` 和 `Sprite` 是分开的——布局可被多个精灵共享，改 `index` 就换帧。这正好复用 010 的 `Timer`。
- **练习**：做走路 6 帧循环 + 站立 1 帧，用状态切换（结合 018）。
- **前置**：010、020。

#### 022 `022_ui.rs` — UI 布局
- **目标**：画出 HUD：血条、分数、文字。
- **核心 API**：`Node`（宽高/padding/margin/flex 布局/`PositionType`）、`Text`/`TextFont`/`TextColor`、`ImageNode`、`BackgroundColor`、`BorderColor`、`ZIndex`、`UiTargetCamera`。
- **观察点**：UI 用的是 flex 布局，坐标原点在**左上角**——和 003 里精灵的"中心 + y 向上"正好相反，这个反差最容易错。
- **练习**：做一个右上角分数 + 底部血条（血条用 `Node` 宽度百分比表示）。
- **前置**：003、009。

#### 023 `023_ui_interaction.rs` — UI 交互
- **目标**：按钮能点、能被鼠标悬停。
- **核心 API**：`Button`、`Interaction`（`None`/`Hovered`/`Pressed`）、`Changed<Interaction>`、UI 上的 `observe`（`Pointer<Click>`/`Pointer<Over>`）、`BackgroundColor` 反馈。
- **观察点**：两种写法对照——**轮询** `Changed<Interaction>` vs **观察者** `On<Pointer<Click>>`；后者是 0.19 的推荐路径，也更省系统。
- **练习**：做一个主菜单（开始/退出两个按钮），点了 `NextState` 切到 018 的 Playing。
- **前置**：015、018、022。

#### 024 `024_audio.rs` — 音频
- **目标**：背景音乐 + 音效。
- **核心 API**：`AudioPlayer::new(handle)`、`PlaybackSettings`（`loop`/音量）、`AudioSink`（运行时控制播放/暂停/音量）、`Volume`。
- **观察点**：音效和 BGM 的区别在于"要不要拿到 `AudioSink` 去控制"——音效放完即弃，BGM 要留句柄。
- **练习**：BGM 循环播放，按 M 静音/恢复；开火事件触发一次性音效。
- **前置**：012、020。

#### 025 `025_camera.rs` — 相机
- **目标**：跟随、缩放、分屏。
- **核心 API**：`Camera2d`、`Projection`（`OrthographicProjection` 的 `scale`/`ScalingMode`）、相机作为子实体跟随、`Viewport`/多相机、`Camera::world_to_viewport`。
- **观察点**：把相机挂成玩家的**子实体** → 零代码跟随（呼应 014）；对比手写"每帧 lerp 到玩家位置"的写法，后者能做平滑跟随。
- **练习**：做成"相机滞后跟随玩家 + 滚轮缩放"。
- **前置**：014、011。

#### 026 `026_gizmos.rs` — Gizmos 调试绘制
- **目标**：把不可见的东西画出来（碰撞盒、速度向量、路径）。
- **核心 API**：`Gizmos`（`line_2d`/`circle_2d`/`rect_2d`/`arrow_2d`）、`GizmoConfigStore`（线宽/开关）、`TransformGizmo`。
- **观察点**：Gizmos 只画一帧、不留痕，所以必须在 `Update` 里每帧重画；它和 `Sprite` 走的是两套渲染路径，不受精灵层级影响。
- **练习**：给 007 的"玩家撞敌人"逻辑画上圆形碰撞范围，让判定过程可视化。
- **前置**：007、025。

---

### 阶段六 · 3D 与渲染（027–030）

#### 027 `027_3d_basic.rs` — 3D 基础
- **目标**：把 2D 的直觉搬到 3D。
- **核心 API**：`Camera3d`、`Mesh3d(handle)`、`MeshMaterial3d::<StandardMaterial>(handle)`、`Assets<Mesh>::add(Cuboid::new(..))`、`PointLight`/`DirectionalLight`（含阴影）、`Transform` 的 3D 旋转。
- **观察点**：2D 里"原点中心 + y 向上 + z 定层序"，3D 里 z 变成**纵深**，`Camera3d` 默认朝 `-z` 看——同一套 `Transform` 组件语境的差异。
- **练习**：摆一个三色立方体 + 地面 + 平行光阴影，用鼠标拖动旋转相机。
- **前置**：003、025。

#### 028 `028_3d_gltf.rs` — 加载 glTF 模型
- **目标**：用美术做好的模型，而不是代码拼几何体。
- **核心 API**：`WorldAssetRoot(handle)`（**0.19 改名，旧名 `SceneRoot`**）、`AssetServer::load` + `GltfAssetLabel::Scene(0)`、`Scene`/`SceneInstance`、glTF 动画播放（`AnimationPlayer`/`AnimationGraph`）。
- **观察点**：glTF 场景**不是**一个实体，而是一棵被实例化出来的实体树；想改其中的材质要用 `SceneInstance` 就绪后再查，不能立刻 `Query`。
- **练习**：加载一个带动画的模型并循环播放，再遍历它内部的网格实体做染色。
- **前置**：020、027。

#### 029 `029_3d_picking.rs` — 3D 拾取与相机控制
- **目标**：鼠标点到 3D 物体上。
- **核心 API**：`MeshPickingPlugin`（**非默认插件，必须手动加**；UI/Sprite 拾取则是默认开启的）、`Pointer<Over>`/`Pointer<Click>`/`Pointer<Drag>`、射线、`Camera::viewport_to_world`、围绕点旋转的轨道相机。
- **观察点**：不加 `MeshPickingPlugin` 时观察者**完全不触发**（不报错），这是最难查的一类问题。
- **练习**：点选立方体高亮 + 拖动旋转 + 滚轮拉近。
- **前置**：013、027。

#### 030 `030_shader.rs` — 自定义着色器
- **目标**：写出自己的 WGSL 材质。
- **核心 API**：`Material` trait + `AsBindGroup`、`ShaderRef`、`MaterialPlugin`、`embedded_asset!`、uniform 传参、`example.com` 的 WGSL 入口约定。
- **观察点**：改 WGSL 存盘 → 热重载即时生效（不用重编译 Rust）；Rust 侧只负责传 uniform。
- **练习**：写一个随时间变色的材质，把 `Time` 通过 uniform 传进着色器；进阶做一个全屏后处理。
- **前置**：020、027。

---

### 阶段七 · 工程质量与综合（031–033）

#### 031 `031_capstone_game.rs` — 综合 2D 小游戏
- **目标**：把 001–026 全部串起来，做成一个真能玩的游戏。
- **要求**：玩家移动（010/011）、射击带冷却（010/011）、敌人成批刷新（008）、碰撞与计分（007/009/012）、菜单与暂停（018/019/023）、音效（024）、相机跟随（025）、血条 HUD（022）、调试 Gizmos（026）。
- **观察点**：本讲重点不是新 API，而是**结构**——所有东西都进插件（016/017），系统顺序明确标注（004），资源划分清晰（009）。
- **练习**：给它加上"敌人随时间变强"和"最高分记录"。
- **吸收旧版**：`010_game.rs`（旧版只有一个不会重生的敌人、靠杀一次就结束，这里补全胜负与重开循环）。
- **前置**：001–026 全部。

#### 032 `032_ecs_test.rs` — ECS 测试
- **目标**：让游戏逻辑可以被自动化验证，不再靠手点。
- **形式**：`#[test]` 用例写在 example 内（或抽到 `tests/`），用 `App::new()` 无窗口驱动。
- **核心 API**：`app.update()` 手动推进帧、`World::resource::<T>()` 断言、`world.query::<..>()`、`App::add_systems` 后不 `.run()`、`Time::advance_by(Duration)` 控制时间、`ButtonInput::press` 模拟输入。
- **关键技巧**（本项目已实测）：无 `DefaultPlugins` 时必须手动 `.init_resource::<ButtonInput<KeyCode>>()` 和 `.init_resource::<Time>()`；`just_pressed` 不会自动清除，跨帧要手动 `.clear()`；时间用 `Time::advance_by(Duration::from_secs_f64(1.0/60.0))` 推进。
- **观察点**：**写一个会失败的反向用例**——把"开火冷却"的判断改成恒真，测试应报出 60 发/秒；说明测试真的在鉴别问题，而不是永远绿。
- **练习**：给 031 的"敌人生成间隔""碰撞计分""冷却射速"各写一条测试。
- **前置**：031。

#### 033 `033_diagnostics_release.rs` — 剖析与发布
- **目标**：知道卡在哪，并把它打包出去。
- **核心 API**：
  - 诊断：`FrameTimeDiagnosticsPlugin`、`EntityCountDiagnosticsPlugin`、自定义 `Diagnostic`、`LogDiagnosticsPlugin`、`bevy_dev_tools` 的 FPS 悬浮窗。
  - 计数与日志：`tracing`（`info!`/`warn!`/`error!`）、`RUST_LOG` 分级、`LogPlugin` 配置。
  - 发布：`[profile.release]`（`lto`/`codegen-units`/`opt-level`）、`[profile.dev] opt-level` 与依赖预优化（本项目 `Cargo.toml` 已配）、体积裁剪、动态链接特性开关。
- **观察点**：同一场景在 dev（`opt-level = 1` + 依赖 `opt-level = 3`）与 release 下的帧时间差异；以及 `cargo run --release` 与 `cargo run` 的编译耗时/运行耗时权衡。
- **练习**：给 031 加上 FPS 与实体数显示，跑 release 对比，写出体积优化前后的二进制大小。
- **前置**：031。

---

## 3. 前置依赖速查

```
001 App
 └─ 002 窗口/相机
     └─ 003 精灵
         ├─ 004 调度 ────────────────┐
         ├─ 022 UI ─── 023 UI交互 ───┤
         └─ 027 3D基础 ─┬─ 028 glTF │
                        ├─ 029 拾取 │
                        └─ 030 着色器
005 组件 ─ 006 查询 ─ 007 过滤 ─ 008 命令 ─ 009 资源 ─ 010 时间 ─ 011 输入 ─ 012 消息
                                                                   └─ 013 观察者 ─ 015 变更检测
008 ─ 014 层级
012/004 ─ 016 插件 ─ 017 模块
016 ─ 018 状态 ─ 019 状态进阶
008/009/003 ─ 020 资产 ─ 021 图集动画
020 ─ 024 音频
014/011 ─ 025 相机 ─ 026 Gizmos
全部 ─ 031 综合 ─ 032 测试
031 ─ 033 剖析发布
```

---

## 4. 实施注意事项

### 4.1 资产准备（020 起需要）
上游 crate 包**不附带** `assets/`（已确认 `bevy-0.19.1/assets` 不存在），需要自建：

| 用途 | 方案 |
|---|---|
| 图片 / 雪碧图 | 用捆绑的 Python + Pillow 程序化生成 PNG（纯色块、棋盘格、4×4 帧动画条） |
| 音效 / BGM | 用 Python 标准库 `wave` 生成正弦波 WAV |
| 字体 | 不额外提供，用 Bevy 内置默认字体；需要自定义字体时用系统字体查询 API，避免分发字体文件 |
| glTF 模型 | 手写一个内嵌 base64 buffer 的极简 `.gltf`（单个立方体 + 一条旋转动画），避免下载二进制 |

> 这样 `assets/` 完全由脚本生成，可复现、可进版本库，不引入外部下载依赖。

### 4.2 参考源
本机已有 Bevy 0.19.1 的全部上游示例源码，**写每一讲时应对照它核对 API**，不要凭记忆：

```
C:\Users\Sycamore\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\bevy-0.19.1\examples\
```

412 个 `.rs` 文件，按主题分目录（`ecs/`、`state/`、`time/`、`ui/`、`picking/`、`shader/` …）；
测试示例在 `bevy-0.19.1\tests\how_to_test_apps.rs` 与 `how_to_test_systems.rs`（对应 032）。

### 4.3 已核实的 0.19.1 API 变更（写代码时别用旧名）

| 旧写法 | 0.19.1 正确写法 | 影响讲次 |
|---|---|---|
| `Event` / `EventWriter` / `EventReader` / `add_event` | `Message` / `MessageWriter` / `MessageReader` / `add_message` | 012 |
| `StateScoped(S)` | `DespawnOnExit(S)` / `DespawnOnEnter(S)` / `DespawnWhen::new(..)` | 019 |
| `SceneRoot(handle)` | `WorldAssetRoot(handle)` | 028 |
| `TextureAtlasSprite` | `Sprite::from_atlas_image(img, TextureAtlas { layout, index })` | 021 |
| `AudioBundle` / `AudioSink` 单独挂 | `AudioPlayer::new(handle)` + `PlaybackSettings` | 024 |
| `Parent` | `ChildOf`（配合 `children![..]` 宏） | 014 |
| `PbrBundle` | `Mesh3d` + `MeshMaterial3d::<StandardMaterial>` | 027 |
| 拾取需要手动开插件 | UI/Sprite 拾取默认开启；**3D 网格拾取需手动加 `MeshPickingPlugin`** | 029 |
| `Query::single()` 返回 `&T` | 返回 `Result` | 006 |

### 4.4 命名与配套改动
- 编号统一 `NNN_snake_name.rs`，三位零填充；017 是唯一的目录式 example。
- `src/main.rs` 保持为指向 README 与示例的极简入口（本项目已如此）。
- 全部实施后重写 `README.md` 的课程表；在实施完成前，README 顶部保留"正在按 `CURRICULUM.md` 重排"的提示，避免编号混淆。
- 每讲结束后跑 `cargo check --all-targets`，要求零警告再进下一讲。

---

## 5. 旧版 → 新版对应关系（内容不丢）

| 旧文件 | 去向 |
|---|---|
| `001_hello.rs` | → 001（并补 `DefaultPlugins` 对照） |
| `002_sprite.rs` | → 002（窗口/相机）+ 003（精灵）；清掉残留死代码 |
| `003_system.rs` | → 010（时间部分）+ 004（多系统与顺序） |
| `004_component.rs` | → 005（`Health` 真正被使用） |
| `005_query.rs` | → 006（遍历）+ 007（`With` 过滤） |
| `006_input.rs` | → 011 |
| `007_resource.rs` | → 009 |
| `008_message.rs` | → 012（`PlayerDied` 消息）+ 013（`EntityDied` 观察者） |
| `009_plugin.rs` | → 016（插件）+ 007（`Without` 反例 + `award_hit`） |
| `010_game.rs` | → 031（补全胜负循环、冷却计时、系统顺序、插件结构） |
| `src/main.rs` 已删的计时器 | → 010 |

---

## 6. 里程碑

| 里程碑 | 完成标志 | 状态 |
|---|---|---|
| M1 阶段一 | 004 跑完，能口头解释"为什么两个系统写同一资源会串行但顺序不定" | ✅ 已完成（001–004 已落地并实跑验证） |
| M2 阶段二 | 012 跑完，能写出帧率无关的移动 + 冷却射击 | ⏳ |
| M3 阶段三 | 015 跑完，能用观察者替代消息、说清变更检测省下了什么 | ⏳ |
| M4 阶段四 | 019 跑完，多文件工程 + 完整状态机可跑 | ⏳ |
| M5 阶段五 | 026 跑完，2D 表现层齐活（资产/动画/UI/音频/相机/调试） | ⏳ |
| M6 阶段六 | 030 跑完，能加载 glTF 并写自定义 WGSL 材质 | ⏳ |
| M7 阶段七 | 033 跑完，有一个可测试、可剖析、可发布的小游戏 | ⏳ |

> 阶段一的实施备注：
> - `003_sprite.rs` 取代了旧 `002_sprite.rs`，旧文件里残留的死代码（`vec2(100.0, 1000.0)` 与注释掉的 `Vec2::new`）随之清除。
> - `001_app.rs` 取代旧 `001_hello.rs`，改用 `println!` 并补充了「为什么只跑一帧」的源码级解释。
> - 旧 `002_sprite.rs` 的窗口/相机部分独立成 `002_window.rs`；旧 `003_system.rs` 的时间部分按计划推迟到 010 讲。
> - 被取代的四个旧文件保存在 git 提交 `21b0c91`（重排前的完整基线）。
