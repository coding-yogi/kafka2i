use std::{thread, time::Duration, error::Error};

use clap::Parser;
use rdkafka::{ClientConfig, Message};
use crossterm::{terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen}, execute};
use ratatui::{prelude::CrosstermBackend, Terminal};
use tui::{app::App, events};

use crate::kafka::consumer::{Consumer, Result as ConsumerResult};
use crate::config::Config;


mod kafka;
mod cmd;
mod config;
mod tui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    env_logger::init();
    
    // Parsing config from command line args
    let config = Config::parse();
    let client_config: ClientConfig = match config.try_into() {
        Ok(c) => c,
        Err(err) => {
            log::error!("Unable to parse config: {}", err);
            return Ok(());
        }
    }; 

    // Setup Kafka consumer
    log::info!("creating new consumer");
    let consumer = match Consumer::new(&client_config){
        Ok(c) => c,
        Err(err) => {
            print!("{:?}",err);
            return Ok(());
        }
    };

    //setup TUI
    setup()?;

    // Run TUI
    let result = run(&consumer).await;

    // Shutdown TUI
    shutdown()?;
    result?;
    
    Ok(())




/*    let metadata = match consumer.metadata() {
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

fn consume(consumer: &Consumer) -> ConsumerResult<()> {
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

async fn run(consumer: &Consumer) -> Result<(), Box<dyn Error>> {
    // ratatui terminal
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
    let mut app = App::new(&mut t, consumer).await;
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
