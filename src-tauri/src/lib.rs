use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{Manager, State};
use enigo::{Enigo, Mouse, Coordinate};
use rand::RngExt; 

// 存储平滑移动的状态
struct MouseState {
    target_pos: Mutex<(f32, f32)>,
    velocity: Mutex<(f32, f32)>,
}

#[tauri::command]
fn move_mouse_to(x: f32, y: f32, state: State<'_, Arc<MouseState>>) {
    let mut target = state.target_pos.lock().unwrap();
    *target = (x, y);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mouse_state = Arc::new(MouseState {
        target_pos: Mutex::new((500.0, 500.0)),
        velocity: Mutex::new((0.0, 0.0)),
    });

    let state_for_thread = Arc::clone(&mouse_state);

    // 开启高频率物理循环线程 (类似 Python 的 MouseController.run)
    thread::spawn(move || {
        let mut enigo = Enigo::new(&enigo::Settings::default()).unwrap();
        let mut rng = rand::rng();
        
        let target_fps = 144.0;
        let frame_time = Duration::from_secs_f32(1.0 / target_fps);

        // 调整这些参数来改变手感
        let friction = 0.85;       // 摩擦系数 (0-1)，越大惯性越强
        let gain = 0.5;           // 加速增益，牵引力
        let jitter_intensity = 0.5; // 微小抖动

        loop {
            let start = Instant::now();
            
            let (curr_x, curr_y) = enigo.location().unwrap_or((0, 0));
            let (tx, ty) = { *state_for_thread.target_pos.lock().unwrap() };
            
            let dx = tx - curr_x as f32;
            let dy = ty - curr_y as f32;
            let dist = (dx*dx + dy*dy).sqrt();

            if dist > 0.5 { // 如果距离很近就不动了
                let mut vel = state_for_thread.velocity.lock().unwrap();

                // 1. 惯性：保留原有速度并应用摩擦力
                let mut vx = vel.0 * friction;
                let mut vy = vel.1 * friction;

                // 2. 加速：根据距离施加牵引力 (先加速)
                // 使用 dist.sqrt() 可以让远距离的拉力增加，但不至于失控
                let pull = dist.sqrt() * gain;
                vx += (dx / dist) * pull;
                vy += (dy / dist) * pull;

                // 3. 减速逻辑：当非常接近目标时强行增加阻尼
                // 这能防止在目标点附近来回摆动 (Overshoot)
                if dist < 20.0 {
                    let slowdown = dist / 20.0;
                    vx *= slowdown;
                    vy *= slowdown;
                }

                // 4. 更新速度状态
                vel.0 = vx;
                vel.1 = vy;

                // 5. 应用抖动 (代表人手的微颤)
                let jx = rng.random_range(-jitter_intensity..jitter_intensity);
                let jy = rng.random_range(-jitter_intensity..jitter_intensity);

                let next_x = curr_x as f32 + vx + jx;
                let next_y = curr_y as f32 + vy + jy;

                let _ = enigo.move_mouse(next_x as i32, next_y as i32, Coordinate::Abs);
            } else {
                // 彻底到达目标，清空速度防止下次启动有奇怪初速度
                let mut vel = state_for_thread.velocity.lock().unwrap();
                vel.0 = 0.0;
                vel.1 = 0.0;
            }

            let elapsed = start.elapsed();
            if elapsed < frame_time {
                thread::sleep(frame_time - elapsed);
            }
        }
    });

    tauri::Builder::default()
        .manage(mouse_state) // 将状态注入 Tauri
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();
            let mask_window = app.get_webview_window("mask").unwrap();

            // 1. 设置 mask 窗口忽略鼠标穿透和显示
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
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![move_mouse_to])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
