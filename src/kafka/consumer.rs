use std::{ time::Duration, fmt::Display};

use rdkafka::{
    consumer::{
        base_consumer::BaseConsumer, 
        Consumer as KafkaConsumer, 
    }, 
    ClientConfig, 
    util::Timeout, 
    error::KafkaError, 
    message::BorrowedMessage, 
    Offset, 
    TopicPartitionList, config::FromClientConfig, 
    
};

use crate::kafka::metadata::{Metadata, Topic};

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

impl From<KafkaError> for ConsumerError {
    fn from(value: KafkaError) -> Self {
        ConsumerError {
            message: value.to_string()
        }
    }
}

const DEFAULT_TIMEOUT_IN_SECS: Duration = Duration::from_secs(30);
const DEFAULT_REFRESH_METADATA_IN_SECS: Duration = Duration::from_secs(30);

// Wraps Kafka Consumer from the lib
pub struct Consumer {
    base_consumer: BaseConsumer,
    default_timeout_in_secs: Timeout,
    pub refresh_metadata_in_secs: Duration,
    metadata: Metadata
}

impl Consumer {

    // New Consumer
    pub fn new(config: &ClientConfig) -> Result<Consumer> {
        // Base Consumer
        let base_consumer = BaseConsumer::from_config(config)?;

        // Time out
        let default_timeout = Timeout::After(DEFAULT_TIMEOUT_IN_SECS);
        
        let consumer = Consumer {
            base_consumer,
            default_timeout_in_secs: default_timeout,
            refresh_metadata_in_secs: DEFAULT_REFRESH_METADATA_IN_SECS,
            metadata: Metadata::new(),
        };

        Ok(consumer)
    }
}

impl Consumer{
    // Fetch Metadata
    pub fn fetch_metadata(&mut self) -> Result<()> {
        // Metadata
        let kafka_metadata = self.base_consumer.fetch_metadata(None, self.default_timeout_in_secs)?; 
        self.metadata.update(&kafka_metadata);
        Ok(())
    }

    // Return Metadata
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
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

    // Assign
    pub fn assign(&self, topic: &str, partition: i32) -> Result<()>{
        let mut tpl = TopicPartitionList::new();
        tpl.add_partition(topic, partition);
        let _ = self.base_consumer.assign(&tpl)?;
        Ok(())
    }

    pub fn assign_all_partitions(&self, topic: &Topic) -> Result<()>{
        let mut tpl = TopicPartitionList::new();
        for p in topic.partitions() {
            tpl.add_partition(topic.name(), p.id());
        }

        let _ = self.base_consumer.assign(&tpl)?;
        Ok(())
    }

    // Seek for a specific topic and partition
    pub fn seek(&self, topic: &str, partition: i32, offset: Offset) -> Result<()> {
        self.base_consumer.seek(topic, partition, offset, DEFAULT_TIMEOUT_IN_SECS)?;
        Ok(())
    }

    // Seek for all topics in the partition
    pub fn seek_for_all_partitions(&self, topic: &Topic, offset: Offset) -> Result<()>{
        for p in topic.partitions() {
            let _ = self.seek(topic.name(), p.id(), offset)?;
        }

        Ok(())
    }

    // set offsets against a timestamp for a TPL fetched from assignment
    pub fn seek_on_timestamp(&self, timestamp: i64) -> Result<()> {
        let tpl = self.base_consumer.offsets_for_timestamp(timestamp, DEFAULT_TIMEOUT_IN_SECS)?;
        for e in tpl.elements() {
            log::debug!("seeking on topic {} & partition {}, offset {} with err {:?}", e.topic(), e.partition(), e.offset().to_raw().unwrap(), e.error());
            self.seek(e.topic(), e.partition(), e.offset())?;
        }

        Ok(())
    }

    pub fn groups(&self) -> Result<()>{
        let group_list = self.base_consumer.fetch_group_list(None, self.default_timeout_in_secs)?;
        for group in group_list.groups() {
            log::info!("group name : {} , state: {}, proto: {}", group.name(), group.state(), group.protocol());
        }

        Ok(())
    }
}

