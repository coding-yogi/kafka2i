use std::io::Stderr;
use std::usize;

use crossterm::event::{KeyEventKind, KeyCode};
use ratatui::style::Stylize;
use ratatui::text::{Text, Span};
use ratatui::widgets::ScrollbarOrientation;
use ratatui::{Terminal, Frame};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Layout, Constraint};
use strum::{EnumIter, FromRepr, Display, VariantNames};
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

/*impl SelectedTab {
    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(Self::from_repr(0).unwrap())
    }
}*/

enum Direction {
    UP,
    DOWN,
}

// App widgets is the collection of all the widgets 
struct AppWidgets<'a> {
    title: UIParagraph<'a>,
    tabs: UITabs<'a>,
    details: UIBlock<'a>,
    footer: UIParagraph<'a>,
   /* namespace_list: UIList<'a>,
    kind_list: UIList<'a>,
    resources_list: UIList<'a>,
    description_paragraph: UIParagraphWithScrollbar<'a>, */
}

impl <'a> AppWidgets<'a> {

    // refresh the whole layout
    pub fn refresh_layout(&mut self, frame: &mut Frame) {
        use Constraint::*;

        //overall layout
        let outer_layout = Layout::vertical([Length(5), Length(2), Fill(1), Length(3)]);
        let [title, tab, details, footer] = outer_layout.areas(frame.size());

        let tab_horizontal = Layout::horizontal([Percentage(10), Percentage(90)]);
        let [component, component_details] = tab_horizontal.areas(details);

        //render
        self.title.render(frame, title);
        self.tabs.render(frame, tab);
        self.details.render(frame, details);
        self.footer.render(frame, footer);
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
    widgets: AppWidgets<'a>,
    state: AppState,
    terminal: &'a mut Terminal<CrosstermBackend<Stderr>>,
    kafka_consumer: &'a Consumer,
}

// This impl block only defines the new state of the app
impl <'a> App<'a> {
    pub async fn new(t: &'a mut Terminal<CrosstermBackend<Stderr>>, kafka_consumer: &'a Consumer) -> App<'a> {
       App {
           widgets: AppWidgets {
                title: UIParagraph::new("", Text::from(vec![
                    Span::from(APP_NAME).bold().green().into_centered_line(),
                    Span::from(APP_VERSION).gray().into_centered_line(),
                ])),

                tabs: UITabs::new("", SelectedTab::VARIANTS),
                details: UIBlock::new(""),
                footer: UIParagraph::new("", Text::from(vec![
                    Span::from(APP_FOOTER).gray().into_centered_line(),
                ]))
                /*namespace_list: UIList::new("Namespace", namespaces),
                kind_list: UIList::new("Kind", k8s.get_kinds().await),
                resources_list: UIList::new("Resource", vec![]),
                description_paragraph: UIParagraphWithScrollbar::new("Description", "".into(), ScrollbarOrientation::VerticalRight),*/
           },
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
        self.widgets.tabs.handle_tab();

       /* match self.state.selected_block {
            SelectedBlock::NONE | SelectedBlock::DETAILS => {
                if self.widgets.namespace_list.state().is_none() {
                    self.widgets.namespace_list.select(Some(0));
                }
                self.state.selected_block = SelectedBlock::NAMESPACE;
                self.widgets.namespace_list.highlight_border();
                self.widgets.description_paragraph.normalise_border();
            },
            SelectedBlock::NAMESPACE => {
                if self.widgets.kind_list.state().is_none() {
                    self.widgets.kind_list.select(Some(0));
                }
                self.state.selected_block = SelectedBlock::KIND;
                self.widgets.kind_list.highlight_border();
                self.widgets.namespace_list.normalise_border();
            },
            SelectedBlock::KIND => {
                if self.widgets.resources_list.state().is_none() {
                    self.widgets.resources_list.select(Some(0));
                }
                self.state.selected_block = SelectedBlock::RESOURCE;
                self.widgets.resources_list.highlight_border();
                self.widgets.kind_list.normalise_border();
            },

            SelectedBlock::RESOURCE => {
                self.state.selected_block = SelectedBlock::DETAILS;
                self.widgets.description_paragraph.highlight_border();
                self.widgets.resources_list.normalise_border();
            }
        };
        */
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
            self.widgets.refresh_layout(f); 
        });
    }
}
