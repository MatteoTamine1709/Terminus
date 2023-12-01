use std::path::PathBuf;

use crossterm::{cursor, event::Event, execute, queue, terminal};
use std::io::{stdout, Write};

use super::widget::widget::{CursorPosition, Widget};

pub struct TextEditor {
    /// save_path
    save_path: PathBuf,

    pub running: bool,
    /// widgets
    widgets: Vec<Widget>,
}

impl TextEditor {
    pub fn new(save_path: &PathBuf) -> Self {
        Self {
            running: true,
            save_path: save_path.clone(),
            widgets: Vec::new(),
        }
    }

    pub fn add_widget(&mut self, widget: Widget) {
        self.widgets.push(widget);
        let pos = self
            .widgets
            .last_mut()
            .unwrap()
            .update_cursor_position_and_view();
        execute!(stdout(), cursor::MoveTo(pos.0 as u16, pos.1 as u16)).unwrap();
        stdout().flush().unwrap();
    }

    pub fn render(&mut self, cursor_position: CursorPosition) {
        queue!(std::io::stdout(), terminal::Clear(terminal::ClearType::All)).unwrap();
        for widget in &mut self.widgets {
            widget.render();
        }
        queue!(
            stdout(),
            cursor::MoveTo(cursor_position.0 as u16, cursor_position.1 as u16)
        )
        .unwrap();

        // Flush the terminal
        stdout().flush().unwrap();
    }

    pub fn event(&mut self, event: Event) {
        let len = self.widgets.len();
        let mut cursor_position = (0, 0);
        for i in 0..len {
            if self.widgets[i].focused {
                let mut widget = std::mem::take(&mut self.widgets[i]);
                let (cursor_position_local, _) = (widget.event)(&mut widget, self, event.clone());
                self.widgets[i] = widget;
                cursor_position = cursor_position_local;
            }
        }
        self.render(cursor_position);
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
