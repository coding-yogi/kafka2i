use ratatui::{
    widgets::{List, Block, ListItem, Borders, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Tabs, Table, Row}, 
    text::{self, Span, Text}, style::{Style, Modifier, Color, palette::tailwind}, 
    prelude::Rect, Frame, symbols, layout::Constraint,
};

const HIGHLIGHT_COLOR: Color = Color::Yellow;
const NORMAL_COLOR: Color = Color::Green;

pub enum Direction {
    UP,
    DOWN,
}

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

    pub fn selected(&self) -> usize {
        self.state
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
    pub fn new(name: &'a str, items: Vec<String>) -> UIList<'a>{ 
        let items_clone = items.clone();
        let list_items = get_list_items(items_clone);
        
        UIList {
            name,
            items,
            list: get_list(name, list_items),
            state: ListState::default(),
            area: Rect::default(),
        }
    }

    pub fn update(&mut self, items: Vec<String>) {
        let items_clone = items.clone();
        let list_items = get_list_items(items_clone);

        self.items = items;
        self.list = get_list(self.name, list_items);
        self.state = ListState::default();
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

    pub fn handle_navigation(&mut self, direction: Direction) {
        match direction {
            Direction::UP => self.handle_up(),
            Direction::DOWN => self.handle_down(), 
        }
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

fn get_list_items(items: Vec<String>) -> Vec<ListItem<'static>> {
    items
        .into_iter()
        .map(|i| ListItem::new(vec![text::Line::from(Span::raw(i))]))
        .collect::<Vec<ListItem>>()
}

fn get_list<'a>(name: &'a str, list_items: Vec<ListItem<'a>>) -> List<'a> {
    List::new(list_items)
        .block(create_block(NORMAL_COLOR, name, true))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(HIGHLIGHT_COLOR))
        .highlight_symbol("> ")
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

    pub fn update(&mut self, text: Text<'a>) {
        let content_length = text.lines.len();
        self.paragraph.update(text); 
        self.scrollbar.update(content_length);
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

    pub fn update(&mut self, text: Text<'a>) {
        self.paragraph = Paragraph::new(text).block(create_block(NORMAL_COLOR, self.name, true))
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

    pub fn update(&mut self, content_length: usize) {
        self.state = ScrollbarState::new(content_length);
        self.scroll_state = 0;
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

#[derive(Clone)]
pub struct UITable<'a> {
    table : Table<'a>,
}

impl <'a> UITable<'a> {
    pub fn new(columns: Vec<&'a str>, column_widths: &'a[u16], data: Vec<Vec<String>>) -> UITable<'a> {
        let mut constraints = vec![];
        for column_width in column_widths {
            constraints.push(Constraint::Percentage(*column_width))
        }

        let mut rows: Vec<Row> = vec![];
        for data_row in data {
            rows.push(Row::new(data_row));
        }

        UITable {
            table: Table::new(rows, constraints).header(Row::new(columns))
        }
    }
}

