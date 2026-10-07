use bevy::prelude::*;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

fn move_player(mut query: Query<&mut Transform, With<Player>>, time: Res<Time>) {
    let speed = 200.0 * time.delta_secs();
    for mut transform in &mut query {
        transform.translation.x += speed;
    }
}

fn print_enemy(query: Query<&Transform, With<Enemy>>) {
    for transform in &query {
        let translation = transform.translation;
        println!("Enemy at: ({:.1}, {:.1})", translation.x, translation.y);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_color(Color::srgb(0.1, 0.2, 0.3), Vec2::new(30.0, 50.0)),
        Transform::from_xyz(-200.0, 0.0, 0.0),
        Player,
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(1.0, 0.2, 0.3), Vec2::new(20.0, 30.0)),
        Transform::from_xyz(-200.0, 0.0, 0.0),
        Enemy,
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (move_player, print_enemy))
        .run();
}
