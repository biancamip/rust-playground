use std::time::{Duration, Instant};

use log::Level;
use redis::Pipeline;

use crate::{format_log, redis_log_message::LogMessage, BackgroundLoggerMessage};

pub struct RedisLogger {
    max_log_level: Level,
    log_tx: crossbeam::channel::Sender<(BackgroundLoggerMessage, log::Level)>,
}

impl RedisLogger {
    pub fn init(config: RedisLoggerConfig) {
        std::thread::spawn(move || Self::init_task(config));

        // Pause app for a bit to avoid a race condition and guarantee errors are logged.
        std::thread::sleep(Duration::from_secs(3));
    }

    fn init_task(config: RedisLoggerConfig) {
        let max_log_level = config.max_log_level;
        let stdout_channel_name = format!("monitoring-nomad:{}.stdout", &config.group_name);
        let warn_channel_name = format!("monitoring-nomad:{}.warn", &config.group_name);
        let stderr_channel_name = format!("monitoring-nomad:{}.stderr", &config.group_name);
        let group_name = config.group_name;
        let index = config.index;
        let redis_connection_string = config.redis_connection_string;

        print!(
            "\
              vv RedisLogger config begin vv\n\
            * max_log_level: {max_log_level}\n\
            * stdout_channel_name: {stdout_channel_name}\n\
            * warn_channel_name: {warn_channel_name}\n\
            * stderr_channel_name: {stderr_channel_name}\n\
            * group_name: {group_name}\n\
            * index: {index}\n\
            * redis_connection_string: {redis_connection_string}\n\
              ^^ RedisLogger config end ^^\n\
            "
        );

        let redis_client = match redis::Client::open(redis_connection_string.clone()) {
            Ok(redis_client) => redis_client,
            Err(error) => {
                eprintln!("Error: RedisLogger failed to open redis log connection: {error}");
                return;
            }
        };

        let connection = match redis_client.get_connection() {
            Ok(connection) => connection,
            Err(error) => {
                eprintln!("Error: RedisLogger failed to establish redis connection: {error}");
                return;
            }
        };

        let (log_tx, log_rx) = crossbeam::channel::bounded(4096 * 8);

        let log_watcher = RedisLogger {
            log_tx,
            max_log_level,
        };

        if let Err(error) = log::set_boxed_logger(Box::new(log_watcher))
            .map(|()| log::set_max_level(config.max_log_level.to_level_filter()))
        {
            eprintln!("Error: RedisLogger failed to set_boxed_logger: {error}");
            return;
        }

        let backgroud_writer = RedisLoggerBackgroundService {
            log_rx,
            buffer: vec![],
            connection,
            client: redis_client,
            last_flush: Instant::now(),
            buffer_size: 500,
            stdout_channel_name,
            warn_channel_name,
            stderr_channel_name,
            group_name,
            index,
            redis_connection_string,
        };

        println!("Redis background logger running.");

        backgroud_writer.run();
    }
}

pub struct RedisLoggerBackgroundService {
    client: redis::Client,
    connection: redis::Connection,
    log_rx: crossbeam::channel::Receiver<(BackgroundLoggerMessage, log::Level)>,
    buffer: Vec<(String, Level)>,
    last_flush: Instant,
    buffer_size: usize,
    stdout_channel_name: String,
    warn_channel_name: String,
    stderr_channel_name: String,
    group_name: String,
    index: String,
    redis_connection_string: String,
}

impl RedisLoggerBackgroundService {
    pub fn run(mut self) {
        let mut consecutive_failures = 0;
        loop {
            let (log, level) = match self.log_rx.recv() {
                Ok(log_level) => {
                    consecutive_failures = 0;
                    log_level
                }
                Err(error) => {
                    consecutive_failures += 1;
                    if consecutive_failures >= 3 {
                        eprintln!("RedisLoggerBackgroundService failed recv(self.log_rx) after {consecutive_failures} retries. Error: {error}");
                        self.publish_log_buffer();
                        return;
                    }
                    continue;
                }
            };

            match log {
                BackgroundLoggerMessage::String(log) => {
                    self.buffer.push((log, level));

                    if self.buffer.len() >= self.buffer_size
                        || self.last_flush.elapsed().as_secs() > 10
                    {
                        self.publish_log_buffer();
                    }
                }
                BackgroundLoggerMessage::Flush => {
                    self.publish_log_buffer();
                }
            }
        }
    }

    fn publish_log_buffer(&mut self) {
        let log_messages = self
            .buffer
            .iter()
            .map(|(msg, level)| {
                let channel = match level {
                    Level::Error => &self.stderr_channel_name,
                    Level::Warn => &self.warn_channel_name,
                    _ => &self.stdout_channel_name,
                };
                LogMessage {
                    channel_name: channel,
                    group: &self.group_name,
                    index: &self.index,
                    message: msg,
                    metadata: None,
                }
            })
            .collect::<Vec<_>>();

        let pipeline = Self::build_pipeline(&log_messages);

        match pipeline.query::<()>(&mut self.connection) {
            Ok(_) => {}
            Err(err) => loop {
                eprintln!("{}", err);
                println!("reconnecting to log redis in 2...");
                std::thread::sleep(Duration::from_secs(2));
                let client = redis::Client::open(self.redis_connection_string.clone());

                match client {
                    Ok(client) => {
                        self.client = client;
                        let connection = match self.client.get_connection() {
                            Ok(connection) => connection,
                            Err(err) => {
                                eprintln!("failed to reconnect to log redis client, retrying in 2");
                                eprintln!("{}", err);
                                continue;
                            }
                        };
                        self.connection = connection;
                    }
                    Err(err) => {
                        eprintln!("failed to reconnect to log redis client, retrying in 2");
                        eprintln!("{}", err);
                        continue;
                    }
                }

                let pipeline = Self::build_pipeline(&log_messages);

                match pipeline.query::<()>(&mut self.connection) {
                    Ok(_) => break,
                    Err(_) => {
                        eprintln!("{}", err);
                        eprintln!("failed to connect to log redis, retrying in 2",)
                    }
                }
            },
        }
        self.buffer.clear();
        self.last_flush = Instant::now()
    }

    #[inline]
    fn build_pipeline(log_messages: &[LogMessage]) -> Pipeline {
        let mut pipeline = Pipeline::new();
        for msg in log_messages {
            pipeline.publish(
                msg.channel_name,
                msg.redis_serialize()
                    .expect("failed to serialize redis log"),
            );
        }
        pipeline
    }
}

impl log::Log for RedisLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.max_log_level
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let log = format_log(record);
        if let Err(error) = self
            .log_tx
            .try_send((BackgroundLoggerMessage::String(log), record.level()))
        {
            eprintln!("{error}")
        }
    }

    fn flush(&self) {
        if let Err(error) = self
            .log_tx
            .try_send((BackgroundLoggerMessage::Flush, self.max_log_level))
        {
            eprintln!("{error}")
        }
    }
}

pub struct RedisLoggerConfig {
    redis_connection_string: String,
    max_log_level: log::Level,
    group_name: String,
    index: String,
}

impl RedisLoggerConfig {
    pub fn new(
        redis_connection_string: String,
        group_name: String,
        alloc_index: String,
    ) -> RedisLoggerConfig {
        Self {
            redis_connection_string,
            group_name,
            index: alloc_index,
            max_log_level: log::Level::Info,
        }
    }
}
