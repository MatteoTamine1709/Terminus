use crossterm::{event::Event, style::Color};
use ropey::Rope;

use crate::editor::TextEditor;

use super::widget::{
    BorderStyle, CursorPosition, CursorPositionByte, ProcessEvent, ShouldExit, WidgetID,
};

pub struct LineNumber {
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

impl LineNumber {
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
        let buffer = Rope::from_str(&text);
        Box::new(Self {
            id: WidgetID::LineNumber,
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

impl Default for LineNumber {
    fn default() -> Self {
        // Return a new Widget with default values here
        Self {
            id: WidgetID::LineNumber,
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

impl ProcessEvent for LineNumber {
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
        WidgetID::LineNumber
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
            if let Some(panel) = editor.get_widget(WidgetID::Panel) {
                let is_relative = true;
                let mut line_number = String::new();
                for j in panel.get_scroll_lines()..(panel.get_scroll_lines() + panel.get_height()) {
                    // Padded to the right
                    if is_relative {
                        let pos = panel.get_cursor_view();
                        let v: i32 = (j as i32) - ((pos.1 + panel.get_scroll_lines()) as i32);
                        let value: String = if v == 0 {
                            (j + 1).to_string()
                        } else {
                            (v.abs()).to_string()
                        };
                        line_number.push_str(&" ".repeat(self.width - value.len()));
                        line_number.push_str(&value);
                        line_number.push('\n');
                    } else {
                        let value: String = (j + 1).to_string();
                        line_number.push_str(&" ".repeat(self.width - value.len()));
                        line_number.push_str(&value);
                        line_number.push('\n');
                    }
                }
                self.set_buffer(ropey::Rope::from_str(&line_number));
            }
        }
        None
    }
}
