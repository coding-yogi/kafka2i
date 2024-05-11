use rdkafka::groups::{GroupInfo, GroupMemberInfo};
use rdkafka::metadata::{Metadata as KafkaMetadata, MetadataTopic, MetadataPartition, MetadataBroker};
use rdkafka::statistics::{Broker as StatsBroker, Topic as StatsTopic, Partition as StatsPartition};

#[derive(Debug, Clone)]
pub struct Metadata {
    brokers: Vec<Broker>,
    topics: Vec<Topic>,
    consumer_groups: Vec<ConsumerGroup>
}

impl Metadata {

    pub fn new() -> Metadata {
        Metadata {
            brokers: vec![],
            topics: vec![],
            consumer_groups: vec![],
        }
    }

    pub fn update(&mut self, metadata: &KafkaMetadata, consumer_groups: Vec<ConsumerGroup>) {
       self.brokers =  metadata.brokers().iter()
            .map(|b| b.into())
            .collect::<Vec<Broker>>();

        self.topics = metadata.topics().iter()
            .map(|t| t.into())
            .collect::<Vec<Topic>>();

        self.consumer_groups = consumer_groups;
    }

    pub fn brokers_list(&self) -> Vec<String> {
        self.brokers.iter()
            .map(|b| b.name.clone())
            .collect()
    }

    pub fn topics_list(&self) -> Vec<String> {
        self.topics.iter()
            .map(|t| t.name.clone())
            .collect()
    }

    pub fn consumer_group_lists(&self) -> Vec<String> {
        self.consumer_groups.iter()
            .map(|g| g.name.clone())
            .collect()
    }

    pub fn get_broker(&self, name: &str) -> Option<Broker> {
        if let Some(broker) = self.brokers.iter().find(|b| b.name == name) {
            return Some((*broker).clone())
        }

        return None
    }

    fn get_topic(&self, name: &str) -> Option<Topic> {
        if let Some(t) = self.topics.iter().find(|t| t.name() == name) {
            return Some((*t).clone())
        }

        return None;
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

#[derive(Debug, Clone)]
pub struct ConsumerGroup {
    name: String,
    members: Vec<ConsumerGroupMember>,
    state: String,
}

impl ConsumerGroup {
    fn name(&self) -> &str {
        &self.name
    }
}

impl From<&GroupInfo> for ConsumerGroup {
    fn from(value: &GroupInfo) -> Self {

        let members = value.members().iter()
            .map(|m| m.into())
            .collect::<Vec<ConsumerGroupMember>>();

        ConsumerGroup {
            name: value.name().to_string(),
            members,
            state: value.state().to_string()
        }
    }
}


#[derive(Debug, Clone)]
pub struct ConsumerGroupMember {
    id: String,
}

impl From<&GroupMemberInfo> for ConsumerGroupMember {
    fn from(value: &GroupMemberInfo) -> Self {
        ConsumerGroupMember {
            id: value.id().to_string()
        }
    }
}

