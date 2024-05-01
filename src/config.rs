use std::{collections::HashMap, io::Stderr};

use rdkafka::config::{ClientConfig, RDKafkaLogLevel};

const BOOTSTRAP_SERVERS: &str = "bootstrap.servers";
const GROUP_ID: &str = "group.id";
const SESSION_TIMEOUT_MS: &str = "session.timeout.ms";
const ENABLE_AUTO_COMMIT: &str = "enable.auto.commit";

enum LogLevel {
    Debug,
    Info,
    Error,
}

impl Into<RDKafkaLogLevel> for LogLevel {
    fn into(self) -> RDKafkaLogLevel {
        match self {
            Self::Debug => RDKafkaLogLevel::Debug,
            Self::Info => RDKafkaLogLevel::Info,
            Self::Error => RDKafkaLogLevel::Error,
        }
    }
}

struct Config {
    log_level: LogLevel,
    bootstrap_server: String,
    session_timeout_ms: u16,
    enable_auto_commit: bool,
    group_id: String,
}

/*
impl TryInto<HashMap<String,String>> for Config {
    type Error = Stderr;

    fn try_into(self) -> Result<HashMap<String,String>, Self::Error> {
        let map = HashMap::new();
        if self.bootstrap_server != "" {
            map.insert(BOOTSTRAP_SERVERS.to_string(), self.bootstrap_server);
        } else {
            return Err(Stderr::from("bootstrap servers cannot be empty"));
        }

        map.insert(GROUP_ID.to_string(), self.group_id);
        map.insert(SESSION_TIMEOUT_MS.to_string(), self.session_timeout_ms.to_string());
        map.insert(ENABLE_AUTO_COMMIT.to_string(), self.enable_auto_commit.to_string());

        Ok(map)
    }
}*/
