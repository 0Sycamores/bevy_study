# legacy_examples —— 旧版例子（存档，**不再是 cargo 目标**）

这里放的是**课程重排之前的旧例子**。它们已从 `examples/` 移出，
因此 `cargo check` / `cargo run --example` **都不再涉及它们**，文件只作参考。

## 为什么留着

按 [`CURRICULUM.md`](../CURRICULUM.md) 的规划，这些例子的内容会被吸收进新的讲次，
但对应讲次**还没写**。现在就删会让课程暂时缺掉「输入 / 资源 / 消息」这几块，所以先存档。

## 清单与去向

| 文件 | 主题 | 去向 | 状态 |
|---|---|---|---|
| `009_plugin.rs` | 插件、`PluginGroup`、用 `Without` 消除查询冲突 | → 新 016（`Without` 那部分已在 007 讲过） | 待吸收 |
| `010_game.rs` | 综合小游戏 | → 新 031 `031_capstone_game.rs` | 待吸收 |

> 已经吸收完毕、随后删除的有三个：
> `006_input.rs`（→ 011）、`007_resource.rs`（→ 009）、
> `008_message.rs`（→ 012 讲消息、013 讲观察者，两半都讲完了）。
> 需要它们的话从 git 历史里取，或用 `git show <基线>:examples/008_message.rs`。

## 使用注意

- **它们不再被编译。** Bevy 升级或仓库结构调整后，这些文件可能失效而**没有任何提示**。
  需要确认还能不能跑，可以临时移回：`git mv legacy_examples/006_input.rs examples/`，
  或者 `git mv` 之后 `cargo run --example 006_input`。
- 重排前的**完整基线**（还包括已删除的 `001_hello.rs`、`002_sprite.rs`、`003_system.rs`、
  `004_component.rs`、`005_query.rs`）保存在 git 提交 `21b0c91` 里。
  取出任意一个：`git show 21b0c91:examples/001_hello.rs`。
- 新课程在 [`../examples/`](../examples/)，索引见 [`../README.md`](../README.md)。
