
use rdkafka::{metadata::{Metadata as KafkaMetadata, MetadataTopic, MetadataPartition, MetadataBroker}};

pub struct Metadata {
    brokers: Vec<Broker>,
    topics: Vec<Topic>,
}

impl Metadata {

    pub fn new() -> Metadata {
        Metadata {
            brokers: vec![],
            topics: vec![],
        }
    }

    pub fn update(&mut self, metadata: &KafkaMetadata) {
        let mut brokers = vec![];
        let brokers_metadata = metadata.brokers();
        for b in brokers_metadata {
            brokers.push(b.into())
        }
 
        let mut topics = vec![];
        let topics_metadata = metadata.topics();
        for t in topics_metadata {
           topics.push(t.into()) 
        }

        self.brokers = brokers;
        self.topics = topics;
    }
}

pub struct Broker {
    id: i32,
    host: String,
    port: i32,
}

impl Broker {
    pub fn new(id: i32, host: &str, port: i32) -> Broker {
        Broker {
            id,
            host: host.to_string(),    
            port,
        }
    }

    pub fn name(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn id(&self) -> i32 {
        self.id
    }
}

impl From<&MetadataBroker> for Broker {
    fn from(value: &MetadataBroker) -> Self {
        Broker { 
            id: value.id(), 
            host: value.host().to_string(), 
            port: value.port() 
        }
    }
}

#[derive(Debug)]
pub struct Topic {
    name: String,
    partitions: Vec<Partition>,
}

impl Topic {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn partitions(&self) -> &[Partition] {
        &self.partitions
    }
}

impl From<&MetadataTopic> for Topic {
    fn from(value: &MetadataTopic) -> Self {

        let mut partitions = vec![];

        let partitions_metadata = value.partitions();
        for p in partitions_metadata {
           partitions.push(p.into()) ;
        }

        Topic { 
            name: value.name().to_string(), 
            partitions 
        }
    }
}

#[derive(Debug)]
pub struct Partition {
    id: i32,
    leader: i32,
    isr: Vec<i32>,
    replicas: Vec<i32>,
}

impl Partition {
    pub fn id(&self) -> i32 {
        self.id
    }
}

impl From<&MetadataPartition> for Partition {
    fn from(value: &MetadataPartition) -> Partition {
        Partition { 
            id: value.id(), 
            leader: value.leader(), 
            isr: value.isr().to_vec(),
            replicas: value.replicas().to_vec() 
        }
    }
}


