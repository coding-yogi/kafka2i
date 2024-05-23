use ratatui::{layout::{Constraint, Layout, Rect, self}, Frame, text::{Text, Span, Line}, widgets::ScrollbarOrientation, style::{Stylize, Color}};
use crate::kafka::metadata::Metadata;

use super::widgets::{UIParagraph, UIParagraphWithScrollbar, UIList, AppWidget, Direction, UITable};

const APP_NAME: &str = "Kafka2i - TUI for Kafka";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_FOOTER: &str = "<TAB> Navigate Lists | <ESC> Quit | <UP/DOWN> Scroll List";

pub const BROKERS_LIST_NAME: &str = "Brokers";
pub const CONSUMER_GROUPS_LIST_NAME: &str = "Consumer Groups";
pub const TOPICS_LIST_NAME: &str = "Topics";
pub const PARTITIONS_LIST_NAME: &str = "Partitions";

// Top level application layout
pub struct AppLayout<'a> {
    pub header_layout: HeaderLayout<'a>,
    pub main_layout: MainLayout<'a>,
    pub footer_layout: FooterLayout<'a>,
}

impl <'a> AppLayout<'a> {
    pub fn new(metadata: &Metadata) -> AppLayout<'a> {
        AppLayout{
            header_layout: HeaderLayout::new(),
            main_layout: MainLayout::new(metadata),
            footer_layout: FooterLayout::new(),
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

// Tabs Layout
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

// Brokers Layout
pub struct ListsLayout<'a> {
    selected_list: usize,
    pub lists: Vec<UIList<'a>>,
}

impl <'a> ListsLayout<'a> {
    pub fn new(metadata: &Metadata) -> ListsLayout<'a> {
        // initlaise all UI Lists
        let mut lists = vec![];
        lists.push(UIList::new(BROKERS_LIST_NAME.to_string(), metadata.brokers_list()));
        lists.push(UIList::new(CONSUMER_GROUPS_LIST_NAME.to_string(), metadata.consumer_group_lists()));
        lists.push(UIList::new(TOPICS_LIST_NAME.to_string(), metadata.topics_list()));
        lists.push(UIList::new(PARTITIONS_LIST_NAME.to_string(), vec![]));

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

    pub fn hande_tab(&mut self) {
        // normalise current block
        self.lists[self.selected_list].normalise_border();

        let mut new_idx = self.selected_list.saturating_add(1);
        if new_idx == self.lists.len() {
            new_idx = 0
        }
        
        // higlight selected list border
        self.selected_list = new_idx;
        self.lists[self.selected_list].highlight_border();
    }

    pub fn selected_list(&self) -> &UIList<'a> {
        &self.lists[self.selected_list]
    }
}

// Topics and Partitions Layout
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
        let layout = Layout::vertical([Constraint::Percentage(15), Constraint::Fill(1)]);
        let [details, message] = layout.areas(area);
        self.details.render(frame, details);
        self.message.render(frame, message);
    }
}

// Footer Layout
pub struct FooterLayout<'a> {
    pub mode: UIParagraph<'a>,
    pub footer: UIParagraph<'a>
}

impl <'a> FooterLayout<'a> {
    pub fn new() -> FooterLayout<'a> {
        FooterLayout {
            mode: UIParagraph::new("".to_string(), Text::default()),
            footer: UIParagraph::new("".to_string(), Text::from(vec![
                Span::from(APP_FOOTER).gray().into_centered_line(),
            ]))
        }
    }

    pub fn update_mode(&mut self, mode: String) {
        self.mode.update(Text::from(vec![vec![
                                    Span::from(" Mode: ").gray().bold(),
                                    Span::from(mode).bold().green()].into()]));
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::horizontal([Constraint::Percentage(20), Constraint::Fill(1)]);
        let [mode, key_mappings] = layout.areas(area);
        self.mode.render(frame, mode);
        self.footer.render(frame, key_mappings);
    }
}
