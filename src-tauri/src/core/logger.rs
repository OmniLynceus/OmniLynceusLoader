use crate::config::CONFIG;

use chrono::Local;
use fern::Dispatch;
use log::info;


/// Initializes the logging system using the `fern` crate, configuring both file and console outputs with a custom format.
pub fn init() -> Result<(), fern::InitError> {
    CONFIG.paths.ensure()?;

    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let log_file = CONFIG.paths.dir("logs").join(format!("{}.log", timestamp));

    Dispatch::new()
        // 配置格式
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        // 文件输出
        .chain(fern::log_file(&log_file)?)
        // 控制台输出
        .chain(std::io::stdout())
        // 日志级别过滤
        .level(CONFIG.metadata.log_level)
        .level_for("enigo::platform::win_impl", log::LevelFilter::Info)
        // 应用
        .apply()?;

    info!("日志系统初始化成功，日志文件路径: {}", log_file.display());
    Ok(())
}