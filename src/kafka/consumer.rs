use std::{ time::Duration, fmt::Display, error::Error};
use crossbeam::channel::Sender;
use log::debug;
use rdkafka::{
    config::FromClientConfigAndContext, consumer::{
        base_consumer::BaseConsumer, 
        Consumer as KafkaConsumer, ConsumerContext, 
    }, error::KafkaError, metadata::Metadata as KafkaMetadata, types::RDKafkaErrorCode, util::Timeout, ClientConfig, ClientContext, Message, Offset, Statistics, TopicPartitionList
};

use crate::kafka::metadata::{Metadata, Topic};

use super::metadata::ConsumerGroup;

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

#[derive(Clone, Debug, Default)]
pub struct DefaultContext;

impl ConsumerContext for DefaultContext {}

impl ClientContext for DefaultContext {
    // Overriding stats as we do not wish to log the stats as part of the default implementatoion
    fn stats(&self, _statistics: rdkafka::Statistics) {
      ()
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
    default_timeout_in_secs: Timeout,
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
        
        let consumer = Consumer {
            base_consumer,
            default_timeout_in_secs: default_timeout,
            refresh_metadata_in_secs: DEFAULT_REFRESH_METADATA_IN_SECS,
            metadata: Metadata::new(),
            stats: Statistics::default(),
        };

        Ok(consumer)
    }

    // Fetch Metadata
    pub fn fetch_metadata(&self) -> Result<KafkaMetadata> {
        // Metadata
        debug!("fetching metadata ...");
        let kafka_metadata = self.base_consumer.fetch_metadata(None, self.default_timeout_in_secs)?; 
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
        let group_list = self.base_consumer.fetch_group_list(None, self.default_timeout_in_secs)?;

        Ok(group_list.groups().iter()
            .map(|g| g.into())
            .collect::<Vec<ConsumerGroup>>())
    }

    pub fn fetch_watermarks(&self, topic: &str, partition: i32) -> Result<(i64, i64)>{
        debug!("fetching watermarks for topic {}/{}", topic, partition);
        let watermarks = self.base_consumer.fetch_watermarks(topic, partition, self.default_timeout_in_secs)?;
        Ok(watermarks)
    }

    pub fn fetch_offset(&self, topic: &str, partition: i32) -> Result<i64> {
        debug!("fetching offset for topic {}/{}", topic, partition);
        let mut tpl = TopicPartitionList::new();
        tpl.add_partition(topic, partition);

        tpl = self.base_consumer.committed_offsets(tpl, self.default_timeout_in_secs)?;
        if let Some(tp) = tpl.elements().first() {
            if let Some(raw_offset) = tp.offset().to_raw() {
                return Ok(raw_offset);
            }
        }

        log::error!("failed to retrieve offset for topic {} & partition {}", topic, partition);
        Err(KafkaError::OffsetFetch(RDKafkaErrorCode::Fail).into())
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
    pub fn consume(&self, timeout: Duration) -> Result<Option<String>> {
        debug!("polling for a message");
        if let Some(msg_result) = self.base_consumer.poll(timeout) {
            let msg = msg_result?;
            if let Some(payload) = msg.payload_view::<str>().take() {
                return Ok(Some(payload.unwrap().to_owned()));
            } else {
                return Ok(Some("No Payload".to_string()));
            }
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
        tpl.add_partition_offset(topic, partition, Offset::End)?;
        let _ = self.base_consumer.assign(&tpl)?;
        Ok(())
    }

    pub fn assign_all_partitions(&self, topic: &Topic) -> Result<()>{
        debug!("assigning all partitions of topic {}", topic.name());
        let mut tpl = TopicPartitionList::new();
        for p in topic.partitions() {
            tpl.add_partition_offset(topic.name(), p.id(), Offset::End)?;
        }

        let _ = self.base_consumer.assign(&tpl)?;
        Ok(())
    }

    // Seek for a specific topic and partition
    pub fn seek(&self, topic: &str, partition: i32, offset: i64) -> Result<()> {
        debug!("seeking offset {}, on topic {}/{}", offset, topic, partition);
        self.base_consumer.seek(topic, partition, Offset::Offset(offset), DEFAULT_TIMEOUT_IN_SECS)?;
        
       
        Ok(())
    }

    // Seek for all topics in the partition
    pub fn seek_for_all_partitions(&self, topic: &Topic, offset: i64) -> Result<()>{
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
            self.seek(e.topic(), e.partition(), e.offset().to_raw().unwrap())?;
        }

        Ok(())
    }
}