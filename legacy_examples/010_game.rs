use bevy::prelude::*;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Bullet {
    speed: f32,
}

#[derive(Component)]
struct ScoreDisplay;

#[derive(Message)]
struct BulletFired {
    position: Vec2,
}

#[derive(Resource, Default)]
struct Score {
    total: u32,
}

/// 开火冷却。没有它的话，`keyboard.pressed` 在按住时每帧都为真，
/// 就变成每秒 60 发子弹。
#[derive(Resource)]
struct FireCooldown(Timer);

fn setup_score_display(mut commands: Commands) {
    commands.spawn((
        Text::new("Score: 0"),
        TextFont {
            font_size: FontSize::Px(32.0),
            ..Default::default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..Default::default()
        },
        ScoreDisplay,
    ));
}

fn update_score_display(
    score: Res<Score>,
    mut score_display: Query<&mut Text, With<ScoreDisplay>>,
) {
    if score.is_changed() {
        for mut text in &mut score_display {
            text.0 = format!("Score: {}", score.total);
        }
    }
}

fn setup_common(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_player(mut commands: Commands) {
    commands.spawn((
        Player,
        Sprite::from_color(Color::srgb(0.2, 0.6, 1.0), Vec2::new(50.0, 50.0)),
        Transform::from_xyz(0.0, -300.0, 0.0),
    ));
}

fn setup_enemy(mut commands: Commands) {
    commands.spawn((
        Enemy,
        Sprite::from_color(Color::srgb(1.0, 0.3, 0.3), Vec2::new(40.0, 40.0)),
        Transform::from_xyz(100.0, 200.0, 0.0),
    ));
}

fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player: Query<&mut Transform, With<Player>>,
) {
    let speed = 300.0 * time.delta_secs();
    for mut transform in &mut player {
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

fn shoot(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    player: Query<&Transform, With<Player>>,
    mut cooldown: ResMut<FireCooldown>,
    mut writer: MessageWriter<BulletFired>,
) {
    // 每帧推进冷却；只有冷却结束且空格被按住时才开火
    cooldown.0.tick(time.delta());
    if !keyboard.pressed(KeyCode::Space) || !cooldown.0.is_finished() {
        return;
    }
    let Ok(player_transform) = player.single() else {
        return;
    };
    writer.write(BulletFired {
        position: player_transform.translation.xy(),
    });
    cooldown.0.reset();
}

fn spawn_bullet(mut reader: MessageReader<BulletFired>, mut commands: Commands) {
    for bullet in reader.read() {
        commands.spawn((
            Sprite::from_color(Color::srgb(1.0, 0.9, 0.0), Vec2::new(10.0, 20.0)),
            Transform::from_xyz(bullet.position.x, bullet.position.y, 0.0),
            Bullet { speed: 500.0 },
        ));
    }
}

fn move_bullet(mut bullets: Query<(&mut Transform, &Bullet)>, time: Res<Time>) {
    for (mut transform, bullet) in &mut bullets {
        transform.translation.y += bullet.speed * time.delta_secs();
    }
}

fn cleanup_bullets(bullets: Query<(Entity, &Transform), With<Bullet>>, mut commands: Commands) {
    for (entity, transform) in &bullets {
        if transform.translation.y > 500.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn check_collision(
    bullets: Query<(Entity, &Transform), With<Bullet>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    mut commands: Commands,
    mut score: ResMut<Score>,
) {
    for (enemy_entity, et) in &enemies {
        // 敌人放外层
        for (bullet_entity, bt) in &bullets {
            // 子弹放内层
            if bt.translation.distance(et.translation) < 30.0 {
                commands.entity(bullet_entity).despawn();
                commands.entity(enemy_entity).despawn();
                score.total += 100;
                break; // 这个敌人本帧只计一次
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Score>()
        .insert_resource(FireCooldown(Timer::from_seconds(0.15, TimerMode::Once)))
        .add_message::<BulletFired>()
        .add_systems(
            Startup,
            (setup_common, setup_score_display, setup_player, setup_enemy),
        )
        .add_systems(
            Update,
            (
                // 输入 -> 写消息 -> 读消息建子弹。
                // 消息是双缓冲的：读取者若排在写入者之前，就得等下一帧才读到。
                (shoot, spawn_bullet).chain(),
                // 先移动子弹，再判定碰撞，最后刷新 UI；否则分数显示会慢一帧。
                (move_bullet, check_collision, update_score_display).chain(),
                // 下面这两个和上面的流程没有共享数据，顺序无所谓。
                move_player,
                cleanup_bullets,
            ),
        )
        .run();
}
