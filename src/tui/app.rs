use std::{io::Stderr, sync::Arc};
use crossterm::event::{KeyEventKind, KeyCode};
use parking_lot::Mutex;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use rdkafka::{ClientContext, consumer::ConsumerContext};
use crate::kafka::consumer::Consumer;
use crate::tui::events::TuiEvent;
use crate::tui::widgets::Direction;

use super::single_layout::{AppLayout, DetailsLayout, ListsLayout, BROKERS_LIST_NAME, CONSUMER_GROUPS_LIST_NAME, PARTITIONS_LIST_NAME, TOPICS_LIST_NAME};

// App state maintains the state at app level
struct AppState {
    // should_quit tells the main loop to terminate the app
    should_quit: bool,
}

// App is the high level struct containing
// layout, state of the app itself
// event handler to update the state of the app or the undelying widget
// and a kafka client
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
        self.layout.main_layout.lists_layout.hande_tab();
    }

    // Handles the list navigation for the list in focus 
    async fn handle_list_navigation(&mut self, direction: Direction){
        let lists_layout = &mut self.layout.main_layout.lists_layout;
        lists_layout.handle_navigation(direction);

        // handle navigation events
        match lists_layout.selected_list().name() {
            BROKERS_LIST_NAME => self.handle_broker_list_navigation().await,
            TOPICS_LIST_NAME => self.handle_topic_list_navigation().await,
            CONSUMER_GROUPS_LIST_NAME => self.handle_cg_list_navigation().await,
            PARTITIONS_LIST_NAME => self.handle_partition_list_navigation().await,
            _ => log::error!("Selected list has an invalid name")
        }
    }

    // Handles broker list navigation
    // populates TUI with details of the broker selected in the list
    async fn handle_broker_list_navigation(&mut self) {
        let lists_layout = &mut self.layout.main_layout.lists_layout;
        let details_layout = &mut self.layout.main_layout.details_layout;

        if let Some(selected_broker) = lists_layout.get_list_by_name(BROKERS_LIST_NAME).unwrap().selected_item() {
            let broker_id = self.kafka_consumer.lock().metadata().get_broker(&selected_broker).unwrap().id();
            let partition_leader_count = self.kafka_consumer.lock().metadata().leader_for_paritions(broker_id);
            let broker_details = generate_broker_details(broker_id, "UP", partition_leader_count);
            details_layout.details.update_cell_data(BROKERS_LIST_NAME, 0, broker_details);
        }
    }

    // Handles topic list navigation
    // populates the TUI with details of the topic selected
    // populates the parition list with paritions of the selected topic
    async fn handle_topic_list_navigation(&mut self) {
        let lists_layout = &mut self.layout.main_layout.lists_layout;
        let details_layout = &mut self.layout.main_layout.details_layout;

        if let Some(selected_topic) = lists_layout.get_list_by_name(TOPICS_LIST_NAME).unwrap().selected_item() {
            // get partitions for the topic
            if let Some(topic) = self.kafka_consumer.lock().metadata().get_topic(&selected_topic) {
                let partitions_names = topic.partition_names();
                self.layout.main_layout.lists_layout.get_list_by_name(PARTITIONS_LIST_NAME).unwrap().update(partitions_names);
            }
        }
    }

    // Handles partition list navidation
    // populates the TUI with details of the partition selected
    async fn handle_partition_list_navigation(&mut self) {
        let lists_layout = &mut self.layout.main_layout.lists_layout;
        let details_layout = &mut self.layout.main_layout.details_layout;

        if let Some(selected_partition) = lists_layout.get_list_by_name(PARTITIONS_LIST_NAME).unwrap().selected_item() {
            let partition = self.kafka_consumer.lock().metadata().get_partition(&selected_partition).unwrap();
            let partition_details = generate_partition_details(partition.leader(), partition.isr().len(), partition.replicas().len());
            details_layout.details.update_cell_data(PARTITIONS_LIST_NAME, 0, partition_details);
        }
    }

    // Handles consumer group list navigation
    // populates the TUI with the details of selected consumer groups
    async fn handle_cg_list_navigation(&mut self) {
        let lists_layout = &mut self.layout.main_layout.lists_layout;
        let details_layout = &mut self.layout.main_layout.details_layout;

        if let Some(selected_cg) = lists_layout.get_list_by_name(CONSUMER_GROUPS_LIST_NAME).unwrap().selected_item() {
            let cg = self.kafka_consumer.lock().metadata().get_consumer_group(&selected_cg).unwrap();
            let cg_details = generate_consumer_group_details(cg.state(), cg.members_count());
            details_layout.details.update_cell_data(CONSUMER_GROUPS_LIST_NAME, 0, cg_details);
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

fn generate_broker_details(id: i32, status: &str, partitions: usize) -> String {
    format!("\nID         : {}\nStatus     : {}\nPartitions : {}", id, status, partitions)
}

fn generate_consumer_group_details(state: &str, members: usize) -> String {
    format!("\nState   : {}\nMembers : {}", state, members)
}

fn generate_partition_details(leader: i32, isr: usize, replicas: usize) -> String {
    format!("\nLeader : {}\nISR    : {} / {}", leader, isr, replicas)
}