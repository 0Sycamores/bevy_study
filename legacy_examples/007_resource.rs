use bevy::prelude::*;

#[derive(Resource, Default, Debug)]
struct Score {
    total: u32,
    multiplier: f32,
}

fn show_score(score: Res<Score>) {
    println!("Score: {} (x{:.1})", score.total, score.multiplier);
}

fn add_score(mut score: ResMut<Score>) {
    score.total += 100;
    score.multiplier += 0.1;
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Score>()
        .add_systems(Update, (add_score, show_score))
        .run();
}
