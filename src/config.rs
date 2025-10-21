use std::{error::Error, fmt::Display};

use clap::{Parser, ValueEnum};
use log::info;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use strum::{Display};

// connection config params
const BOOTSTRAP_SERVERS: &str = "bootstrap.servers";
const GROUP_ID: &str = "group.id";
const SOCKET_KEEP_ALIVE: &str = "socket.keepalive.enable";
const STATS_INTERVAL_MS: &str = "statistics.interval.ms";
const SECURITY_PROTOCOL: &str = "security.protocol";

// SSL config
const SSL_KEY_LOCATION: &str = "ssl.key.location";
const SSL_CERT_LOCATION: &str = "ss.certificate.location";
const CA_CERT_LOCATION: &str = "ssl.ca.location";
const ENABLE_CERT_VALIDATION: &str = "enable.ssl.certificate.verification";

// SASL mechanism
const SASL_MECHANISM: &str = "sasl.mechanism";

// SASL plain confif
const SASL_USERNAME: &str = "sasl.username";
const SASL_PASSWORD: &str = "sasl.password";

// SASL OAuth config
const OAUTH_BEARER_METHOD: &str = "sasl.oauthbearer.method";
const OAUTH_CLIENT_ID: &str = "sasl.oauthbearer.client.id";
const OAUTH_CLIENT_SECRET: &str = "sasl.oauthbearer.client.secret";
const OAUTH_SCOPE: &str = "sasl.oauthbearer.scope";
const OAUTH_TOKEN_ENDPOINT: &str = "sasl.oauthbearer.token.endpoint.url";
const HTTPS_CA_LOCATION: &str = "https.ca.location";

// Log config
const DEBUG: &str = "debug";

const DEFAULT_GROUP_ID: &str = "cg.krust";

#[derive(Debug, Display, Clone, ValueEnum, PartialEq, Copy)]
pub enum LogLevel {
    #[strum(serialize = "debug")]
    Debug,
    #[strum(serialize = "info")]
    Info,
    #[strum(serialize = "error")]
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

#[derive(Debug, Display, Clone, ValueEnum)]
pub enum SaslMechanism {
    #[strum(serialize = "PLAIN")]
    #[value(name = "PLAIN")]
    Plain,

    #[strum(serialize = "OAUTHBEARER")]
    #[value(name = "OAUTHBEARER")]
    OauthBearer,
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
#[derive(Parser, Debug, Clone)]
#[command(name = "kafka2i")]
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
    #[arg(short, long, default_value_t = Protocol::Ssl)]
    pub protocol: Protocol,

    /// Full path to CA location for validating SSL certificate
    #[arg(long)]
    pub ssl_ca_location: Option<String>,

    /// SSL client key
    #[arg(long)]
    pub ssl_client_key_location: Option<String>,

    /// SSL client certificate
    #[arg(long)]
    pub ssl_client_certificate_location: Option<String>,

    /// Disable server certificate vertification
    #[arg(short, long)]
    pub disable_ssl_cert_vertification: bool,

    // SASL mechanism
    #[arg(long)]
    pub sasl_mechanism: Option<SaslMechanism>,

    /// SASL username
    #[arg(long, required_if_eq("sasl_mechanism", "PLAIN"))]
    pub sasl_username: Option<String>,

    /// SASL password
    #[arg(long, required_if_eq("sasl_mechanism", "PLAIN"))]
    pub sasl_password: Option<String>,

    /// SASL OAuth bearer method
    #[arg(short, long, default_value = "oidc")]
    pub oauth_bearer_method: String,

    /// OAuth token endpoint
    #[arg(long, required_if_eq("sasl_mechanism", "OAUTHBEARER"))]
    pub oauth_token_endpoint: Option<String>,

    /// OAuth client id
    #[arg(long)]
    #[arg(long, required_if_eq("sasl_mechanism", "OAUTHBEARER"))]
    pub oauth_client_id: Option<String>,

    /// OAuth client secret
    #[arg(long)]
    #[arg(long, required_if_eq("sasl_mechanism", "OAUTHBEARER"))]
    pub oauth_client_secret: Option<String>,

    /// OAuth scope
    #[arg(long)]
    pub oauth_scope: Option<String>,

    /// Https CA location will be used to validate server cerification for the token endpoint
    #[arg(long)]
    pub https_ca_location: Option<String>,
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

        // keepalive
        client_config.set(SOCKET_KEEP_ALIVE, "true");

        // stats interval
        client_config.set(STATS_INTERVAL_MS, "5000");
       
        // handle protocol
        client_config.set(SECURITY_PROTOCOL.to_string(), self.protocol.to_string());

        // handle SSL config
        match self.protocol {
            Protocol::Ssl | Protocol::SaslSsl => {
                if self.ssl_ca_location != None {
                    client_config.set(CA_CERT_LOCATION, self.ssl_ca_location.unwrap());
                } else {
                    info!("ssl.ca.location is not provided, client will fall back to default ca location");
                }

                // check if both client key & certificate is provided
                if self.ssl_client_key_location != None && self.ssl_client_certificate_location != None {
                    client_config.set(SSL_KEY_LOCATION, self.ssl_client_key_location.unwrap());
                    client_config.set(SSL_CERT_LOCATION, self.ssl_client_certificate_location.unwrap());
                } else {
                    info!("either of client key, client cert or both are not provided, wil continue without using both")
                }

                // check is we need to disable cert verification
                if self.disable_ssl_cert_vertification {
                    client_config.set(ENABLE_CERT_VALIDATION, "false");
                }
            },
            _ => (),
        }

        // handle SASL config
        if let Some(sasl_mechanism) = self.sasl_mechanism {
            client_config.set(SASL_MECHANISM, sasl_mechanism.to_string());

            match sasl_mechanism {
                SaslMechanism::Plain => {
                    // check if both username and password is provided
                    if self.sasl_username == None || self.sasl_password == None {
                        return Err(ConfigError::new("username and password must be set while using SASL_PLAIN mechanism"));
                    }

                    client_config.set(SASL_USERNAME, self.sasl_username.unwrap());
                    client_config.set(SASL_PASSWORD, self.sasl_password.unwrap());
                },

                SaslMechanism::OauthBearer => {
                    client_config.set(OAUTH_BEARER_METHOD, self.oauth_bearer_method);

                    // check if the token endpoint, client id and secret is provided
                    if self.oauth_token_endpoint == None || self.oauth_client_id == None || self.oauth_client_secret == None {
                        return Err(ConfigError::new("token endpoint, client id & client secret must be set while using SASL_OAUTHBEARER mechanism"));
                    }

                    client_config.set(OAUTH_TOKEN_ENDPOINT, self.oauth_token_endpoint.unwrap());
                    client_config.set(OAUTH_CLIENT_ID, self.oauth_client_id.unwrap());
                    client_config.set(OAUTH_CLIENT_SECRET, self.oauth_client_secret.unwrap());

                    // set scope if provided
                    if let Some(scope) = self.oauth_scope  {
                        client_config.set(OAUTH_SCOPE, scope);
                    }

                    // check if https ca is set
                    if let Some(https_ca_location) = self.https_ca_location {
                        client_config.set(HTTPS_CA_LOCATION, https_ca_location);
                    }
                }
            }
        }

        Ok(client_config)
    }
}
