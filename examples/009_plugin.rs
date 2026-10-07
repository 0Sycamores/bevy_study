use bevy::{app::PluginGroupBuilder, prelude::*};

#[derive(Resource, Default)]
struct Score {
    total: u32,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

fn setup_common(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_player(mut commands: Commands) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.1, 0.0, 0.1), Vec2::new(20.0, 30.0)),
        Transform::from_xyz(-200.0, 100.0, 0.0),
        Player,
    ));
}

fn setup_enemy(mut commands: Commands) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.1, 0.2, 0.3), Vec2::new(20.0, 20.0)),
        Transform::from_xyz(-100.0, 100.0, 0.0),
        Enemy,
    ));
}

fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let speed = 200.0 * time.delta_secs();
    for mut transform in &mut query {
        if keyboard.pressed(KeyCode::KeyW) {
            transform.translation.y += speed;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            transform.translation.y -= speed;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            transform.translation.x -= speed;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            transform.translation.x += speed;
        }
    }
}

fn move_enemy(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Enemy>>,
) {
    let speed = 200.0 * time.delta_secs();
    for mut transform in &mut query {
        if keyboard.pressed(KeyCode::ArrowUp) {
            transform.translation.y += speed;
        }
        if keyboard.pressed(KeyCode::ArrowDown) {
            transform.translation.y -= speed;
        }
        if keyboard.pressed(KeyCode::ArrowLeft) {
            transform.translation.x -= speed;
        }
        if keyboard.pressed(KeyCode::ArrowRight) {
            transform.translation.x += speed;
        }
    }
}

/// 玩家撞到敌人就得 10 分，并把敌人镜像到对角，方便反复触发。
/// 这里同时用到 `With` / `Without`：只有加入 `Without<Player>`，
/// 借用检查才能证明两个查询访问的 Transform 互不相交（读 vs 写）。
fn award_hit(
    player: Query<&Transform, With<Player>>,
    mut enemy: Query<&mut Transform, (With<Enemy>, Without<Player>)>,
    mut score: ResMut<Score>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    for mut enemy_transform in &mut enemy {
        if enemy_transform.translation.distance(player_transform.translation) < 30.0 {
            score.total += 10;
            let mirrored = -enemy_transform.translation;
            enemy_transform.translation = mirrored;
        }
    }
}

/// `is_changed()` 只在 Score 被修改过的那一帧返回 true，
/// 所以这里不会像直接 println! 那样每帧刷屏。
fn show_score(score: Res<Score>) {
    if score.is_changed() {
        info!("Score: {}", score.total);
    }
}

struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>().add_systems(Update, show_score);
    }
}

struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
            .add_systems(Update, move_player);
    }
}

struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_enemy)
            .add_systems(Update, (move_enemy, award_hit));
    }
}

struct GamePlugin;

impl PluginGroup for GamePlugin {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(PlayerPlugin)
            .add(EnemyPlugin)
            .add(ScorePlugin)
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_common)
        .add_plugins(GamePlugin)
        .run();
}
