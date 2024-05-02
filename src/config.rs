use std::fmt::Display;

use rdkafka::config::{ClientConfig, RDKafkaLogLevel};

const BOOTSTRAP_SERVERS: &str = "bootstrap.servers";
const GROUP_ID: &str = "group.id";
const SESSION_TIMEOUT_MS: &str = "session.timeout.ms";
const ENABLE_AUTO_COMMIT: &str = "enable.auto.commit";

const DEFAULT_GROUP_ID: &str = "cg.krust";

pub enum LogLevel {
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

#[derive(Debug, Clone)]
pub struct ConfigError {
    message: String
}

impl ConfigError {
    fn new(message: &str) -> ConfigError {
        ConfigError {
            message: message.to_string(),
        }
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub struct Config {
    pub log_level: LogLevel,
    pub bootstrap_server: String,
    pub session_timeout_ms: u16,
    pub enable_auto_commit: bool,
    pub group_id: String,
}

impl TryInto<ClientConfig> for Config {
    type Error = ConfigError;

    fn try_into(self) -> Result<ClientConfig, Self::Error> {
        let mut client_config = ClientConfig::new();
        client_config.log_level = self.log_level.into();

        // bootstrap server
        if self.bootstrap_server != "" {
            client_config.set(BOOTSTRAP_SERVERS.to_string(), self.bootstrap_server);
        } else {
            return Err(ConfigError::new("bootstrap servers cannot be empty"));
        }

        // group id
        if self.group_id != "" {
            client_config.set(GROUP_ID.to_string(), self.group_id);
        } else {
            client_config.set(GROUP_ID.to_string(), DEFAULT_GROUP_ID.to_string());
        }
        
        // session time out
        if self.session_timeout_ms != 0 {
            client_config.set(SESSION_TIMEOUT_MS.to_string(), self.session_timeout_ms.to_string());
        }

        // enable auto commit
        if self.enable_auto_commit {
            client_config.set(ENABLE_AUTO_COMMIT.to_string(), self.enable_auto_commit.to_string());
        }

        Ok(client_config)
    }
}
