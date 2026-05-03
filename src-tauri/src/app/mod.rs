use crate::core::logger;
use crate::config::CONFIG;

use tauri::Manager;
use log::debug;

fn close() {
    debug!("执行程序收尾工作，即将关闭应用");
    CONFIG.save().ok();
}


pub fn init(app: &mut tauri::App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    logger::init()?;
    let main_window = app.get_webview_window("main").unwrap();
    let mask_window = app.get_webview_window("mask").unwrap();

    mask_window.set_ignore_cursor_events(true).ok();
    mask_window.show().ok();

    mask_window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
        }
    });

    main_window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { .. } = event {
            mask_window.destroy().ok();
            close();
        }
    });

    Ok(())
}