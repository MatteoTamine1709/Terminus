use std::path::PathBuf;

use crossterm::{cursor, event::Event, queue, style::Color, terminal};
use ropey::Rope;

use std::io::{stdout, Write};

type ShouldExit = bool;
type CursorPosition = (usize, usize);
type CursorPositionByte = usize;

pub enum BorderStyle {
    None,
    Solid,
    Dashed,
}

/// a widget
pub struct Widget {
    /// the text
    pub buffer: Rope,

    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,

    /// the color
    pub color: Color,
    pub background_color: Color,

    pub focused: bool,

    pub render: fn(&mut Widget, &mut TextEditor),
    pub event: fn(&mut Widget, &mut TextEditor, Event) -> (CursorPosition, ShouldExit),

    /// scolled lines
    pub scroll_lines: usize,

    /// scrolled columns
    pub scroll_columns: usize,

    pub boder_style: BorderStyle,
    pub cursor: CursorPositionByte,
    pub cursor_position: CursorPosition,

    pub focused_widget: Option<usize>,
    pub widgets: Vec<Widget>,
}

impl Widget {
    pub fn new(
        text: String,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        color: Color,
        background_color: Color,
        focused: bool,
        boder_style: BorderStyle,
        render: fn(&mut Widget, &mut TextEditor),
        event: fn(&mut Widget, &mut TextEditor, Event) -> (CursorPosition, ShouldExit),
    ) -> Self {
        Self {
            buffer: Rope::from_str(&text),
            x,
            y,
            width,
            height,
            color,
            background_color,
            focused,
            boder_style,
            render,
            event,
            ..Default::default()
        }
    }

    pub fn add_widget(&mut self, widget: Widget) {
        self.widgets.push(widget);
    }
}

impl Default for Widget {
    fn default() -> Self {
        // Return a new Widget with default values here
        Self {
            buffer: Rope::from_str(""),
            scroll_lines: 0,
            scroll_columns: 0,
            color: Color::White,
            background_color: Color::Black,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            focused: false,
            render: |_, _| {},
            event: |_, _, _| ((0, 0), false),
            boder_style: BorderStyle::None,
            focused_widget: None,
            cursor: 0,
            cursor_position: (0, 0),
            widgets: Vec::new(),
        }
    }
}

pub struct TextEditor {
    /// save_path
    save_path: PathBuf,

    /// widgets
    widgets: Vec<Widget>,

    /// focused widget
    focused_widget: Option<usize>,
}

impl TextEditor {
    pub fn new(save_path: &PathBuf) -> Self {
        Self {
            save_path: save_path.clone(),
            widgets: Vec::new(),
            focused_widget: None,
        }
    }

    pub fn add_widget(&mut self, widget: Widget) {
        if widget.focused {
            self.focused_widget = Some(self.widgets.len());
        }
        self.widgets.push(widget);
    }

    pub fn render(&mut self, cursor_position: CursorPosition) {
        queue!(std::io::stdout(), terminal::Clear(terminal::ClearType::All)).unwrap();
        for i in 0..self.widgets.len() {
            let mut widget = std::mem::take(&mut self.widgets[i]);
            (widget.render)(&mut widget, self);
            self.widgets[i] = widget;
        }
        queue!(
            stdout(),
            cursor::MoveTo(cursor_position.0 as u16, cursor_position.1 as u16)
        )
        .unwrap();

        // Flush the terminal
        stdout().flush().unwrap();
    }

    pub fn event(&mut self, event: Event) -> ShouldExit {
        if let Some(focused_widget) = self.focused_widget {
            let mut widget = std::mem::take(&mut self.widgets[focused_widget]);
            let (cursor_position, should_exit) = (widget.event)(&mut widget, self, event);
            self.widgets[focused_widget] = widget;
            self.render(cursor_position);
            should_exit
        } else {
            false
        }
    }

    // pub fn get_text(&self) -> RopeSlice {
    //     self.text.slice(..)
    // }

    // /// get the number of lines scrolled
    // pub fn get_lines_scrolled(&self) -> usize {
    //     self.scroll_lines
    // }

    // /// get the number of columns scrolled
    // pub fn get_columns_scrolled(&self) -> usize {
    //     self.scroll_columns
    // }

    // /// convert the contents of the editor to a string
    // pub fn to_string(&self) -> String {
    //     self.text.to_string()
    // }

    // /// get the number of lines
    // pub fn len_lines(&self) -> usize {
    //     self.text.len_lines()
    // }

    // /// get the top row that is visible
    // pub fn get_first_visible_line(&self) -> usize {
    //     self.scroll_lines
    // }

    // /// get the current line the cursor is on
    // pub fn get_current_line(&self) -> usize {
    //     self.text.byte_to_line(self.cursor)
    // }

    // pub fn get_row_and_column(&self) -> (usize, usize) {
    //     // get the line
    //     let line_num = self.text.byte_to_line(self.cursor);

    //     // get the real line
    //     let line = self.text.line(line_num);

    //     // get where it starts
    //     let line_start = self.text.line_to_char(line_num);

    //     // get where we are, in chars
    //     let line_pos = self.text.byte_to_char(self.cursor) - line_start;

    //     // and loop over the line until we are at the right width
    //     let column = line_pos;

    //     (line_num, column)
    // }
}
