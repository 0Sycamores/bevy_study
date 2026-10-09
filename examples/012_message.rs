//! 012 · 消息
//!
//! 运行：`cargo run --example 012_message`
//!
//! 新增概念
//!   Message                        全局广播的"发生了一件事"
//!   MessageWriter / MessageReader  写 / 读
//!   MessageMutator                 同一个类型既读又写（Reader + Writer 会冲突）
//!   双缓冲                         一条消息只活两帧
//!
//! 使用场景
//!   让"事情发生方"和"关心方"互不认识：受伤、死亡、得分、播音效
//!
//! 注意：Bevy **不会**自动把写者排在读者之前，必须自己 chain —— 否则消息晚一帧才被读到
//! 注意：消息只活两帧，长期存在的状态该用组件或资源

use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(LogPlugin::default());
    // 消息类型必须先注册，否则读写会 panic。
    app.add_message::<Heartbeat>();
    app.add_message::<Damage>();
    app.add_systems(
        Update,
        (
            // ① 「晚一帧」的读者，被显式排在最前面（在写者之前）。
            //    现实中不会故意这么写 —— 但不声明顺序时，引擎就可能这么排（见文末实测）。
            read_heartbeat_late,
            // ② 正确的顺序：写者在前、读者紧随其后，同一帧就能读到。
            (send_heartbeat, read_heartbeat_ordered).chain(),
            // ③ 伤害链路：写 → 改写 → 两个读者。
            (send_damage, apply_armor, apply_damage_to_health, log_damage).chain(),
        )
            .chain(),
    );

    for frame in 1..=4 {
        println!("═══ 第 {frame} 帧 ═══");
        app.update();
    }
}

/// 每帧发一条心跳，带上是第几帧。
#[derive(Message)]
struct Heartbeat(u32);

/// 一条带数据的伤害消息。`MessageMutator` 会在半路修改它。
#[derive(Message)]
struct Damage(i32);

/// 没有和写者排先后。注意 `MessageReader` 必须声明为 `mut` ——
/// 它内部要记录"这个系统已经读到哪儿了"。
fn read_heartbeat_late(mut reader: MessageReader<Heartbeat>) {
    let got: Vec<u32> = reader.read().map(|beat| beat.0).collect();
    println!("   [晚一帧读者] 读到 {:?}", got);
}

fn send_heartbeat(mut writer: MessageWriter<Heartbeat>, mut frame: Local<u32>) {
    *frame += 1;
    println!("   [写] Heartbeat({})", *frame);
    writer.write(Heartbeat(*frame));
}

/// 和写者 `chain` 在一起，所以同一帧就能读到刚发的那条。
fn read_heartbeat_ordered(mut reader: MessageReader<Heartbeat>) {
    let got: Vec<u32> = reader.read().map(|beat| beat.0).collect();
    println!("   [有序读者]   读到 {:?}", got);
}

/// 只在第 1 帧发一次伤害，方便对照。
fn send_damage(mut writer: MessageWriter<Damage>, mut done: Local<bool>) {
    if !*done {
        *done = true;
        println!("   [写] Damage(10)");
        writer.write(Damage(10));
    }
}

/// 同一个消息类型**既读又写**时，不能同时用 `MessageReader` + `MessageWriter`
/// （两者借用同一份底层资源，会冲突）。这时用 `MessageMutator`。
/// 它读出来的是 `&mut T`，可以直接改。
fn apply_armor(mut damage: MessageMutator<Damage>) {
    for value in damage.read() {
        value.0 -= 3;
        println!("   [护甲] 减伤 3 → Damage({})", value.0);
    }
}

/// 两个系统读同一个消息类型是**允许的**，各自独立记录读取进度。
fn apply_damage_to_health(mut reader: MessageReader<Damage>) {
    for damage in reader.read() {
        println!("   [扣血] 受到 {} 点伤害", damage.0);
    }
}

fn log_damage(mut reader: MessageReader<Damage>) {
    for damage in reader.read() {
        println!("   [日志] 伤害事件：{}", damage.0);
    }
}

// ─────────────────────────────────────────────────────────────────────
// 实测输出
//
//   ═══ 第 1 帧 ═══
//      [晚一帧读者] 读到 []
//      [写] Heartbeat(1)
//      [有序读者]   读到 [1]
//      [写] Damage(10)
//      [护甲] 减伤 3 → Damage(7)
//      [扣血] 受到 7 点伤害
//      [日志] 伤害事件：7
//   ═══ 第 2 帧 ═══
//      [晚一帧读者] 读到 [1]
//      [写] Heartbeat(2)
//      [有序读者]   读到 [2]
//   ═══ 第 3 帧 ═══
//      [晚一帧读者] 读到 [2]
//      [写] Heartbeat(3)
//      [有序读者]   读到 [3]
//   ═══ 第 4 帧 ═══
//      [晚一帧读者] 读到 [3]
//      [写] Heartbeat(4)
//      [有序读者]   读到 [4]
//
// 同一帧里同一个消息类型，两个读者读到的却不一样：
//   · 有序读者  永远读到**本帧**那条
//   · 晚一帧读者 永远读到**上一帧**那条
//
// （实跑，同一份产物连跑 11 次输出逐字相同 —— 因为三个系统组之间都 `chain` 了，
//   顺序被钉死。这里刻意没有留下任何"时序未定"的地方。）
//
// 这就是本讲最关键的一点：**Bevy 不会自动把写者排在读者前面。**
// 顺序不声明，就由调度器自己挑；挑错了，表现就是消息"晚一帧生效"。
//
// ─────────────────────────────────────────────────────────────────────
// 消息只活两帧（双缓冲）
//
// 上面第 3 帧的晚一帧读者读到的是第 2 帧发的 `Heartbeat(2)`。
// 第 1 帧发的 `Heartbeat(1)` 去哪了？**被丢掉了。**
//
// Bevy 用**双缓冲**维护消息：一份是"本帧正在写"的，一份是"本帧正在读"的，
// 每帧末交换并清空旧的那份。所以一条消息总共只存在**两帧**：
//
//     写在第 N 帧 → 第 N 帧可读、第 N+1 帧仍可读 → 第 N+2 帧消失
//
// 单独实测（临时 example：一条只在第 1 帧写入的消息，
// 读者刻意排在写者之前，两者 `chain` 在一起）：
//
//     第 1 帧 读到 0 条      ← 读者在写者前面，本帧还没写完
//     第 2 帧 读到 1 条      ← 上一帧写的那条还在
//     第 3 帧 读到 0 条      ← 已经过期丢弃
//     第 4 帧 读到 0 条
//
// 也就是说，消息的"存活"是**恰好两帧**：写它的那一帧 + 下一帧。
// 读者只要排在写者之后，本帧就能读到；排在之前就得等下一帧 —— 但也就只剩那一次机会。
//
// 这条性质决定了消息的适用场景：
//   · ✅ 一次性通知：受伤、死亡、得分、播放音效
//   · ❌ 需要长期存在的状态：当前血量、是否暂停 —— 那些该用组件或资源
//
// 没人读的消息会自己消失，不会无限堆积。
//
// ─────────────────────────────────────────────────────────────────────
// 三种角色
//
//   `MessageWriter<T>`     只写：`write(msg)` / `write_default()`（T 实现 Default 时）
//   `MessageReader<T>`     只读：`read()` 返回"本系统还没读过的那些"的迭代器
//   `MessageMutator<T>`    既读又写：`read()` 返回 `&mut T` 的迭代器
//
// 几个容易踩的点：
//
// 1. `MessageReader` / `MessageMutator` **必须声明成 `mut`**，
//    因为它要跨帧记住"我读到哪儿了"。不写 `mut` 编译不过。
// 2. 同一个类型不能同时要 `MessageReader<T>` 和 `MessageWriter<T>` —— 会冲突，
//    要用 `MessageMutator<T>`。
// 3. 不同消息类型互不影响，一个系统里可以有多个 reader / writer。
// 4. **每个读者各自独立记录进度**：`apply_damage_to_health` 读过的，
//    `log_damage` 照样能读到。一条消息可以被任意多个系统消费。
//
// ─────────────────────────────────────────────────────────────────────
// 什么时候用消息，什么时候用观察者
//
// 本讲讲的是**全局广播**：写出去谁都能读。013 会讲**观察者(Observer)**：
// 事件带一个**目标实体**，只有监听那个实体的观察者才会收到。
//
//   消息     "有件事发生了"（玩家死了、得分变了）—— 关心的人自己来读
//   观察者   "这个实体出事了"（这个敌人被击中、这个按钮被点了）—— 精准投递
//
// ─────────────────────────────────────────────────────────────────────
