use std::env;

use log::{Level, Metadata, Record};

use crate::format_log;

pub struct ConsoleLogger {
    max_log_level: Level,
}

impl log::Log for ConsoleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.max_log_level
    }

    fn log(&self, record: &Record) {
        let log = format_log(record);

        if record.level() == Level::Error {
            eprint!("{}", log);
        } else {
            print!("{}", log);
        }
    }

    fn flush(&self) {}
}

pub struct ConsoleLoggerConfig {
    max_log_level: Level,
}

impl ConsoleLoggerConfig {
    pub fn new(max_log_level: Level) -> Self {
        Self { max_log_level }
    }
}

impl Default for ConsoleLoggerConfig {
    fn default() -> Self {
        let level = match env::var("RUST_LOG") {
            Ok(level) if level == "error" => Level::Error,
            Ok(level) if level == "warn" => Level::Warn,
            Ok(level) if level == "info" => Level::Info,
            Ok(level) if level == "debug" => Level::Debug,
            Ok(level) if level == "trace" => Level::Trace,
            _ => Level::Info,
        };
        Self::new(level)
    }
}

impl ConsoleLogger {
    pub fn init(config: ConsoleLoggerConfig) {
        let logger = ConsoleLogger {
            max_log_level: config.max_log_level,
        };

        if let Err(error) = log::set_boxed_logger(Box::new(logger))
            .map(|()| log::set_max_level(config.max_log_level.to_level_filter()))
        {
            eprintln!("Error: ConsoleLogger failed to set_boxed_logger: {error}");
        }
    }
}

#[cfg(test)]
mod test {
    use log::{error, info, warn};

    use crate::loggers::console_logger::{ConsoleLogger, ConsoleLoggerConfig};

    #[test]
    #[ignore]
    fn test_log_filter() {
        std::env::set_var("LOG_FILTERS", "service_logger::console_logger::test=warn");

        ConsoleLogger::init(ConsoleLoggerConfig::default());

        dbg!(module_path!());
        info!("don't look at me!");
        warn!("warn");
        error!("look at me!");
    }
}
