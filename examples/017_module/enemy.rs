//! 敌人功能。
//!
//! 注意两个跨模块的引用：`crate::common::GameSet`（共享分组）与
//! `crate::score::Score`（别的模块的资源）。除此之外它不需要知道任何外部细节 ——
//! 尤其**不需要**知道 `ScorePlugin` 里有哪些系统。

use bevy::prelude::*;

use crate::common::GameSet;
use crate::score::Score;

#[derive(Component)]
struct Enemy {
    /// 出场顺序。不靠名字或遍历顺序排序（006 讲的），显式给个号最稳。
    id: u32,
    name: &'static str,
    /// 走到 9 就算抵达终点
    progress: i32,
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_enemies).add_systems(
            Update,
            (move_enemies, check_arrival).chain().in_set(GameSet::Enemy),
        );
    }
}

fn spawn_enemies(mut commands: Commands) {
    commands.spawn(Enemy {
        id: 1,
        name: "小兵甲",
        progress: 0,
    });
    commands.spawn(Enemy {
        id: 2,
        name: "小兵乙",
        progress: 0,
    });
}

fn move_enemies(mut enemies: Query<&mut Enemy>) {
    for mut enemy in &mut enemies {
        enemy.progress += 3;
    }
}

/// 抵达终点就加分并销毁。
///
/// `ResMut<Score>` 就是模块之间的接口：**通过共享资源协作，而不是互相调用**。
/// 代价是"谁拥有 Score"成了需要约定的问题 —— 本讲约定归属 `score.rs`，
/// 并在 `main.rs` 里声明"分数展示要排在敌人结算之后"。
fn check_arrival(
    mut commands: Commands,
    enemies: Query<(Entity, &Enemy)>,
    mut score: ResMut<Score>,
) {
    let mut arrived: Vec<(Entity, &Enemy)> = enemies
        .iter()
        .filter(|(_, enemy)| enemy.progress >= 9)
        .collect();
    arrived.sort_by_key(|(_, enemy)| enemy.id);

    for (entity, enemy) in arrived {
        println!("   [敌人插件] {} 抵达终点，+10 分", enemy.name);
        score.0 += 10;
        commands.entity(entity).despawn();
    }
}
