// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt()
        // .with_max_level(tracing::Level::DEBUG) // Optional: Set a default max level
        .init();
    tracing::info!("Test log message in tracing!");
    basic_test_app_lib::run()
}
