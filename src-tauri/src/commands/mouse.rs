
use crate::core::mouse::Controller;
use crate::config::CONFIG;

use rand::RngExt;
use tauri::{AppHandle, Emitter, Manager, State};



#[tauri::command]
pub fn random_move(app: AppHandle, controller: State<'_, Controller>) {
    let mut rng = rand::rng();

    let(x, y)= (
        rng.random_range(1..CONFIG.metadata.screen.width),
        rng.random_range(1..CONFIG.metadata.screen.height)
    );

    

    controller.move_to(x, y);
    if let Some(mask) = app.get_webview_window("mask") {
        mask.emit("move-to", serde_json::json!({"x": x, "y": y})).ok();
    }
}