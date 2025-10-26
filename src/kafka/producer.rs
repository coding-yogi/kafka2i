use std::{error::Error, fmt::Display, time::Duration};
use rdkafka::{config::FromClientConfigAndContext, error::KafkaError, message::Header, producer::{FutureProducer, FutureRecord, Partitioner}, util::Timeout, ClientConfig, ClientContext};

pub type Result<T> = std::result::Result<T, ProducerError>;

#[derive(Debug, Clone)]
pub struct ProducerError {
    message: String,
}

impl Display for ProducerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ProducerError {}

impl From<KafkaError> for ProducerError {
    fn from(value: KafkaError) -> Self {
        ProducerError {
            message: value.to_string()
        }
    }
}

const DEFAULT_QUEUE_TIMEOUT_IN_MS: Duration = Duration::from_millis(500);

// Wraps Kafka producer from the lib
pub struct Producer<T>
where T: ClientContext + Partitioner + 'static{
    producer: FutureProducer<T>,
    queue_timeout: Timeout
}

impl <T> Producer<T>
where T: ClientContext + Partitioner + 'static{
    // Create new producer
    pub fn new(config: &ClientConfig, context: T) -> Result<Producer<T>> {
        let producer: FutureProducer<T> = FutureProducer::from_config_and_context(
            config,
            context,
        )?;

        let queue_timeout = Timeout::After(DEFAULT_QUEUE_TIMEOUT_IN_MS);

        Ok(Producer{
            producer,
            queue_timeout
        })
    }

    // Send message
    pub async fn send_message(
        &self,
        topic: &str,
        key: Option<&str>,
        headers: Vec<(String, String)>,
        payload: Option<&[u8]>,
    ) -> Result<()> {
        // headers
        let mut msg_headers = rdkafka::message::OwnedHeaders::new();
        for (k, v) in headers {
            msg_headers= msg_headers.insert(Header {
                key: &k,
                value: Some(v.as_bytes()),
            });
        }

        let record = FutureRecord::to(topic)
            .key(key.unwrap_or(""))
            .headers(msg_headers)
            .payload(payload.unwrap_or(&[]));

        match self.producer.send(record, self.queue_timeout).await {
            Ok(_) => Ok(()),
            Err((e, _)) => Err(ProducerError {
                message: e.to_string(),
            }),
        }
    }
}