use std::{thread, time::Duration};

use consumer::Result;
use rdkafka::{ClientConfig, Message};

use crate::consumer::Consumer;

mod consumer;
mod producer;
mod cmd;
mod config;
mod metadata;

fn main() {

    let mut client_config = ClientConfig::new();
    client_config.set("bootstrap.servers", "localhost:9092")
        .set("group.id", "cd.krust.1")
        .set("auto.offset.reset", "earliest");

    env_logger::init();
    log::info!("creating new consumer");

    let consumer = match Consumer::new(&client_config){
        Ok(c) => c,
        Err(err) => {
            print!("{:?}",err);
            return;
        }
    };

    let metadata = match consumer.metadata() {
        Ok(m) => m,
        Err(err) => {
            print!("{:?}",err);
            return;
        }
    };

    // Assign all topics to this consumer and reset offset to 
    let test_topic = &metadata.topics()[0];
        
    // assign topic
    match consumer.assign_all_partitions(test_topic) {
        Ok(()) => (),
        Err(err) => {
            log::error!("{}", err);
            return;
        }
    };

    // Probably should wait for some event
    thread::sleep(Duration::from_secs(2));

    // seek to beginning
    match consumer.seek_for_all_partitions(test_topic, rdkafka::Offset::Beginning) {
        Ok(()) => (),
        Err(err) => {
            log::error!("{}", err)
        }
    };
    
   // thread::sleep(Duration::from_secs(3));

    log::info!("consuming from topics");

    for i in 1..5 {
        if consume(&consumer).is_err() {
            return;
        }
    }

    // seek based on timestamp
    match consumer.seek_on_timestamp(1714732185000) {
        Ok(()) => (),
        Err(err) => {
            log::error!("{}", err);
            return;
        }
    };

    // consumer
    for i in 1..2 {
        if consume(&consumer).is_err() {
            return;
        }
    }

    // seek based on offset number
    match consumer.seek_for_all_partitions(test_topic, rdkafka::Offset::Offset(1)){
        Ok(()) => (),
        Err(err) => {
            log::error!("{}", err);
            return;
        }
    };
   
    for i in 1..2 {
        if consume(&consumer).is_err() {
            return;
        }
    }
 } 

fn consume(consumer: &Consumer) -> Result<()> {
    match consumer.consume() {
        Ok(opt_message) => {
            if let Some(msg) = opt_message {
                log::info!("{}", std::str::from_utf8(msg.payload().unwrap()).unwrap());
            } else {
                log::info!("no message");
            }

            return Ok(());
        },
        Err(err) => {
            log::error!("{}",err);
            return Err(err);
        }
    }
}
