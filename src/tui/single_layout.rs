use ratatui::{layout::{Constraint, Layout, Rect}, style::Stylize, text::{Line, Span, Text}, widgets::{Clear, ScrollbarOrientation}, Frame};
use crate::kafka::metadata::Metadata;

use super::widgets::{AppWidget, Direction, InputEvent, UIInput, UIList, UIParagraph, UIParagraphWithScrollbar, UITable};

const APP_NAME: &str = "Kafka2i - TUI for Kafka";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_FOOTER: &str = "Press <H> for help | Press <ESC> to quit";

pub const BROKERS_LIST: &str = "Brokers";
pub const CONSUMER_GROUPS_LIST: &str = "Consumer Groups";
pub const TOPICS_LIST: &str = "Topics";
pub const PARTITIONS_LIST: &str = "Partitions";



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
        AppLayout{
            header_layout: HeaderLayout::new(),
            main_layout: MainLayout::new(metadata),
            footer_layout: FooterLayout::new(),
            help_layout: HelpLayout::new(),
            show_help: false,
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        use Constraint::*;

        //overall layout
        let outer_layout = Layout::vertical([Length(5), Fill(1), Length(3)]);
        let [title, main, footer] = outer_layout.areas(frame.size());

        self.header_layout.render(frame, title);
        self.main_layout.render(frame, main);
        self.footer_layout.render(frame, footer);

        // centered help layout
        if self.show_help {
            self.help_layout.render(frame, self.centered_help_area(frame));
        }
    }

    // function to get a rect of 60 x 40 in the center of the terminal
    fn centered_help_area(&self, frame: &Frame) -> Rect {
        let area = frame.size();
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
        HeaderLayout{
            title: UIParagraph::new("".to_string(), Text::from(vec![
                Span::from(APP_NAME).bold().green().into_centered_line(),
                Span::from(APP_VERSION).gray().into_centered_line()
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
    pub details_layout: DetailsLayout<'a>
}

impl <'a> MainLayout<'a> {
    pub fn new(metadata: &Metadata) -> MainLayout<'a> {
        MainLayout {
            lists_layout: ListsLayout::new(metadata),
            details_layout: DetailsLayout::new()
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        use Constraint::*;
        let horizontal_layout = Layout::horizontal([Percentage(25), Fill(1)]);
        let [list_layout, details_layout] = horizontal_layout.areas(area);

        self.lists_layout.render(frame, list_layout);
        self.details_layout.render(frame, details_layout);
    }
}

// Lists Layout
pub struct ListsLayout<'a> {
    selected_list: usize,
    pub lists: Vec<UIList<'a>>,
}

impl <'a> ListsLayout<'a> {
    pub fn new(metadata: &Metadata) -> ListsLayout<'a> {
        // initlaise all UI Lists
        let mut lists = vec![];
        lists.push(UIList::new(BROKERS_LIST.to_string(), metadata.brokers_list()));
        lists.push(UIList::new(CONSUMER_GROUPS_LIST.to_string(), metadata.consumer_group_lists()));
        lists.push(UIList::new(TOPICS_LIST.to_string(), metadata.topics_list()));
        lists.push(UIList::new(PARTITIONS_LIST.to_string(), vec![]));

        // select and highlight first list
        let selected_list = 0; 
        lists[selected_list].highlight_border();

        ListsLayout {
            selected_list,
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
        self.lists[self.selected_list].handle_navigation(direction);
    }

    pub fn handle_tab(&mut self, back_tab: bool) {
        // normalise current block
        self.lists[self.selected_list].normalise_border();

        let mut new_idx = self.selected_list;

        if back_tab {
            if new_idx == 0 {
                new_idx = self.lists.len() - 1;
            } else {
                new_idx = self.selected_list.saturating_sub(1);
            }

        } else {
            new_idx = self.selected_list.saturating_add(1);
            if new_idx == self.lists.len() {
                new_idx = 0
            }
        }
        
        // higlight selected list border
        self.selected_list = new_idx;
        self.lists[self.selected_list].highlight_border();
    }

    pub fn selected_list(&self) -> &UIList<'a> {
        &self.lists[self.selected_list]
    }
}

// Details Layout
pub struct DetailsLayout<'a> {
    pub details: UITable<'a>,
    pub message: UIParagraphWithScrollbar<'a>
}

impl <'a> DetailsLayout<'a> {
    pub fn new() -> DetailsLayout<'a> {
        let column_headers = vec!["Broker", "Consumer Group", "Topic", "Partition"];
        let column_constraints: Vec<u16> = vec![25, 25, 25, 25];
        let data = vec![vec!["".to_string(); column_constraints.len()]];

        DetailsLayout {
            details: UITable::new(column_headers, column_constraints, data),
            message: UIParagraphWithScrollbar::new("Message".to_string(), "".into(), ScrollbarOrientation::VerticalRight),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // The data in the details panel won't display if height is lesser than 13
        let layout = Layout::vertical([Constraint::Length(13), Constraint::Fill(1)]);
        let [details, message] = layout.areas(area);
        self.details.render(frame, details);
        self.message.render(frame, message);
    }
}

// Footer Layout
pub struct FooterLayout<'a> {
    pub mode: UIParagraph<'a>,
    pub footer: UIParagraph<'a>,
    pub input: UIInput<'a>,
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

    pub fn update_mode(&mut self, mode: String) {
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
            help_option(" M        ", "Scroll down the message pane"),
            help_option(" N        ", "Scroll up the message pane"),
            help_option(" RIGHT    ", "Move to next offset"),
            help_option(" Left     ", "Move to previous offset"),
            help_option(" :        ", "Enter edit mode"),
            help_option(" H        ", "Show/Hide help menu"),
            help_option(" ESC      ", "Exist the application"),
            Span::from("").into(),
            Line::from(Span::from(" Commands (edit mode):").green()),
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