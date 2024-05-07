use ratatui::{layout::{Constraint, Layout, Rect}, Frame, text::{Text, Span}, widgets::ScrollbarOrientation, style::Stylize};
use strum::{EnumIter, FromRepr, Display, VariantNames};
use super::widgets::{UIParagraph, UITabs, UIParagraphWithScrollbar, UIList, AppWidget};

const APP_NAME: &str = "Kafka2i - TUI for Kafka";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_FOOTER: &str = "<TAB> Switch Tabs | <ESC> Quit | <UP/DOWN> Scroll List";

const LIST_AND_DETAILS_CONSTRAINTS: [Constraint; 2] = [Constraint::Percentage(20), Constraint::Fill(1)];

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter, PartialEq, VariantNames)]
pub enum SelectedTab {
    #[default]
    #[strum(to_string = "BROKERS")]
    BROKERS,
    #[strum(to_string = "CONSUMER GROUPS")]
    CONSUMERGROUPS,
    #[strum(to_string = "TOPICS & PARTITIONS")]
    TOPICS,
}

// Top level application layout
pub struct AppLayout<'a> {
    pub header_layout: HeaderLayout<'a>,
    pub tabs_layout: TabsLayout<'a>,
    pub footer_layout: FooterLayout<'a>,
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
pub struct TabsLayout<'a> {
    pub tabs: UITabs<'a>,
    pub broker_layout: BrokerTabLayout<'a>,
    pub cg_layout : ConsumerGroupTabLayout<'a>,
    pub topics_layout: TopicsAndPartitionsTabLayout<'a>
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

// Brokers Layout
pub struct BrokerTabLayout<'a> {
    pub brokers_list: UIList<'a>,
    pub broker_details: UIParagraphWithScrollbar<'a>
}

impl <'a> BrokerTabLayout<'a> {
    pub fn new() -> BrokerTabLayout<'a> {
        BrokerTabLayout {
            brokers_list: UIList::new("Brokers", vec![]),
            broker_details: UIParagraphWithScrollbar::new("Details", "".into(), ScrollbarOrientation::VerticalRight),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let horizontal_layout = Layout::horizontal(LIST_AND_DETAILS_CONSTRAINTS);
        let [list, details] = horizontal_layout.areas(area);
        self.brokers_list.render(frame, list);
        self.broker_details.render(frame, details);
    }
}

// Consumer Group Layout
pub struct ConsumerGroupTabLayout<'a> {
    pub cg_list: UIList<'a>,
    pub cg_details: UIParagraphWithScrollbar<'a>
}

impl <'a> ConsumerGroupTabLayout<'a> {
    pub fn new() -> ConsumerGroupTabLayout<'a> {
        ConsumerGroupTabLayout {
            cg_list: UIList::new("Consumer Groups", vec![]),
            cg_details: UIParagraphWithScrollbar::new("Details", "".into(), ScrollbarOrientation::VerticalRight),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let horizontal_layout = Layout::horizontal(LIST_AND_DETAILS_CONSTRAINTS);
        let [list, details] = horizontal_layout.areas(area);

        self.cg_list.render(frame, list);
        self.cg_details.render(frame, details);
    }
}

// Topics and Partitions Layout
pub struct TopicsAndPartitionsTabLayout<'a> {
    pub topics_list: UIList<'a>,
    pub partitions_list: UIList<'a>,
    pub topic_parition_details: UIParagraphWithScrollbar<'a>
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
        let horizontal_layout = Layout::horizontal(LIST_AND_DETAILS_CONSTRAINTS);
        let [list, details] = horizontal_layout.areas(area);
        let vertical_layout = Layout::vertical([Constraint::Percentage(50), Constraint::Fill(1)]);
        let [topics_list, partitions_list] = vertical_layout.areas(list);

        self.topics_list.render(frame, topics_list);
        self.partitions_list.render(frame, partitions_list);
        self.topic_parition_details.render(frame, details);
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
