use std::{char, sync::Arc, thread};
use crossbeam::channel::Receiver;
use log::{error, info};
use parking_lot::Mutex;
use rdkafka::{ClientContext, consumer::ConsumerContext};
use strum::{self, Display};
use crate::kafka::consumer::Consumer;
use crate::tui::widgets::Direction;

use super::{single_layout::{AppLayout, BROKERS_LIST_NAME, CONSUMER_GROUPS_LIST_NAME, PARTITIONS_LIST_NAME, TOPICS_LIST_NAME}, widgets::InputEvent};

// Mode of App
#[derive(Clone, Debug, Display, Default)]
enum Mode {
    #[default]
    #[strum(to_string="Consumer")]
    Consumer,
    #[strum(to_string="Producer")]
    Producer
}

#[derive(PartialEq)]
enum EditMode {
    Normal,
    Editing
}

pub enum AppEvent {
    Tab,
    Up,
    Down,
    Left,
    Right,
    Esc,
    Edit,
    Input(char),
    Backspace,
    Enter,
}

// App state maintains the state at app level
struct AppState {
    // should_quit tells the main loop to terminate the app
    should_quit: bool,
    //mode
    mode: Mode,
    //edit mode
    edit_mode: EditMode,
}

// App is the high level struct containing
// layout, state of the app itself
// event handler to update the state of the app or the undelying widget
// and a kafka client
pub struct App<'a, T> 
where T: ClientContext + ConsumerContext
{
    layout: Arc<Mutex<AppLayout<'a>>>,
    state: AppState,
    //terminal: &'a mut Terminal<CrosstermBackend<Stderr>>,
    kafka_consumer: Arc<Mutex<Consumer<T>>>,
    app_event_recv: Receiver<AppEvent>,
}

// This impl block only defines the new state of the app
impl <'a, T> App<'a, T> 
where T: ClientContext + ConsumerContext
{
    pub async fn new(kafka_consumer: Arc<Mutex<Consumer<T>>>, app_event_recv: Receiver<AppEvent>) -> App<'a, T> {
       let metadata = kafka_consumer.lock().metadata().clone();
       let mode = Mode::default();

        let app = App {
           layout: Arc::new(Mutex::new(AppLayout::new(&metadata))),
           state: AppState {
                should_quit: false,
                mode: mode.clone(),
                edit_mode: EditMode::Normal,
           },
           //terminal: t,
           kafka_consumer,
           app_event_recv
        };

        app.layout.lock().footer_layout.update_mode(mode.to_string());
        app
    }

    pub fn layout(&self) -> Arc<Mutex<AppLayout<'a>>> {
        self.layout.clone()
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
    pub fn event_handler(&mut self) {
        loop {
            match self.app_event_recv.recv() {
                Ok(event) => {
                    match self.state.edit_mode {
                        EditMode::Normal => {
                            match event {
                                AppEvent::Tab => self.handle_tab(),
                                AppEvent::Up => self.handle_list_navigation(Direction::UP),
                                AppEvent::Down => self.handle_list_navigation(Direction::DOWN),
                                AppEvent::Esc => {
                                    self.state.should_quit = true;
                                    break;
                                },
                                AppEvent::Edit => self.toogle_edit_mode(EditMode::Editing),
                                _ => (),
                            }
                        },
                        EditMode::Editing => {
                            match event {
                                AppEvent::Esc => self.toogle_edit_mode(EditMode::Normal),
                                AppEvent::Input(char) => self.handle_input_event(InputEvent::NewChar(char)),
                                AppEvent::Backspace => self.handle_input_event(InputEvent::RemovePrevChar),
                                AppEvent::Left => self.handle_input_event(InputEvent::MoveCursor(Direction::LEFT)),
                                AppEvent::Right => self.handle_input_event(InputEvent::MoveCursor(Direction::RIGHT)),
                                AppEvent::Enter => {
                                    self.handle_input_submission();
                                    self.toogle_edit_mode(EditMode::Normal);
                                },
                                _ => (),
                            }
                        },
                    }
                },
                Err(_) => log::error!("error occured while receiving app event")
            }
        }
    }

    // Handles tab event which switches between the available tabs
    fn handle_tab(&mut self) {
        self.layout.lock().main_layout.lists_layout.hande_tab();
    }

    // Handles the list navigation for the list in focus 
    fn handle_list_navigation(&mut self, direction: Direction){
        let mut layout_guard = self.layout.lock();
        let lists_layout = &mut layout_guard.main_layout.lists_layout;
        lists_layout.handle_navigation(direction);
        let seleted_list_name = lists_layout.selected_list().name().to_string().clone();
        drop(layout_guard);

        // handle navigation events
        match seleted_list_name.as_str() {
            BROKERS_LIST_NAME => self.handle_broker_list_navigation(),
            TOPICS_LIST_NAME => self.handle_topic_list_navigation(),
            CONSUMER_GROUPS_LIST_NAME => self.handle_cg_list_navigation(),
            PARTITIONS_LIST_NAME => self.handle_partition_list_navigation(),
            _ => log::error!("Selected list has an invalid name")
        }
    }

    // Handles broker list navigation
    // populates TUI with details of the broker selected in the list
    fn handle_broker_list_navigation(&mut self) {
        if let Some(selected_broker) = self.get_selected_item_for_list(BROKERS_LIST_NAME) {
            let broker = match self.kafka_consumer.lock().metadata().get_broker(&selected_broker) {
                Some(broker) => broker,
                None => {
                    error!("Unable to get broker details for broker with name: {}", selected_broker);
                    return;
                }
            };

            // update broker details
            let broker_id = broker.id();
            let partition_leader_count = self.kafka_consumer.lock().metadata().no_of_partitions_for_broker(broker_id);
            let broker_details = generate_broker_details(broker_id, "UP", partition_leader_count);
            self.layout.lock().main_layout.details_layout.details.update_cell_data(BROKERS_LIST_NAME, 0, broker_details);
        }
    }

    // Handles topic list navigation
    // populates the TUI with details of the topic selected
    // populates the parition list with paritions of the selected topic
    fn handle_topic_list_navigation(&mut self) {
        if let Some(selected_topic) = self.get_selected_item_for_list(TOPICS_LIST_NAME) {
            let topic = match self.kafka_consumer.lock().metadata().get_topic(&selected_topic) {
                Some(topic) => topic,
                None => {
                    error!("Unable to get topic details for topic with name: {}", selected_topic);
                    return;
                }
            };

            let topic_details = generate_topic_details(topic.partitions().len());
            self.layout.lock().main_layout.details_layout.details.update_cell_data(TOPICS_LIST_NAME, 0, topic_details);

            let partitions_names = topic.partition_names();
            match self.layout.lock().main_layout.lists_layout.get_list_by_name(PARTITIONS_LIST_NAME) {
                Some(list) => list.update(partitions_names),
                None => {
                    error!("No list found by name: {}", PARTITIONS_LIST_NAME);
                    return;
                }
            };
        }
    }

    // Handles partition list navidation
    // populates the TUI with details of the partition selected
    fn handle_partition_list_navigation(&mut self) {
        if let Some(selected_partition) = self.get_selected_item_for_list(PARTITIONS_LIST_NAME) {
            let partition = match self.kafka_consumer.lock().metadata().get_partition(&selected_partition) {
                Some(partition) => partition,
                None => {
                    error!("Unable to get details for partition with name: {}", selected_partition);
                    return;
                }
            };
            
            // get high water mark for the topic
            let mut high_watermark: i64 = -999;
            let mut offset: i64 = -999;

            if let Some((topic_name, partition_id)) = get_topic_and_parition_id(&selected_partition) {
                thread::scope(|s| {
                    s.spawn(|| {
                        match self.kafka_consumer.lock().fetch_watermarks(topic_name, partition_id) {
                            Ok((_, h)) => {
                                high_watermark = h;
                            },
                            Err(err) => error!("error while fetching watermark {}", err),
                        };
                    });

                    s.spawn(|| {
                       match self.kafka_consumer.lock().fetch_offset(topic_name, partition_id) {
                            Ok(o) => {
                                offset = o;
                            },
                            Err(err) => error!("error while fetching offsets {}", err),
                       };
                    });
                });
            }

            let partition_details = generate_partition_details(partition.leader(), partition.isr().len(), partition.replicas().len(), high_watermark, offset);
            self.layout.lock().main_layout.details_layout.details.update_cell_data(PARTITIONS_LIST_NAME, 0, partition_details);

        }
    }

    // Handles consumer group list navigation
    // populates the TUI with the details of selected consumer groups
    fn handle_cg_list_navigation(&mut self) {
        if let Some(selected_cg) = self.get_selected_item_for_list(CONSUMER_GROUPS_LIST_NAME) {
            match self.kafka_consumer.lock().metadata().get_consumer_group(&selected_cg) {
                Some(cg) => {
                    let cg_details = generate_consumer_group_details(cg.state(), cg.members_count());
                    self.layout.lock().main_layout.details_layout.details.update_cell_data(CONSUMER_GROUPS_LIST_NAME, 0, cg_details);
                },
                None => {
                    error!("Unable to get details for cg with name {}", selected_cg);
                    return;
                }
            };
        }
    }

    // Gets the selected item for the list
    fn get_selected_item_for_list(&mut self, list_name: &str) -> Option<String> {
        let mut layout_guard = self.layout.lock();
        let lists_layout = &mut layout_guard.main_layout.lists_layout;

        let selected_item = match lists_layout.get_list_by_name(list_name) {
            Some(list) => list.selected_item(),
            None => {
                error!("No list found by name: {}", list_name);
                return None;
            }
        };

        drop(layout_guard);

        let item = match selected_item {
            Some(item) => item,
            None => {
                return None;
            }
        };

        return Some(item);
    }

    fn toogle_edit_mode(&mut self, mode: EditMode) {
        match mode {
            EditMode::Normal => {
                self.layout.lock().footer_layout.handle_input_event(InputEvent::Reset);
                self.state.edit_mode = EditMode::Normal;
            },
            EditMode::Editing => {
                 self.state.edit_mode = EditMode::Editing;
                self.layout.lock().footer_layout.handle_input_event(InputEvent::NewChar(':'));
            }
        }
    }   

    fn handle_input_event(&mut self, input_event: InputEvent) {
       self.layout.lock().footer_layout.handle_input_event(input_event);
    }

    fn handle_input_submission(&mut self) {
        let mut layout_guard = self.layout.lock();
        let footer_layout = &mut layout_guard.footer_layout;
        let input_value = footer_layout.input_value();
        footer_layout.handle_input_event(InputEvent::Reset);
        info!("received input from command: {}",input_value);
    }
}

fn generate_broker_details(id: i32, status: &str, partitions: usize) -> String {
    format!("\nID         : {}\nStatus     : {}\nPartitions : {}", id, status, partitions)
}

fn generate_consumer_group_details(state: &str, members: usize) -> String {
    format!("\nState   : {}\nMembers : {}", state, members)
}

fn generate_partition_details(leader: i32, isr: usize, replicas: usize, hwm: i64, offset: i64) -> String {
    format!("\nLeader : {}\nISR    : {} / {}\nHWM    : {}\nOffset : {}", leader, isr, replicas, hwm, offset)
}

fn generate_topic_details(parition_count: usize) -> String {
    format!("\nParitions: {}", parition_count)
}

fn get_topic_and_parition_id(partition_name: &str) -> Option<(&str, i32)> {
    let topic_and_partition = partition_name.split("/").collect::<Vec<&str>>();
    if topic_and_partition.len() != 2 {
        log::error!("error splitting parition name into topic name and partition id for {}", partition_name);
        return None;
    }

    let paritition_id = match topic_and_partition[1].parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            log::error!("unable to parse partition id {} into integer", topic_and_partition[1]);
            return None;
        }
    };

    Some((topic_and_partition[0], paritition_id))
}
