use std::io::stdout;

use crossterm::{
    cursor::{self},
    event::Event,
    queue,
    style::{self, Color, Stylize},
};

use ropey::{Rope, RopeSlice};

use super::super::editor::TextEditor;

pub type ShouldExit = bool;
pub type CursorPosition = (i32, i32);
pub type CursorPositionByte = usize;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum WidgetType {
    None,
    Popup,
    Panel,
    StatusBar,
    LineNumber,
    CommandLine,
    _WidgetCount,
}

// Display trait
impl std::fmt::Display for WidgetType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            WidgetType::None => write!(f, "None"),
            WidgetType::Popup => write!(f, "Popup"),
            WidgetType::Panel => write!(f, "Panel"),
            WidgetType::StatusBar => write!(f, "StatusBar"),
            WidgetType::LineNumber => write!(f, "LineNumber"),
            WidgetType::CommandLine => write!(f, "CommandLine"),
            WidgetType::_WidgetCount => write!(f, "WidgetCount"),
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum BorderStyle {
    None,
    Solid,
    Dashed,
}

// https://doc.rust-lang.org/book/ch10-02-traits.html

pub trait ProcessEvent {
    fn event(
        &mut self,
        editor: &mut TextEditor,
        event: &Event,
    ) -> Option<(CursorPosition, ShouldExit)>;

    fn get_border_style(&self) -> BorderStyle;
    fn get_x(&self) -> usize;
    fn get_y(&self) -> usize;
    fn get_width(&self) -> usize;
    fn get_height(&self) -> usize;
    fn get_scroll_lines(&self) -> usize;
    fn get_scroll_columns(&self) -> usize;
    fn get_default_fg(&self) -> Color;
    fn get_default_bg(&self) -> Color;
    fn get_buffer(&self) -> &Rope;
    fn get_text_position(&self) -> CursorPositionByte;
    fn get_focused(&self) -> bool;
    fn get_targetable(&self) -> bool;
    fn get_type(&self) -> WidgetType;
    fn get_id(&self) -> usize;
    fn get_z_idx(&self) -> usize;

    fn set_border_style(&mut self, border_style: BorderStyle);
    fn set_x(&mut self, x: usize);
    fn set_y(&mut self, y: usize);
    fn set_width(&mut self, width: usize);
    fn set_height(&mut self, height: usize);
    fn set_scroll_lines(&mut self, scroll_lines: usize);
    fn set_scroll_columns(&mut self, scroll_columns: usize);
    fn set_default_fg(&mut self, default_fg: Color);
    fn set_default_bg(&mut self, default_bg: Color);
    fn set_buffer(&mut self, buffer: Rope);
    fn set_text_position(&mut self, text_position: CursorPositionByte);
    fn set_focused(&mut self, focused: bool);
    fn set_targetable(&mut self, targetable: bool);
    fn set_type(&mut self, id: WidgetType);
    fn set_id(&mut self, id: usize);
    fn set_z_idx(&mut self, z_idx: usize);

    fn get_offset(&self) -> usize {
        match self.get_border_style() {
            BorderStyle::None => 0,
            BorderStyle::Solid | BorderStyle::Dashed => 1,
        }
    }

    fn render_box(&self, chars: [&str; 6]) {
        let mut stdout = stdout();
        queue!(
            stdout,
            cursor::MoveTo(self.get_x() as u16, self.get_y() as u16)
        )
        .unwrap();
        queue!(stdout, style::Print(chars[0])).unwrap();
        queue!(
            stdout,
            cursor::MoveTo(
                self.get_x() as u16 + self.get_width() as u16,
                self.get_y() as u16
            )
        )
        .unwrap();
        queue!(stdout, style::Print(chars[1])).unwrap();
        queue!(
            stdout,
            cursor::MoveTo(
                self.get_x() as u16,
                self.get_y() as u16 + self.get_height() as u16
            )
        )
        .unwrap();
        queue!(stdout, style::Print(chars[2])).unwrap();
        queue!(
            stdout,
            cursor::MoveTo(
                self.get_x() as u16 + self.get_width() as u16,
                self.get_y() as u16 + self.get_height() as u16
            )
        )
        .unwrap();
        queue!(stdout, style::Print(chars[3])).unwrap();
        for i in 1..self.get_width() {
            queue!(
                stdout,
                cursor::MoveTo(self.get_x() as u16 + i as u16, self.get_y() as u16)
            )
            .unwrap();
            queue!(stdout, style::Print(chars[4])).unwrap();
            queue!(
                stdout,
                cursor::MoveTo(
                    self.get_x() as u16 + i as u16,
                    self.get_y() as u16 + self.get_height() as u16
                )
            )
            .unwrap();
            queue!(stdout, style::Print(chars[4])).unwrap();
        }
        for i in 1..self.get_height() {
            queue!(
                stdout,
                cursor::MoveTo(self.get_x() as u16, self.get_y() as u16 + i as u16)
            )
            .unwrap();
            queue!(stdout, style::Print(chars[5])).unwrap();
            queue!(
                stdout,
                cursor::MoveTo(
                    self.get_x() as u16 + self.get_width() as u16,
                    self.get_y() as u16 + i as u16
                )
            )
            .unwrap();
            queue!(stdout, style::Print(chars[5])).unwrap();
        }
    }

    fn render(&self) {
        let mut stdout = stdout();
        queue!(
            stdout,
            cursor::MoveTo(self.get_x() as u16, self.get_y() as u16)
        )
        .unwrap();
        let offset = self.get_offset();
        let height = self.get_height() - offset;
        let width = self.get_width() - offset;
        let x = self.get_x() + offset;
        let mut y = self.get_y() + offset;

        let buffer = self.get_buffer();
        let lines: Vec<RopeSlice> = buffer
            .lines()
            .skip(self.get_scroll_lines())
            .take(height)
            .collect();
        for line in lines.iter() {
            let mut line_to_display: String = line
                .chars()
                .skip(self.get_scroll_columns())
                .take(width)
                .collect();
            if line_to_display.ends_with('\n') {
                line_to_display.pop();
            }
            if line_to_display.len() + x < width {
                line_to_display.push_str(&" ".repeat(width - line_to_display.len() - x));
            }

            queue!(stdout, cursor::MoveTo(x as u16, y as u16)).unwrap();
            queue!(
                stdout,
                style::PrintStyledContent(
                    line_to_display
                        .with(self.get_default_fg())
                        .on(self.get_default_bg())
                )
            )
            .unwrap();
            y += 1;
        }
        for i in 0..(height - lines.len()) {
            queue!(stdout, cursor::MoveTo(x as u16, (y + i) as u16)).unwrap();
            queue!(
                stdout,
                style::PrintStyledContent(
                    " ".repeat(width)
                        .with(self.get_default_fg())
                        .on(self.get_default_bg())
                )
            )
            .unwrap();
        }

        match self.get_border_style() {
            BorderStyle::None => {}
            BorderStyle::Solid => {
                self.render_box(["┌", "┐", "└", "┘", "─", "│"]);
            }
            BorderStyle::Dashed => {
                self.render_box(["┌", "┐", "└", "┘", "┄", "┆"]);
            }
        }
    }

    fn update_cursor_position_and_view(&mut self) -> CursorPosition {
        let offset = self.get_offset();
        let mut y = self.get_buffer().byte_to_line(self.get_text_position());
        let mut x = self.get_text_position() - self.get_buffer().line_to_byte(y);

        if y < self.get_scroll_lines() {
            self.set_scroll_lines(y);
        }
        if y > self.get_scroll_lines() + (self.get_height() - 1) - offset - offset {
            self.set_scroll_lines(y - (self.get_height() - 1 - offset));
        }
        if x < self.get_scroll_columns() {
            self.set_scroll_columns(x);
        }
        if x > self.get_scroll_columns() + self.get_width() - offset - offset {
            self.set_scroll_columns((x + offset) - self.get_width());
        }
        x += offset + self.get_x();
        y += offset + self.get_y();
        y -= self.get_scroll_lines();
        x -= self.get_scroll_columns();
        (x as i32, y as i32)
    }

    fn get_cursor_view(&self) -> CursorPosition {
        let offset = self.get_offset();
        let y = self.get_buffer().byte_to_line(self.get_text_position());
        let x = self.get_text_position() - self.get_buffer().line_to_byte(y);
        let scroll_lines = self.get_scroll_lines();
        let scroll_columns = self.get_scroll_columns();

        // if y < scroll_lines {
        //     scroll_lines = y;
        // }
        // if y > scroll_lines + (self.get_height() - 1) - offset - offset {
        //     scroll_lines = y - (self.get_height() - 1 - offset);
        // }
        // if x < scroll_columns {
        //     scroll_columns = x;
        // }
        // if x > scroll_columns + self.get_width() - offset - offset {
        //     scroll_columns = (x + offset) - self.get_width();
        // }

        let y: i32 = y as i32 + offset as i32 + self.get_y() as i32 - scroll_lines as i32;
        let x: i32 = x as i32 + offset as i32 + self.get_x() as i32 - scroll_columns as i32;
        (x, y)
    }

    fn is_cursor_visible(&self) -> bool {
        let y = self.get_buffer().byte_to_line(self.get_text_position());
        let x = self.get_text_position() - self.get_buffer().line_to_byte(y);
        let scroll_lines = self.get_scroll_lines();
        let scroll_columns = self.get_scroll_columns();

        if y < scroll_lines {
            return false;
        }
        if y > scroll_lines + (self.get_height() - 1) {
            return false;
        }
        if x < scroll_columns {
            return false;
        }
        if x > scroll_columns + self.get_width() {
            return false;
        }
        true
    }
}
pub struct Widget {
    pub typ: WidgetType,
    pub id: usize,
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

    pub z_idx: usize,
}

impl Widget {
    pub fn _new() -> Box<Self> {
        Box::new(Self {
            typ: WidgetType::None,
            id: 0,
            buffer: Rope::from_str(""),
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            default_fg: Color::Reset,
            default_bg: Color::Reset,
            focused: false,
            targetable: false,
            scroll_lines: 0,
            scroll_columns: 0,
            boder_style: BorderStyle::None,
            text_position: 0,
            z_idx: 0,
        })
    }
}

impl ProcessEvent for Widget {
    fn event(
        &mut self,
        _editor: &mut TextEditor,
        _event: &Event,
    ) -> Option<(CursorPosition, ShouldExit)> {
        Some((self.update_cursor_position_and_view(), true))
    }

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
        self.typ
    }
    fn get_id(&self) -> usize {
        self.id
    }
    fn get_z_idx(&self) -> usize {
        self.z_idx
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
        self.z_idx = z_idx;
    }
}
