// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
// use tracing_subscriber;
// use tracing;
#[tauri::command]
fn greet() {
    println!("testing, Hello World, Lorem Ipsum, etc...");
}

/* #[cfg_attr(mobile, tauri::mobile_entry_point)] */
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_custom_dialog::init::<tauri::Wry>())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
