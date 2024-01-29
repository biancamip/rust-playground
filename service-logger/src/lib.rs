use log::Record;
use loggers::{
    console_logger::{ConsoleLogger, ConsoleLoggerConfig},
    redis_logger::{RedisLogger, RedisLoggerConfig},
};

pub mod loggers;
mod redis_log_message;

#[derive(Debug)]
pub enum BackgroundLoggerMessage {
    String(String),
    Flush,
}

#[derive(Debug)]
pub enum ServiceLoggerKind {
    ConsoleLogger,
    RedisLogger,
}

pub struct ServiceLogger {}

#[derive(Debug)]
pub struct ServiceLoggerEnv {
    log_redis_connection_string: Option<String>,
    log_kind: ServiceLoggerKind,
    group_name: Option<String>,
    alloc_index: Option<String>,
}

impl ServiceLoggerEnv {
    pub fn from_env() -> Self {
        let log_kind = match std::env::var("LOG_KIND") {
            Ok(log_kind) if log_kind == "REDIS" => ServiceLoggerKind::RedisLogger,
            _ => ServiceLoggerKind::ConsoleLogger,
        };

        Self {
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
