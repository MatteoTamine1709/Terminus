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
pub type CursorPosition = (usize, usize);
pub type CursorPositionByte = usize;

pub enum WidgetID {
    Main,
    Status,
    LineNumber,
    CommandLine,
}

pub enum BorderStyle {
    None,
    Solid,
    Dashed,
}

/// a widget
pub struct Widget {
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
    pub colors: Vec<(Option<Color>, Option<Color>)>,

    pub focused: bool,
    pub targetable: bool,

    pub event: fn(&mut Widget, &mut TextEditor, Event) -> (CursorPosition, ShouldExit),
    pub processed: bool,

    /// scolled lines
    pub scroll_lines: usize,

    /// scrolled columns
    pub scroll_columns: usize,

    pub boder_style: BorderStyle,
    pub text_position: CursorPositionByte,

    pub widgets: Vec<Widget>,
}

impl Widget {
    pub fn new(
        id: usize,
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
        event: fn(&mut Widget, &mut TextEditor, Event) -> (CursorPosition, ShouldExit),
    ) -> Self {
        let buffer = Rope::from_str(&text);
        let mut colors = Vec::new();
        colors.resize(buffer.len_chars(), (None, None));
        Self {
            id,
            buffer,
            x,
            y,
            width,
            height,
            default_fg: fg,
            default_bg: bg,
            colors,
            focused,
            targetable,
            boder_style,
            event,
            ..Default::default()
        }
    }

    pub fn add_widget(&mut self, widget: Widget) {
        self.widgets.push(widget);
    }

    pub fn get_offset(&self) -> usize {
        match self.boder_style {
            BorderStyle::None => 0,
            BorderStyle::Solid | BorderStyle::Dashed => 1,
        }
    }

    fn render_box(&self, chars: [&str; 6]) {
        let mut stdout = stdout();
        queue!(stdout, cursor::MoveTo(self.x as u16, self.y as u16)).unwrap();
        queue!(stdout, style::Print(chars[0])).unwrap();
        queue!(
            stdout,
            cursor::MoveTo(self.x as u16 + self.width as u16, self.y as u16)
        )
        .unwrap();
        queue!(stdout, style::Print(chars[1])).unwrap();
        queue!(
            stdout,
            cursor::MoveTo(self.x as u16, self.y as u16 + self.height as u16)
        )
        .unwrap();
        queue!(stdout, style::Print(chars[2])).unwrap();
        queue!(
            stdout,
            cursor::MoveTo(
                self.x as u16 + self.width as u16,
                self.y as u16 + self.height as u16
            )
        )
        .unwrap();
        queue!(stdout, style::Print(chars[3])).unwrap();
        for i in 1..self.width {
            queue!(
                stdout,
                cursor::MoveTo(self.x as u16 + i as u16, self.y as u16)
            )
            .unwrap();
            queue!(stdout, style::Print(chars[4])).unwrap();
            queue!(
                stdout,
                cursor::MoveTo(self.x as u16 + i as u16, self.y as u16 + self.height as u16)
            )
            .unwrap();
            queue!(stdout, style::Print(chars[4])).unwrap();
        }
        for i in 1..self.height {
            queue!(
                stdout,
                cursor::MoveTo(self.x as u16, self.y as u16 + i as u16)
            )
            .unwrap();
            queue!(stdout, style::Print(chars[5])).unwrap();
            queue!(
                stdout,
                cursor::MoveTo(self.x as u16 + self.width as u16, self.y as u16 + i as u16)
            )
            .unwrap();
            queue!(stdout, style::Print(chars[5])).unwrap();
        }
    }

    pub fn render(&self) {
        let mut stdout = stdout();
        queue!(stdout, cursor::MoveTo(self.x as u16, self.y as u16)).unwrap();
        let offset = self.get_offset();
        let height = self.height - offset;
        let width = self.width - offset;
        let x = self.x + offset;
        let mut y = self.y + offset;

        let lines: Vec<RopeSlice> = self
            .buffer
            .lines()
            .skip(self.scroll_lines)
            .take(height)
            .collect();
        for line in lines.iter() {
            let mut line_to_display: String =
                line.chars().skip(self.scroll_columns).take(width).collect();
            if line_to_display.ends_with('\n') {
                line_to_display.pop();
            }
            if line_to_display.len() < width {
                line_to_display.push_str(&" ".repeat(width - line_to_display.len()));
            }
            queue!(stdout, cursor::MoveTo(x as u16, y as u16)).unwrap();
            queue!(
                stdout,
                style::PrintStyledContent(
                    line_to_display.with(self.default_fg).on(self.default_bg)
                )
            )
            .unwrap();
            y += 1;
        }

        for child in self.widgets.iter() {
            child.render();
        }

        match self.boder_style {
            BorderStyle::None => {}
            BorderStyle::Solid => {
                self.render_box(["┌", "┐", "└", "┘", "─", "│"]);
            }
            BorderStyle::Dashed => {
                self.render_box(["┌", "┐", "└", "┘", "┄", "┆"]);
            }
        }
    }

    pub fn update_cursor_position_and_view(&mut self) -> CursorPosition {
        let offset = self.get_offset();
        let mut y = self.buffer.byte_to_line(self.text_position);
        let mut x = self.text_position - self.buffer.line_to_byte(y);

        if y < self.scroll_lines {
            self.scroll_lines = y;
        }
        if y > self.scroll_lines + (self.height - 1) - offset - offset {
            self.scroll_lines = y - (self.height - 1 - offset);
        }
        if x < self.scroll_columns {
            self.scroll_columns = x;
        }
        if x > self.scroll_columns + self.width - offset - offset {
            self.scroll_columns = (x + offset) - self.width;
        }
        y -= self.scroll_lines;
        x -= self.scroll_columns;
        self.text_position =
            self.buffer.line_to_byte(y + self.scroll_lines) + x + self.scroll_columns;
        (x + offset + self.x, y + offset + self.y)
    }
}

impl Default for Widget {
    fn default() -> Self {
        // Return a new Widget with default values here
        Self {
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
            colors: Vec::new(),
            focused: false,
            event: |_, _, _| ((0, 0), false),
            processed: false,
            targetable: false,
            boder_style: BorderStyle::None,
            text_position: 0,
            widgets: Vec::new(),
        }
    }
}
