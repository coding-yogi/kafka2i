use std::{error::Error, io::Stderr, time::Duration, sync::Arc};

use clap::Parser;
use crossbeam::channel::bounded;
use kafka::consumer::StatsContext;
use parking_lot::Mutex;
use rdkafka::{consumer::ConsumerContext, ClientConfig, ClientContext, Statistics};
use crossterm::{terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen}, execute};
use ratatui::{prelude::CrosstermBackend, Terminal};
use tokio::time;
use tui::{app::App, events};

use crate::kafka::consumer::{Consumer, DefaultContext};
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
    let client_config: ClientConfig = config.try_into()?;

    // Setup Kafka consumer to consume messages
    log::debug!("creating new kafka consumer to consume messages");
    let message_consumer = Arc::new(Mutex::new(Consumer::new(&client_config, DefaultContext).unwrap()));
    let metadata = message_consumer.lock().fetch_metadata()?;
    let consumer_groups = message_consumer.lock().fetch_groups()?;
    message_consumer.lock().update_metadata(metadata, consumer_groups);

    // Clone
    let message_consumer_clone = message_consumer.clone();

    let (stats_sender, stats_receiver) = bounded::<Statistics>(5);
        
    // Another consumer to fetch metadata and stats
    log::debug!("creating a new stats consumer to consume metadata and stats");
    let stats_consumer = Consumer::new(&client_config, StatsContext::new(stats_sender)).unwrap();
    
    // spawn a task to poll stats consumer at regular interval
    // polling is required to receive stats from the callback
    let handle = tokio::spawn(async move {
        loop {
            // poll to pull stats
            let _ = stats_consumer.consume();

            // receive stats
            match stats_receiver.recv_timeout(Duration::from_secs(1)) {
                Ok(_stats) => {
                    //Update stats for message consumer
                    //message_consumer_clone.lock().update_stats(stats);
                },
                Err(_) => log::error!("timed out while receiving stats")
            }
        
            // Reason for pulling metadata using StatsConsumer and then updating message consumer 
            // is to avoid message consumer from being locked for longer time during fetching of metadata
            // thus avoiding the lag on TUI
            log::debug!("refreshing metadata");
            let metadata = stats_consumer.fetch_metadata().unwrap();
            let consumer_groups = stats_consumer.fetch_groups().unwrap();
            message_consumer_clone.lock().update_metadata(metadata, consumer_groups);
            let refresh_time = message_consumer_clone.lock().refresh_metadata_in_secs;

            // sleep for refresh duration
            time::sleep(refresh_time).await;
        }
    });

    //setup TUI
    setup()?;
    
    // Run TUI
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr())).unwrap();
    let result = run(&mut t, message_consumer).await;

    // Shutdown TUI
    shutdown()?;    
    result?; 

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

/*fn consume<T: ClientContext + ConsumerContext>(consumer: &Consumer<T>) -> ConsumerResult<()> {
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
}*/

fn setup() -> Result<(), Box<dyn Error>>{
    log::debug!("setting up TUI");
    enable_raw_mode()?;
    execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

fn shutdown() -> Result<(), Box<dyn Error>> {
    log::debug!("shutting down TUI");
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
