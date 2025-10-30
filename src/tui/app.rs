use std::collections::HashMap;
use std::str::FromStr;
use std::{char, sync::Arc, time::Duration};
use crossbeam::channel::Receiver;
use chrono::{DateTime};
use log::{debug, error, info};
use parking_lot::Mutex;
use rdkafka::{consumer::ConsumerContext, ClientContext};
use strum::{self, Display, EnumString};
use crate::kafka::consumer::{Consumer, ConsumerError, KafkaMessage};
use crate::tui::app;
use crate::tui::single_layout::AppMode;
use crate::tui::widgets::{AppWidget, Direction};

use super::{single_layout::{self, AppLayout, BROKERS_LIST, CONSUMER_GROUPS_LIST, PARTITIONS_LIST, TOPICS_LIST}, widgets::InputEvent};

#[derive(PartialEq)]
enum EditMode {
    Normal,
    Editing
}

pub enum AppEvent {
    Tab,
    BackTab,
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

// AppCMDs
#[derive(Clone, Debug, Display, PartialEq, EnumString)]
enum Command {
    #[strum(serialize = ":offset")]
    Offset,
    #[strum(serialize = ":ts")]
    Timestamp,
    Invalid,
}

// AppErr
const ERR_INVALID_CMD: &str = "err:InvalidCMD";
const ERR_INVALID_OFFSET: &str = "err:InvalidOffset";
const ERR_INVALID_TIMESTAMP: &str = "err:InvalidTimestamp";
const ERR_NO_SELECTED_PARTITION: &str = "err:NoSelectedPartition";
const ERR_FETCHING_OFFSET: &str = "err:FetchingOffset";
const ERR_OFFSET_NOT_FOUND: &str = "err:OffsetNotFound";

const UNINITIALISED_OFFSET: i64 = -999;

// App state maintains the state at app level
struct AppState {
    // app mode
    app_mode: AppMode,
    //edit mode
    edit_mode: EditMode,
    //offset for selected partition
    offset: i64,
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
    layout: Arc<Mutex<AppLayout<'a>>>,
    state: AppState,
    kafka_consumer: Arc<Mutex<Consumer<T>>>,
    app_event_recv: Receiver<AppEvent>,
    clipboard: Option<arboard::Clipboard>,
}

// This impl block only defines the new state of the app
impl <'a, T> App<'a, T> 
where T: ClientContext + ConsumerContext
{
    pub async fn new(kafka_consumer: Arc<Mutex<Consumer<T>>>, app_event_recv: Receiver<AppEvent>) -> App<'a, T> {
        let metadata = kafka_consumer.lock().metadata().clone();

        let app = App {
            layout: Arc::new(Mutex::new(AppLayout::new(&metadata))),
            state: AppState {
                should_quit: false,
                app_mode: AppMode::default(),
                edit_mode: EditMode::Normal,
                offset: UNINITIALISED_OFFSET,
            },
            //terminal: t,
            kafka_consumer,
            app_event_recv,
            clipboard: match arboard::Clipboard::new() {
                Ok(c) => Some(c),
                Err(err) => {
                    error!("error initiating clipboard {}", err);
                    None
                } 
            },
        };

        app
    }

    pub fn layout(&self) -> Arc<Mutex<AppLayout<'a>>> {
        self.layout.clone()
    }
}

// This impl block for the app event handler
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
                                AppEvent::Tab => self.handle_tab(false),
                                AppEvent::BackTab => self.handle_tab(true),
                                AppEvent::Up => self.handle_list_navigation(Direction::UP),
                                AppEvent::Down => self.handle_list_navigation(Direction::DOWN),
                                AppEvent::Left => self.handle_offset_navigation(Direction::LEFT),
                                AppEvent::Right => self.handle_offset_navigation(Direction::RIGHT),
                                AppEvent::Input(char) => match char {
                                    ':' => self.toggle_edit_mode(EditMode::Editing),
                                    'm' => self.handle_message_scroll(Direction::DOWN),
                                    'n' => self.handle_message_scroll(Direction::UP),
                                    'h' => self.handle_help_command(),
                                    'c' => self.set_app_mode(single_layout::AppMode::Consumer),
                                    'p' => self.set_app_mode(single_layout::AppMode::Producer),
                                    'q' | 'Q' => {
                                        self.state.should_quit = true;
                                        break;
                                    },
                                    _ => (),
                                },
                                _ => (),
                            }
                        },
                        EditMode::Editing => {
                            match event {
                                AppEvent::Esc => self.toggle_edit_mode(EditMode::Normal),
                                AppEvent::Input(char) => self.handle_input_event(InputEvent::NewChar(char)),
                                AppEvent::Backspace => self.handle_input_event(InputEvent::RemovePrevChar),
                                AppEvent::Left => self.handle_input_event(InputEvent::MoveCursor(Direction::LEFT)),
                                AppEvent::Right => self.handle_input_event(InputEvent::MoveCursor(Direction::RIGHT)),
                                AppEvent::Up => self.handle_input_event(InputEvent::MoveCursor(Direction::UP)),
                                AppEvent::Down => self.handle_input_event(InputEvent::MoveCursor(Direction::DOWN)),
                                AppEvent::Enter => {
                                    match self.state.app_mode {
                                        AppMode::Consumer => {
                                            // For consumer, we handle the command entered post hitting enter
                                            self.handle_input_submission();
                                            self.toggle_edit_mode(EditMode::Normal);
                                        },
                                        AppMode::Producer => {
                                            // For Producer, we accept it as an input event
                                            self.handle_input_event(InputEvent::NewChar('\n'));
                                        }
                                    }
                                },
                                AppEvent::Tab => self.handle_input_event(InputEvent::NewChar('\t')),
                                _ => (),
                            }
                        },
                    }
                },
                Err(_) => log::error!("error occured while receiving app event")
            }
        }
    }

    // set mode of the app
    fn set_app_mode(&mut self, mode: single_layout::AppMode) {
        self.state.app_mode = mode.clone();
        self.layout.lock().set_app_mode(mode);
    }
}

// Implementation block to handle all list navigations
impl <T> App<'_, T>
where T: ClientContext + ConsumerContext {

    // Handles tab event which switches between the available tabs
    fn handle_tab(&mut self, back_tab: bool) {
        // enable edit mode if any of the input blocks is selected
        if self.layout.lock().main_layout.handle_tab(back_tab) && self.state.app_mode == AppMode::Producer {
            self.toggle_edit_mode(EditMode::Editing);
        }
    }

    // Handles the list navigation for the list in focus 
    fn handle_list_navigation(&mut self, direction: Direction){
        // selected list
        self.layout.lock().main_layout.lists_layout.handle_navigation(direction);
        let selected_list_name = self.layout.lock().main_layout.lists_layout.selected_list().name().to_string().clone();

        // handle navigation events
        match selected_list_name.as_str() {
            BROKERS_LIST => self.handle_broker_list_navigation(),
            TOPICS_LIST => self.handle_topic_list_navigation(),
            CONSUMER_GROUPS_LIST => self.handle_cg_list_navigation(),
            PARTITIONS_LIST => self.handle_partition_list_navigation(),
            _ => log::error!("Selected list has an invalid name")
        }
    }

    // Handles broker list navigation
    // populates TUI with details of the broker selected in the list
    fn handle_broker_list_navigation(&mut self) {
        if let Some(selected_broker) = self.get_selected_item_for_list(BROKERS_LIST) {
            let broker = match self.kafka_consumer.lock().metadata().get_broker(&selected_broker) {
                Some(broker) => broker,
                None => return
            };

            // update broker details
            let broker_id = broker.id();
            let partition_leader_count = self.kafka_consumer.lock().metadata().no_of_partitions_for_broker(broker_id);
            let broker_details = generate_broker_details(broker_id, "UP", partition_leader_count);
            self.layout.lock().main_layout.details_layout.metadata.update_cell_data(BROKERS_LIST, 0, broker_details);
        }
    }

    // Handles topic list navigation
    // populates the TUI with details of the topic selected
    // populates the parition list with paritions of the selected topic
    fn handle_topic_list_navigation(&mut self) {
        if let Some(selected_topic) = self.get_selected_item_for_list(TOPICS_LIST) {
            if let Some(topic) = self.kafka_consumer.lock().metadata().get_topic(&selected_topic) {
                let topic_details = generate_topic_details(topic.partitions().len());
                self.layout.lock().main_layout.details_layout.metadata.update_cell_data(TOPICS_LIST, 0, topic_details);

                // Fetching all partition names
                let partitions_names = topic.partition_names();
                match self.layout.lock().main_layout.lists_layout.get_list_by_name(PARTITIONS_LIST) {
                    Some(list) => list.update(partitions_names),
                    None => {
                        error!("No list found by name: {}", PARTITIONS_LIST);
                        return;
                    }
                };
            }
        }
    }

    // Handles partition list navidation
    // populates the TUI with details of the partition selected
    fn handle_partition_list_navigation(&mut self) {
        if let Some(selected_partition) = self.get_selected_item_for_list(PARTITIONS_LIST) {
            // reset the stored offset after selecting a new partition
            self.state.offset = UNINITIALISED_OFFSET;

            // fetch message only in consumer mode
            if self.state.app_mode == AppMode::Consumer {
                self.fetch_message(&selected_partition, -1)
            }
        }
    }

    // Handles consumer group list navigation
    // populates the TUI with the details of selected consumer groups
    fn handle_cg_list_navigation(&mut self) {
        if let Some(selected_cg) = self.get_selected_item_for_list(CONSUMER_GROUPS_LIST) {
            if let Some(cg) = self.kafka_consumer.lock().metadata().get_consumer_group(&selected_cg) {
                let cg_details = generate_consumer_group_details(cg.state(), cg.members_count());
                self.layout.lock().main_layout.details_layout.metadata.update_cell_data(CONSUMER_GROUPS_LIST, 0, cg_details);
            }
        }
    }

    // Gets the selected item for the list
    fn get_selected_item_for_list(&mut self, list_name: &str) -> Option<String> {
        if let Some(list) = self.layout.lock().main_layout.lists_layout.get_list_by_name(list_name) {
            return list.selected_item()
        }

        return None;
    }
}

// Implementation block for all message block related events
impl <T> App<'_, T>
where T: ClientContext + ConsumerContext {
    fn handle_message_scroll(&mut self, direction: Direction) {
        match direction {
            Direction::DOWN => self.layout.lock().main_layout.details_layout.consumed_message.scroll_down(),
            Direction::UP => self.layout.lock().main_layout.details_layout.consumed_message.scroll_up(),
            _ => ()
        }
    }
}

// Implementation block for consuming messages
impl <T> App<'_, T>
where T: ClientContext + ConsumerContext {
    // assign and poll the consumer for the given topic, partition and offset
    fn assign_and_poll(&mut self, topic_name: &str, partition_id: i32) -> Result<(), ConsumerError> {
        debug!("assigning partition {} for the topic {}", partition_id, topic_name);
        match self.kafka_consumer.lock().assign(topic_name, partition_id) {
            Ok(()) => (),
            Err(err) => {
                error!("error while assigning partitions for the topic {}: {}", topic_name, err);
                return Err(err);
            }
        }

        // Poll after assigning paritions
        // we do not want to capture the message just yet
        match self.kafka_consumer.lock().consume(Duration::from_secs(5), false) {
            Ok(_) => (),
            Err(err) => {
                error!("error while polling post assignment {}", err);
                return Err(err);
            }
        }

        Ok(())
    }

    // Seek and consume message from the given offset
    fn seek_and_consume(&mut self, topic_name: &str, partition_id: i32, offset: i64) -> Option<KafkaMessage>{
        match self.kafka_consumer.lock().seek(topic_name, partition_id, offset) {
            Ok(()) => (),
            Err(err) => {
                error!("consumer error while seeking offset {} on partition {}/{} {}", offset, topic_name, partition_id, err);
                return None;
            }
        }

        // consumer the message from the seeked offset
        match self.kafka_consumer.lock().consume(Duration::from_secs(5), true) {
            Ok(msg) =>  msg,
            Err(err) => {
                error!("error consuming message on topic {}/{} at offset {}: {}", topic_name, partition_id, offset, err);
                None
            }
        }
    }

    // Write message to TUI
    fn write_message(&mut self, message: KafkaMessage) {
                let message_timestamp = message.timestamp_or_default();
                let message_offset = message.offset;
                let message_payload = format!("Key: {}\n\nHeaders: {}\n\nPayload: {}",
                    message.key_or_default(), pretty_print_headers(&message.headers), pretty_print_json(&message.payload_or_default()));

        // copy to clipboard
        if let Err(err) = self.copy_to_clipboard(&message_payload) {
            error!("error while copying message to clipboard: {}", err);
        }

        // write to TUI
        info!("message fetched at offset {} of partition {}/{}: {}", message_offset, message.topic, message.partition, message_payload);
        self.layout.lock().main_layout.details_layout.consumed_message.update_title_and_text(format!("Message offset:{} ts:{}", message_offset, message_timestamp), message_payload.into());
    }

    // fetch message based on the parition name and offset
    fn fetch_message(&mut self, partition_str:&str, offset: i64) {
        // Clear the message block
        self.layout.lock().main_layout.details_layout.consumed_message.update_text("".into());

        let mut offset = offset;

        let partition = match self.kafka_consumer.lock().metadata().get_partition(partition_str) {
            Some(partition) => partition,
            None => {
                error!("Unable to get details for partition with name: {}", partition_str);
                return;
            }
        };

        if let Some((topic_name, partition_id)) = get_topic_and_parition_id(partition_str) {
            // get high water mark for the topic
            let high_watermark: i64;
            let low_watermark: i64;

            // Update status in message block
            self.layout.lock().main_layout.details_layout.consumed_message.update_text("fetching watermarks ...".into());

            // fetch watermarks for the give topic and partition id
            match self.kafka_consumer.lock().fetch_watermarks(topic_name, partition_id) {
                Ok((l, h)) => {
                    low_watermark = l;
                    high_watermark = h;
                },
                Err(err) => {
                    error!("error while fetching watermark on topic {}/{}: {}", topic_name, partition_id, err);
                    return;
                }
            };

            // Update UI
            let partition_details = generate_partition_details(partition.leader(), partition.isr().len(), partition.replicas().len(), low_watermark, high_watermark);
            self.layout.lock().main_layout.details_layout.metadata.update_cell_data(PARTITIONS_LIST, 0, partition_details);

            // check if there are messages available to consume on the selected topic & partition
            if high_watermark == low_watermark {
                self.log_error_and_update(format!("No messages in partition {}/{}", topic_name, partition_id));
                return;
            }

            // set correct offset
            if offset == -1 {
                // set offset to the end based on HWM
                offset = high_watermark - 1;
            } else if  offset < low_watermark || offset >= high_watermark {
                self.layout.lock().footer_layout.set_value(ERR_INVALID_OFFSET);
                self.log_error_and_update(format!("invalid offset {}, should be between {} and {}", offset, low_watermark, high_watermark));
                return;
            }

            // Assign current partition to consumer
            self.layout.lock().main_layout.details_layout.consumed_message.update_text("assigning partition ...".into());
            if let Err(err) = self.assign_and_poll(topic_name, partition_id) {
                self.log_error_and_update(format!("error assigning and polling for partition {}/{}: {}", topic_name, partition_id, err));
                return;
            }

            // seek high watermark -1 by default and consume the message
            self.layout.lock().main_layout.details_layout.consumed_message.update_text("seeking offset & fetching message ...".into());
            if let Some(msg) = self.seek_and_consume(topic_name, partition_id, offset) {
                self.write_message(msg);

                // Update offset in the state after fetching the msg successfully
                self.state.offset = offset;
            } else {
                self.log_error_and_update(format!("no message was returned"));
                return;
            }
        }
    }

    // Copy message to clipboard
    fn copy_to_clipboard(&mut self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        match &mut self.clipboard {
            Some(cb) => cb.set_text(message.to_string())?,
            None => (),
        }
        
        Ok(())
    }

    // log error and update TUI
    fn log_error_and_update(&mut self, message: String) {
        error!("{}", message);
        self.layout.lock().main_layout.details_layout.consumed_message.update_text(message.into());
    }
}

// Implementation block to handle all input events
impl <T> App<'_, T>
where T: ClientContext + ConsumerContext {
    // Toggle the edit mode to accept input
    fn toggle_edit_mode(&mut self, mode: EditMode) {
        match mode {
            EditMode::Normal => {
                self.state.edit_mode = EditMode::Normal;

                match self.state.app_mode {
                    AppMode::Consumer => self.layout.lock().footer_layout.input.normalise_border(),
                    AppMode::Producer => self.layout.lock().main_layout.normalise_border(),
                }
            },
            EditMode::Editing => {
                self.state.edit_mode = EditMode::Editing;

                // send relevant input events to footer only in consumer mode
                if self.state.app_mode == AppMode::Consumer {
                    self.layout.lock().footer_layout.handle_input_event(InputEvent::Reset);
                    self.layout.lock().footer_layout.input.highlight_border();
                    self.layout.lock().footer_layout.handle_input_event(InputEvent::NewChar(':'));
                }
            }
        }
    }   

    // Handle input event
    fn handle_input_event(&mut self, input_event: InputEvent) {
        match self.state.app_mode {
            AppMode::Consumer => self.layout.lock().footer_layout.handle_input_event(input_event),
            AppMode::Producer => self.layout.lock().main_layout.details_layout.handle_input_event(input_event),
        }
    }

    // handle input submission
    fn handle_input_submission(&mut self) {
        let input_value = self.layout.lock().footer_layout.input_value();
        self.layout.lock().footer_layout.handle_input_event(InputEvent::Reset);

        //handle cmd
        self.handle_command(&input_value)
    }

    // validate cmd
    fn handle_command(&mut self, input: &str)  {
        // For now handle commands only for consumer mode
        if self.state.app_mode == AppMode::Producer {
            return;
        }

        let inputs = input.split("!").collect::<Vec<&str>>();
        if inputs.len() < 2 {
            self.layout.lock().footer_layout.set_value(ERR_INVALID_CMD);
            error!("invalid command {}: command should be of format :<command>!<arg>", input);
            return;
        }

        let (command, arg) = match Command::from_str(inputs[0]) {
           Ok(cmd) => (cmd, inputs[1]),
           Err(err) => {
               self.layout.lock().footer_layout.set_value(ERR_INVALID_CMD);
               error!("error parsing command {}: {}", input, err);
               return;
           }
       };

       match command {
           Command::Invalid => return,
           Command::Offset => self.handle_offset_command(arg),
           Command::Timestamp => self.handle_timestamp_command(arg),
       }
    }
}

// Handle all commands
impl <T> App<'_, T>
where T: ClientContext + ConsumerContext {
    // Handle offset command
    pub fn handle_offset_command(&mut self, offset_str: &str)  {
        //check if offset is a number
        let offset = match offset_str.parse::<i64>() {
            Ok(o) => o,
            Err(_) => {
                self.layout.lock().footer_layout.set_value(ERR_INVALID_OFFSET);
                error!("invalid offset {}", offset_str);
                return;
            }
        };

        let selected_partition = match self.get_selected_item_for_list(PARTITIONS_LIST) {
            Some(p) => p,
            None => {
                self.layout.lock().footer_layout.set_value(ERR_NO_SELECTED_PARTITION);
                error!("no partition selected to seek");
                return;
            }
        };

        self.fetch_message(&selected_partition, offset);
    }

    // handle timestamp command
    pub fn handle_timestamp_command(&mut self, timestamp_str: &str)  {
        //check if timestamp is a number
        let _timestamp = match timestamp_str.parse::<i64>() {
            Ok(t) => t,
            Err(_) => {
                self.layout.lock().footer_layout.set_value(ERR_INVALID_TIMESTAMP);
                error!("invalid timestamp {}. timestamp should be a number representing an epoch in milliseconds", timestamp_str);
                return;
            }
        };

        // check if it is a valid epoch timestamp
        if DateTime::from_timestamp_millis(_timestamp) == None {
            self.layout.lock().footer_layout.set_value(ERR_INVALID_TIMESTAMP);
            error!("invalid timestamp {}. timestamp should be an epoch in milliseconds", timestamp_str);
            return;
        }

        // fetch offset for a given timestamp
        let selected_partition = match self.get_selected_item_for_list(PARTITIONS_LIST) {
            Some(p) => p,
            None => {
            self.layout.lock().footer_layout.set_value(ERR_NO_SELECTED_PARTITION);
            error!("no partition selected to seek");
            return;
            }
        };

        // get the offset based on the timestamp for a given topic and partition
        if let Some((topic_name, partition_id)) = get_topic_and_parition_id(&selected_partition) {
            let offset = match self.kafka_consumer.lock().offsets_for_timestamp(topic_name, partition_id, _timestamp) {
                Ok(offset) => match offset {
                    Some(o) => o,
                    None => {
                        self.layout.lock().footer_layout.set_value(ERR_OFFSET_NOT_FOUND);
                        error!("no offset found for topic {} & partition {} for timestamp {}", topic_name, partition_id, _timestamp);
                        return;
                    }
                },
                Err(err) => {
                    self.layout.lock().footer_layout.set_value(ERR_FETCHING_OFFSET);
                    error!("error fetching offset for timestamp {}: {}", _timestamp, err);
                    return;
                }
            };

            self.fetch_message(&selected_partition, offset);
        }
    }

    // Handle offset navigation
    pub fn handle_offset_navigation(&mut self, direction: Direction){
        // get current offset on the topic
        let selected_partition = match self.get_selected_item_for_list(PARTITIONS_LIST) {
            Some(p) => p,
            None => {
                self.layout.lock().footer_layout.set_value(ERR_NO_SELECTED_PARTITION);
                error!("no partition selected to seek");
                return;
            }
        };

        // fetch current offset from state
        let mut offset = self.state.offset;

        // Increment / decrement offset based on the direction
        match direction {
            Direction::LEFT => offset-=1,
            Direction::RIGHT => offset+=1,
            _ => ()
        }

        self.fetch_message(&selected_partition, offset);
    }

    pub fn handle_help_command(&mut self) {
        let current_state = self.layout.lock().show_help;
        self.layout.lock().show_help = !current_state;
    }
}

// Generate broker deatils
fn generate_broker_details(id: i32, status: &str, partitions: usize) -> String {
    format!("\nID         : {}\nStatus     : {}\nPartitions : {}", id, status, partitions)
}

// Generate consumer group details
fn generate_consumer_group_details(state: &str, members: usize) -> String {
    format!("\nState   : {}\nMembers : {}", state, members)
}

// Generate parition details
fn generate_partition_details(leader: i32, isr: usize, replicas: usize, lwm: i64, hwm: i64) -> String {
    format!("\nLeader : {}\nISR    : {} / {}\nLWM    : {}\nHWM    : {}", leader, isr, replicas, lwm, hwm)
}

// Generate topic details
fn generate_topic_details(parition_count: usize) -> String {
    format!("\nParitions: {}", parition_count)
}

// Get Topic Name and the partition ids from partition name
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

// Pretty print json
fn pretty_print_json(json_str: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(json) => {
            match serde_json::to_string_pretty(&json) {
                Ok(pretty_json) => pretty_json,
                Err(_) => json_str.to_string()
            }
        },
        Err(_) => json_str.to_string()
    }
}

// Pretty print headers
fn pretty_print_headers(headers: &HashMap<String, String>) -> String {
    match serde_json::to_string_pretty(&headers) {
        Ok(pretty_json) => pretty_json,
        Err(_) => format!("{:?}", headers)
    }
}