use bevy::prelude::*;

fn hello_system() {
    println!("hello bevy!")
}

fn count_system(time: Res<Time>) {
    println!("Delta time: {:.4}s", time.delta_secs());
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, (hello_system, count_system))
        .run();
}
