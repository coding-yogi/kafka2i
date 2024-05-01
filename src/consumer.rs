use std::{ time::Duration};

use rdkafka::{consumer::{base_consumer::BaseConsumer, Consumer as KafkaConsumer}, ClientConfig, config::FromClientConfig, util::Timeout, error::KafkaError, metadata::MetadataBroker};

use crate::metadata::Metadata;

type Result<T> = std::result::Result<T, ConsumerError>;

#[derive(Debug, Clone)]
struct ConsumerError {
    message: String,
}

impl ConsumerError {
    pub fn new(message: String) -> ConsumerError {
        ConsumerError {
            message
        }
    }
}

impl From<KafkaError> for ConsumerError {
    fn from(value: KafkaError) -> Self {
        ConsumerError {
            message: value.to_string()
        }
    }
}

struct ConsumerConfig {


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
    fn metadata(&self) -> Result<Metadata> {
        // Metadata
        let kafka_metadata = self.base_consumer.fetch_metadata(None, self.default_timeout_in_secs)?;

        // Consumer group Metadata
        let cg_metadata = self.base_consumer.group_metadata();
        Ok(Metadata::new(kafka_metadata, cg_metadata))
    }

}
