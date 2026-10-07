use bevy::prelude::*;

fn setup(mut commands: Commands) {
    // 初始化一个相机
    commands.spawn(Camera2d);
    // 创建一个纯色方块精灵
    commands.spawn(Sprite::from_color(
        Color::srgb(0.2, 0.6, 1.0),
        // Vec2::new(100.0, 100.0),
        vec2(100.0, 1000.0),
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}
