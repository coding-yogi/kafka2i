use std::{io::Stderr, sync::Arc};
use crossterm::event::{KeyEventKind, KeyCode};
use parking_lot::Mutex;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use rdkafka::{ClientContext, consumer::ConsumerContext};
use strum::{EnumIter, FromRepr, EnumCount, Display};

use crate::kafka::metadata;
use crate::kafka::consumer::Consumer;
use crate::tui::events::TuiEvent;
use crate::tui::widgets::Direction;

use super::single_layout::{AppLayout, TOPICS_LIST_NAME, PARTITIONS_LIST_NAME};

#[derive(Clone, Display, EnumIter, FromRepr, Default, EnumCount)]
enum SelectedBlock {
    #[default]
    #[strum(to_string = "Brokers")]
    Brokers = 1,

    #[strum(to_string = "Consumer Groups")]
    ConsumerGroups = 2,

    #[strum(to_string = "Topics")]
    Topics = 3,

    #[strum(to_string = "Partitions")]
    Partitions = 4
}

impl SelectedBlock {
    fn next(self) -> Self {
        let idx = self as usize;
        let next_idx = idx.saturating_add(1);

        // this unwrap won't panic as idx has been handled
        SelectedBlock::from_repr(next_idx).unwrap_or(SelectedBlock::default())
    }
}

// App state maintains the state at app level
struct AppState {
    // should quit tells the main loop to terminate the app
    should_quit: bool,

    // List selected 
    selected_block: SelectedBlock,
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
                selected_block: SelectedBlock::default(),
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
        self.layout.main_layout.lists_layout.normalise_border(&self.state.selected_block.to_string());
        self.state.selected_block = self.state.selected_block.clone().next();
        self.layout.main_layout.lists_layout.highlight_border(&self.state.selected_block.to_string());
    }

    // Handles the list navigation for the list in focus 
    async fn handle_list_navigation(&mut self, direction: Direction){
        let lists_layout = &mut self.layout.main_layout.lists_layout;
        lists_layout.handle_navigation(&self.state.selected_block.to_string(), direction);

        match self.state.selected_block {
            // If selected block is Topics, populate the paritions list
            SelectedBlock::Topics => {
                if let Some(selected_topic) = lists_layout.get_list_by_name(TOPICS_LIST_NAME).unwrap().selected_item() {
                    // get partitions for the topic
                    if let Some(topic) = self.kafka_consumer.lock().metadata().get_topic(&selected_topic) {
                        let partitions_names = topic.partition_names();
                        self.layout.main_layout.lists_layout.get_list_by_name(PARTITIONS_LIST_NAME).unwrap().update(partitions_names);
                    }
                }
            },
            
            // If selected block is partition, show current offset
            SelectedBlock::Partitions => {
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


