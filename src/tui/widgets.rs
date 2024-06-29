use std::char;

use crossterm::event;
use ratatui::{
    widgets::{List, Block, ListItem, Borders, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Tabs, Table, Row, TableState}, 
    text::{self, Span, Text}, style::{Style, Modifier, Color, palette::tailwind, Stylize}, 
    prelude::Rect, Frame, symbols, layout::Constraint,
};
use tui_input::{Input, InputRequest};

const HIGHLIGHT_COLOR: Color = Color::Yellow;
const NORMAL_COLOR: Color = Color::Green;

pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT
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
    name: String,
    items: Vec<String>,
    list: List<'a>,
    state: ListState,
    area: Rect,
}

impl <'a> UIList <'a> {
    pub fn new(name: String, items: Vec<String>) -> UIList<'a>{ 
        let items_clone = items.clone();
        let list_items = get_list_items(items_clone);

        let list_count = list_items.len();
        let name = format!("{} ({})", name, list_count);

        UIList {
            name: name.clone(),
            items,
            list: get_list(name, list_items),
            state: ListState::default(),
            area: Rect::default(),
        }
    }

    pub fn name(&self) -> &str {
        // return base name w/o count
        &self.name.split("(").collect::<Vec<&str>>()[0].trim()
    }

    pub fn update(&mut self, items: Vec<String>) {
        let items_clone = items.clone();
        let list_items = get_list_items(items_clone);
        let list_count = list_items.len();

        self.name = format!("{} ({})", self.name(), list_count);
        self.items = items;
        self.list = get_list(self.name.clone(), list_items);
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
            _ => (),
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

fn get_list<'a>(name: String, list_items: Vec<ListItem<'a>>) -> List<'a> {
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
        self.list = self.list.clone().block(create_block(HIGHLIGHT_COLOR, self.name.clone(), true));
    }

    fn normalise_border(&mut self) {
        self.list = self.list.clone().block(create_block(NORMAL_COLOR, self.name.clone(), true));
    } 
}

#[derive(Clone)]
pub struct UIParagraphWithScrollbar<'a> {
    paragraph: UIParagraph<'a>,
    scrollbar: UIScrollbar<'a>,
}

impl <'a> UIParagraphWithScrollbar<'a> {
    pub fn new(name: String, text: Text<'a>, orientation: ScrollbarOrientation) -> UIParagraphWithScrollbar<'a> {
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
    name: String,
    paragraph: Paragraph<'a>,
    area: Rect,
}

impl <'a> UIParagraph<'a> {
    pub fn new(name: String, text: Text<'a>) -> UIParagraph<'a> {
        UIParagraph {
            name: name.clone(),
            paragraph: Paragraph::new(text).block(create_block(NORMAL_COLOR, name, true)),
            area: Rect::default()
        }
    }

    pub fn update(&mut self, text: Text<'a>) {
        self.paragraph = Paragraph::new(text).block(create_block(NORMAL_COLOR, self.name.clone(), true))
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
        self.paragraph = self.paragraph.clone().block(create_block(NORMAL_COLOR, self.name.clone(), true));
    }

    fn highlight_border(&mut self) {
        self.paragraph = self.paragraph.clone().block(create_block(HIGHLIGHT_COLOR, self.name.clone(), true));
    }
}

fn create_block<'a>(color: Color, name: String, with_border: bool) -> Block<'a> {
    let block = Block::default();
    if with_border {
        return block.borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::new().fg(color)).title(name);
    }

    block
}

pub enum InputEvent {
    NewChar(char),
    RemovePrevChar,
    RemoveNextChar,
    MoveCursor(Direction),
    Reset,
}

#[derive(Clone)]
pub struct UIInput<'a> {
    paragraph: UIParagraph<'a>,
    input: Input,
}

impl <'a> UIInput<'a> {
    pub fn new(name: String) -> UIInput<'a> {
        UIInput {
            paragraph: UIParagraph::new(name, "".into()),
            input: Input::default(),
        }
    }

    pub fn handle_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::NewChar(c) => self.enter_char(c),
            InputEvent::RemovePrevChar => self.remove_previous_char(),
            InputEvent::RemoveNextChar => (),
            InputEvent::MoveCursor(d) => self.move_cursor(d),
            InputEvent::Reset => self.reset(),
        }
    }

    fn reset(&mut self) {
        self.input.reset();
        self.paragraph.update("".into());
    }

    fn enter_char(&mut self, new_char: char) {
        self.input.handle(InputRequest::InsertChar(new_char));
        self.paragraph.update(self.input.value().to_string().into());
    }

    fn remove_previous_char(&mut self) {
        self.input.handle(InputRequest::DeletePrevChar);
        self.paragraph.update(self.input.value().to_string().into());
    }

    fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::LEFT => self.input.handle(InputRequest::GoToPrevChar),
            Direction::RIGHT => self.input.handle(InputRequest::GoToNextChar),
            _ => None,
        };
    }

    pub fn value(&mut self) -> String {
        self.input.value().to_string()
    }
}

impl <'a> AppWidget for UIInput<'a> {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.paragraph.render(frame, area);
    }

    fn normalise_border(&mut self) {
        self.paragraph.normalise_border();
    }

    fn highlight_border(&mut self) {
        self.paragraph.highlight_border();
    }
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
    area: Rect,
    state: TableState,
    columns: Vec<&'a str>,
    data: Vec<Vec<String>>
}

impl <'a> UITable<'a> {
    pub fn new(columns: Vec<&'a str>, column_widths: Vec<u16>, data: Vec<Vec<String>>) -> UITable<'a> {
        let mut constraints = vec![];
        for column_width in column_widths {
            constraints.push(Constraint::Percentage(column_width))
        }

        let mut rows: Vec<Row> = vec![];
        for data_row in data.iter() {
            rows.push(Row::new(data_row.clone()).height(5));
        }

        UITable {
            table: Table::new(rows, constraints)
                .header(Row::new(columns.clone()).bold())
                .block(create_block(NORMAL_COLOR, "".to_string(), true)),
            area: Rect::default(),
            state: TableState::default(),
            columns,
            data,
        }
    }

    pub fn update_cell_data(&mut self, column_name: &str, row: usize, cell_data: String) {
        let column = self.get_column_idx_for(column_name); 
        let data_row = self.data.get_mut(row).unwrap();
        data_row[column] = cell_data;
        
        let mut rows: Vec<Row> = vec![];
        for data_row in &self.data {
            rows.push(Row::new(data_row.clone()).height(5));
        }
        
        self.table = self.table.clone().rows(rows);
    }

    pub fn get_column_idx_for(&self, name: &str) -> usize {
        let mut col_idx = 0;
        for col_name in self.columns.iter() {
            if name.contains(col_name) {
                return col_idx
            }

            col_idx += 1;
        }

        0
    }
}

impl <'a> AppWidget for UITable<'a> {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.area = area;
        frame.render_stateful_widget::<Table>(self.table.clone(), self.area, &mut self.state);
    }

    fn normalise_border(&mut self) {
    }

    fn highlight_border(&mut self) {
    }
}

