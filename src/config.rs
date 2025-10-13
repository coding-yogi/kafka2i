use std::{error::Error, fmt::Display};

use clap::{Parser, ValueEnum};
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use strum::{Display};

// config params
const BOOTSTRAP_SERVERS: &str = "bootstrap.servers";
const GROUP_ID: &str = "group.id";
const SESSION_TIMEOUT_MS: &str = "session.timeout.ms";
const ENABLE_AUTO_COMMIT: &str = "enable.auto.commit";
const STATS_INTERVAL_MS: &str = "statistics.interval.ms";
const SECURITY_PROTOCOL: &str = "security.protocol";
const CA_CERT_LOCATION: &str = "ssl.ca.location";
const ENABLE_CERT_VALIDATION: &str = "enable.ssl.certificate.verification";
const DEBUG: &str = "debug";
const AUTO_OFFSET_RESET: &str = "auto.offset.reset";

const DEFAULT_GROUP_ID: &str = "cg.krust";

#[derive(Debug, Clone, ValueEnum, PartialEq, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Error,
}

#[derive(Debug, Display, Clone, ValueEnum)]
pub enum Protocol {
    #[strum(serialize = "PLAINTEXT")]
    #[value(name = "PLAINTEXT")]
    PlainText,

    #[strum(serialize = "SSL")]
    #[value(name = "SSL")]
    Ssl,

    #[strum(serialize = "SASL_SSL")]
    #[value(name = "SASL_SSL")]
    SaslSsl,

    #[strum(serialize = "SASL_PLAINTEXT")]
    #[value(name = "SASL_PLAINTEXT")]
    SaslPlainText,
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

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        &self.message
    }
}

/// TUI for kafka written in Rust
#[derive(Parser, Debug)]
#[command(name = "krust")]
#[command(about = "TUI for kafka written in Rust", long_about = None)]
pub struct Config {
    /// Log level to be set for kafka client
    #[arg(short, long, value_enum, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,

    /// Bootstrap servers in kafka format
    #[arg(short, long, required=true)]
    pub bootstrap_servers: String,
    
    /// Consumer group ID
    #[arg(short, long, default_value = DEFAULT_GROUP_ID)]
    pub group_id: String,

    /// Protocol to use
    #[arg(short, long, default_value_t = Protocol::PlainText)]
    pub protocol: Protocol,

    /// Full path to CA location for validating SSL Certificate, If not set, certificate validation will be ignored
    #[arg(short, long, default_value = "")]
    pub ssl_ca_location: String,
}

impl TryInto<ClientConfig> for Config {
    type Error = ConfigError;

    fn try_into(self) -> Result<ClientConfig, Self::Error> {
        let mut client_config = ClientConfig::new();
        client_config.log_level = self.log_level.into();

        // set debufg level
        if self.log_level == LogLevel::Debug {
            client_config.set(DEBUG.to_string(), "all".to_string());
        }
       
        // bootstrap server
        if self.bootstrap_servers != "" {
            client_config.set(BOOTSTRAP_SERVERS.to_string(), self.bootstrap_servers);
        } else {
            return Err(ConfigError::new("bootstrap servers cannot be empty"));
        }

        // group id
        client_config.set(GROUP_ID.to_string(), self.group_id);

        // test config
        //client_config.set("auto.offset.reset", "earliest");
        //client_config.set("enable.auto.commit", "false");
        //client_config.set("enable.auto.offset.store", "false");

        client_config.set("socket.keepalive.enable", "true");

        // stats interval
        client_config.set(STATS_INTERVAL_MS, "5000");
       
        // handle protocol
        client_config.set(SECURITY_PROTOCOL.to_string(), self.protocol.to_string());

        match self.protocol {
            Protocol::Ssl => {
                if self.ssl_ca_location != "" {
                    client_config.set(CA_CERT_LOCATION, self.ssl_ca_location);
                } else {
                    // Disable the cert validation if not cert location is found
                    client_config.set(ENABLE_CERT_VALIDATION, "false");
                }
            },
            Protocol::SaslSsl => {

            },

            Protocol::SaslPlainText => {

            },
            Protocol::PlainText => (),
        }

        Ok(client_config)
    }
}
