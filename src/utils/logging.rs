use log::Level;
use std::thread;
use env_logger::Env;
use chrono::Local;
use colored::*;

/// Initializes the env_logger with a custom format
/// that also logs the thread names
pub fn init_logger() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use std::io::Write;
            let color = get_level_style(record.level());
            writeln!(
                buf,
                "{:<25} {:<45}| {} {}: {}",
                format!("thread::{}", thread::current().name().unwrap_or("main")).dimmed(),
                record.target().dimmed().italic(),
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record
                    .level()
                    .to_string()
                    .to_lowercase()
                    .as_str()
                    .color(color),
                record.args()
            )
        })
        .init();
}

fn get_level_style(level: Level) -> colored::Color {
    match level {
        Level::Trace => colored::Color::Magenta,
        Level::Debug => colored::Color::Blue,
        Level::Info => colored::Color::Green,
        Level::Warn => colored::Color::Yellow,
        Level::Error => colored::Color::Red,
    }
}