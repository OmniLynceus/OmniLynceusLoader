pub mod mouse;

#[macro_export]
macro_rules! commands_register {
    () => {{
        use $crate::commands::{mouse};
        
        tauri::generate_handler![
            mouse::random_move,
        ]
    }};
}