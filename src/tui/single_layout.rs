use ratatui::{layout::{Constraint, Layout, Rect}, Frame, text::{Text, Span}, widgets::ScrollbarOrientation, style::Stylize};
use crate::kafka::metadata::Metadata;

use super::widgets::{UIParagraph, UITabs, UIParagraphWithScrollbar, UIList, AppWidget};

const APP_NAME: &str = "Kafka2i - TUI for Kafka";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_FOOTER: &str = "<TAB> Switch Tabs | <ESC> Quit | <UP/DOWN> Scroll List";

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
        let vertical_layout = Layout::vertical([Percentage(30), Fill(1)]);
        let [list_layout, details_layout] = vertical_layout.areas(area);

        self.lists_layout.render(frame, list_layout);
        self.details_layout.render(frame, details_layout);
    }
}

// Brokers Layout
pub struct ListsLayout<'a> {
    pub brokers_list: UIList<'a>,
    pub cg_list: UIList<'a>,
    pub topics_list: UIList<'a>,
    pub partitions_list: UIList<'a>
}

impl <'a> ListsLayout<'a> {
    pub fn new(metadata: &Metadata) -> ListsLayout<'a> {
        ListsLayout {
            brokers_list: UIList::new("Brokers", metadata.brokers_list()),
            cg_list: UIList::new("Consumer Groups" , metadata.consumer_group_lists()),
            topics_list: UIList::new("Topics", metadata.topics_list()),
            partitions_list: UIList::new("Partitions", vec![]),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        use Constraint::*;
        let horizontal_layout = Layout::horizontal([Percentage(25), Percentage(25), Percentage(25), Fill(1)]);
        let [brokers_list, cg_list, topic_list, partitions_list] = horizontal_layout.areas(area);
        self.brokers_list.render(frame, brokers_list);
        self.cg_list.render(frame, cg_list);
        self.topics_list.render(frame, topic_list);
        self.partitions_list.render(frame, partitions_list);
    }
}

// Topics and Partitions Layout
pub struct DetailsLayout<'a> {
    pub details: UIParagraphWithScrollbar<'a>
}

impl <'a> DetailsLayout<'a> {
    pub fn new() -> DetailsLayout<'a> {
        DetailsLayout {
            details: UIParagraphWithScrollbar::new("Details", "".into(), ScrollbarOrientation::VerticalRight),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.details.render(frame, area);
    }
}

// Footer Layout
pub struct FooterLayout<'a> {
    pub footer: UIParagraph<'a>
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
