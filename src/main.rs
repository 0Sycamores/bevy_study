//! 这个仓库的学习内容全部在 `examples/` 里，每个例子都能单独运行：
//!
//! ```text
//! cargo run --example 001_hello
//! cargo run --example 010_game
//! cargo check --examples
//! ```
//!
//! 课程目录、每个例子的观察点和练习见 `README.md`。

fn main() {
    println!("这个仓库的例子都在 examples/ 目录下，请用下面的方式运行：");
    println!();
    println!("    cargo run --example 001_hello");
    println!("    cargo run --example 010_game");
    println!();
    println!("完整课程目录见 README.md。");
}
