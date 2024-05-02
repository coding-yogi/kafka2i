use std::{ time::Duration, fmt::Display};

use rdkafka::{consumer::{base_consumer::BaseConsumer, Consumer as KafkaConsumer}, ClientConfig, config::FromClientConfig, util::Timeout, error::KafkaError, message::BorrowedMessage, Offset};

use crate::metadata::Metadata;

type Result<T> = std::result::Result<T, ConsumerError>;

#[derive(Debug, Clone)]
pub struct ConsumerError {
    message: String,
}

impl Display for ConsumerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<KafkaError> for ConsumerError {
    fn from(value: KafkaError) -> Self {
        ConsumerError {
            message: value.to_string()
        }
    }
}

const DEFAULT_TIMEOUT_IN_SECS: u64 = 30;

// Wraps Kafka Consumer from the lib
pub struct Consumer {
    base_consumer: BaseConsumer,
    default_timeout_in_secs: Timeout,
}

impl Consumer {

    // New Consumer
    pub fn new(config: &ClientConfig) -> Result<Consumer> {
        // Base Consumer
        let base_consumer = BaseConsumer::from_config(config)?;

        // Time out
        let default_timeout = Timeout::After(Duration::new(DEFAULT_TIMEOUT_IN_SECS, 0));
        
        let consumer = Consumer {
            base_consumer,
            default_timeout_in_secs: default_timeout,
        };

        Ok(consumer)
    }

    // Fetch Metadata
    pub fn metadata(&self) -> Result<Metadata> {
        // Metadata
        let kafka_metadata = self.base_consumer.fetch_metadata(None, self.default_timeout_in_secs)?; 

        // Consumer group Metadata
        let cg_metadata = self.base_consumer.group_metadata();
        Ok(Metadata::new(kafka_metadata, cg_metadata))
    }

    // Consume
    pub fn consume(&self) -> Result<Option<BorrowedMessage>> {
        if let Some(msg_result) = self.base_consumer.poll(None) {
            let msg = msg_result?;
            return Ok(Some(msg));
        }    

        Ok(None)
    }

    // subscribe to a topic
    pub fn subscribe(&self, topics: &[&str]) -> Result<()>{
        self.base_consumer.subscribe(topics)?;
        Ok(())
    }

    // unsubscribe
    pub fn unsubscribe(&self) -> Result<()>{
        self.base_consumer.unsubscribe();
        Ok(())
    }

    // Seek
    pub fn seek(&self, topic: &str) -> Result<()> {
        self.base_consumer.seek(topic, 0, Offset::Beginning, Duration::from_secs(DEFAULT_TIMEOUT_IN_SECS))?;
        Ok(())
    }

}
