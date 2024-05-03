use std::{thread, time::Duration};

use log::info;
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

    let topics = metadata.topics();
    log::info!("List of topics: {:?}", topics.iter().map(|t| t.name().to_string()).collect::<Vec<String>>());

    log::info!("Partitions for topic {}: {:?}", topics[0].name() , topics[0].partitions().iter().map(|p| p.id()).collect::<Vec<i32>>());
    
        
    // resetting offset to beginning
    match consumer.seek(topics[0].name(), rdkafka::Offset::Beginning){

        Ok(()) => (),
        Err(err) => {
            log::error!("{}", err);
            return;
        }
    };

    log::info!("subscribing to topic {}", topics[0].name());
    
    match consumer.subscribe(&vec![topics[0].name()]) {
        Ok(()) => (),
        Err(err) => {
            log::error!("{}", err);
            return;
        }
    };

    thread::sleep(Duration::from_secs(3));

    /*
    log::info!("seeking offset to beginning");
    match consumer.seek_to_beginning() {
        Ok(()) => (),
        Err(err) => {
            log::error!("{}", err);
            return;
        }
    };
    */

    log::info!("consuming from topics");
    match consumer.consume() {
        Ok(opt_message) => {
            if let Some(msg) = opt_message {
                log::info!("{}", std::str::from_utf8(msg.payload().unwrap()).unwrap());
            } else {
                log::info!("no message");
            }
        },
        Err(err) => {
            log::error!("{}",err);
            return;
        }
    }
 }  
