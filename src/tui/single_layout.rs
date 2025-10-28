use ratatui::{layout::{Constraint, Layout, Rect}, style::Stylize, text::{Line, Span, Text}, widgets::{Clear, ScrollbarOrientation}, Frame};
use strum::Display;
use crate::kafka::metadata::Metadata;

use super::widgets::{AppWidget, Direction, InputEvent, UIInput, UIList, UIParagraph, UIParagraphWithScrollbar, UITable};

const APP_NAME: &str = "Kafka2i - TUI for Kafka";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_FOOTER: &str = "Press <H> for help | Press <ESC> to quit";

pub const BROKERS_LIST: &str = "Brokers";
pub const CONSUMER_GROUPS_LIST: &str = "Consumer Groups";
pub const TOPICS_LIST: &str = "Topics";
pub const PARTITIONS_LIST: &str = "Partitions";

// Mode of App
#[derive(Clone, Debug, Display, Default, PartialEq)]
pub enum AppMode {
    #[default]
    #[strum(to_string="Consumer")]
    Consumer,
    #[strum(to_string="Producer")]
    Producer
}

// Top level application layout
pub struct AppLayout<'a> {
    pub header_layout: HeaderLayout<'a>,
    pub main_layout: MainLayout<'a>,
    pub footer_layout: FooterLayout<'a>,
    pub help_layout: HelpLayout<'a>,
    pub show_help: bool,
}

impl <'a> AppLayout<'a> {
    pub fn new(metadata: &Metadata) -> AppLayout<'a> {
        let mut app_layout = AppLayout{
            header_layout: HeaderLayout::new(),
            main_layout: MainLayout::new(metadata),
            footer_layout: FooterLayout::new(),
            help_layout: HelpLayout::new(),
            show_help: false,
        };

        app_layout.footer_layout.set_mode(AppMode::Consumer.to_string());
        app_layout
    }

    pub fn render(&mut self, frame: &mut Frame) {
        use Constraint::*;

        //overall layout
        let outer_layout = Layout::vertical([Length(5), Fill(1), Length(3)]);
        let [title, main, footer] = outer_layout.areas(frame.area());

        self.header_layout.render(frame, title);
        self.main_layout.render(frame, main);
        self.footer_layout.render(frame, footer);

        // centered help layout
        if self.show_help {
            self.help_layout.render(frame, self.centered_help_area(frame));
        }
    }

    pub fn set_app_mode(&mut self, mode: AppMode) {
        self.main_layout.details_layout.mode = mode.clone();
        self.footer_layout.set_mode(mode.to_string());
    }

    // function to get a rect of 60 x 40 in the center of the terminal
    fn centered_help_area(&self, frame: &Frame) -> Rect {
        let area = frame.area();
        let width = 45;
        let height = 45;

        let popup_layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - height) / 2),
                Constraint::Percentage(height),
                Constraint::Percentage((100 - height) / 2),
            ]
            .as_ref(),
        )
        .split(area);

        let vertical = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - width) / 2),
                    Constraint::Percentage(width),
                    Constraint::Percentage((100 - width) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1]);

        vertical[1]
    }

}

// Header Layout
pub struct HeaderLayout<'a> {
    title: UIParagraph<'a>
}

impl <'a> HeaderLayout<'a> {
    pub fn new() -> HeaderLayout<'a> {
        let crab = emojis::get_by_shortcode("crab").unwrap();
        let heart = emojis::get_by_shortcode("heart").unwrap();

        HeaderLayout{
            title: UIParagraph::new("".to_string(), Text::from(vec![
                Span::from(format!("{} (v{})", APP_NAME, APP_VERSION)).bold().green().into_centered_line(),
                Span::from(format!("Built in {} with {}", crab.as_str(), heart.as_str())).bold().gray().into_centered_line()
            ]))
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.title.render(frame, area)
    }
}

// Main layout
pub struct MainLayout<'a> {
    pub lists_layout: ListsLayout<'a>,
    pub details_layout: DetailsLayout<'a>,
    // To keep track of the widget selected
    selected_widget: usize
}

impl <'a> MainLayout<'a> {
    pub fn new(metadata: &Metadata) -> MainLayout<'a> {
        MainLayout {
            lists_layout: ListsLayout::new(metadata),
            details_layout: DetailsLayout::new(),
            selected_widget: 0,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        use Constraint::*;
        let horizontal_layout = Layout::horizontal([Percentage(25), Fill(1)]);
        let [list_layout, details_layout] = horizontal_layout.areas(area);

        self.lists_layout.render(frame, list_layout);
        self.details_layout.render(frame, details_layout);
    }

    // handle_tab will highlight the required widget and also return a boolean stating if edit mode should be enabled
    pub fn handle_tab(&mut self, back_tab: bool) -> bool {
        // collect all list widgets
        let mut selectable_widgets: Vec<&mut (dyn AppWidget + Send)> = self.lists_layout.lists.iter_mut()
                    .map(|l| l as &mut (dyn AppWidget + Send)).collect();

        // length of lists
        let lists_cnt = selectable_widgets.len();

        // collect all input widgets
        let mut input_widgets: Vec<&mut (dyn AppWidget + Send)> = vec![
            &mut self.details_layout.key,
            &mut self.details_layout.headers,
            &mut self.details_layout.payload
        ];

        // calculate length before appending to selectable_widgets, as len() post append step will be 0
        let inputs_cnt = input_widgets.len();

        selectable_widgets.append(&mut input_widgets);

        // normalise border of already selected widget
        selectable_widgets[self.selected_widget].normalise_border();

        // If producer mode, consider all widgets
        let mut len = selectable_widgets.len();

        // If in consumer mode, reduce the length by length of input widgets
        if self.details_layout.mode == AppMode::Consumer {
            len = len - inputs_cnt;
        }

        let mut new_idx = self.selected_widget;
        if back_tab {
            if new_idx == 0 {
                new_idx = len - 1;
            } else {
                new_idx = self.selected_widget.saturating_sub(1);
                // After subtracting if the new index is greater than the len-1 of widgets, then it should be set to max
                // This scenario can happen if an input widget on producer screen is selected currently and we switch to consumer before backtab
                if new_idx >= len {
                    new_idx =  len - 1;
                }
            }
        } else {
            new_idx = self.selected_widget.saturating_add(1);
            // After increasing the index, if it matches or exceeds the length, then set to 0
            // exceeding length would occur if UI is in producer mode with selected input widget
            // and then switched back to consumer before tab
            if new_idx >= len {
                new_idx = 0
            }
        }

        // higlight selected list border
        self.selected_widget = new_idx;
        selectable_widgets[self.selected_widget].highlight_border();

        // if any of the input widget is selected we need to enable edit mode
        new_idx >= lists_cnt
    }

    pub fn normalise_border(&mut self) {
         // collect all list widgets
        let mut selectable_widgets: Vec<&mut (dyn AppWidget + Send)> = self.lists_layout.lists.iter_mut()
                    .map(|l| l as &mut (dyn AppWidget + Send)).collect();

        // collect all input widgets
        let mut input_widgets: Vec<&mut (dyn AppWidget + Send)> = vec![
            &mut self.details_layout.key,
            &mut self.details_layout.headers,
            &mut self.details_layout.payload
        ];

        selectable_widgets.append(&mut input_widgets);
        selectable_widgets[self.selected_widget].normalise_border();
    }
}

// Lists Layout
pub struct ListsLayout<'a> {
    pub lists: Vec<UIList<'a>>,
}

impl <'a> ListsLayout<'a> {
    pub fn new(metadata: &Metadata) -> ListsLayout<'a> {
        // initialise all UI Lists
        let mut lists = vec![];
        lists.push(UIList::new(BROKERS_LIST.to_string(), metadata.brokers_list()));
        lists.push(UIList::new(CONSUMER_GROUPS_LIST.to_string(), metadata.consumer_group_lists()));
        lists.push(UIList::new(TOPICS_LIST.to_string(), metadata.topics_list()));
        lists.push(UIList::new(PARTITIONS_LIST.to_string(), vec![]));

        // select and highlight first list
        let selected_list = 0; 
        lists[selected_list].highlight_border();

        ListsLayout {
            lists
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        use Constraint::*;
        let vertical_layout = Layout::vertical([Percentage(10), Percentage(30), Percentage(30), Fill(1)]);
        let list_areas: [Rect; 4] = vertical_layout.areas(area);

        for i in 0..self.lists.len() {
            self.lists[i].render(frame, list_areas[i]);
        }
    }

    pub fn get_list_by_name(&mut self, name: &str) -> Option<&mut UIList<'a>> {
        self.lists.iter_mut()
            .filter(|l| l.name().starts_with(name))
            .next()
    }

    pub fn handle_navigation(&mut self, direction: Direction) {
        if let Some(selected_list) = self.selected_list_index() {
            self.lists[selected_list].handle_navigation(direction);
        }
    }

    pub fn selected_list(&self) -> &UIList<'a> {
        &self.lists[self.selected_list_index().unwrap_or(0)]
    }

    pub fn selected_list_index(&self) -> Option<usize> {
        self.lists.iter().position(|l| l.is_focused())
    }
}

// Details Layout
pub struct DetailsLayout<'a> {
    mode: AppMode,
    pub metadata: UITable<'a>,

    // consumer mode fields
    pub consumed_message: UIParagraphWithScrollbar<'a>,

    // producer mode fields
    pub key: UIInput<'a, UIParagraph<'a>>,
    pub headers: UIInput<'a, UIParagraphWithScrollbar<'a>>,
    pub payload: UIInput<'a, UIParagraphWithScrollbar<'a>>,
}

impl <'a> DetailsLayout<'a> {
    pub fn new() -> DetailsLayout<'a> {
        let column_headers = vec!["Broker", "Consumer Group", "Topic", "Partition"];
        let column_constraints: Vec<u16> = vec![25, 25, 25, 25];
        let data = vec![vec!["".to_string(); column_constraints.len()]];

        DetailsLayout {
            mode: AppMode::default(),
            metadata: UITable::new(column_headers, column_constraints, data),
            consumed_message: UIParagraphWithScrollbar::new_with_scrollbar_orientation("Message".to_string(),
            "".into(), ScrollbarOrientation::VerticalRight),
            key: UIInput::new("Key".to_string()),
            headers: UIInput::new("Headers".to_string()),
            payload: UIInput::new("Payload".to_string())
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        match self.mode {
            AppMode::Consumer => {
                let layout = Layout::vertical([Constraint::Length(9), Constraint::Fill(1)]);
                let [metadata, message] = layout.areas(area);
                self.metadata.render(frame, metadata);
                self.consumed_message.render(frame, message);
            },
            AppMode::Producer => {
                let layout = Layout::vertical([Constraint::Length(9), Constraint::Length(4), Constraint::Length(9), Constraint::Fill(1)]);
                let [metadata, key, headers, payload] = layout.areas(area);
                self.metadata.render(frame, metadata);
                self.key.render(frame, key);
                self.headers.render(frame, headers);
                self.payload.render(frame, payload);
            }
        }
    }

    pub fn handle_input_event(&mut self, event: InputEvent) {
        //check which input is focused and delegate call to its input handler
        if self.key.is_focused() {
            self.key.handle_event(event);
        } else if self.headers.is_focused() {
            self.headers.handle_event(event);
            self.headers.scroll_to_end();
        } else if self.payload.is_focused() {
            self.payload.handle_event(event);
            self.headers.scroll_to_end();
        }
    }
}

// Footer Layout
pub struct FooterLayout<'a> {
    pub mode: UIParagraph<'a>,
    pub footer: UIParagraph<'a>,
    pub input: UIInput<'a, UIParagraph<'a>>,
}

impl <'a> FooterLayout<'a> {
    pub fn new() -> FooterLayout<'a> {
        FooterLayout {
            mode: UIParagraph::new("".to_string(), Text::default()),
            footer: UIParagraph::new("".to_string(), Text::from(vec![
                Span::from(APP_FOOTER).gray().into_centered_line(),
            ])),
            input: UIInput::new("".to_string()),
        }
    }

    pub fn set_mode(&mut self, mode: String) {
        self.mode.update(Text::from(vec![vec![
                                    Span::from(" Mode: ").gray().bold(),
                                    Span::from(mode).bold().green()].into()]));
    }

    pub fn handle_input_event(&mut self, event: InputEvent) {
        self.input.handle_event(event);
    }

    pub fn input_value(&mut self) -> String {
        self.input.value()
    }

    pub fn set_value(&mut self, value: &'a str) {
        self.input.set_value(value);
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(60), Constraint::Percentage(20)]);
        let [mode, key_mappings, input] = layout.areas(area);
        self.mode.render(frame, mode);
        self.footer.render(frame, key_mappings);
        self.input.render(frame, input);
    }
}

// Help Layout
pub struct HelpLayout<'a> {
    help: UIParagraph<'a>,
}

impl <'a> HelpLayout<'a> {
    pub fn new() -> HelpLayout<'a> {
        let help_text = Text::from(vec![
            Line::from(Span::from("Help Menu").bold().underlined().green().into_centered_line()),
            Span::from("").into(),
            Line::from(Span::from(" Key Mappings:").green()),
            Span::from("").into(),
            help_option(" TAB      ", "Navigate between lists"),
            help_option(" UP/DOWN  ", "Scroll thru the selected lists"),
            help_option(" m        ", "Scroll down the message pane"),
            help_option(" n        ", "Scroll up the message pane"),
            help_option(" RIGHT    ", "Move to next offset"),
            help_option(" Left     ", "Move to previous offset"),
            help_option(" :        ", "Enter edit mode for consumer"),
            help_option(" c        ", "Switch to consumer mode"),
            help_option(" p        ", "Switch to producer mode"),
            help_option(" h        ", "Show/Hide help menu"),
            help_option(" ESC      ", "Exit the edit mode for consumer & producer"),
            help_option(" q        ", "Quit the application"),
            Span::from("").into(),
            Line::from(Span::from(" Consumer Commands (edit mode):").green()),
            Span::from("").into(),
            help_option(" offset!<num>  ", "Fetches the message at a given offset"),
            help_option(" ts!<epoch>    ", "Fetches the message for a given timestamp"),

        ]);

        let mut paragraph = UIParagraph::new_with_color("Help".to_string(), ratatui::style::Color::Gray, help_text);
        paragraph.highlight_border();

        HelpLayout {
            help: paragraph,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // clear existing area before showing help dialog
        frame.render_widget(Clear, area);
        self.help.render(frame, area);
    }

    
}

// Generate a line for a given help option
fn help_option<'a>(key: &'a str, purpose: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::from(key).bold().green().into(), 
        Span::from(purpose).into(),
    ])
}