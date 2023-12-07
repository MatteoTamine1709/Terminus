use std::{
    fs,
    io::{self, ErrorKind},
    path::Path,
};

use crossterm::{event::Event, style::Color};
use ropey::Rope;

use crate::editor::TextEditor;

use super::widget::{
    BorderStyle, CursorPosition, CursorPositionByte, ProcessEvent, ShouldExit, WidgetID,
};

fn get_git_branch_name(repo_path: &Path) -> io::Result<String> {
    let head_path = repo_path.join(".git/HEAD");
    let content = fs::read_to_string(head_path)?;
    content
        .split_whitespace()
        .last()
        .and_then(|s| s.split('/').last())
        .map(String::from)
        .ok_or(io::Error::new(ErrorKind::Other, "Branch name not found"))
}

static TOTAL_FILE_INFO_WIDTH: usize = 25;
static TOTAL_POS_INFO_WIDTH: usize = 20;

pub struct StatusBar {
    pub id: WidgetID,
    /// the text
    pub buffer: Rope,

    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,

    /// the color
    pub default_fg: Color,
    pub default_bg: Color,

    pub focused: bool,
    pub targetable: bool,

    /// scolled lines
    pub scroll_lines: usize,

    /// scrolled columns
    pub scroll_columns: usize,

    pub boder_style: BorderStyle,
    pub text_position: CursorPositionByte,
}

impl StatusBar {
    pub fn new(
        text: String,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        fg: Color,
        bg: Color,
        focused: bool,
        targetable: bool,
        boder_style: BorderStyle,
    ) -> Box<Self> {
        let mut buffer = Rope::from_str(&text);
        {
            let current = buffer.to_string();
            let parts = current.split(' ').collect::<Vec<&str>>();
            let file_info = parts[0].to_string();

            let mut status_bar = String::new();
            status_bar.push_str(&file_info);
            status_bar.push_str(&" ".repeat(TOTAL_FILE_INFO_WIDTH - file_info.len()));

            let mut pos_info = String::new();
            pos_info.push_str("Position unavailable");
            status_bar.push_str(&" ".repeat(TOTAL_POS_INFO_WIDTH - pos_info.len()));

            // Get git branch

            match get_git_branch_name(Path::new(".")) {
                Ok(branch_name) => {
                    status_bar.push_str(&format!("Git: {}", branch_name));
                }
                Err(_e) => {
                    status_bar.push_str(&format!("Git: /"));
                }
            }

            buffer = ropey::Rope::from_str(&status_bar);
        }
        Box::new(Self {
            id: WidgetID::StatusBar,
            buffer,
            x,
            y,
            width,
            height,
            default_fg: fg,
            default_bg: bg,
            focused,
            targetable,
            boder_style,
            ..Default::default()
        })
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        // Return a new Widget with default values here
        Self {
            id: WidgetID::StatusBar,
            buffer: Rope::from_str(""),
            scroll_lines: 0,
            scroll_columns: 0,
            default_fg: Color::White,
            default_bg: Color::Black,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            focused: false,
            targetable: false,
            boder_style: BorderStyle::None,
            text_position: 0,
        }
    }
}

impl ProcessEvent for StatusBar {
    fn get_border_style(&self) -> BorderStyle {
        self.boder_style
    }
    fn get_buffer(&self) -> &Rope {
        &self.buffer
    }
    fn get_height(&self) -> usize {
        self.height
    }
    fn get_width(&self) -> usize {
        self.width
    }
    fn get_x(&self) -> usize {
        self.x
    }
    fn get_y(&self) -> usize {
        self.y
    }
    fn get_scroll_lines(&self) -> usize {
        self.scroll_lines
    }
    fn get_scroll_columns(&self) -> usize {
        self.scroll_columns
    }
    fn get_default_fg(&self) -> Color {
        self.default_fg
    }
    fn get_default_bg(&self) -> Color {
        self.default_bg
    }
    fn get_text_position(&self) -> CursorPositionByte {
        self.text_position
    }
    fn get_focused(&self) -> bool {
        self.focused
    }
    fn get_targetable(&self) -> bool {
        self.targetable
    }
    fn get_id(&self) -> WidgetID {
        WidgetID::StatusBar
    }

    fn set_border_style(&mut self, border_style: BorderStyle) {
        self.boder_style = border_style;
    }
    fn set_buffer(&mut self, buffer: Rope) {
        self.buffer = buffer;
    }
    fn set_height(&mut self, height: usize) {
        self.height = height;
    }
    fn set_width(&mut self, width: usize) {
        self.width = width;
    }
    fn set_x(&mut self, x: usize) {
        self.x = x;
    }
    fn set_y(&mut self, y: usize) {
        self.y = y;
    }
    fn set_scroll_lines(&mut self, scroll_lines: usize) {
        self.scroll_lines = scroll_lines;
    }
    fn set_scroll_columns(&mut self, scroll_columns: usize) {
        self.scroll_columns = scroll_columns;
    }
    fn set_default_fg(&mut self, default_fg: Color) {
        self.default_fg = default_fg;
    }
    fn set_default_bg(&mut self, default_bg: Color) {
        self.default_bg = default_bg;
    }
    fn set_text_position(&mut self, text_position: CursorPositionByte) {
        self.text_position = text_position;
    }
    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
    fn set_targetable(&mut self, targetable: bool) {
        self.targetable = targetable;
    }
    fn set_id(&mut self, id: WidgetID) {
        self.id = id;
    }

    fn event(
        &mut self,
        editor: &mut TextEditor,
        event: &Event,
    ) -> Option<(CursorPosition, ShouldExit)> {
        if self.focused {
            if let Event::Key(key_event) = event {
                match key_event.modifiers {
                    crossterm::event::KeyModifiers::NONE => match key_event.code {
                        crossterm::event::KeyCode::Esc => {
                            self.focused = false;
                            return Some((self.update_cursor_position_and_view(), false));
                        }
                        crossterm::event::KeyCode::Char(c) => {
                            self.buffer.insert_char(self.text_position, c);
                            self.text_position += 1;
                        }
                        crossterm::event::KeyCode::Backspace => {
                            if self.text_position > 0 {
                                self.buffer
                                    .remove(self.text_position - 1..self.text_position);
                                self.text_position -= 1;
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        {
            let current = self.get_buffer().to_string();
            let parts = current.split(' ').collect::<Vec<&str>>();
            let mut file_info = parts[0].to_string();
            let last_char = file_info.chars().last().unwrap();
            if last_char != '*' && editor.written {
                file_info.push('*');
            }
            if editor.saved {
                file_info = file_info.replace("*", "");
            }

            let mut status_bar = String::new();
            status_bar.push_str(&file_info);
            status_bar.push_str(&" ".repeat(TOTAL_FILE_INFO_WIDTH - file_info.len()));

            let mut pos_info = String::new();
            if let Some(panel) = editor.get_widget(WidgetID::Panel) {
                let pos = panel.get_cursor_view();
                let x = pos.0 + 1 - panel.get_x() + panel.get_scroll_columns();
                let y = pos.1 + 1 - panel.get_y() + panel.get_scroll_lines();
                let percent = y * 100 / panel.get_buffer().len_lines();
                pos_info.push_str(&format!("{}% ({},{})", percent, x, y));
                status_bar.push_str(&pos_info);
            } else {
                pos_info.push_str("Position unavailable");
            }
            status_bar.push_str(&" ".repeat(TOTAL_POS_INFO_WIDTH - pos_info.len()));

            // Get git branch

            match get_git_branch_name(Path::new(".")) {
                Ok(branch_name) => {
                    status_bar.push_str(&format!("Git: {}", branch_name));
                }
                Err(_e) => {
                    status_bar.push_str(&format!("Git: /"));
                }
            }

            self.set_buffer(ropey::Rope::from_str(&status_bar));
        }
        None
    }
}
