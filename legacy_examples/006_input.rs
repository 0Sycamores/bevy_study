use bevy::prelude::*;

#[derive(Component)]
struct Player;

fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let speed = 300.0 * time.delta_secs();
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

fn on_space(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Space) {
        println!("Space Pressed")
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_color(Color::srgb(0.2, 0.3, 0.4), Vec2::new(20.0, 30.0)),
        Player,
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (on_space, move_player))
        .run();
}
