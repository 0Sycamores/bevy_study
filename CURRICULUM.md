# bevy_study 课程规划（Bevy 0.19.1）

> 全栈进阶版：33 讲，7 个阶段，从最小 App 到 3D + 着色器 + 性能剖析 + 发布构建。
> 本文是**规划**。当前 `examples/` 里仍是旧的 10 个例子，按本规划实施后会被替换（对应关系见文末）。

---

## 0. 规划原则

1. **一讲一个概念**，每讲一个可独立运行的 `examples/NNN_name.rs`，只依赖编号更小的内容。
2. **不出现前置知识泄漏**。每讲只依赖编号更小的讲次：比如 `Time` 必须排在 `Resource` 之后，
   不能出现"用到了还没教的东西"。依赖顺序见第 3 节。
3. **每讲必须有"能看见/能验证"的现象**。纯打印的讲次会让位给可观察的行为。
4. **同一件事给出多种写法**（这是本项目的定位）：例如计时器讲 `Timer` 的三种驱动方式、查询讲 `single`/`iter`/`par_iter`。
5. **踩坑即内容**。每讲用注释就地演示一个"写错了会怎样"，而不是只在 README 里描述。
6. **深度锚点**：调度顺序（004）、变更检测（015）、状态机（018/019）、ECS 测试（032）是四个"别人教程常跳过但实战必踩"的深水区，单独成讲。
7. **不写「练习」和「观察实验」小节**。要的是能读懂的例子，不是作业本。
   现象与结论一律用**陈述句**写进正文——不写"把 X 注释掉试试，会看到 Y"，
   而写"去掉 X 的后果是 Y，原因是……"。布局上：现象放「观察点」，原理放「要点」。
   此规则对后续所有讲次（005–033）生效。
8. **每讲的例子只服务本讲的主题**，不要把前面讲次的机制堆进来显得更"完整"。
   010 讲时间就只讲时间，不顺手示范移动方块；011 讲输入就只讲输入，不写成射击教程。
   确实需要借用前面讲次的东西时，让它当**配角**、在注释里点明"细节在 X 讲"，不要让它抢戏。
   （这条是实施 010/011 时发现走偏后补上的，见文末阶段二备注。）
9. **文件顶部注释统一成「新增概念 / 使用场景 / 注意」三段**（001–012 已全部改齐）：
   - **新增概念**：本讲新出现的 API 与名词，每条一行解释，不展开
   - **使用场景**：真实开发里什么时候会用到它
   - **注意**：只写最容易踩的那一个坑（可省略），细节指向文末

   目的是让人扫一眼就知道"这讲教什么、什么时候用得上"。
   原理、源码位置、实测输出、历史背景等**全部放文末**，顶部不写记叙文。
   顶部注释控制在 15 行以内。

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

#### 002 `002_window.rs` — 窗口与相机
- **目标**：让画面真的出现，理解"渲染需要相机"。
- **核心 API**：`DefaultPlugins`、`WindowPlugin`/`Window`（标题、分辨率、`WindowMode`）、`ClearColor`、`Camera2d`、`Startup` 系统。
- **观察点**：注释掉 `Camera2d` → 窗口全黑但程序正常，证明"没有相机 = 没有渲染"，而不是崩溃。

#### 003 `003_sprite.rs` — 第一个精灵 ✅ 已实现
- **目标**：生成实体、挂组件、被渲染；**并把坐标系一次讲清**（本讲是 `Transform` 首次出现处）。
- **核心 API**：`commands.spawn((..))`、`Sprite::from_color(..)`、`Transform::from_xyz`、`with_scale`/`with_rotation`、required components（`Sprite` 自动带 `Transform`/`Visibility`）。
- **坐标系与手性**（实施时新增，原规划未包含）：Bevy 是**右手系 Y-up**，2D/3D 共用一套；
  +X 右、+Y 上、+Z 指向观察者；与 Godot/Maya/OpenGL 一致，与 Unity 的 Z 轴相反。
  引用对照图：<https://topkg.github.io/bevy-cheatbook/img/handedness.png>
  （出处 <https://topkg.github.io/bevy-cheatbook/fundamentals/coords.html>，原图作者 @FreyaHolmer）。
  同时点明 **UI 坐标是例外**（左上角原点、Y 向下），为 022 埋伏笔。
- **观察点**：坐标原点在屏幕中心，`y` 向上；z 决定层叠（越大越靠前）。
- **已验证的手性实验**：`Quat::from_rotation_z(+90°) * Vec3::X == Vec3::Y`（实测），
  即右手系里绕 +Z 的正向旋转在屏幕上表现为**逆时针**。

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

#### 005 `005_component.rs` — 组件与查询 ✅ 已实现
- **目标**：自定义组件，读写组件数据。
- **核心 API**：`#[derive(Component)]`、元组 spawn、`Query<(&A, &mut B)>`、`Single<&mut T>`。
- **观察点**：`Query` 只返回**同时拥有**所声明组件的实体；给敌人实体不加 `Player` 标记，它就自动被排除。

#### 006 `006_query.rs` — 查询的几种写法 ✅ 已实现
- **目标**：同一件事的多种查询姿势，对应本项目"同一件事多种写法对照"的定位。
- **核心 API**：`query.iter()`、`iter_mut()`、`single()`（返回 `Result`）、`get(entity)`、`par_iter()`、`Query::iter_many`。
- **观察点**：`single()` 在 0.19 返回 `Result`——实体不存在时是 `Err` 而不是 panic，顺势讲"为什么 Bevy 把参数失败设计成跳过系统"。
- **前置**：005。

#### 007 `007_query_filter.rs` — 查询过滤与冲突 ✅ 已实现
- **目标**：`With`/`Without` 做筛选，以及"为什么两个查询会冲突"。
- **核心 API**：`With`、`Without`、`Or`、`Has`，以及 `Without` 解开可变借用冲突的经典用法。
- **观察点**：同时写 `Query<&mut Transform, With<Player>>` 和 `Query<&mut Transform, With<Enemy>>` → 编译期报冲突；加上 `Without` 互斥标注后编译通过。这是 Bevy 新手最常撞的墙。
- **前置**：006。

#### 008 `008_commands.rs` — 命令与增删改 ✅ 已实现
- **目标**：理解 `Commands` 是**延迟执行**的，以及同步点在哪。
- **核心 API**：`spawn`/`despawn`/`insert`/`remove`、`entity(id)`、`Commands::get_entity`、`apply_deferred`（隐式同步点）。
- **观察点**：在同一系统里 `spawn` 之后立刻 `query` 查不到——命令还没落地；把 spawn 和 query 拆成两个系统（或加 `.chain()`）就好了。这个"为什么查不到"是新手的第二大坑。
- **前置**：005、007。

#### 009 `009_resource.rs` — 资源 ✅ 已实现
- **目标**：全局单例数据。
- **核心 API**：`#[derive(Resource)]`、`init_resource`、`insert_resource`、`Res`/`ResMut`、`resource_exists` 运行条件。
- **观察点**：访问未初始化的资源 → 参数获取失败 → 系统被**静默跳过**（不 panic）。用一条 `info!` 验证"系统根本没跑"。
- **前置**：008。

#### 010 `010_time.rs` — 时间与计时器 ✅ 已实现
- **目标**：**只讲时间本身** —— `Time` 怎么读、`Timer` 怎么写、`Stopwatch` 怎么用、怎么暂停与倍速。
- **核心 API**：`Time`（`delta_secs`/`elapsed_secs`）、`Timer`/`TimerMode`/`Stopwatch`、
  `tick`/`just_finished`/`is_finished`/`elapsed`/`remaining`/`fraction`/`duration`/`pause`/`unpause`/`reset`、
  `Time<Real>`/`Time<Virtual>`/`Time<Fixed>`。
- **结构**（实施时定为两段、两个 App，原因见下）：
  1. **第一段**：`Timer` 的三种驱动写法对照（手动累加 / 资源 / 组件），第 3 帧三种同时触发；
     逐帧打印 `Time` 与两个 `Timer` 的完整状态；`Stopwatch` 的 `pause`/`unpause`/`reset`。
  2. **第二段**：`Time<Virtual>` 的暂停与倍速，实测三行对比 `Time` / `Real` / `Virtual` 的 delta。
- **⚠️ 实施时修正**：原规划的观察点是"用 `delta_secs` 和无 `Time` 的每帧加固定值跑同一个移动系统，对比位移"，
  这把重点带到了**移动**上（那是 Transform 的事，见 003），而 `Time<Virtual>` 这个列出的核心 API
  反倒只能写在注释里。现按"每讲只服务本讲主题"（规划原则第 8 条）改成上面这样：
  去掉移动演示，把 `Time<Virtual>` 提升为现场演示。
- **为什么分两段**：`Time<Virtual>` 由 `TimePlugin` 提供，而装了它会**覆盖手动 `advance_by`**
  （实测，见 4.5）—— 想精确控时就不能装插件，想用虚拟时钟就必须装。两个需求互斥，只能分两个 App。
- **前置**：009。

#### 011 `011_input.rs` — 键盘与鼠标输入 ✅ 已实现
- **目标**：**只讲输入本身** —— 输入从哪几个资源来、怎么查、以及"按下"的三种语义。
- **核心 API**：`ButtonInput<KeyCode>` / `ButtonInput<Key>` / `ButtonInput<MouseButton>`、
  `pressed`/`just_pressed`/`just_released`、`AccumulatedMouseMotion`、`AccumulatedMouseScroll`、
  `Window::cursor_position()`、`Camera::viewport_to_world_2d`。
- **观察点**：每秒一行汇总。按住空格时 `just_pressed` 1 次 / `pressed` 60 次 —— 那个 60 就是帧率。
- **⚠️ 实施时修正**：原规划写的是"用 `pressed` 做按一次开一枪会变成每秒 60 发，换成 `just_pressed` + 冷却计时器"，
  方向对，但落地时滑向了**射击教程**（移动方块 + 冷却 + 射速），本讲自己的东西（`Key` vs `KeyCode`、
  鼠标移动量 / 滚轮、光标换算）只剩注释。现按规划原则第 8 条重写：去掉移动与射击叙事，
  把五种输入资源、离散/连续两类形态、以及光标 → 世界坐标换算全部做成**现场演示**。
- **配角标注**：`Time` / `Timer` 只用来把刷屏输入压成每秒一行，注释里明说"细节在 010"；
  相机的玩法（跟随/缩放/分屏）指向 025。
- **前置**：009、010。

#### 012 `012_message.rs` — 消息（0.17 起由 Event 改名） ✅ 已实现
- **目标**：系统间解耦通信。
- **核心 API**：`#[derive(Message)]`、`app.add_message::<T>()`、`MessageWriter::write`、`MessageReader::read`、`MessageMutator`、`MessageReader` 的双缓冲语义。
- **观察点**：
  1. 消息**只活两帧**（双缓冲）：写在第 N 帧 → 第 N、N+1 帧可读 → 第 N+2 帧消失，没人读就自己丢弃。
  2. **Bevy 不会自动把 `MessageWriter` 排在 `MessageReader` 之前** —— 见 4.5 的实测记录。
     原规划此处曾写"引擎自动保证写者在前"，**该说法是错的**，已按实测更正。
     不声明顺序时读者会排在写者前面，表现为"晚一帧"；必须 `.chain()` / `.after()`。
- **前置**：009、010。

---

### 阶段三 · 事件与关系（013–015）

#### 013 `013_observer.rs` — 观察者与实体事件 ✅ 已实现
- **目标**：比消息更精确的"针对某个实体"的响应式编程。
- **核心 API**：`#[derive(EntityEvent)]`、`.observe(|on: On<E>| ..)`、`commands.trigger(..)`、内置生命周期事件（`Add`/`Insert`/`Remove`）、`Observer` 冒泡传播。
- **观察点**：同一个自定义事件，用**全局消息**广播 vs 用**观察者**只发给目标实体，后者不会误伤别的实体；观察者能拿到被触发实体本身。
- **前置**：012。

#### 014 `014_hierarchy.rs` — 父子层级 ✅ 已实现
- **目标**：把实体组织成树，理解变换继承。
- **核心 API**：`ChildOf`、`Children`、`children![..]` 宏、`with_children`、`add_child`/`remove_child`、`Transform` 传播。
- **观察点**：父实体旋转/缩放，子实体跟着动（`Transform` 继承）；子实体的 `Transform.translation` 是**相对**父的局部坐标。
- **前置**：008。

#### 015 `015_change_detection.rs` — 变更检测 ✅ 已实现
- **目标**：只处理变化过的数据，这是 Bevy 性能模型的基石。
- **核心 API**：`Changed<T>`、`Added<T>`、`Res::is_changed()`、`Ref<T>`（`is_added`/`is_changed`/`last_changed`）、`RemovedComponents`。
- **观察点**：不加过滤的每次遍历 vs 加 `Changed<T>` 后只在真正改动时命中；在仅 1/60 帧改动的场景下命中次数差异巨大（用计数器打印验证）。
- **前置**：009、013。

---

### 阶段四 · 组织与状态（016–019）

#### 016 `016_plugin.rs` — 插件 ✅ 已实现
- **目标**：把一组系统/资源/事件打包成可复用单元。
- **核心 API**：`impl Plugin`（`build`/`finish`/`cleanup`）、`PluginGroup`、`PluginGroupBuilder`、`DefaultPlugins.set(..)`、`disable::<T>()`。
- **观察点**：把 016 之前所有讲的系统收进自定义插件，`main` 里只剩插件列表；再演示 `DefaultPlugins` 的插件组结构（用 `RUST_LOG` 打印加载了哪些插件）。
- **前置**：004、012。

#### 017 `017_module/` — 多文件工程结构（目录式 example） ✅ 已实现
- **目标**：从"单文件例子"过渡到"能长大的工程"。
- **形式**：本讲是**目录式 example**：`examples/017_module/main.rs` + `player.rs` + `enemy.rs` + `common.rs`。Cargo 会把 `examples/<name>/main.rs` 自动识别为名为 `<name>` 的 example（上游 bevy 仓库即用此模式）。
- **核心 API**：`mod`/`pub use`、`prelude.rs` 汇总导出、`plugin` 与 `module` 的配合。
- **观察点**：同名的 `PlayerPlugin` 分布在各自文件里，`main.rs` 只做组装；对比 016 的单文件版本，改动量集中在哪。
- **前置**：016。

#### 018 `018_states.rs` — 状态机 ✅ 已实现
- **目标**：菜单 / 游戏中 / 暂停 / 结算 的流程控制。
- **核心 API**：`#[derive(States)]`、`init_state::<T>()`、`OnEnter(S)`/`OnExit(S)`、`in_state(S)` 运行条件、`NextState<S>::set(..)`、`StateTransition` 调度。
- **观察点**：`OnEnter` 只在进入的那一帧跑；`in_state` 的系统在状态外完全不执行（不是"执行了但提前 return"）。
- **前置**：016。

#### 019 `019_state_advanced.rs` — 状态进阶 ✅ 已实现
- **目标**：复杂流程下的状态组合。
- **核心 API**：`SubStates`（依附父状态存在）、`ComputedStates`（由其他状态推导）、`DespawnOnExit(S)`/`DespawnOnEnter(S)`/`DespawnWhen::new(..)`（0.19 的自动清理，旧名 `StateScoped` 已废弃）。
- **观察点**：`DespawnOnExit` 挂上去之后，切状态时实体**自动**销毁，不用手写清理系统——顺手对比"手写 `OnExit` 清理"的写法有多啰嗦。
- **前置**：018。

---

### 阶段五 · 2D 表现层（020–026）

> 从本阶段起需要 `assets/` 目录。资产准备方案见第 4 节。

#### 020 `020_asset.rs` — 资产加载 ✅ 已实现
- **目标**：从磁盘加载图片，并正确处理"还没加载完"。
- **核心 API**：`AssetServer::load`、`Handle<T>`、`Assets<T>::add`、`LoadState`/`AssetServer::is_loaded_with_dependencies`、热重载。
- **观察点**：`load` 是**异步**的，句柄立刻返回但图可能还没到；不加加载判断会看到一帧空白/默认贴图。
- **前置**：008。

#### 021 `021_atlas_animation.rs` — 图集与帧动画 ✅ 已实现
- **目标**：用一张雪碧图做逐帧动画。
- **核心 API**：`TextureAtlasLayout::from_grid(..)`、`Assets<TextureAtlasLayout>`、`TextureAtlas { layout, index }`、`Sprite::from_atlas_image(..)`、用 `Timer` 驱动 `index` 递增。
- **观察点**：`TextureAtlasLayout` 和 `Sprite` 是分开的——布局可被多个精灵共享，改 `index` 就换帧。这正好复用 010 的 `Timer`。
- **前置**：010、020。

#### 022 `022_ui.rs` — UI 布局 ✅ 已实现
- **目标**：画出 HUD：血条、分数、文字。
- **核心 API**：`Node`（宽高/padding/margin/flex 布局/`PositionType`）、`Text`/`TextFont`/`TextColor`、`ImageNode`、`BackgroundColor`、`BorderColor`、`ZIndex`、`UiTargetCamera`。
- **观察点**：UI 用的是 flex 布局，坐标原点在**左上角**——和 003 里精灵的"中心 + y 向上"正好相反，这个反差最容易错。
- **前置**：003、009。

#### 023 `023_ui_interaction.rs` — UI 交互 ✅ 已实现
- **目标**：按钮能点、能被鼠标悬停。
- **核心 API**：`Button`、`Interaction`（`None`/`Hovered`/`Pressed`）、`Changed<Interaction>`、UI 上的 `observe`（`Pointer<Click>`/`Pointer<Over>`）、`BackgroundColor` 反馈。
- **观察点**：两种写法对照——**轮询** `Changed<Interaction>` vs **观察者** `On<Pointer<Click>>`；后者是 0.19 的推荐路径，也更省系统。
- **前置**：015、018、022。

#### 024 `024_audio.rs` — 音频 ✅ 已实现
- **目标**：背景音乐 + 音效。
- **核心 API**：`AudioPlayer::new(handle)`、`PlaybackSettings`（`loop`/音量）、`AudioSink`（运行时控制播放/暂停/音量）、`Volume`。
- **观察点**：音效和 BGM 的区别在于"要不要拿到 `AudioSink` 去控制"——音效放完即弃，BGM 要留句柄。
- **前置**：012、020。

#### 025 `025_camera.rs` — 相机 ✅ 已实现
- **目标**：跟随、缩放、分屏。
- **核心 API**：`Camera2d`、`Projection`（`OrthographicProjection` 的 `scale`/`ScalingMode`）、相机作为子实体跟随、`Viewport`/多相机、`Camera::world_to_viewport`。
- **观察点**：把相机挂成玩家的**子实体** → 零代码跟随（呼应 014）；对比手写"每帧 lerp 到玩家位置"的写法，后者能做平滑跟随。
- **前置**：014、011。

#### 026 `026_gizmos.rs` — Gizmos 调试绘制 ✅ 已实现
- **目标**：把不可见的东西画出来（碰撞盒、速度向量、路径）。
- **核心 API**：`Gizmos`（`line_2d`/`circle_2d`/`rect_2d`/`arrow_2d`）、`GizmoConfigStore`（线宽/开关）、`TransformGizmo`。
- **观察点**：Gizmos 只画一帧、不留痕，所以必须在 `Update` 里每帧重画；它和 `Sprite` 走的是两套渲染路径，不受精灵层级影响。
- **前置**：007、025。

---

### 阶段六 · 3D 与渲染（027–030）

#### 027 `027_3d_basic.rs` — 3D 基础 ✅ 已实现
- **目标**：把 2D 的直觉搬到 3D。
- **核心 API**：`Camera3d`、`Mesh3d(handle)`、`MeshMaterial3d::<StandardMaterial>(handle)`、`Assets<Mesh>::add(Cuboid::new(..))`、`PointLight`/`DirectionalLight`（含阴影）、`Transform` 的 3D 旋转。
- **观察点**：2D 里"原点中心 + y 向上 + z 定层序"，3D 里 z 变成**纵深**，`Camera3d` 默认朝 `-z` 看——同一套 `Transform` 组件语境的差异。
- **前置**：003、025。

#### 028 `028_3d_gltf.rs` — 加载 glTF 模型 ✅ 已实现
- **目标**：用美术做好的模型，而不是代码拼几何体。
- **核心 API**：`WorldAssetRoot(handle)`（**0.19 改名，旧名 `SceneRoot`**）、`AssetServer::load` + `GltfAssetLabel::Scene(0)`、`Scene`/`SceneInstance`、glTF 动画播放（`AnimationPlayer`/`AnimationGraph`）。
- **观察点**：glTF 场景**不是**一个实体，而是一棵被实例化出来的实体树；想改其中的材质要用 `SceneInstance` 就绪后再查，不能立刻 `Query`。
- **前置**：020、027。

#### 029 `029_3d_picking.rs` — 3D 拾取与相机控制 ✅ 已实现
- **目标**：鼠标点到 3D 物体上。
- **核心 API**：`MeshPickingPlugin`（**非默认插件，必须手动加**；UI/Sprite 拾取则是默认开启的）、`Pointer<Over>`/`Pointer<Click>`/`Pointer<Drag>`、射线、`Camera::viewport_to_world`、围绕点旋转的轨道相机。
- **观察点**：不加 `MeshPickingPlugin` 时观察者**完全不触发**（不报错），这是最难查的一类问题。
- **前置**：013、027。

#### 030 `030_shader.rs` — 自定义着色器 ✅ 已实现
- **目标**：写出自己的 WGSL 材质。
- **核心 API**：`Material` trait + `AsBindGroup`、`ShaderRef`、`MaterialPlugin`、`embedded_asset!`、uniform 传参、`example.com` 的 WGSL 入口约定。
- **观察点**：改 WGSL 存盘 → 热重载即时生效（不用重编译 Rust）；Rust 侧只负责传 uniform。
- **前置**：020、027。

---

### 阶段七 · 工程质量与综合（031–033）

#### 031 `031_capstone_game.rs` — 综合 2D 小游戏
- **目标**：把 001–026 全部串起来，做成一个真能玩的游戏。
- **要求**：玩家移动（010/011）、射击带冷却（010/011）、敌人成批刷新（008）、碰撞与计分（007/009/012）、菜单与暂停（018/019/023）、音效（024）、相机跟随（025）、血条 HUD（022）、调试 Gizmos（026）。
- **观察点**：本讲重点不是新 API，而是**结构**——所有东西都进插件（016/017），系统顺序明确标注（004），资源划分清晰（009）。
- **前置**：001–026 全部。

#### 032 `032_ecs_test.rs` — ECS 测试
- **目标**：让游戏逻辑可以被自动化验证，不再靠手点。
- **形式**：`#[test]` 用例写在 example 内（或抽到 `tests/`），用 `App::new()` 无窗口驱动。
- **核心 API**：`app.update()` 手动推进帧、`World::resource::<T>()` 断言、`world.query::<..>()`、`App::add_systems` 后不 `.run()`、`Time::advance_by(Duration)` 控制时间、`ButtonInput::press` 模拟输入。
- **关键技巧**（本项目已实测）：无 `DefaultPlugins` 时必须手动 `.init_resource::<ButtonInput<KeyCode>>()` 和 `.init_resource::<Time>()`；`just_pressed` 不会自动清除，跨帧要手动 `.clear()`；时间用 `Time::advance_by(Duration::from_secs_f64(1.0/60.0))` 推进。
- **观察点**：**写一个会失败的反向用例**——把"开火冷却"的判断改成恒真，测试应报出 60 发/秒；说明测试真的在鉴别问题，而不是永远绿。
- **前置**：031。

#### 033 `033_diagnostics_release.rs` — 剖析与发布
- **目标**：知道卡在哪，并把它打包出去。
- **核心 API**：
  - 诊断：`FrameTimeDiagnosticsPlugin`、`EntityCountDiagnosticsPlugin`、自定义 `Diagnostic`、`LogDiagnosticsPlugin`、`bevy_dev_tools` 的 FPS 悬浮窗。
  - 计数与日志：`tracing`（`info!`/`warn!`/`error!`）、`RUST_LOG` 分级、`LogPlugin` 配置。
  - 发布：`[profile.release]`（`lto`/`codegen-units`/`opt-level`）、`[profile.dev] opt-level` 与依赖预优化（本项目 `Cargo.toml` 已配）、体积裁剪、动态链接特性开关。
- **观察点**：同一场景在 dev（`opt-level = 1` + 依赖 `opt-level = 3`）与 release 下的帧时间差异；以及 `cargo run --release` 与 `cargo run` 的编译耗时/运行耗时权衡。
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

### 4.5 0.19.1 实测行为（不是改名，是语义）

以下几条都是**实跑验证过的**，写后续讲次时直接采信，不要再凭印象：

| 事实 | 实测结论 | 影响讲次 |
|---|---|---|
| 系统参数校验失败 | **分两类**：① `Single` / `Option<Single>` / `Populated` 条件不满足 → **静默跳过整个系统**（无输出、无报错、不 panic）；② 缺 `Res<T>` / `ResMut<T>` → **直接 panic**（`Resource does not exist`，exit 101）。错误处理器是 `FallbackErrorHandler`（0.19 由 `DefaultErrorHandler` 改名），默认 `match_severity`，`DefaultPlugins` **不会**改它 | 006 |
| 同一系统内两个查询的借用冲突 | **运行时 panic**，错误码 `B0001`，**不是编译错误** —— `cargo check` 照样通过。修法：`Without<T>` 造互斥查询，或 `ParamSet` 合并 | 007 |
| `Query` 遍历顺序 | **不是生成顺序**。Bevy 按原型(archetype)分组存储，遍历**逐组**进行，组间先后与生成时间无关，不可依赖 | 006 |
| `Commands` 何时落地 | `ScheduleBuildSettings::auto_insert_apply_deferred` **默认为 `true`**：`.chain()` / `.after()` 会在"有延迟参数的系统 → 读相关数据的系统"这条边上自动插 `ApplyDeferred`。不声明顺序则落地时机不定（实测同帧内可查到 0 个）。`chain_ignore_deferred()` 只排序、不插同步点 | 008 |
| `Has<T>` 的位置 | 它是**取数项**（写在元组里、返回 `bool`），**不是**过滤器；`With` / `Without` / `Or` 才是过滤器（写在第二个参数位置） | 007 |
| `Time` / `Time<Real>` / `Time<Virtual>` | 系统里 `Res<Time>` 拿到的就是**虚拟时钟**。实测（每帧真实流逝 ~100ms）：正常 Time=Real=Virtual≈100ms；`pause()` 后 Time=Virtual=**0** 而 Real 仍 ≈100ms；`set_relative_speed(3.0)` 后 Time=Virtual≈**300ms**、Real≈100ms。暂停/慢动作只需改 `Time<Virtual>` | 010 |
| 手动推进时间 | 不装 `TimePlugin` 时 `init_resource::<Time>()` + `Time::advance_by(dur)` 可精确控制（delta 即所给值）；**装了 `TimePlugin` 则手动 `advance_by(Time<Real>)` 会被插件覆盖**（实测无效） | 010 / 012 |
| 无窗口多帧推进 | 直接 `app.update()` 循环即可，**不需要** `run()`，也不会因"插件还在构建"而 panic（实测） | 010 / 012 |
| 消息的生命周期 | **双缓冲，只活两帧**：写在第 N 帧 → 第 N、N+1 帧可读 → 第 N+2 帧消失。实测一条只写一次的消息：读者排在前时第 1 帧读 0、第 2 帧读 1、第 3 帧读 0 | 012 |
| 消息的读写顺序 | **Bevy 不会自动把 `MessageWriter` 排在 `MessageReader` 之前**（与常见的相反说法不符，实测读者会先跑）。不声明顺序就晚一帧生效，必须 `.chain()` / `.after()` | 012 |
| `init_resource` 的语义 | **资源已存在时不覆盖**（实测 `insert(42)` 后再 `init` 仍是 42）；`insert_resource` 会覆盖，所以"重置资源"要用后者 | 009 |
| `Stopwatch` 的位置 | **不在 prelude 里**，需要 `use bevy::time::Stopwatch;` | 010 |
| 几个不在 prelude 的类型 | `Stopwatch` → `bevy::time::Stopwatch`；`AccumulatedMouseMotion` / `AccumulatedMouseScroll` → `bevy::input::mouse::{..}`；`Key` → `bevy::input::keyboard::Key`。用到时都要单独 `use` | 010 / 011 |
| 状态机需要插件 | `StatesPlugin` 是状态机的前提（`DefaultPlugins` 内置）。只用 `LogPlugin` 时必须在 `init_state` 之前加上，否则 panic：`The StateTransition schedule is missing` | 018 / 019 |
| 状态切换的生效时机 | `NextState::set(..)` 只是请求，切换发生在**帧末**的 `StateTransition` 调度里 —— 同一帧内 `State<S>` 仍是旧值，`OnExit(旧)` 先于 `OnEnter(新)`。`OnEnter` 里的 `commands.spawn` 还要再等一个同步点，所以"请求 → 看见成果"隔一两帧是正常的 | 018 / 019 |
| 子状态不存在时读不到 | 父状态不满足时子状态**整个不存在**，`Res<State<子状态>>` 会 panic；必须用 `Option<Res<State<..>>>`。`NextState<子状态>` 同理 | 019 |
| `init_state` 会触发一次 `OnEnter` | 即使"没有发生切换"，初始状态的 `OnEnter` 也会跑一次 —— 初始化的准备可以放心放进去 | 018 |
| 目录式 example | `examples/<名字>/main.rs` 会被 Cargo 自动识别为名为 `<名字>` 的 example，同级子模块文件**不会**变成独立目标。已在本仓库用 `cargo metadata` 实测确认 | 017 |
| `assets/` 根怎么找 | 依次是 `BEVY_ASSET_ROOT` → `CARGO_MANIFEST_DIR`（`cargo run` 自动设置）→ **可执行文件所在目录**。所以直接跑 `target/debug/examples/xxx.exe` 会去 exe 旁边找 `assets/`，**全部加载失败**（已实测，报 `Path not found: ...\target\debug\examples\assets\...`）。要么用 `cargo run`，要么先设 `BEVY_ASSET_ROOT` | 020 |
| 加载失败不会 panic | `LoadState::Failed(..)` 只是永远不 `Loaded`，程序照常跑 —— 典型表现是"东西不出现但什么都不报"。排查第一步永远是打印 `get_load_state(..)` | 020 |
| `file_watcher` 不在默认 feature | `default = ["2d","3d","ui","audio"]`（只有四个）；热重载在 `dev = ["debug","bevy_dev_tools","file_watcher"]` 里。实测本项目直接改 PNG 等 10 秒也无 `AssetEvent::Modified`。开启还需额外依赖 `notify-debouncer-full` | 020 |
| 默认字体无中文字形 | 内置字体是 `FiraMono-subset.ttf`（拉丁字母）。UI 里写中文会刷 `ICU4X data error: No segmentation model for complex script: Chinese/Japanese` 且画面上出不来字。必须自带中文字体（`system_font_discovery` + `Font::from_system`，或放进 `assets/fonts/`）。**注意 `println!` 不受影响** —— 终端渲染与 Bevy 渲染是两回事 | 022 |
| 默认音频只支持 OGG | `audio = ["bevy_audio", "vorbis"]` —— 不含 `wav`。播 `.wav` 会在解码处 **panic**（`bevy_audio/src/audio_source.rs:101`，`unwrap()` on Err）。要 WAV 得开 `wav` feature（引入 `hound`）；全套用 `audio-all-formats`。`Pitch` 程序合成音不需要任何文件与 feature | 024 |
| 又一批不在 prelude 的类型 | `Viewport` → `bevy::camera::Viewport`；`PlaybackMode` / `Volume` → `bevy::audio::{..}`；`AccumulatedMouseScroll` → `bevy::input::mouse::..`；`GizmoConfigStore` / `DefaultGizmoConfigGroup` → `bevy::gizmos::config::{..}` | 024–026 |
| `observe(..)` 不是 bundle | 它是 `EntityCommands` 的方法（也导出一个 `EntityCommand`），**不能**写进 `children![..]`。要挂观察者的子实体得先 `spawn` 再 `.observe(..)` | 023 |
| UI 交互不要用 `Interaction::Pressed` 当"点击" | `Pressed` 在按住期间**持续为真**（与 011 的 `pressed` 同理）。要单次触发，用 `Changed<Interaction>` 过滤，或改用 `On<Pointer<Click>>` 观察者 | 023 |
| 3D 网格拾取必须手动装插件 | `MeshPickingPlugin` **不是**默认插件（UI / 2D 精灵拾取才是）。不加时观察者完全不触发，且 **stderr 无任何 WARN/ERROR**（已实测：临时注释该行后程序照常启动、stderr 为空） | 029 |
| glTF 的节点 ≠ 网格实体 | glTF 里带 `Mesh3d` 的是命名节点的**子实体**（名字形如 `Pyramid.PyramidMaterial`）。按节点名直接改材质会**静默失败**。文件 3 个节点实例化出 6 个实体（scene 本身 + 每个图元各一个） | 028 |
| `WorldInstanceReady` 不在 prelude | 要 `use bevy::world_serialization::WorldInstanceReady`。另注意 `WorldAssetRoot` 装的是 `Handle<WorldAsset>`，旧名 `SceneRoot` | 028 |
| 环境光 0.19 改名 | `GlobalAmbientLight` 是**资源**（全局默认）；`AmbientLight` 现在是挂相机上覆盖用的**组件** | 027 |
| 光源开阴影的字段名 | `shadow_maps_enabled`（旧版叫 `shadows_enabled`） | 027 |
| 自定义 shader 运行时校验失败 | `@group` 编号**不能写死**。0.19 里材质 bind group 由引擎按特性动态决定并注入，须写 `@group(#{MATERIAL_BIND_GROUP})`。写死 `2` 会撞到 storage buffer，报 `ResourceBinding { group: 2, binding: 0 } is not available in the pipeline layout` 并 `Quitting the application due to Validation RenderError` | 030 |
| 自定义材质什么都不显示也不报错 | 忘了 `MaterialPlugin::<M>::default()` | 030 |
| `TimerMode::Once` + `is_finished()` | 到点后**每帧都为真**（并非只在到点那一帧），拿它做"触发一次"会变成每帧触发；要用 `just_finished()` | 010 |

两条相关取舍：
- `Query` 命中 0 个**永远不会**导致系统被跳过；只有 `Single` 那一类才会。
- 需要固定遍历次序时，把结果收集成 `Vec` 后显式排序，不要依赖引擎的遍历顺序。

---

## 6. 里程碑

| 里程碑 | 完成标志 | 状态 |
|---|---|---|
| M1 阶段一 | 004 跑完，能口头解释"为什么两个系统写同一资源会串行但顺序不定" | ✅ 已完成（001–004 已落地并实跑验证） |
| M2 阶段二 | 012 跑完，能写出帧率无关的移动 + 冷却射击 | ✅ 已完成（005–012 全部落地并实跑验证） |
| M3 阶段三 | 015 跑完，能用观察者替代消息、说清变更检测省下了什么 | ✅ 已完成（013–015 全部落地并实跑验证） |
| M4 阶段四 | 019 跑完，多文件工程 + 完整状态机可跑 | ✅ 已完成（016–019 全部落地并实跑验证） |
| M5 阶段五 | 026 跑完，2D 表现层齐活（资产/动画/UI/音频/相机/调试） | ✅ 已完成（020–026 全部落地并实跑验证） |
| M6 阶段六 | 030 跑完，能加载 glTF 并写自定义 WGSL 材质 | ✅ 已完成（027–030 全部落地并实跑验证） |
| M7 阶段七 | 033 跑完，有一个可测试、可剖析、可发布的小游戏 | ⏳ |

> 阶段一（001–004）的实施备注：
> - `003_sprite.rs` 是 `Transform` 首次出现的地方，顺带把**坐标系与手性**讲透（含各引擎对照图）。
> - `001_app.rs` 用 `println!` 并补充了「为什么只跑一帧」的源码级解释（附 `bevy_app` 源码位置）。
> - 窗口与相机独立成 `002_window.rs`；时间相关的内容按依赖顺序推迟到 010 讲。

> 阶段二（005–008）的实施备注：
> - **005 是可视化的**（窗口 + 精灵），用"两个方块动、一个不动"直观证明"查询按组件组合筛选"；
>   006/007/008 是**无窗口纯控制台**的，因为它们讲的是查询语义与命令时序，文本比画面清楚。
>   这个"概念可视化、机制文本化"的分工是实施时定下的，后续讲次沿用。
> - `006_query.rs` 新增了原规划没有的两个知识点：① 取不到数据时 Bevy 的三种反应；
>   ② **遍历顺序 ≠ 生成顺序**（原型分组存储）。后者是实跑时发现的，原规划完全没提。
> - `007_query_filter.rs` 修正了原规划的一个说法：借用冲突是**运行时 panic（B0001）**而非编译错误。
> - `008_commands.rs` 用一条六步 `.chain()` 流水线演示命令延迟与自动同步点，
>   并在注释里回指 005 的 `Startup`（同一套同步点规则）。

> 阶段二（009–012）的实施备注：
> - **009 与 010 是无窗口控制台，011 是窗口交互，012 是无窗口逐帧推进。**
>   010/012 都用手动 `app.update()` 逐帧推进（见 4.5 的"无窗口多帧推进"），
>   因为时间与消息都是**跨帧**机制，帧长不可控就看不清。
> - `010_time.rs` 落地了 `Timer` 的三种驱动写法（手动累加 / 资源 / 组件）对照，
>   并用"前 5 帧 100ms、后 5 帧 50ms"实测出帧率相关写法的 33% 误差。
> - `010` 初版让 `Time<Virtual>` 只出现在注释里（因为它需要 `TimePlugin`，
>   而该插件会覆盖手动推进的时间）；返工时已把它提升为**现场演示**，
>   办法是分成两段、两个 App —— 一段精确控时，一段用插件。
> - `011_input.rs` 把经典 bug 量化了。实测（固定 60 帧 × 1/60 秒）：
>   `just_pressed` 触发 **1** 次、`pressed` 触发 **60** 次 —— 那个 60 就是帧率。
> - `012_message.rs` 实测纠正了一条流传很广的说法：
>   **Bevy 并不会自动把 `MessageWriter` 排在 `MessageReader` 之前**。
>   同帧同类型的两个读者，有序的读到本帧、无序的读到上一帧。消息**只活两帧**。
> - **返工记录（010 / 011）**：初版把"移动方块 / 射击冷却"当成了主线，本讲自己的 API 反倒靠边。
>   按"每讲只服务本讲主题"改写：010 去掉移动演示、把虚拟时钟提为现场演示；
>   011 去掉移动与射击叙事、把五种输入资源与两类形态全做成演示。
>   相应地补了规划原则第 8 条，010 / 011 的讲次规格也已改写。

> 阶段三（013–015）的实施备注：
> - 013 用两帧输出同时证明了四件事：**生命周期事件自动触发**（`On<Add, Health>`，没写 trigger）、
>   **精准投递**（三个敌人都被全局观察者看到，但 `[专属观察者]` 只在敌人A 身上出现，
>   敌人C 全程没出现）、**链式反应**（击中 → 掉血 → 阵亡 → 销毁）、
>   以及 `commands.trigger` 的**延迟性**（`[触发]` 两行总排在观察者输出之前）。
> - 014 用"太阳 → 行星 → 卫星 + 彗星"演示变换继承：**没有任何一行代码在移动子实体**。
>   `children![..]` 与 `add_child` 两种建层级方式都覆盖；树结构靠
>   `Children` 递归 + `Without<ChildOf>` 找根打印出来。
> - 015 的核心是一个**反例**：敌人B 的值从未改变，却每帧都被 `Changed` 命中；
>   敌人A 只在真正扣血时命中。由此讲清"`Changed` 判的是可变访问、不是值不同"，
>   并引出 `set_if_neq`。
> - 013 规格里的"观察者冒泡"（沿 `ChildOf` 向上传播）**没有现场演示** ——
>   它是层级的能力，放进 013 会抢戏（规划原则第 8 条），故留在文末说明并指向 014。

> 阶段四（016–019）的实施备注：
> - 016 初版漏了跨插件的顺序声明，结果 `Score = 20` **加了却永远没打印出来**
>   （整个演示只有 3 帧，错过就没了）。修法是引入 `GameSet` + `configure_sets`
>   集中声明组间顺序 —— 这正好把 `SystemSet` 从"004 里提过一句"提升为**插件的必要配套**。
> - 017 是本项目唯一的**目录式 example**，此前无法确认该模式在本仓库可行。
>   实施时已实测：`cargo metadata` 显示 `017_module` 的来源是
>   `examples/017_module/main.rs`，同级子模块未被识别为独立目标。
>   017 的输出与 016 **逐字相同** —— 这正是本讲要证明的"只是搬了家"。
> - 018 用计时器自动循环三个状态（不依赖输入），并明确了三条最容易搞混的规则：
>   `set()` 是请求不是执行、`OnEnter`/`OnExit` 只跑一次、状态是资源不是组件。
>   实测还发现 **`init_state` 也会触发一次 `OnEnter`**，已写进例子。
> - 019 实测踩到两个坑，都写进了例子：① 状态机需要 `StatesPlugin`，
>   只用 `LogPlugin` 会在 `init_state` 处 panic；② 读子状态必须用
>   `Option<Res<State<..>>>`，因为父状态不满足时它**整个不存在**。
>   另外状态切换在帧末生效，所以本讲用了 6 帧而不是 5 帧，否则看不到
>   `DespawnOnExit` 的清理效果。
> - 019 规格里的 `ComputedStates` **未现场演示**（与 `SubStates` 的演示目标重叠，
>   按规划原则第 8 条留在文末说明）。
> 阶段五（020–026）的实施备注：
> - 本阶段第一次需要 `assets/`。按规划采用**全程序化生成**：新增 `tools/make_assets.py`
>   （用捆绑的 Python + Pillow）生成 `logo.png` 与 6 帧雪碧图 `runner.png`。
>   音频素材最终**没有生成** —— 原因见下。
> - 实测纠正了三处会导致"照着写跑不起来"的认知，都已写进例子与 4.5：
>   ① **`assets/` 的根会回落到可执行文件目录**，直接跑 exe 必然加载失败（本讲开发时踩到）；
>   ② **`file_watcher` 不在默认 feature**，热重载在本项目根本不生效
>      （初版误以为它是默认开启的，已更正）；
>   ③ **默认音频只支持 OGG**，播 WAV 会直接 panic，而 `wav` / `file_watcher`
>      所需的额外依赖在本机无法下载。
> - 024 因此**改用 `Pitch` 程序合成音**：不需要音频文件、不需要额外 feature，
>   反而把"频率"这个参数暴露出来，比文件名更能说明 `AudioPlayer` 在播什么。
>   文件播放的正确姿势（转 OGG）写在文末。
> - 022 实测发现 **Bevy 内置字体不含中文字形**：UI 里写中文会刷 ICU4X 警告且显示不出来。
>   本讲 UI 文本因此统一改成 ASCII，并把"要自带中文字体"写成例子里的重点提示。
> - 025 是本阶段唯一"没有输入也能看"的一讲：用 `Viewport` 把同一份世界画在左右两半，
>   左相机可滚轮缩放 —— 一眼看出 `scale` 是"视野倍数"而不是"物体倍数"。
> 阶段六（027–030）的实施备注：
> - 本轮又纠正了四处会让"照着老教程写跑不起来"的 0.19 变更，全部写进例子与 4.5：
>   ① 环境光拆成 `GlobalAmbientLight`（资源）+ `AmbientLight`（组件）；
>   ② 光源阴影字段改名 `shadow_maps_enabled`；
>   ③ **glTF 的节点与网格是两个实体**（本讲第一版按节点名改材质，静默失败，
>      跑起来看输出才发现）；
>   ④ **WGSL 里材质 bind group 的编号不能写死** —— 初版写 `@group(2)`，
>      编译正常但一跑就 `Validation Error` 并退出。引擎自己的 `pbr_bindings.wgsl`
>      用的是预处理器变量 `#{MATERIAL_BIND_GROUP}`，跟着改才对。
> - 028 的 glTF 素材由 `make_pyramid_gltf()` **手写生成**（三节点树、18 顶点、
>   6 三角形、base64 内嵌 432 字节缓冲），不依赖任何 3D 工具 ——
>   顺带把顶点、法线、绕序这些平时被工具藏起来的东西摊开了。
>   glTF 动画**未现场演示**：0.19 播动画还需 `AnimationGraphHandle`（一套动画图），
>   主体是"动画系统"而非"加载 glTF"，按原则第 8 条留给后续；
>   素材里也没有动画，注意力集中在"场景 = 实体树"。
> - 029 的交互部分**机器验证不了**（需要真实鼠标），已如实标注；但"不加插件会静默失效"
>   这一条用 A/B（临时注释插件行）实测确认了 stderr 为空。
> - 030 的热重载**在本项目不可用**（`file_watcher` 属 `dev` 组、开启需下载依赖），
>   文末已如实说明，并给出 `embedded_asset!` 的替代路径。
