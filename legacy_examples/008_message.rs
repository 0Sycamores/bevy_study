use bevy::prelude::*;

// =========================================================================
// 方式一：Message（缓冲消息，一对多，读取者按顺序消费）
// 适合「一件事发生，多个系统分别响应」。
// 注意：消息是双缓冲的，写入者必须排在读取者之前，否则要等到下一帧才被读到。
// =========================================================================
#[derive(Message, Debug)]
struct PlayerDied {
    score: u32,
}

// =========================================================================
// 方式二：EntityEvent + 观察者（Observer，精确指向某个实体）
// 适合「只关心某个具体实体上发生的事」。
// 与 Message 的区别：观察者绑定在实体上，只接收 event_target 指向它的那一次触发。
// =========================================================================
#[derive(EntityEvent, Debug)]
struct EntityDied {
    entity: Entity,
    points: u32,
}

#[derive(Resource, Default, Debug)]
struct Score {
    total: u32,
}

/// 保存被观察的敌人实体，方便后续对它触发事件。
#[derive(Resource)]
struct Enemy(Entity);

// ------------------------------------------------------------------ 消息通路

/// 按空格：发送一条 PlayerDied 消息。
fn death_system(keyboard: Res<ButtonInput<KeyCode>>, mut writer: MessageWriter<PlayerDied>) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }
    writer.write(PlayerDied { score: 100 });
    println!("Sent: PlayerDied {{ score: 100 }}");
}

/// 读取消息并结算分数。同一帧内可以有多条消息，用 for 全部消费。
fn score_system(mut reader: MessageReader<PlayerDied>, mut score: ResMut<Score>) {
    for death in reader.read() {
        score.total += death.score;
        println!(
            "Received: score += {} -> total = {}",
            death.score, score.total
        );
    }
}

// ------------------------------------------------------ EntityEvent / 观察者

/// 生成敌人，并在它自己身上挂一个观察者。
/// observe 只监听指向这个实体的 EntityDied，其它实体被击杀不会触发它。
fn spawn_entity(mut commands: Commands) {
    let enemy = commands
        .spawn((Name::new("Enemy"), Transform::default()))
        .observe(|on: On<EntityDied>, mut score: ResMut<Score>| {
            let event = on.event();
            score.total += event.points;
            println!(
                "Entity died! +{} points -> target: {:?}, total = {}",
                event.points, event.entity, score.total
            );
        })
        .id();
    commands.insert_resource(Enemy(enemy));
}

/// 按 K：对敌人触发 EntityDied。
/// commands.trigger 是延迟执行的，所以这行日志会先于上面观察者里的日志打印出来。
fn kill_entity(keyboard: Res<ButtonInput<KeyCode>>, enemy: Res<Enemy>, mut commands: Commands) {
    if !keyboard.just_pressed(KeyCode::KeyK) {
        return;
    }
    commands.trigger(EntityDied {
        entity: enemy.0,
        points: 50,
    });
    println!("Triggered: EntityDied (target = {:?})", enemy.0);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Score>()
        .add_message::<PlayerDied>()
        .add_systems(Startup, spawn_entity)
        // 写入者 (death_system) 必须排在读取者 (score_system) 之前
        .add_systems(Update, (death_system, score_system).chain())
        .add_systems(Update, kill_entity)
        .run();
}
