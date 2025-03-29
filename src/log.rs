use anyhow::Result;
use nihility_common::config::get_config;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use time::format_description::well_known::Iso8601;
use tracing::debug;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, fmt};

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub enum LogOutType {
    #[default]
    Console,
    File(String),
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LogConfig {
    pub log: Vec<LogConfigChunk>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LogConfigChunk {
    pub enable: bool,
    pub out_type: LogOutType,
    pub level: LogLevel,
    pub with_file: bool,
    pub with_line_number: bool,
    pub with_thread_ids: bool,
    pub with_target: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log: vec![LogConfigChunk::default()],
        }
    }
}

pub struct Log;

static CONSOLE_WORK_GUARD: OnceLock<WorkerGuard> = OnceLock::new();
static FILE_WORK_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

const LOG_FILE_NAME: &str = ".log";

impl Default for LogConfigChunk {
    fn default() -> Self {
        LogConfigChunk {
            enable: true,
            out_type: LogOutType::default(),
            level: LogLevel::default(),
            with_file: false,
            with_line_number: false,
            with_thread_ids: true,
            with_target: true,
        }
    }
}

impl Log {
    pub async fn init() -> Result<()> {
        let configs = get_config::<LogConfig>(env!("CARGO_PKG_NAME").to_string())
            .await?
            .log;
        let mut layers = Vec::new();

        for config in configs {
            if !config.enable {
                continue;
            }
            let non_blocking = match &config.out_type {
                LogOutType::Console => {
                    let (non_blocking, guard) = non_blocking(std::io::stdout());
                    CONSOLE_WORK_GUARD.get_or_init(|| guard);
                    non_blocking
                }
                LogOutType::File(out_path) => {
                    let file_appender = rolling::daily(out_path, LOG_FILE_NAME);
                    let (non_blocking, guard) = non_blocking(file_appender);
                    FILE_WORK_GUARD.get_or_init(|| guard);
                    non_blocking
                }
            };
            let layer = fmt::layer()
                .with_ansi(false)
                .with_file(config.with_file)
                .with_line_number(config.with_line_number)
                .with_thread_ids(config.with_thread_ids)
                .with_target(config.with_target)
                .with_timer(LocalTime::new(Iso8601::DATE_TIME_OFFSET))
                .with_writer(non_blocking);
            let layer = match config.level {
                LogLevel::Trace => layer.with_filter(LevelFilter::TRACE),
                LogLevel::Debug => layer.with_filter(LevelFilter::DEBUG),
                LogLevel::Info => layer.with_filter(LevelFilter::INFO),
                LogLevel::Warn => layer.with_filter(LevelFilter::WARN),
                LogLevel::Error => layer.with_filter(LevelFilter::ERROR),
            }
            .boxed();
            layers.push(layer);
        }
        tracing_subscriber::registry().with(layers).init();
        debug!("Log Subscriber Init Success");
        Ok(())
    }
}
