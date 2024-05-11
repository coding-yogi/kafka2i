use std::{io::Stderr, sync::Arc};
use crossterm::event::{KeyEventKind, KeyCode};
use parking_lot::Mutex;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::ScrollbarOrientation;
use rdkafka::{ClientContext, consumer::ConsumerContext};

use crate::kafka::metadata;
use crate::kafka::{consumer::Consumer, stats::Stats};
use crate::tui::events::TuiEvent;
use crate::tui::widgets::Direction;

use super::widgets::UIParagraphWithScrollbar;
use super::single_layout::AppLayout;

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
    
    }

    // Handles the list navigation for the list in focus 
    async fn handle_list_navigation(&mut self, direction: Direction){
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
