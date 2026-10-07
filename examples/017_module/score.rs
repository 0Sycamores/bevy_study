//! 分数功能。
//!
//! 一个功能 = 一个文件 = 一个插件。文件里三样东西：
//! 专属资源、专属系统、（组件）、以及把它们装起来的插件。

use bevy::prelude::*;

use crate::common::GameSet;

/// `Score` 归属本模块。别的模块可以读写它，但"怎么展示"由这里决定。
#[derive(Resource, Default)]
pub struct Score(pub i32);

/// 对外只需要导出这个。
pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>()
            .add_systems(Update, report_score.in_set(GameSet::Score));
    }
}

// ── 下面是本模块的实现细节，故意不加 `pub` ──
// 别的模块看不到 `report_score`，也就不可能绕过插件直接调用它。
// 这既是封装，也让"谁会用到这个函数"这个问题有唯一答案：本文件的插件。

/// 只在分数变过的时候打印（015 讲的 `is_changed`），避免每帧刷屏。
fn report_score(score: Res<Score>) {
    if score.is_changed() {
        println!("   [分数插件] Score = {}", score.0);
    }
}
