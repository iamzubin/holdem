use std::sync::OnceLock;
use tracing::{info, Level};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

pub fn setup_logging() {
    let log_dir = std::env::var("LOCALAPPDATA")
        .map(|p| std::path::PathBuf::from(p).join("holdem").join("logs"))
        .unwrap_or_else(|_| std::path::PathBuf::from(".").join("logs"));

    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Failed to create log directory: {}", e);
    }

    // A week of daily logs; older ones are deleted.
    let file_appender = match RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("holdem")
        .filename_suffix("log")
        .max_log_files(7)
        .build(&log_dir)
    {
        Ok(appender) => appender,
        Err(e) => {
            eprintln!("Failed to set up rotating logs: {}", e);
            RollingFileAppender::new(Rotation::DAILY, &log_dir, "holdem.log")
        }
    };

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let _ = LOG_GUARD.set(guard);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .with(fmt::layer().with_writer(std::io::stdout))
        .init();

    info!("Logging initialized. Log directory: {:?}", log_dir);
}
