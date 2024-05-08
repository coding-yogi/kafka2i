use std::io::Stderr;
use crossterm::event::{KeyEventKind, KeyCode};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use rdkafka::{ClientContext, consumer::ConsumerContext};

use crate::kafka::consumer::Consumer;
use crate::tui::events::TuiEvent;
use crate::tui::widgets::Direction;

use super::{layout::{AppLayout, SelectedTab}, widgets::UIList};

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
    kafka_consumer: Consumer<T>,
}

// This impl block only defines the new state of the app
impl <'a, T> App<'a, T> 
where T: ClientContext + ConsumerContext
{
    pub async fn new(t: &'a mut Terminal<CrosstermBackend<Stderr>>, kafka_consumer: Consumer<T>) -> App<'a, T> {
       App {
           layout: AppLayout::new(),
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
        self.layout.tabs_layout.tabs.handle_tab();
        let selected_tab = SelectedTab::from_repr(self.layout.tabs_layout.tabs.selected()).unwrap();


        match selected_tab {
            SelectedTab::BROKERS => {
                let broker_names = self.kafka_consumer.metadata().brokers_list();
                self.layout.tabs_layout.broker_layout.brokers_list = UIList::new("Brokers", broker_names);
            },
            SelectedTab::TOPICS => {
                let topics = self.kafka_consumer.metadata().topics_list();
                self.layout.tabs_layout.topics_layout.topics_list = UIList::new("Topics", topics);
            },
            _ => ()
        }
    }

    // Handles the list navigation for the list in focus 
    async fn handle_list_navigation(&mut self, direction: Direction){
        let selected_tab = SelectedTab::from_repr(self.layout.tabs_layout.tabs.selected()).unwrap();

        match selected_tab {
            SelectedTab::BROKERS => self.layout.tabs_layout.broker_layout.brokers_list.handle_navigation(direction),
            SelectedTab::CONSUMERGROUPS => self.layout.tabs_layout.cg_layout.cg_list.handle_navigation(direction),
            SelectedTab::TOPICS => self.layout.tabs_layout.topics_layout.topics_list.handle_navigation(direction),
        }

        // TODO: Handle population of details as per the option selected on the list
              
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
