use std::io::Stderr;
use std::sync::Arc;
use std::usize;

use crossterm::event::{KeyEventKind, KeyCode};
use ratatui::style::Stylize;
use ratatui::text::{Text, Span};
use ratatui::widgets::ScrollbarOrientation;
use ratatui::{Terminal, Frame};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Layout, Constraint, Rect};
use strum::{EnumIter, FromRepr, Display, VariantNames};
use tokio::sync::Mutex;
use crate::kafka::consumer::Consumer;
use crate::tui::events::TuiEvent;
use crate::tui::widgets::{AppWidget, UIList, UIBlock, UIParagraph, UIScrollbar, UIParagraphWithScrollbar};

use super::widgets::UITabs;

#[derive(PartialEq)]
enum SelectedBlock {
    NAMESPACE,
    KIND,
    RESOURCE,
    DETAILS,
    NONE,
}

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter, PartialEq, VariantNames)]
enum SelectedTab {
    #[default]
    #[strum(to_string = "BROKERS")]
    BROKERS,
    #[strum(to_string = "CONSUMER GROUPS")]
    CONSUMERGROUPS,
    #[strum(to_string = "TOPICS & PARTITIONS")]
    TOPICS,
}

enum Direction {
    UP,
    DOWN,
}

struct AppLayout<'a> {
    header_layout: HeaderLayout<'a>,
    tabs_layout: TabsLayout<'a>,
    footer_layout: FooterLayout<'a>,
}

impl <'a> AppLayout<'a> {
    pub fn new() -> AppLayout<'a> {
        AppLayout{
            header_layout: HeaderLayout::new(),
            tabs_layout: TabsLayout::new(),
            footer_layout: FooterLayout::new(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        use Constraint::*;

        //overall layout
        let outer_layout = Layout::vertical([Length(5), Fill(1), Length(3)]);
        let [title, tab, footer] = outer_layout.areas(frame.size());

        self.header_layout.render(frame, title);
        self.tabs_layout.render(frame, tab);
        self.footer_layout.render(frame, footer);
    }
}

struct HeaderLayout<'a> {
    title: UIParagraph<'a>
}

impl <'a> HeaderLayout<'a> {
    pub fn new() -> HeaderLayout<'a> {
        HeaderLayout{
            title: UIParagraph::new("", Text::from(vec![
                Span::from(APP_NAME).bold().green().into_centered_line(),
                Span::from(APP_VERSION).gray().into_centered_line(),
            ])),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.title.render(frame, area)
    }
}

struct TabsLayout<'a> {
    tabs: UITabs<'a>,
    broker_layout: BrokerTabLayout<'a>,
    cg_layout : ConsumerGroupTabLayout<'a>,
    topics_layout: TopicsAndPartitionsTabLayout<'a>
}

impl <'a> TabsLayout<'a> {
    pub fn new() -> TabsLayout<'a> {
        TabsLayout {
            tabs: UITabs::new("", SelectedTab::VARIANTS),
            broker_layout: BrokerTabLayout::new(),
            cg_layout: ConsumerGroupTabLayout::new(),
            topics_layout: TopicsAndPartitionsTabLayout::new()
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        use Constraint::*;
        let vertical_layout = Layout::vertical([Length(2), Fill(1)]);
        let [tabs, details] = vertical_layout.areas(area);

        self.tabs.render(frame, tabs);

        let selected_tab = SelectedTab::from_repr(self.tabs.selected()).unwrap();
        match selected_tab {
            SelectedTab::BROKERS => self.broker_layout.render(frame, details),
            SelectedTab::CONSUMERGROUPS => self.cg_layout.render(frame, details),
            SelectedTab::TOPICS => self.topics_layout.render(frame, details),
        }
    }
}

struct BrokerTabLayout<'a> {
    brokers_list: UIList<'a>,
    broker_details: UIParagraphWithScrollbar<'a>
}

impl <'a> BrokerTabLayout<'a> {
    pub fn new() -> BrokerTabLayout<'a> {
        BrokerTabLayout {
            brokers_list: UIList::new("Brokers", vec![]),
            broker_details: UIParagraphWithScrollbar::new("Details", "".into(), ScrollbarOrientation::VerticalRight),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let horizontal_layout = Layout::horizontal([Constraint::Percentage(10), Constraint::Fill(1)]);
        let [list, details] = horizontal_layout.areas(area);
        self.brokers_list.render(frame, list);
        self.broker_details.render(frame, details);
    }
}

struct ConsumerGroupTabLayout<'a> {
    cg_list: UIList<'a>,
    cg_details: UIParagraphWithScrollbar<'a>
}

impl <'a> ConsumerGroupTabLayout<'a> {
    pub fn new() -> ConsumerGroupTabLayout<'a> {
        ConsumerGroupTabLayout {
            cg_list: UIList::new("Consumer Groups", vec![]),
            cg_details: UIParagraphWithScrollbar::new("Details", "".into(), ScrollbarOrientation::VerticalRight),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let horizontal_layout = Layout::horizontal([Constraint::Percentage(10), Constraint::Fill(1)]);
        let [list, details] = horizontal_layout.areas(area);

        self.cg_list.render(frame, list);
        self.cg_details.render(frame, details);
    }
}

struct TopicsAndPartitionsTabLayout<'a> {
    topics_list: UIList<'a>,
    partitions_list: UIList<'a>,
    topic_parition_details: UIParagraphWithScrollbar<'a>
}

impl <'a> TopicsAndPartitionsTabLayout<'a> {
    pub fn new() -> TopicsAndPartitionsTabLayout<'a> {
        TopicsAndPartitionsTabLayout {
            topics_list: UIList::new("Topics", vec![]),
            partitions_list: UIList::new("Partitions", vec![]),
            topic_parition_details: UIParagraphWithScrollbar::new("Details", "".into(), ScrollbarOrientation::VerticalRight),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let horizontal_layout = Layout::horizontal([Constraint::Percentage(10), Constraint::Fill(1)]);
        let [list, details] = horizontal_layout.areas(area);
        let vertical_layout = Layout::vertical([Constraint::Percentage(50), Constraint::Fill(1)]);
        let [topics_list, partitions_list] = vertical_layout.areas(list);

        self.topics_list.render(frame, topics_list);
        self.partitions_list.render(frame, partitions_list);
        self.topic_parition_details.render(frame, details);
    }
}

struct FooterLayout<'a> {
    footer: UIParagraph<'a>
}

impl <'a> FooterLayout<'a> {
    pub fn new() -> FooterLayout<'a> {
        FooterLayout {
            footer: UIParagraph::new("", Text::from(vec![
                Span::from(APP_FOOTER).gray().into_centered_line(),
            ]))
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.footer.render(frame, area)
    }
}

// App state maintains the state at app level
struct AppState {
    // selected_tab holds the state of currently selected Tab
    selected_tab: SelectedTab,

    // selected_block holds the state of currently selected section of the tui
    // it may be a list , a block or any widget
   // selected_block: SelectedBlock,

    // should quit tells the main loop to terminate the app
    should_quit: bool,
}

impl AppState {
}

const APP_NAME: &str = "Kafka2i - TUI for Kafka";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_FOOTER: &str = "<TAB> Switch Tabs | <ESC> Quit | <UP/DOWN> Scroll List";
//const APP_TABS: &'static [&str] = &["Brokers", "Consumer Groups", "Topics"]; 

// App is the high level struct containing
// Widgets
// State of the app itself
// Event Handler to update the state of the app or the undelying widget
// And a Kafka client
pub struct App<'a> {
    //widgets: AppWidgets<'a>,
    layout: AppLayout<'a>,
    state: AppState,
    terminal: &'a mut Terminal<CrosstermBackend<Stderr>>,
    kafka_consumer: Arc<Mutex<Consumer>>,
}

// This impl block only defines the new state of the app
impl <'a> App<'a> {
    pub async fn new(t: &'a mut Terminal<CrosstermBackend<Stderr>>, kafka_consumer: Arc<Mutex<Consumer>>) -> App<'a> {
       App {
           layout: AppLayout::new(),
           state: AppState {
                selected_tab: SelectedTab::default(),
                should_quit: false,
           },
           terminal: t,
           kafka_consumer,
        }
    }
}

// This impl block defines all the methods related to state of the app
impl App<'_> {
    // should_quit is defined at app level so its easier to call from main method
    pub fn should_quit(&self) -> bool {
        self.state.should_quit
    }

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

    async fn handle_tab(&mut self) {
        self.layout.tabs_layout.tabs.handle_tab();
    }

    async fn handle_list_navigation(&mut self, direction: Direction){
        match direction {
            Direction::UP => {
                 match self.state.selected_tab {
                   /* SelectedBlock::KIND => self.widgets.kind_list.handle_up(),
                    SelectedBlock::NAMESPACE => self.widgets.namespace_list.handle_up(),
                    SelectedBlock::RESOURCE => self.widgets.resources_list.handle_up(),
                    SelectedBlock::DETAILS => self.widgets.description_paragraph.handle_up(),*/
                    _ => (),
                }
            },
            Direction::DOWN => {
                match self.state.selected_tab {
                 /*   SelectedBlock::KIND => self.widgets.kind_list.handle_down(),
                    SelectedBlock::NAMESPACE => self.widgets.namespace_list.handle_down(),
                    SelectedBlock::RESOURCE => self.widgets.resources_list.handle_down(),
                    SelectedBlock::DETAILS => self.widgets.description_paragraph.handle_down(),*/
                    _ => (),
                }
            }
        }
        
        // if the event happens on namespace or kind, get resources
       /* if  (self.state.selected_block == SelectedBlock::NAMESPACE || self.state.selected_block == SelectedBlock::KIND) &&
            self.widgets.kind_list.selected_item().is_some() {
             // get resources
            match self.k8s.get_resource_list(&self.widgets.namespace_list.selected_item().unwrap(), &self.widgets.kind_list.selected_item().unwrap()).await {
                Ok(rs) => {
                    self.widgets.resources_list = UIList::new("Resources", rs)
                },
                Err(_) => () //handle later
            };
        }

        // if the event happens on a resource, fetch the resource yaml
        if self.state.selected_block == SelectedBlock::RESOURCE && self.widgets.resources_list.selected_item().is_some() {
            match self.k8s.get_resource_yaml(&self.widgets.namespace_list.selected_item().unwrap(), &self.widgets.kind_list.selected_item().unwrap(), &self.widgets.resources_list.selected_item().unwrap()).await {
                Ok(ry) => {
                    self.widgets.description_paragraph = UIParagraphWithScrollbar::new("Details", ry.into(), ScrollbarOrientation::VerticalRight);
                }, 
                Err(_) => () // TODO: Handle error
            };
        }*/
    }
}

// this impl block handles app rendering logic
impl App<'_> {
    pub fn render(&mut self) {
        let _ = self.terminal.draw(|f| {
            self.layout.render(f); 
        });
    }
}
