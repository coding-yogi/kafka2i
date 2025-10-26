use std::{ collections::HashMap, error::Error, fmt::Display, time::Duration};
use crossbeam::channel::Sender;
use log::debug;
use rdkafka::{
    client::OAuthToken, config::FromClientConfigAndContext, consumer::{
        base_consumer::BaseConsumer, 
        Consumer as KafkaConsumer, ConsumerContext, 
    }, error::KafkaError, message::Headers, metadata::Metadata as KafkaMetadata, types::RDKafkaErrorCode, util::Timeout, ClientConfig, ClientContext, Message, Offset, Statistics, TopicPartitionList
};
use reqwest::blocking::Client as http_client;
use serde::Deserialize;

use crate::{config::Config, kafka::metadata::{ConsumerGroup, Metadata}};

pub type Result<T> = std::result::Result<T, ConsumerError>;

#[derive(Debug, Clone)]
pub struct ConsumerError {
    message: String,
}

impl Display for ConsumerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ConsumerError {}

impl From<KafkaError> for ConsumerError {
    fn from(value: KafkaError) -> Self {
        ConsumerError {
            message: value.to_string()
        }
    }
}

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
    sub: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DefaultContext {
    config: Config
}

impl  DefaultContext {
    pub fn new(config: Config) -> DefaultContext {
        DefaultContext {
            config: config
        }
    }
}

impl ConsumerContext for DefaultContext {}

impl ClientContext for DefaultContext {
    // required to override to enable token refresh
    const ENABLE_REFRESH_OAUTH_TOKEN: bool = true;

    // Overriding stats as we do not wish to log the stats as part of the default implementatoion
    fn stats(&self, _statistics: rdkafka::Statistics) {
      ()
    }

    // Overriding Oauth callback
    fn generate_oauth_token(&self, _oauthbearer_config: Option<&str>) -> std::result::Result<OAuthToken, Box<dyn Error>> {
        debug!("config received in oauth callback: {:?}", _oauthbearer_config);
        if self.config.oauth_token_endpoint == None || self.config.oauth_client_id == None || self.config.oauth_client_secret == None {
            return Err("token endpoint, client id & client secret must be provided".into());
        }

        // Set params
        let mut params = vec![
            ("grant_type", "client_credentials"),
            ("client_id", self.config.oauth_client_id.as_ref().unwrap()),
            ("client_secret", self.config.oauth_client_secret.as_ref().unwrap()),
        ];

        if let Some(scope) = &self.config.oauth_scope {
            params.push(("scope", scope));
        }

        // https client builder
        let mut http_client_builder = http_client::builder();

        // check if the https ca location is provided
        if let Some(ca_location) = &self.config.https_ca_location {
            http_client_builder = http_client_builder
                .add_root_certificate(
                    reqwest::Certificate::from_pem(&std::fs::read(ca_location)?)?
                );
        }

        // Make request
        debug!("making request to oauth token endpoint");
        let http_client = http_client_builder.build()?;
        let response = http_client
            .post(self.config.oauth_token_endpoint.as_ref().unwrap())
            .form(&params)
            .send()?
            .error_for_status()?
            .json::<TokenResponse>()?;
 
        // Calculate expiry
        let expiry= std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() + response.expires_in;

        // Build OAuthBearerToken for librdkafka
        let token = OAuthToken{
            token: response.access_token,
            principal_name: response.sub.unwrap_or("".to_string()),
            lifetime_ms: i64::try_from(expiry)?, // in milliseconds
        };

        debug!("returning generated token");
        Ok(token)
    }

}
pub struct StatsContext {
   stats_sender: Sender<Statistics> 
}

impl StatsContext {
    pub fn new(stats_sender: Sender<Statistics>) -> StatsContext {
        StatsContext {
            stats_sender
        }
    }
}

impl ConsumerContext for StatsContext {}

impl ClientContext for StatsContext {
    fn stats(&self, statistics: rdkafka::Statistics) {
      let _ =  self.stats_sender.send(statistics); 
    }
}

const DEFAULT_TIMEOUT_IN_SECS: Duration = Duration::from_secs(30);
const DEFAULT_REFRESH_METADATA_IN_SECS: Duration = Duration::from_secs(30);

// Wraps Kafka Consumer from the lib
pub struct Consumer<T>
where T: ClientContext + ConsumerContext {
    base_consumer: BaseConsumer<T>,
    default_timeout: Timeout,
    pub refresh_metadata_in_secs: Duration,
    metadata: Metadata,
    stats: Statistics,
}
 
impl <T> Consumer<T> 
where T: ClientContext + ConsumerContext 
{
    // New Consumer
    pub fn new(config: &ClientConfig, context: T) -> Result<Consumer<T>> {
        // Base Consumer
        let base_consumer = BaseConsumer::from_config_and_context(config, context)?;

        // Time out
        let default_timeout = Timeout::After(DEFAULT_TIMEOUT_IN_SECS);
        
        Ok(Consumer {
            base_consumer,
            default_timeout,
            refresh_metadata_in_secs: DEFAULT_REFRESH_METADATA_IN_SECS,
            metadata: Metadata::new(),
            stats: Statistics::default(),
        })
    }

    // Fetch Metadata
    pub fn fetch_metadata(&self) -> Result<KafkaMetadata> {
        // Metadata
        debug!("fetching metadata ...");
        let kafka_metadata = self.base_consumer.fetch_metadata(None, self.default_timeout)?;
        Ok(kafka_metadata)
    }

    // Update metadata
    pub fn update_metadata(&mut self, metadata: KafkaMetadata, consumer_groups: Vec<ConsumerGroup>) {
        log::debug!("Updating metadata ...");
        self.metadata.update(&metadata, consumer_groups);
    }

    // Return Metadata
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

     pub fn fetch_groups(&self) -> Result<Vec<ConsumerGroup>>{
        debug!("fetching groups ...");
        let group_list = self.base_consumer.fetch_group_list(None, self.default_timeout)?;

        Ok(group_list.groups().iter()
            .map(|g| g.into())
            .collect::<Vec<ConsumerGroup>>())
    }

    pub fn fetch_watermarks(&self, topic: &str, partition: i32) -> Result<(i64, i64)>{
        debug!("fetching watermarks for topic {}/{}", topic, partition);
        let watermarks = self.base_consumer.fetch_watermarks(topic, partition, self.default_timeout)?;
        Ok(watermarks)
    }

    // Update stats
    pub fn update_stats(&mut self, stats: Statistics) {
        self.stats = stats
    }

    // Stats
    pub fn stats(&self) -> &Statistics {
        &self.stats
    }

    // Consume
    pub fn consume(&self, timeout: Duration, with_retries: bool) -> Result<Option<KafkaMessage>> {
        debug!("polling for a message");

        // retry consume upto 3 times with a backoff of 100ms when error if error is Broker transport failure
        let mut retries = 3;
        let sleep_duration_in_ms = 100;
        loop {
            match self.base_consumer.poll(timeout) {
                Some(Ok(msg)) => return Ok(Some(KafkaMessage::new(&msg))),
                Some(Err(err)) => {
                    if let KafkaError::MessageConsumption(err_msg) = &err && *err_msg == RDKafkaErrorCode::BrokerTransportFailure {
                        // We don't use with_retries flag here as this is a specific error we want to retry on
                        if retries > 0 {
                            log::warn!("consume resulted in broker transport failure, retrying ...");
                            retries -= 1;
                            std::thread::sleep(Duration::from_millis(sleep_duration_in_ms));
                            continue;
                        } else {
                            log::error!("consume resulted in broker transport failure, failing consume");
                            return Err(err.into());
                        }
                    } else {
                        return Err(err.into());
                    }
                },
                None => {
                    // log and continue retries
                    if retries > 0 && with_retries {
                        log::warn!("no message received, retrying ...");
                        retries -= 1;
                        std::thread::sleep(Duration::from_millis(sleep_duration_in_ms));
                        continue;
                    } else {
                        debug!("no message received");
                        break;
                    }
                },
            }
        }

        debug!("message is none");
        Ok(None)
    }

    // Assign
    pub fn assign(&self, topic: &str, partition: i32) -> Result<()>{
        let mut tpl = TopicPartitionList::new();
        tpl.add_partition_offset(topic, partition, Offset::End)?;
        let _ = self.base_consumer.assign(&tpl)?;
        Ok(())
    }

    // Seek for a specific topic and partition
    pub fn seek(&self, topic: &str, partition: i32, offset: i64) -> Result<()> {
        debug!("seeking offset {}, on topic {}/{}", offset, topic, partition);

        //retry seek upto 3 times with a backoff of 100ms when error is Erroneous state
        let mut retries = 3;
        loop {
            match self.base_consumer.seek(topic, partition, Offset::Offset(offset), DEFAULT_TIMEOUT_IN_SECS) {
                Ok(_) => break,
                Err(err) => {
                    if let KafkaError::Seek(err_msg) = &err && err_msg == "Local: Erroneous state" {
                        if retries > 0 {
                            log::warn!("seek resulted in erroneous state, retrying ...");
                            retries -= 1;
                            std::thread::sleep(Duration::from_millis(100));
                            continue;
                        } else {
                            log::error!("seek resulted in erroneous state even after retries, failing seek");
                            return Err(err.into());
                        }
                    } else {
                        return Err(err.into());
                    }
                }
            }
        }

        Ok(())
    }

    // return the offset for a specific parition & timestamp
    pub fn offsets_for_timestamp(&self, topic: &str, partition: i32, timestamp: i64) -> Result<Option<i64>> {
        let tpl = self.base_consumer.offsets_for_timestamp(timestamp, DEFAULT_TIMEOUT_IN_SECS)?;
        for e in tpl.elements() {
            if e.topic() == topic && e.partition() == partition {
                return Ok(e.offset().to_raw());
            }
        }

        Ok(None)
    }
}

pub struct KafkaMessage {
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub key: Option<String>,
    pub headers: HashMap<String, String>,
    pub payload: Option<String>,
    pub timestamp: Option<i64>,
}

impl KafkaMessage {
    pub fn new<M: Message>(msg: &M) -> KafkaMessage {
        KafkaMessage {
            topic: msg.topic().to_string(),
            partition: msg.partition(),
            offset: msg.offset(),
            key: retrieve_key(msg),
            payload: retrieve_payload(msg),
            headers: retrieve_headers(msg),
            timestamp: match msg.timestamp() {
                rdkafka::message::Timestamp::NotAvailable => None,
                rdkafka::message::Timestamp::CreateTime(t) => Some(t),
                rdkafka::message::Timestamp::LogAppendTime(t) => Some(t),
            },
        }
    }

    // payload or default
    pub fn payload_or_default(&self) -> String {
        return self.payload.clone().unwrap_or("No Payload".to_string())
    }

    // timestamp or default
    pub fn timestamp_or_default(&self) -> String {
        return self.timestamp.unwrap_or(0).to_string();
    }

    // Key or default
    pub fn key_or_default(&self) -> String {
        return self.key.clone().unwrap_or("No key".to_string())
    }
}

// retrieve key from original kafka message
fn retrieve_key<M: Message>(msg: &M) -> Option<String> {
    match msg.key_view::<str>() {
        Some(k) => {
            match k {
                Ok(kk) => Some(kk.to_string()),
                Err(_) => None,
            }
        },
        None => None,
    }
}

// retrieve headers from original kafka message
fn retrieve_headers<M: Message>(msg: &M) -> HashMap<String, String> {
    let mut headersMap = HashMap::new();
    if let Some(headers) = msg.headers() {
        headers.iter().for_each(|header| {
            if let Some(value) = header.value {
                headersMap.insert(header.key.to_string(), String::from_utf8_lossy(value).to_string());
            } else {
                headersMap.insert(header.key.to_string(), "".to_string());
            }
        });
    }

    headersMap
}

// retrieve payload from original kafka message
fn retrieve_payload<M: Message>(msg: &M) -> Option<String> {
    match msg.payload_view::<str>() {
        Some(p) => {
            match p {
                Ok(pp) => Some(pp.to_string()),
                Err(_) => None,
            }
        },
        None => None,
    }
}