use rdkafka::Statistics;

use super::metadata::{Broker, Topic};

pub trait Stats {
    fn brokers(&self) -> Vec<Broker>;
    fn get_broker(&self, name: &str) -> Option<Broker>;
    fn brokers_list(&self) -> Vec<String>;
    fn topics_and_partitions(&self) -> Vec<Topic>;
    fn topics_list(&self) -> Vec<String>;
    fn get_topic(&self, name:  &str) -> Option<Topic>;
}

impl Stats for Statistics {
    fn brokers(&self) -> Vec<Broker> {
        self.brokers.iter()
            .filter(|(_, v)| v.source == "learned")
            .map(|(_,v)| v.into())
            .collect::<Vec<Broker>>()
    }

    fn get_broker(&self, name: &str) -> Option<Broker> {
       if let Some(b) = self.brokers().iter().find(|b| b.name() == name) {
           return Some((*b).clone())
       }

       return None
    }

    fn topics_and_partitions(&self) -> Vec<Topic> {
        self.topics.iter()
            .map(|(_, t)| t.into())
            .collect::<Vec<Topic>>()
    }

    fn get_topic(&self, name: &str) -> Option<Topic> {
        if let Some(t) = self.topics_and_partitions().iter().find(|t| t.name() == name) {
            return Some((*t).clone())
        }

        return None;
    }

    fn brokers_list(&self) -> Vec<String> {
        self.brokers().iter().map(|b| b.name().to_string()).collect::<Vec<String>>()
    }

    fn topics_list(&self) -> Vec<String> {
        self.topics_and_partitions().iter().map(|t| t.name().to_string()).collect::<Vec<String>>()
    }
}
