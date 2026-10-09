//! 024 · 音频
//!
//! 运行：`cargo run --example 024_audio`
//!
//! 新增概念
//!   AudioPlayer        播放某个音频源（是个组件，挂在实体上）
//!   PlaybackSettings   开始播放时的设置：循环、音量、初始暂停/静音
//!   PlaybackMode       `Once` 播一次 / `Loop` 循环 / `Despawn` 播完自动销毁实体 / `Remove` 播完摘掉组件
//!   AudioSink          正在播放的句柄：**运行中**改音量、暂停、快进都靠它
//!   Pitch              程序合成的单音（给个频率 + 时长就有声音，**不需要音频文件**）
//!
//! 使用场景
//!   背景音乐、音效、可暂停的语音；`Pitch` 适合做提示音与程序化音乐
//!
//! 注意：`PlaybackSettings` 的改动**不会**作用于已经在播放的声音 ——
//!       它是"开播参数"，不是"遥控器"。运行中控制必须拿到 `AudioSink`

use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "024 · 音频：自动演奏 + 空格音效 + M 静音".into(),
                resolution: (960, 640).into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<Muted>()
        .insert_resource(Melody::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (play_melody, play_sfx_on_space, toggle_mute_on_m))
        .run();
}

#[derive(Resource, Default)]
struct Muted(bool);

/// 程序化"背景音乐"的状态：一个计时器 + 当前走到第几个音。
#[derive(Resource)]
struct Melody {
    timer: Timer,
    step: usize,
}

impl Default for Melody {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            step: 0,
        }
    }
}

/// 一段 C 大调琶音，循环演奏 —— 相当于"程序生成的 BGM"。
const NOTES: [f32; 8] = [
    261.63, 329.63, 392.00, 523.25, 392.00, 329.63, 293.66, 261.63,
];

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    println!("── 每 0.5 秒自动演奏一个音符（Pitch 程序合成，不用音频文件）");
    println!("   空格 = 放一声音效；M = 静音/恢复");
}

/// 当前该用的音量 —— 静音时不放声音。
fn volume_for(muted: bool, base: f32) -> Volume {
    if muted {
        Volume::Linear(0.0)
    } else {
        Volume::Linear(base)
    }
}

/// 定时演奏旋律。每次 `spawn` 一个**临时实体**播放一个音，播完自动销毁。
///
/// 这正是音效/短音的标准做法：**每次要响就 spawn 一个**。好处是天然支持
/// 多个声音重叠（连按空格会听到叠加），不必操心"同一个实体能否重播"。
fn play_melody(
    time: Res<Time>,
    mut melody: ResMut<Melody>,
    muted: Res<Muted>,
    mut pitch_assets: ResMut<Assets<Pitch>>,
    mut commands: Commands,
) {
    if !melody.timer.tick(time.delta()).just_finished() {
        return;
    }

    let frequency = NOTES[melody.step % NOTES.len()];
    melody.step += 1;

    commands.spawn((
        // `Pitch` 是内置的音频源类型，所以直接用 `Assets<Pitch>` 造一个即可。
        // 注意构造方式是 `AudioPlayer(handle)`（元组），不是 `AudioPlayer::new(..)` ——
        // 后者只适用于默认的 `AudioSource`（也就是文件）。
        AudioPlayer(pitch_assets.add(Pitch::new(frequency, Duration::from_millis(420)))),
        PlaybackSettings {
            // 播完自动销毁整个实体，不用手写清理（相当于给音频挂了个"用完即弃"）
            mode: PlaybackMode::Despawn,
            // ⚠️ 开播参数：静音状态在**创建时**就要读一次
            volume: volume_for(muted.0, 0.25),
            ..default()
        },
    ));
}

/// 空格：放一声音效（比旋律音高，短促）。
fn play_sfx_on_space(
    keyboard: Res<ButtonInput<KeyCode>>,
    muted: Res<Muted>,
    mut pitch_assets: ResMut<Assets<Pitch>>,
    mut commands: Commands,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }
    commands.spawn((
        AudioPlayer(pitch_assets.add(Pitch::new(880.0, Duration::from_millis(160)))),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: volume_for(muted.0, 0.6),
            ..default()
        },
    ));
    println!("   [音效] 880Hz 短音（实体播完自动销毁）");
}

/// M：静音开关。
///
/// 这里要**同时干两件事**，正好体现两个组件的分工：
///   ① 改 `Muted` 资源 —— 之后新创建的音频会用新音量（`PlaybackSettings`）
///   ② 改正在播放的 `AudioSink` —— 已经在响的那些立刻变音量
///
/// 只做 ① 的话，静音要等到下一个音符才生效；只做 ② 的话，
/// 新音符又会用旧音量冒出来。
fn toggle_mute_on_m(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut muted: ResMut<Muted>,
    mut sinks: Query<&mut AudioSink>,
) {
    if !keyboard.just_pressed(KeyCode::KeyM) {
        return;
    }
    muted.0 = !muted.0;

    let target = volume_for(muted.0, 0.25);
    let mut affected = 0;
    for mut sink in &mut sinks {
        // `AudioSink` 上还有 `pause()` / `play()` / `is_paused()` / `stop()` 等，
        // 做一个"能暂停的音乐播放器"就靠这一套。
        sink.set_volume(target);
        affected += 1;
    }
    println!(
        "   [静音] 现在 {}（立刻改了 {} 个正在播放的声音）",
        if muted.0 { "静音" } else { "恢复" },
        affected
    );
}

// ─────────────────────────────────────────────────────────────────────
// 跑起来会听到什么
//
//   · 一进画面就有 0.5 秒一拍的琶音（C-E-G-高C-G-E-D-C 循环）
//   · 按空格 → 一声更高的"嘀"（可以和旋律音重叠）
//   · 按 M    → 立刻安静；再按 M 恢复。打印会告诉你有几个声音被改了
//   · 连按空格会有多个音效实体并存 —— 这就是"多实例"，也正是音效需要的行为
//
// ─────────────────────────────────────────────────────────────────────
// 两个组件分工明确
//
//   `PlaybackSettings`   开播参数。音频开始播放后，改它**没有效果**。
//                        写死"这条音效音量 0.6、播完销毁"。
//
//   `AudioSink`          播放句柄。音频开始播放后引擎自动挂上，
//                        有它才能运行中调音量 / 暂停 / 停止 / 查询进度。
//
// 所以"做个能调音量的设置界面"走 `AudioSink`；
// "这条音效就是小一点"直接写在 `PlaybackSettings` 里。
// 本讲的 `toggle_mute_on_m` 两样都用，就是在演示它们缺一不可。
//
// ─────────────────────────────────────────────────────────────────────
// `PlaybackMode` 的四个取值
//
//   `Once`     播一次就结束（实体和 sink 还在，但音频已经放完）
//   `Loop`     无限循环 —— 背景音乐的标准选择
//   `Despawn`  播完**自动销毁整个实体** —— 音效的标准选择，不用手写清理
//   `Remove`   播完**摘掉音频组件**，实体留着
//
// ⚠️ `Once` 有个坑：官方文档写明音频播完后 **`AudioPlayer` 不能复用** ——
//    想重播同一实体，必须把音频组件移除再加回去。所以"反复播放的音效"
//    更简单的做法就是每次 `spawn` 一个临时实体（本讲的做法）。
//
// ─────────────────────────────────────────────────────────────────────
// ⚠️ 音频格式：默认只支持 OGG，不支持 WAV
//
// 这条很容易踩，而且是**运行时 panic**，不是编译错误：
//
//     bevy 的 `audio` feature = ["bevy_audio", "vorbis"]
//                                    ↑ 只有 vorbis（OGG）
//
// 拿一个 `.wav` 去 `asset_server.load(..)` 再播放，会在解码处直接 panic：
//
//     panicked at bevy_audio/src/audio_source.rs:101:
//     called `Result::unwrap()` on an `Err` value
//
// 想播 WAV 得开 `wav` feature（它会引入 `hound` 依赖）：
//
//     bevy = { version = "0.20", features = ["wav"] }
//
// 想要全套格式用 `audio-all-formats`（aac / flac / mp3 / mp4 / vorbis / wav）。
//
// **本讲因此改用 `Pitch` 程序合成音** —— 它不需要任何音频文件、不需要额外
// feature，给个频率就有声音，正好也把 API 讲全了。
// 用它还有个额外好处：你能直接读到"频率"这个参数，
// 比"文件名"更能看清 `AudioPlayer` 到底在播什么。
//
// 真实项目里播文件的常规做法：
//   1. 素材转成 **OGG**（默认就支持，体积也小）
//   2. `assets.load("audio/xxx.ogg")` 拿到 `Handle<AudioSource>`
//   3. `commands.spawn((AudioPlayer::new(handle), PlaybackSettings { .. }))`
//      注意文件走的是 `AudioPlayer::new(..)`，而 `Pitch` 走 `AudioPlayer(handle)` ——
//      因为 `AudioPlayer<Source = AudioSource>` 的默认泛型就是"文件"。
//
// ─────────────────────────────────────────────────────────────────────
// 音效与背景音乐的区别（实战经验）
//
//   BGM ：数量少、要循环、要能被设置界面统一调节 → `Loop` + 留着 `AudioSink`
//   音效：数量多、同时播、播完就扔                 → `Despawn` + 每次 spawn 新实体
//
// 本讲用 `Despawn` 把这两类都覆盖了：旋律每个音是一个临时实体，
// 音效也是临时实体 —— 它们唯一的区别只是频率与时长。
// ─────────────────────────────────────────────────────────────────────
