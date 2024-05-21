use std::{error::Error, io::Stderr, time::Duration, thread, sync::Arc};

use clap::Parser;
use crossbeam::channel::bounded;
use kafka::consumer::StatsContext;
use parking_lot::Mutex;
use rdkafka::{ClientConfig, Message, consumer::{DefaultConsumerContext, ConsumerContext}, Statistics, ClientContext};
use crossterm::{terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen}, execute};
use ratatui::{prelude::CrosstermBackend, Terminal};
use tokio::time;
use tui::{app::App, events};

use crate::kafka::consumer::{Consumer, Result as ConsumerResult, DefaultContext};
use crate::config::Config;

mod kafka;
mod cmd;
mod config;
mod tui;
mod logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    logger::initiate();
    
    // Parsing config from command line args
    let config = Config::parse();
    let client_config: ClientConfig = config.try_into().unwrap();

    // Setup Kafka consumer to consume messages
    log::info!("creating new kafka consumer to consume messages");
    let message_consumer = Arc::new(Mutex::new(Consumer::new(&client_config, DefaultContext).unwrap()));
    let _ = message_consumer.lock().fetch_metadata();

    // Clone
    let message_consumer_clone = message_consumer.clone();

    //let (stats_sender, stats_receiver) = bounded::<Statistics>(5);
        
    //Another consumer to fetch metadata and stats
    // log::info!("creating a new consumer to consumer metadata and stats");
    // let mut stats_consumer = Consumer::new(&client_config, StatsContext::new(stats_sender)).unwrap();
    
    let handle = tokio::spawn(async move {
        loop {
            
        /*
            //fetch metadata
            let _ = stats_consumer.fetch_metadata();
            let refresh_time = stats_consumer.refresh_metadata_in_secs;
            
            // poll to pull stats
            let _ = stats_consumer.consume();

            // receive stats
            match stats_receiver.recv_timeout(Duration::from_secs(1)) {
                Ok(s) => {
                    //Update stats for message consumer
                    log::info!("{:?}", s);
                    message_consumer_clone.lock().update_stats(s);
                },
                Err(_) => ()
            }
        */
            let _ = message_consumer_clone.lock().fetch_metadata();
            let refresh_time = message_consumer_clone.lock().refresh_metadata_in_secs;

            // sleep
            time::sleep(refresh_time).await;

        }
    });

    //setup TUI
    setup().unwrap();
    
    // Run TUI
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr())).unwrap();
    let result = run(&mut t, message_consumer).await;

    // Shutdown TUI
    shutdown().unwrap();    
    result.unwrap(); 
    // Handle
    drop(handle);
    Ok(())



/*
    loop {
        thread::sleep(Duration::from_secs(1));
    }

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

    // print group list
    consumer.groups().unwrap();
    */
 } 

fn consume<T: ClientContext + ConsumerContext>(consumer: &Consumer<T>) -> ConsumerResult<()> {
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

fn setup() -> Result<(), Box<dyn Error>>{
    enable_raw_mode()?;
    execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

fn shutdown() -> Result<(), Box<dyn Error>> {
  execute!(std::io::stderr(), LeaveAlternateScreen)?;
  disable_raw_mode()?;
  Ok(())
}

async fn run<'a, T: ClientContext + ConsumerContext>(t: &'a mut Terminal<CrosstermBackend<Stderr>>, consumer: Arc<Mutex<Consumer<T>>>) -> Result<(), Box<dyn Error>> {
    // ratatui terminal
    let mut app = App::new(t, consumer).await;
    let mut events = events::EventHandler::new(1.0, 30.0);

    loop {
        let event = events.next().await?;
        app.event_handler(event).await;

        if app.should_quit() {
            break;
        }
    }
       
    Ok(())
}
