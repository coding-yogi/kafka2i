use std::{error::Error, io::Stderr, sync::Arc, thread, time::Duration};

use clap::Parser;
use crossbeam::channel::{bounded, unbounded};
use crossterm::event::{KeyEventKind, KeyCode};
use kafka::consumer::StatsContext;
use parking_lot::Mutex;
use rdkafka::{consumer::ConsumerContext, ClientConfig, ClientContext, Statistics};
use crossterm::{terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen}, execute};
use ratatui::{prelude::CrosstermBackend, Terminal};
use tokio::time;
use tui::{app::App, app::AppEvent, events};

use crate::kafka::{consumer::{Consumer, DefaultContext}};
use crate::config::Config;
use crate::tui::events::TuiEvent;

mod kafka;
mod cmd;
mod config;
mod tui;
mod logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _logger = logger::initiate();
    
    // Parsing config from command line args
    let config = Config::parse();
    let client_config: ClientConfig = config.try_into()?;

    // Setup Kafka consumer to consume messages
    log::debug!("creating new kafka consumer to consume messages");
    let message_consumer = Arc::new(Mutex::new(Consumer::new(&client_config, DefaultContext).unwrap()));

    log::debug!("fetching metadata for first time");
    let metadata = message_consumer.lock().fetch_metadata()?;
    let consumer_groups = message_consumer.lock().fetch_groups()?;
    message_consumer.lock().update_metadata(metadata, consumer_groups);

    // Clone
    let message_consumer_clone = message_consumer.clone();
    let refresh_metadata_duration = message_consumer_clone.lock().refresh_metadata_in_secs;

    //let (stats_sender, stats_receiver) = bounded::<Statistics>(5);
        
    // Another consumer to fetch metadata and stats
    // This consumer will be sent to a thread to refresh metadata at a certain frequency
    //log::debug!("creating a new stats consumer to consume metadata and stats");
    //let stats_consumer = Consumer::new(&client_config, StatsContext::new(stats_sender)).unwrap();
  
    // spawn a task to poll stats consumer at regular interval
    // polling is required to receive stats from the callback
     let handle = tokio::spawn(async move {
        loop {
            // poll to pull stats
            //let _ = stats_consumer.consume();
            let _ = message_consumer_clone.lock().consume(Duration::from_secs(1));

            // receive stats
            //match stats_receiver.recv_timeout(Duration::from_secs(5)) {
            //    Ok(_stats) => {
            //        //Update stats for message consumer
            //        //message_consumer_clone.lock().update_stats(stats);
            //    },
            //    Err(_) => log::error!("timed out while receiving stats")
           // }
        
            // Reason for pulling metadata using StatsConsumer and then updating message consumer 
            // is to avoid message consumer from being locked for longer time during fetching of metadata
            // thus avoiding the lag on TUI
            log::debug!("refreshing metadata");
            let metadata = message_consumer_clone.lock().fetch_metadata().unwrap();
            let consumer_groups = message_consumer_clone.lock().fetch_groups().unwrap();
            message_consumer_clone.lock().update_metadata(metadata, consumer_groups);

            // sleep for refresh duration
            time::sleep(refresh_metadata_duration).await;
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

}
        
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
    let (sender, receiver) = unbounded::<AppEvent>();
    let mut app = App::new(consumer, receiver).await;
    let app_layout = app.layout();
    let mut events = events::EventHandler::new(1.0, 30.0);

    let should_quit = Arc::new(Mutex::new(false));
    let should_quit_clone = should_quit.clone();

    // spawn 2 scoped threads

    thread::scope(|s| {
        s.spawn(|| {
            loop {
                let event = events.next().unwrap();
                match event {
                    TuiEvent::Key(key) => {
                        match key.kind {
                            KeyEventKind::Press => {
                                let _ = match key.code {
                                    KeyCode::Tab => sender.send(AppEvent::Tab),
                                    KeyCode::Up => sender.send(AppEvent::Up),
                                    KeyCode::Down => sender.send(AppEvent::Down),
                                    KeyCode::Left => sender.send(AppEvent::Left),
                                    KeyCode::Right => sender.send(AppEvent::Right),
                                    KeyCode::Esc => {
                                        let res = sender.send(AppEvent::Esc);
                                        if *should_quit.lock() {
                                            break;
                                        }
                                        res
                                    },
                                    KeyCode::Enter => sender.send(AppEvent::Enter),
                                    KeyCode::Char(':') => sender.send(AppEvent::Edit),
                                    KeyCode::Char(input) => sender.send(AppEvent::Input(input)),
                                    KeyCode::Backspace => sender.send(AppEvent::Backspace),
                                    _ => Ok(())
                                };
                            }
                            // for any other KeyEventKind
                            _ => ()
                        }
                    },
                    TuiEvent::Render => {
                       let _ =  t.draw(|f| {
                            app_layout.lock().render(f)
                        });
                    } ,
                    // ignore any other events for now
                    _ => ()
                }
            }
        });

        s.spawn(|| {
            app.event_handler();
            *should_quit_clone.lock() = true;
        });
    });
   
    Ok(())
}
