use rdkafka::{metadata::{Metadata as KafkaMetadata, MetadataTopic, MetadataPartition, MetadataBroker}};
use rdkafka::{statistics::{Broker as StatsBroker, Topic as StatsTopic, Partition as StatsPartition}};

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

    pub fn brokers_list(&self) -> Vec<String> {
        self.brokers.iter()
            .map(|b| b.name().to_string())
            .collect()
    }

    pub fn topics_list(&self) -> Vec<String> {
        self.topics.iter()
            .map(|t| t.name().to_string())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Broker {
    id: i32,
    name: String,
    state: String,
}

impl Broker {
    pub fn new(id: i32, host: &str, port: i32) -> Broker {
        Broker {
            id,
            name: format!("{}:{}/{}", host.to_string(), port, id),    
            state: "".to_string(),

        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn state(&self) -> &str {
        &self.state
    }
}

impl From<&MetadataBroker> for Broker {
    fn from(value: &MetadataBroker) -> Self {
        Broker::new(value.id(), value.host(), value.port())
    }
}

impl From<&StatsBroker> for Broker {
    fn from(value: &StatsBroker) -> Self {
        Broker {
            id: value.nodeid,
            name: value.name.clone(),
            state: value.state.clone(),
        }
    }
}

#[derive(Debug, Clone)]
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

impl From<&StatsTopic> for Topic {
    fn from(value: &StatsTopic) -> Self {
       let mut partitions = vec![];

       let partition_stats = &value.partitions;
       for (_, v) in partition_stats {
           partitions.push(v.into());
       }

       Topic {
           name: value.topic.clone(),
           partitions
       }
    }
}

#[derive(Debug, Clone)]
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

impl From<&StatsPartition> for Partition {
    fn from(value: &StatsPartition) -> Self {
        Partition {
            id: value.partition,
            leader: value.leader,
            isr: vec![],
            replicas: vec![],
        }
    }
}


