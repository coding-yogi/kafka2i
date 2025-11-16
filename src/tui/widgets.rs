use std::char;
use ratatui::{
    layout::Constraint, 
    prelude::Rect, 
    style::{Color, Modifier, Style, Stylize}, 
    symbols, 
    text::{self, Span, Text},
     widgets::{
        Block,
        Borders,
        List,
        ListItem,
        ListState,
        Paragraph,
        Row,
        Table,
        TableState,
        Wrap
    }, 
    Frame
};
use tui_textarea::{CursorMove, TextArea};
use ratatui_explorer::{File, FileExplorer, Theme};

pub const HIGHLIGHT_COLOR: Color = Color::Yellow;
pub const NORMAL_COLOR: Color = Color::Green;
pub const ERROR_COLOR: Color = Color::Red;

#[derive(PartialEq)]
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

// UIList conatins the stateful widget's type and Rect which holds the widget
#[derive(Clone)]
pub struct UIList <'a> {
    name: String,
    items: Vec<String>,
    list: List<'a>,
    state: ListState,
    area: Rect,
    focused: bool,
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
            focused: false,
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
    
    pub fn selected_item(&self) -> Option<String> {
        if let Some(idx) = self.state() {
            if let Some(item) = self.items.get(idx) {
                return Some(item.clone());
            }
        }

        None
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }
    
    pub fn state(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn handle_navigation(&mut self, direction: &Direction) {
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
        //.map(|i| ListItem::new(Paragraph::new(i).wrap(Wrap { trim: false })))
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
        self.focused = true
    }

    fn normalise_border(&mut self) {
        self.list = self.list.clone().block(create_block(NORMAL_COLOR, self.name.clone(), true));
         self.focused = false
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
            paragraph: Paragraph::new(text)
            .block(create_block(NORMAL_COLOR, name, true)),
            area: Rect::default()
        }
    }

    pub fn new_with_color(name: String, bg_color: Color, text: Text<'a>) -> UIParagraph<'a> {
        UIParagraph {
            name: name.clone(),
            paragraph: Paragraph::new(text)
            .block(create_block(bg_color, name, true)),
            area: Rect::default()
        }
    }

    pub fn update(&mut self, text: Text<'a>) {
        self.update_with_name(self.name.clone(), text);
    }

    pub fn update_with_name(&mut self, name: String, text: Text<'a>) {
        self.paragraph = Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .block(create_block(HIGHLIGHT_COLOR, name, true))
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

#[derive(PartialEq)]
pub enum InputEvent {
    NewChar(char),
    RemovePrevChar,
    RemoveNextChar,
    MoveCursor(Direction),
    Reset,
}

pub struct UITextArea<'a> {
    name: String,
    text_area: TextArea<'a>,
    area: Rect,
    focused: bool,
    errored: bool,
}

impl <'a> UITextArea<'a> {
    pub fn new(name: String) -> UITextArea<'a> {
        let mut text_area = TextArea::default();
        text_area.set_block(create_block(NORMAL_COLOR, name.clone(), true));
        text_area.set_cursor_line_style(Style::default());
        text_area.set_cursor_style(Style::default());

        UITextArea {
            name: name,
            text_area: text_area,
            area: Rect::default(),
            focused: false,
            errored: false,
        }
    }

    pub fn set_title(&mut self, title: String) {
        self.name = title
    }

    pub fn set_errored(&mut self, errored: bool) {
        self.errored = errored
    }

    pub fn handle_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::NewChar(c) => self.text_area.insert_char(c),
            InputEvent::RemovePrevChar => { self.text_area.delete_char(); },
            InputEvent::RemoveNextChar => { self.text_area.delete_next_char(); },
            InputEvent::MoveCursor(d) => {
                match d {
                    Direction::LEFT => self.text_area.move_cursor(CursorMove::Back),
                    Direction::RIGHT => self.text_area.move_cursor(CursorMove::Forward),
                    Direction::UP => self.text_area.move_cursor(CursorMove::Up),
                    Direction::DOWN => self.text_area.move_cursor(CursorMove::Down),
                }
            },
            InputEvent::Reset => self.reset(),
        }
    }

    fn reset(&mut self) {
        self.update_text("".to_string());
    }

    pub fn update_text(&mut self, text: String) {
        self.update_title_and_text(self.name.clone(), text);
    }

    pub fn update_title_and_text(&mut self, title: String, text: String) {
        self.text_area = TextArea::new(text.split('\n').map(|s| s.to_string()).collect());

        // If the text area was already focused, keep the border highlighting
        if self.focused {
            self.text_area.set_block(create_block(HIGHLIGHT_COLOR, title, true));
        } else {
            self.text_area.set_block(create_block(NORMAL_COLOR, title, true));
        }

        self.cursor_visibility(false);
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn scroll(&mut self, direction: &Direction) {
        match direction {
            Direction::UP => self.text_area.scroll((-1, 0)),
            Direction::DOWN => self.text_area.scroll((1, 0)),
            _ => ()
        }
    }

    pub fn text(&self) -> String {
        self.text_area.lines().join("\n")
    }

    pub fn cursor_visibility(&mut self, visible: bool) {
        if visible {
            self.text_area.set_cursor_style(Style::default().bg(Color::Green));
            self.text_area.set_cursor_line_style(Style::default());
        } else {
            self.text_area.set_cursor_style(Style::default());
            self.text_area.set_cursor_line_style(Style::default());
        }
    }
}

impl <'a> AppWidget for UITextArea<'a> {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.area = area;
        frame.render_widget::<&TextArea>(&self.text_area, self.area);
    }

    fn normalise_border(&mut self) {
        self.focused = false;
        let mut color = NORMAL_COLOR;

        if self.errored {
            color = ERROR_COLOR
        }

        self.text_area.set_block(create_block(color, self.name.clone(), true));
        self.text_area.set_cursor_style(Style::default());
    }

    fn highlight_border(&mut self) {
        self.focused = true;
        self.errored = false;
        self.text_area.set_block(create_block(HIGHLIGHT_COLOR, self.name.clone(), true));
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

const ROW_HEIGHT: u16 = 5;

impl <'a> UITable<'a> {
    pub fn new(columns: Vec<&'a str>, column_widths: Vec<u16>, data: Vec<Vec<String>>) -> UITable<'a> {
        let mut constraints = vec![];
        for column_width in column_widths {
            constraints.push(Constraint::Percentage(column_width))
        }

        let mut rows: Vec<Row> = vec![];
        for data_row in data.iter() {
            rows.push(Row::new(data_row.clone()).height(ROW_HEIGHT));
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
            rows.push(Row::new(data_row.clone()).height(ROW_HEIGHT));
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
        // No implementation required
    }

    fn highlight_border(&mut self) {
        // No implementation required
    }
}

#[derive(PartialEq)]
pub struct UIFileExplorer {
    area: Rect,
    file_explorer: Option<FileExplorer>,
}

impl UIFileExplorer {
    pub fn new() -> UIFileExplorer {
        let file_explorer = match FileExplorer::with_theme(file_explorer_default_theme()) {
            Ok(fe) => Some(fe),
            Err(err) => {
                log::error!("failed to open explorer: {}", err);
                None
            }
        };

        UIFileExplorer {
            area: Rect::default(),
            file_explorer: file_explorer
        }
    }

    pub fn handle_input(&mut self, direction: Direction) {
        if self.file_explorer == None {
            return
        }

        let fe = self.file_explorer.as_mut().unwrap();

        // reset theme if set to error
        fe.set_theme(file_explorer_default_theme());

        let result = match direction {
            Direction::DOWN => fe.handle(ratatui_explorer::Input::Down),
            Direction::UP => fe.handle(ratatui_explorer::Input::Up),
            Direction::LEFT => fe.handle(ratatui_explorer::Input::Left),
            Direction::RIGHT => fe.handle(ratatui_explorer::Input::Right),
        };

        if let Err(err) = result {
            log::error!("file explorer failed to handle an input event {}", err);
        }
    }

    pub fn get_selected_file(&mut self) -> Option<File> {
        if self.file_explorer == None {
            return None
        }

        let fe = self.file_explorer.as_mut().unwrap();
        if fe.current().is_file() {
            return Some(fe.current().clone());
        }

        self.show_error("Error: Select a file".to_string());
        None
    }

    pub fn show_error(&mut self, error: String) {
        self.file_explorer.as_mut().unwrap().set_theme(file_explorer_error_theme(error));
    }
}

impl  AppWidget for UIFileExplorer {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        match &self.file_explorer {
            Some(fe) => {
                self.area = area;
                frame.render_widget(&fe.widget(), area);
            },
            None => (),
        }
    }

    fn normalise_border(&mut self) {
        // No implementation required
    }

    fn highlight_border(&mut self) {
        // No implementation required
    }
}

fn file_explorer_base_theme() -> Theme {
    Theme::default()
        .with_highlight_item_style(Style::default().fg(HIGHLIGHT_COLOR))
        .with_highlight_dir_style(Style::default().fg(HIGHLIGHT_COLOR).add_modifier(Modifier::BOLD))
        .with_highlight_symbol("> ".into())
        .with_title_bottom(|_| " ← Parent | → Child | ↑ Prev File | ↓ Next File".into())
}

fn file_explorer_default_theme() -> Theme {
    file_explorer_base_theme()
        .with_block(create_block(HIGHLIGHT_COLOR, "Select File".to_string(), true))
}

fn file_explorer_error_theme(text: String) -> Theme {
    file_explorer_base_theme()
        .with_block(create_block(ERROR_COLOR, text, true))
}