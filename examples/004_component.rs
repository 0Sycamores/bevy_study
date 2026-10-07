use bevy::prelude::*;

#[derive(Component, Debug)]
struct Player {
    speed: f32,
}

#[allow(dead_code)]
#[derive(Component, Debug)]
struct Health {
    current: f32,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_color(Color::srgb(0.2, 0.6, 1.0), Vec2::new(100.0, 100.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Player { speed: 30.0 },
        Health { current: 100.0 },
    ));
}

fn move_player(mut query: Query<(&Player, &mut Transform)>, time: Res<Time>) {
    for (player, mut transform) in &mut query {
        transform.translation.x += 1.0 * player.speed * time.delta_secs();
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, move_player)
        .run();
}
