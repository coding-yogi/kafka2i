use ratatui::{
    widgets::{List, Block, ListItem, Borders, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Tabs, Padding}, 
    text::{self, Span, Text}, style::{Style, Modifier, Color, palette::tailwind}, 
    prelude::Rect, Frame, symbols,
};


const HIGHLIGHT_COLOR: Color = Color::Yellow;
const NORMAL_COLOR: Color = Color::Green;

pub trait AppWidget {
    fn render(&mut self, frame: &mut Frame, area: Rect);
    fn highlight_border(&mut self);
    fn normalise_border(&mut self);
}


// UIWidget contains the specific widget type and the Rect which holds the Widget
#[derive(Clone)]
pub struct UIBlock<'a> {
    block: Block<'a>,
    area: Rect,
}

impl <'a> UIBlock<'a> {
    pub fn new(name: &'a str) -> UIBlock<'a> {
        UIBlock{
            area: Rect::default(),
            block: Block::default()
                .borders(Borders::ALL)
                .border_set(symbols::border::ROUNDED)
                .border_style(Style::new().fg(NORMAL_COLOR)).title(name)
        }
    }
}

impl <'a> AppWidget for UIBlock<'a> {
     fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.area = area;
        frame.render_widget::<&Block>(&self.block, self.area);
    }

    fn highlight_border(&mut self) {
        self.block = self.block.clone().border_style(Style::new().fg(HIGHLIGHT_COLOR))
    }

    fn normalise_border(&mut self) {
        self.block = self.block.clone().border_style(Style::new().fg(NORMAL_COLOR));
    }
}

#[derive(Clone)]
pub struct UITabs<'a> {
    titles: &'a [&'a str],
    state: usize,
    area: Rect,
    tabs: Tabs<'a>,
}

impl <'a> UITabs<'a> {
    pub fn new(name: &'a str, titles: &'a[&'a str]) -> UITabs<'a> {

        let highlight_style = (Color::default(), tailwind::GREEN.c700);

        UITabs {
            titles,
            state: 0,
            area: Rect::default(),
            tabs: Tabs::new(titles.to_vec())
                .block(Block::default()
                       .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                       .border_style(Style::new().fg(NORMAL_COLOR)).title(name))
                .select(0)
                .highlight_style(highlight_style),
        }
    }

    pub fn select(&mut self, idx: usize) {
        self.tabs = self.tabs.clone().select(idx);
        self.state = idx;
    }

    pub fn selected_title(&self) -> &str {
        self.titles[self.state]
    }

    pub fn handle_tab(&mut self) {
        let no_of_titles = self.titles.len();
        if self.state == no_of_titles - 1 {
            self.select(0);
        } else {
            self.select(self.state.saturating_add(1));
        }
    }
}

impl <'a> AppWidget for UITabs<'a> {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.area = area;
        frame.render_widget::<&Tabs>(&self.tabs, self.area);
    }

    fn highlight_border(&mut self) {
        // No implementation required
    }

    fn normalise_border(&mut self) {
        // No implementation required
    }
}

// UIList conatins the stateful widget's type and Rect which holds the widget
#[derive(Clone)]
pub struct UIList <'a> {
    name: &'a str,
    items: Vec<String>,
    list: List<'a>,
    state: ListState,
    area: Rect,
}

impl <'a> UIList <'a> {
    pub fn new(name: &'a str, items: Vec<String>) -> UIList<'a>
    { 
        let items_clone = items.clone();
        let list_items =  items_clone
                .into_iter()
                .map(|i| ListItem::new(vec![text::Line::from(Span::raw(i))]))
                .collect::<Vec<ListItem>>();

        UIList {
            name,
            items,
            list: List::new(list_items)
                .block(create_block(NORMAL_COLOR, name, true))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(HIGHLIGHT_COLOR))
                .highlight_symbol("> "),
            state: ListState::default(),
            area: Rect::default(),
        }
    }
    
    pub fn select(&mut self, idx: Option<usize>) {
        self.state.select(idx)
    }

    pub fn selected_item(&self) -> Option<String> {
        if let Some(idx) = self.state() {
            return Some(self.items.get(idx).unwrap().clone());
        } 

        None
    }
    
    pub fn state(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn handle_down(&mut self) {
        if let Some(idx) = self.state.selected() {
            if idx == self.list.len()-1 {
                self.state.select(Some(0));
            } else {
                self.state.select(Some(idx + 1));
            }
        } else {
            self.state.select(Some(0))
        }
    }

    pub fn handle_up(&mut self) {
        if let Some(idx) = self.state.selected() {
            if idx == 0 {
                self.state.select(Some(self.list.len()-1));
            } else {
                self.state.select(Some(idx - 1));
            }
        } else {
            self.state.select(Some(0))
        }
    }
}

impl <'a> AppWidget for UIList<'a> {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.area = area;
        frame.render_stateful_widget::<&List>(&self.list, self.area, &mut self.state);
    }
    
    fn highlight_border(&mut self) {
        self.list = self.list.clone().block(create_block(HIGHLIGHT_COLOR, self.name, true));
    }

    fn normalise_border(&mut self) {
        self.list = self.list.clone().block(create_block(NORMAL_COLOR, self.name, true));
    } 
}

#[derive(Clone)]
pub struct UIParagraphWithScrollbar<'a> {
    paragraph: UIParagraph<'a>,
    scrollbar: UIScrollbar<'a>,
}

impl <'a> UIParagraphWithScrollbar<'a> {
    pub fn new(name: &'a str, text: Text<'a>, orientation: ScrollbarOrientation) -> UIParagraphWithScrollbar<'a> {
        let content_length = text.lines.len();

        UIParagraphWithScrollbar {
            paragraph: UIParagraph::new(name, text),
            scrollbar: UIScrollbar::new(orientation, content_length),
        }
    }

    pub fn handle_down(&mut self) {
        self.scrollbar.handle_down();
        self.paragraph.scroll((self.scrollbar.scroll_state,0));
    }

    pub fn handle_up(&mut self) {
        self.scrollbar.handle_up();
        self.paragraph.scroll((self.scrollbar.scroll_state,0));
    }
}

impl <'a> AppWidget for UIParagraphWithScrollbar<'a> {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.paragraph.render(frame, area);
        self.scrollbar.render(frame, area);
    }

    fn normalise_border(&mut self) {
        self.paragraph.normalise_border();
    }

    fn highlight_border(&mut self) {
        self.paragraph.highlight_border();
    }
}

// UiParagraph
#[derive(Clone)]
pub struct UIParagraph <'a> {
    name: &'a str,
    paragraph: Paragraph<'a>,
    area: Rect,
}

impl <'a> UIParagraph<'a> {
    pub fn new(name: &'a str, text: Text<'a>) -> UIParagraph<'a> {
        UIParagraph {
            name,
            paragraph: Paragraph::new(text).block(create_block(NORMAL_COLOR, name, true)),
            area: Rect::default()
        }
    }

    pub fn scroll(&mut self, offset: (u16, u16)) {
       self.paragraph =  self.paragraph.clone().scroll(offset);
    }
}

impl <'a> AppWidget for UIParagraph<'a> {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.area = area;
        frame.render_widget::<&Paragraph>(&self.paragraph, self.area);
    }

    fn normalise_border(&mut self) {
        self.paragraph = self.paragraph.clone().block(create_block(NORMAL_COLOR, self.name, true));
    }

    fn highlight_border(&mut self) {
        self.paragraph = self.paragraph.clone().block(create_block(HIGHLIGHT_COLOR, self.name, true));
    }
}

fn create_block<'a>(color: Color, name: &'a str, with_border: bool) -> Block<'a> {
    let block = Block::default();
    if with_border {
        return block.borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::new().fg(color)).title(name);
    }

    block
}

#[derive(Clone)]
pub struct UIScrollbar<'a> {
    scrollbar: Scrollbar<'a>,
    area: Rect,
    state: ScrollbarState,
    scroll_state: u16,
}

impl <'a> UIScrollbar<'a> {
    pub fn new(orientation: ScrollbarOrientation, content_length: usize) -> UIScrollbar<'a> {
        UIScrollbar { 
            scrollbar: Scrollbar::new(orientation),
            area: Rect::default(), 
            state: ScrollbarState::new(content_length),
            scroll_state: 0,
        }
    }

    pub fn handle_down(&mut self) {
        self.scroll_state = self.scroll_state.saturating_add(1);
        self.state = self.state.position(self.scroll_state.into());
    }

    pub fn handle_up(&mut self) {
        self.scroll_state = self.scroll_state.saturating_sub(1);
        self.state = self.state.position(self.scroll_state.into());
    }
}

impl <'a> AppWidget for UIScrollbar<'a> {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.area = area;
        frame.render_stateful_widget::<Scrollbar>(self.scrollbar.clone(), self.area, &mut self.state);
    }

    fn normalise_border(&mut self) {
    }

    fn highlight_border(&mut self) {
    }
}


