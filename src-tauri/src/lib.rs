pub mod app;
pub mod commands;
pub mod config;
pub mod core;



#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use core::mouse::Controller;
    // 初始化控制器
    let controller = Controller::new();
    controller.start(); // 启动控制器线程
    
    tauri::Builder::default()
        .manage(controller) // 将控制器托管给 Tauri
        .plugin(tauri_plugin_opener::init())
        .setup(app::init)
        .invoke_handler(commands_register!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}