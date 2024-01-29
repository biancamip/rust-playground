use std::path::{Path, PathBuf};

use console_logger::{ConsoleLogger, ConsoleLoggerConfig};
use file_logger::{FileLogger, FileLoggerConfig};
use log::{Level, Record};
use redis_logger::{RedisLogger, RedisLoggerConfig};

pub mod console_logger;
pub mod file_logger;
mod redis_log_message;
pub mod redis_logger;

#[derive(Debug)]
pub enum BackgroundLoggerMessage {
    String(String),
    Flush,
}

#[derive(Debug)]
pub enum ServiceLoggerKind {
    ConsoleLogger,
    FileLogger,
    RedisLogger,
}

pub struct ServiceLogger {}

#[derive(Debug)]
pub struct ServiceLoggerEnv {
    log_path: Option<String>,
    max_line_count: Option<usize>,
    log_redis_connection_string: Option<String>,
    log_kind: ServiceLoggerKind,
    group_name: Option<String>,
    alloc_index: Option<String>,
}

impl ServiceLoggerEnv {
    pub fn from_env() -> Self {
        let log_kind = match std::env::var("LOG_KIND") {
            Ok(log_kind) if log_kind == "FILE" => ServiceLoggerKind::FileLogger,
            Ok(log_kind) if log_kind == "REDIS" => ServiceLoggerKind::RedisLogger,
            _ => ServiceLoggerKind::ConsoleLogger,
        };

        Self {
            log_path: std::env::var("LOG_PATH").ok(),
            max_line_count: std::env::var("MAX_LINE_COUNT")
                .map(|l| l.parse::<usize>().expect("FAILED TO PARSE MAX_LINE_COUNT"))
                .ok(),
            log_kind,
            log_redis_connection_string: std::env::var("LOG_REDIS_CONNECTION_STRING").ok(),
            group_name: std::env::var("GROUP_NAME").ok(),
            alloc_index: std::env::var("ALLOC_INDEX").ok(),
        }
    }
}

impl ServiceLogger {
    pub fn init_from_env() {
        let env = ServiceLoggerEnv::from_env();
        println!("{:#?}", env);
        match env.log_kind {
            ServiceLoggerKind::ConsoleLogger => Self::init_console(),
            ServiceLoggerKind::FileLogger => {
                let path = env.log_path.expect("MISSING LOG_PATH FOR FILE LOGGER");
                let path = PathBuf::from(path);
                if let Some(folder) = path.parent() {
                    if !folder.exists() {
                        println!("creating {} folder", folder.to_string_lossy());
                        std::fs::create_dir_all(folder).unwrap_or_else(|_| {
                            panic!("Failed to create {} folder", folder.to_string_lossy())
                        });
                    }
                }

                Self::init_file(path, env.max_line_count, Level::Info)
            }
            ServiceLoggerKind::RedisLogger => {
                let redis_connection_string = env
                    .log_redis_connection_string
                    .expect("MISSING LOG_REDIS_CONNECTION_STRING");
                let group_name = env.group_name.expect("GROUP_NAME");
                let alloc_index = env.alloc_index.unwrap_or_else(|| "0".to_string());

                Self::init_redis(redis_connection_string, group_name, alloc_index)
            }
        }
    }

    pub fn init_console() {
        ConsoleLogger::init(ConsoleLoggerConfig::default())
    }

    pub fn init_file(
        path: impl AsRef<Path>,
        max_line_count: Option<usize>,
        max_log_level: log::Level,
    ) {
        FileLogger::init(FileLoggerConfig {
            max_log_level,
            path: path.as_ref().to_owned(),
            max_line_count: max_line_count.unwrap_or(100_000),
        })
    }

    pub fn init_redis(redis_connection_string: String, group_name: String, alloc_index: String) {
        RedisLogger::init(RedisLoggerConfig::new(
            redis_connection_string,
            group_name,
            alloc_index,
        ))
    }
}

#[inline(always)]
fn format_log(record: &Record) -> String {
    format!("{}\n", record.args())
}
