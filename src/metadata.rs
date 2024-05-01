
use rdkafka::{metadata::{Metadata as KafkaMetadata, MetadataTopic, MetadataPartition}, consumer::ConsumerGroupMetadata};

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
            brokers.push(Broker { 
                id: b.id(), 
                host: b.host().to_string(), 
                port: b.port()
            })
        }

        brokers
    }

    pub fn topic(&self) -> Vec<Topic> {
        let mut topics = vec![];
        let topics_metadata = self.metadata.topics();
        for t in topics_metadata {
           topics.push(t.into()) 
        }

        topics
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
}

pub struct Topic<'a> {
    name: String,
    partitions: &'a[Partition<'a>],
}

impl <'a> Topic<'a> {
    pub fn new(name: String, partitions: &'a[Partition]) -> Topic<'a> {
        Topic {
            name,
            partitions
        }
    }
}

impl <'a> From<&MetadataTopic> for Topic<'a> {
    fn from(value: &MetadataTopic) -> Self {

        let mut partitions = vec![];

        let partitions_metadata = value.partitions();
        for p in partitions_metadata {
           partitions.push(p.into()) ;
        }

        Topic { 
            name: value.name().to_string(), 
            partitions: &partitions 
        }
    }
}

pub struct Partition<'a> {
    id: i32,
    leader: i32,
    isr: &'a [i32],
    replicas: &'a [i32],
}

impl<'a> Partition<'a> {
    pub fn new(id: i32, leader: i32, isr: &'a[i32], replicas: &'a[i32]) -> Partition<'a> {
        Partition {
            id,
            leader,
            isr,
            replicas,
        }
    }
}

impl<'a> From<&MetadataPartition> for Partition<'a> {
    fn from(value: &MetadataPartition) -> Self {
        Partition { 
            id: value.id(), 
            leader: value.leader(), 
            isr: value.isr(), 
            replicas: value.replicas() 
        }
    }
}


