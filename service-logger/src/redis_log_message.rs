use std::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub struct LogMessage<'a> {
    pub message: &'a str,
    pub group: &'a str,
    pub index: &'a str,
    pub channel_name: &'a str,
    pub metadata: Option<&'a str>,
}

impl<'a> LogMessage<'a> {
    pub fn redis_serialize(&self) -> Result<String, Box<dyn Error>> {
        let encoded: Vec<u8> = bincode::serialize(&self)?;
        Ok(base64::encode(encoded))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::LogMessage;

    #[derive(Debug, Clone)]
    pub struct LogMessageDeserializer {
        buffer: Vec<u8>,
    }

    impl LogMessageDeserializer {
        pub fn new() -> Self {
            Self { buffer: vec![] }
        }

        pub fn deserialize<'a>(&'a mut self, data: &str) -> Result<LogMessage<'a>, Box<dyn Error>> {
            let buffer = &mut self.buffer;
            buffer.clear();
            base64::decode_engine_vec(data, buffer, &base64::engine::DEFAULT_ENGINE)?;
            Ok(bincode::deserialize(buffer)?)
        }
    }

    impl Default for LogMessageDeserializer {
        fn default() -> Self {
            Self::new()
        }
    }

    #[test]
    fn deserializer() {
        let log_message = LogMessage {
            channel_name: "foo",
            group: "bar",
            index: "1",
            message: "foobar",
            metadata: Some("barfoo"),
        };

        let serialized = log_message.redis_serialize().unwrap();
        let mut reader = LogMessageDeserializer::new();
        let deserialized = reader.deserialize(&serialized).unwrap();

        assert_eq!(deserialized, log_message);
    }
}
