
use rdkafka::{metadata::{Metadata as KafkaMetadata, MetadataTopic, MetadataPartition, MetadataBroker}, consumer::ConsumerGroupMetadata};

pub struct Metadata {
    metadata: KafkaMetadata,
    cg_metadata: Option<ConsumerGroupMetadata>
}

impl Metadata {

    pub fn new(metadata: KafkaMetadata, cg_metadata: Option<ConsumerGroupMetadata>) -> Metadata {
        Metadata { 
            metadata, 
            cg_metadata
        }
    }

    pub fn brokers(&self) -> Vec<Broker> {
        let mut brokers = vec![];
        let brokers_metadata = self.metadata.brokers();
        for b in brokers_metadata {
            brokers.push(b.into())
        }

        brokers
    }

    pub fn topics(&self) -> Vec<Topic> {
        let mut topics = vec![];
        let topics_metadata = self.metadata.topics();
        for t in topics_metadata {
           topics.push(t.into()) 
        }

        topics
    }
}

pub struct Broker<'a> {
    id: i32,
    host: &'a str,
    port: i32,
}

impl<'a> Broker<'a> {
    pub fn new(id: i32, host: &str, port: i32) -> Broker {
        Broker {
            id,
            host,    
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

impl<'a> From<&'a MetadataBroker> for Broker<'a> {
    fn from(value: &'a MetadataBroker) -> Self {
        Broker { 
            id: value.id(), 
            host: value.host(), 
            port: value.port() 
        }
    }
}

#[derive(Debug)]
pub struct Topic<'a> {
    name: &'a str,
    partitions: Vec<Partition<'a>>,
}

impl<'a> Topic<'a> {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn partitions(&self) -> &[Partition] {
        &self.partitions
    }
}

impl <'a> From<&'a MetadataTopic> for Topic<'a> {
    fn from(value: &'a MetadataTopic) -> Self {

        let mut partitions = vec![];

        let partitions_metadata = value.partitions();
        for p in partitions_metadata {
           partitions.push(p.into()) ;
        }

        Topic { 
            name: value.name(), 
            partitions 
        }
    }
}

#[derive(Debug)]
pub struct Partition<'a> {
    id: i32,
    leader: i32,
    isr: &'a [i32],
    replicas: &'a [i32],
}

impl<'a> From<&'a MetadataPartition> for Partition<'a> {
    fn from(value: &'a MetadataPartition) -> Partition<'a> {
        Partition { 
            id: value.id(), 
            leader: value.leader(), 
            isr: value.isr(), 
            replicas: value.replicas() 
        }
    }
}


