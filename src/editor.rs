use std::path::PathBuf;

use crossterm::{
    cursor,
    event::{Event, KeyModifiers},
    execute, queue, terminal,
};
use std::io::{stdout, Write};

use crate::widget::widget::{ProcessEvent, WidgetID};

use super::widget::widget::CursorPosition;

pub struct TextEditor {
    /// save_path
    pub save_path: PathBuf,
    pub written: bool,
    pub saved: bool,

    pub running: bool,
    /// widgets
    widgets: Vec<Box<dyn ProcessEvent>>,
    new_widgets: Vec<Box<dyn ProcessEvent>>,
    pub focused_widget_id: WidgetID,
    pub old_cursor_position: CursorPosition,
}

impl TextEditor {
    pub fn new(save_path: &PathBuf) -> Self {
        Self {
            running: true,
            save_path: save_path.clone(),
            widgets: Vec::new(),
            new_widgets: Vec::new(),
            focused_widget_id: WidgetID::Panel,
            old_cursor_position: (0, 0),
            written: false,
            saved: true,
        }
    }

    pub fn add_widget(&mut self, widget: Box<dyn ProcessEvent>) {
        self.widgets.push(widget);
        let pos = self
            .widgets
            .last_mut()
            .unwrap()
            .update_cursor_position_and_view();
        execute!(stdout(), cursor::MoveTo(pos.0 as u16, pos.1 as u16)).unwrap();
        stdout().flush().unwrap();
    }

    pub fn get_widget(&self, id: WidgetID) -> Option<&Box<dyn ProcessEvent>> {
        for widget in &self.widgets {
            if widget.get_id() == id {
                return Some(widget);
            }
        }
        for widget in &self.new_widgets {
            if widget.get_id() == id {
                return Some(widget);
            }
        }

        None
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

    fn process_uncaught_event(
        &mut self,
        widget: &mut Box<dyn ProcessEvent>,
        event: &Event,
    ) -> Option<CursorPosition> {
        match event {
            Event::Key(key) => {
                if key.modifiers == KeyModifiers::CONTROL {
                    match key.code {
                        crossterm::event::KeyCode::Char('q')
                        | crossterm::event::KeyCode::Char('c') => {
                            self.running = false;
                            return None;
                        }
                        crossterm::event::KeyCode::Right => {
                            while widget.get_text_position() < widget.get_buffer().len_chars()
                                && widget
                                    .get_buffer()
                                    .char(widget.get_text_position())
                                    .is_whitespace()
                            {
                                widget.set_text_position(widget.get_text_position() + 1);
                            }
                            let mut has_punc = false;
                            while widget.get_text_position() < widget.get_buffer().len_chars()
                                && widget
                                    .get_buffer()
                                    .char(widget.get_text_position())
                                    .is_ascii_punctuation()
                            {
                                has_punc = true;
                                widget.set_text_position(widget.get_text_position() + 1);
                            }
                            while !has_punc
                                && widget.get_text_position() < widget.get_buffer().len_chars()
                                && widget
                                    .get_buffer()
                                    .char(widget.get_text_position())
                                    .is_alphanumeric()
                            {
                                widget.set_text_position(widget.get_text_position() + 1);
                            }

                            return Some(widget.update_cursor_position_and_view());
                        }
                        crossterm::event::KeyCode::Left => {
                            while widget.get_text_position() > 0
                                && widget
                                    .get_buffer()
                                    .char(widget.get_text_position() - 1)
                                    .is_whitespace()
                            {
                                widget.set_text_position(widget.get_text_position() - 1);
                            }
                            let mut has_punc = false;
                            while widget.get_text_position() > 0
                                && widget
                                    .get_buffer()
                                    .char(widget.get_text_position() - 1)
                                    .is_ascii_punctuation()
                            {
                                has_punc = true;
                                widget.set_text_position(widget.get_text_position() - 1);
                            }
                            while !has_punc
                                && widget.get_text_position() > 0
                                && widget
                                    .get_buffer()
                                    .char(widget.get_text_position() - 1)
                                    .is_alphanumeric()
                            {
                                widget.set_text_position(widget.get_text_position() - 1);
                            }

                            return Some(widget.update_cursor_position_and_view());
                        }
                        _ => {}
                    }
                }
                if key.modifiers == KeyModifiers::NONE {
                    match key.code {
                        crossterm::event::KeyCode::Right => {
                            if widget.get_text_position() < widget.get_buffer().len_chars() {
                                widget.set_text_position(widget.get_text_position() + 1);
                            }
                            return Some(widget.update_cursor_position_and_view());
                        }
                        crossterm::event::KeyCode::Left => {
                            if widget.get_text_position() > 0 {
                                widget.set_text_position(widget.get_text_position() - 1);
                            }
                            return Some(widget.update_cursor_position_and_view());
                        }
                        crossterm::event::KeyCode::Up => {
                            let line = widget.get_buffer().byte_to_line(widget.get_text_position());
                            if line > 0 {
                                let line_start = widget.get_buffer().line_to_byte(line);
                                let mut column = widget.get_text_position() - line_start;
                                let len_prev_line = widget.get_buffer().line(line - 1).len_chars();
                                if column >= len_prev_line {
                                    column = len_prev_line - 1;
                                }
                                if len_prev_line > 1 {
                                    widget.set_text_position(
                                        widget.get_buffer().line_to_byte(line - 1) + column,
                                    );
                                } else {
                                    widget.set_text_position(
                                        widget.get_buffer().line_to_byte(line - 1),
                                    );
                                }
                            } else {
                                widget.set_text_position(0);
                            }
                            return Some(widget.update_cursor_position_and_view());
                        }
                        crossterm::event::KeyCode::PageUp => {
                            for _i in 0..widget.get_height() - 1 {
                                let line =
                                    widget.get_buffer().byte_to_line(widget.get_text_position());
                                if line > 0 {
                                    let line_start = widget.get_buffer().line_to_byte(line);
                                    let mut column = widget.get_text_position() - line_start;
                                    let len_prev_line =
                                        widget.get_buffer().line(line - 1).len_chars();
                                    if column >= len_prev_line {
                                        column = len_prev_line - 1;
                                    }
                                    if len_prev_line > 1 {
                                        widget.set_text_position(
                                            widget.get_buffer().line_to_byte(line - 1) + column,
                                        );
                                    } else {
                                        widget.set_text_position(
                                            widget.get_buffer().line_to_byte(line - 1),
                                        );
                                    }
                                } else {
                                    widget.set_text_position(0);
                                }
                            }
                            return Some(widget.update_cursor_position_and_view());
                        }
                        crossterm::event::KeyCode::Down => {
                            let line = widget.get_buffer().byte_to_line(widget.get_text_position());
                            if widget.get_buffer().len_lines() >= 2
                                && line < widget.get_buffer().len_lines() - 2
                            {
                                let line_start = widget.get_buffer().line_to_byte(line);
                                let mut column = widget.get_text_position() - line_start;
                                let len_next_line = widget.get_buffer().line(line + 1).len_chars();
                                if column >= len_next_line {
                                    column = len_next_line - 1;
                                }
                                if len_next_line > 1 {
                                    widget.set_text_position(
                                        widget.get_buffer().line_to_byte(line + 1) + column,
                                    );
                                } else {
                                    widget.set_text_position(
                                        widget.get_buffer().line_to_byte(line + 1),
                                    );
                                }
                            } else {
                                widget.set_text_position(widget.get_buffer().len_chars());
                            }
                            return Some(widget.update_cursor_position_and_view());
                        }
                        crossterm::event::KeyCode::PageDown => {
                            for _i in 0..widget.get_height() - 1 {
                                let line =
                                    widget.get_buffer().byte_to_line(widget.get_text_position());
                                if widget.get_buffer().len_lines() >= 2
                                    && line < widget.get_buffer().len_lines() - 2
                                {
                                    let line_start = widget.get_buffer().line_to_byte(line);
                                    let mut column = widget.get_text_position() - line_start;
                                    let len_next_line =
                                        widget.get_buffer().line(line + 1).len_chars();
                                    if column >= len_next_line {
                                        column = len_next_line - 1;
                                    }
                                    if len_next_line > 1 {
                                        widget.set_text_position(
                                            widget.get_buffer().line_to_byte(line + 1) + column,
                                        );
                                    } else {
                                        widget.set_text_position(
                                            widget.get_buffer().line_to_byte(line + 1),
                                        );
                                    }
                                } else {
                                    widget.set_text_position(widget.get_buffer().len_chars());
                                }
                            }
                            return Some(widget.update_cursor_position_and_view());
                        }
                        crossterm::event::KeyCode::Home => {
                            widget.set_text_position(0);
                            return Some(widget.update_cursor_position_and_view());
                        }

                        crossterm::event::KeyCode::End => {
                            widget.set_text_position(widget.get_buffer().len_chars());
                            return Some(widget.update_cursor_position_and_view());
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        None
    }

    pub fn event(&mut self, event: &Event) {
        let mut cursor_position: (usize, usize) = (0, 0);
        self.new_widgets.clear();
        while let Some(mut widget) = self.widgets.pop() {
            if widget.get_id() == self.focused_widget_id {
                widget.set_focused(true);
                if let Some((pos, should_exit)) = widget.event(self, event) {
                    cursor_position = pos;
                    self.old_cursor_position = cursor_position;
                    if !should_exit {
                        self.new_widgets.push(widget);
                    }
                } else {
                    if let Some(pos) = self.process_uncaught_event(&mut widget, event) {
                        cursor_position = pos;
                        self.old_cursor_position = cursor_position;
                    }
                    cursor_position = self.old_cursor_position;
                    self.new_widgets.push(widget);
                }
            } else {
                widget.set_focused(false);
                widget.event(self, event);
                self.new_widgets.push(widget);
            }
        }
        while let Some(widget) = self.new_widgets.pop() {
            self.widgets.push(widget);
        }
        self.render(cursor_position);
    }
}
