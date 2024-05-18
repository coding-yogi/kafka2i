use std::{io::Stderr, sync::Arc};
use crossterm::event::{KeyEventKind, KeyCode};
use parking_lot::Mutex;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use rdkafka::{ClientContext, consumer::ConsumerContext};
use crate::kafka::metadata;
use crate::kafka::consumer::Consumer;
use crate::tui::events::TuiEvent;
use crate::tui::widgets::Direction;

use super::single_layout::{AppLayout, TOPICS_LIST_NAME, PARTITIONS_LIST_NAME};

// App state maintains the state at app level
struct AppState {
    // should quit tells the main loop to terminate the app
    should_quit: bool,
}

// App is the high level struct containing
// Widgets
// State of the app itself
// Event Handler to update the state of the app or the undelying widget
// And a Kafka client
pub struct App<'a, T> 
where T: ClientContext + ConsumerContext
{
    layout: AppLayout<'a>,
    state: AppState,
    terminal: &'a mut Terminal<CrosstermBackend<Stderr>>,
    kafka_consumer: Arc<Mutex<Consumer<T>>>,
}

// This impl block only defines the new state of the app
impl <'a, T> App<'a, T> 
where T: ClientContext + ConsumerContext
{
    pub async fn new(t: &'a mut Terminal<CrosstermBackend<Stderr>>, kafka_consumer: Arc<Mutex<Consumer<T>>>) -> App<'a, T> {
       
       let metadata = kafka_consumer.lock().metadata().clone();

        App {
           layout: AppLayout::new(&metadata),
           state: AppState {
                should_quit: false,
           },
           terminal: t,
           kafka_consumer,
        }
    }
}

// This impl block defines all the methods related to state of the app
impl <T> App<'_, T>
where T: ClientContext + ConsumerContext {
    // should_quit is defined at app level so its easier to call from main method
    pub fn should_quit(&self) -> bool {
        self.state.should_quit
    }
    
    // Event handler which defines the high level handlers for every type of event handled in TUI
    pub async fn event_handler(&mut self, event: TuiEvent) {
        match event {
            TuiEvent::Key(key) => {
                match key.kind {
                    KeyEventKind::Press => {
                        match key.code {
                            KeyCode::Tab => self.handle_tab().await,
                            KeyCode::Up => self.handle_list_navigation(Direction::UP).await,
                            KeyCode::Down => self.handle_list_navigation(Direction::DOWN).await,
                            KeyCode::Esc => self.state.should_quit = true,
                            _ => ()
                        }
                    }
                    // any other KeyEventKind
                    _ => ()
                }
            }
            TuiEvent::Render => self.render(),
            // ignore any other events for now
            _ => ()
        }
    }

    // Handles tab event which switches between the available tabs
    async fn handle_tab(&mut self) {
        //normalize border of current list and highlight of next list
        self.layout.main_layout.lists_layout.hande_tab();
    }

    // Handles the list navigation for the list in focus 
    async fn handle_list_navigation(&mut self, direction: Direction){
        let lists_layout = &mut self.layout.main_layout.lists_layout;
        lists_layout.handle_navigation(direction);

        match lists_layout.selected_list().name() {
            // If selected block is Topics, populate the paritions list
            TOPICS_LIST_NAME => {
                if let Some(selected_topic) = lists_layout.get_list_by_name(TOPICS_LIST_NAME).unwrap().selected_item() {
                    // get partitions for the topic
                    if let Some(topic) = self.kafka_consumer.lock().metadata().get_topic(&selected_topic) {
                        let partitions_names = topic.partition_names();
                        self.layout.main_layout.lists_layout.get_list_by_name(PARTITIONS_LIST_NAME).unwrap().update(partitions_names);
                    }
                }
            },
            
            // If selected block is partition, show current offset
            PARTITIONS_LIST_NAME => {
                if let Some(selected_partition) = lists_layout.get_list_by_name(PARTITIONS_LIST_NAME).unwrap().selected_item() {
                }
            },
            _ => ()
        }
    }
}

// this impl block handles app rendering logic
impl <T> App<'_, T> 
where T: ClientContext + ConsumerContext
{
    pub fn render(&mut self) {
        let _ = self.terminal.draw(|f| {
            self.layout.render(f); 
        });
    }
}


