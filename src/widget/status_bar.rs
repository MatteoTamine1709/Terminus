use std::{
    fs,
    io::{self, ErrorKind},
    path::Path,
};

use crossterm::{event::Event, style::Color};
use ropey::Rope;

use crate::editor::TextEditor;

use super::widget::{
    BorderStyle, ColorText, CursorPosition, CursorPositionByte, ProcessEvent, ShouldExit,
    WidgetType,
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

static TOTAL_FILE_INFO_WIDTH: usize = 45;
static TOTAL_POS_INFO_WIDTH: usize = 20;

pub struct StatusBar {
    pub typ: WidgetType,
    pub id: usize,
    /// the text
    pub buffer: Rope,
    pub colors: Vec<Vec<ColorText>>,

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

    pub z_index: usize,
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
            let mut file_info = parts[0].to_string();

            let mut status_bar = String::new();
            if file_info.len() >= TOTAL_FILE_INFO_WIDTH {
                file_info =
                    String::from(&file_info[(file_info.len() - TOTAL_FILE_INFO_WIDTH + 1)..]);
            }
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
            typ: WidgetType::StatusBar,
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
            typ: WidgetType::StatusBar,
            id: 0,
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
            z_index: 0,
            colors: Vec::new(),
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
    fn get_type(&self) -> WidgetType {
        WidgetType::StatusBar
    }
    fn get_id(&self) -> usize {
        self.id
    }
    fn get_z_idx(&self) -> usize {
        self.z_index
    }

    fn get_colors(&self) -> Vec<Vec<ColorText>> {
        self.colors.clone()
    }
    fn get_colors_mut(&mut self) -> &mut Vec<Vec<ColorText>> {
        &mut self.colors
    }
    fn set_colors(&mut self, colors: Vec<Vec<ColorText>>) {
        self.colors = colors;
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
    fn set_type(&mut self, id: WidgetType) {
        self.typ = id;
    }
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    fn set_z_idx(&mut self, z_idx: usize) {
        self.z_index = z_idx;
    }
    fn event(
        &mut self,
        editor: &mut TextEditor,
        event: &Event,
    ) -> Option<(CursorPosition, ShouldExit)> {
        {
            let current = self.get_buffer().to_string();
            let parts = current.split(' ').collect::<Vec<&str>>();
            let mut file_info = parts[0].to_string();
            let last_char = file_info.chars().last().unwrap();
            if last_char != '*' && editor.written {
                file_info.push('*');
            }
            if !editor.written {
                file_info = file_info.replace("*", "");
            }

            let mut status_bar = String::new();
            if file_info.len() >= TOTAL_FILE_INFO_WIDTH {
                file_info = String::from(&file_info[(file_info.len() - TOTAL_FILE_INFO_WIDTH)..]);
            }
            status_bar.push_str(&file_info);
            status_bar.push_str(&" ".repeat(TOTAL_FILE_INFO_WIDTH - file_info.len()));

            let mut pos_info = String::new();
            if let Some(panel) = editor.get_widget(WidgetType::Panel) {
                let pos = panel.get_cursor_view();

                let x = pos.0 + 1 - panel.get_x() as i32 + panel.get_scroll_columns() as i32;
                let y = pos.1 + 1 - panel.get_y() as i32 + panel.get_scroll_lines() as i32;
                let percent = y * 100 / (panel.get_buffer().len_lines() as i32);
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
