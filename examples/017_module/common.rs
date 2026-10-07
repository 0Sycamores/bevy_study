//! 跨模块共享的东西。
//!
//! 判断标准：**有两个以上模块要用**，才放进来。
//! 只有一个模块用的东西应该留在那个模块里，否则 `common` 会变成新的"巨型 main"。

use bevy::prelude::*;

/// 插件之间声明先后用的分组。
///
/// 它不属于任何一个插件 —— `ScorePlugin` 用它标记自己的展示系统，
/// `EnemyPlugin` 用它标记自己的结算系统，顺序在 `main.rs` 里统一声明。
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSet {
    /// 敌人推进与结算
    Enemy,
    /// 分数展示
    Score,
}
