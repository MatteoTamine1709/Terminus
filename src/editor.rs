use std::path::PathBuf;

use crossterm::{
    cursor,
    event::{Event, KeyModifiers},
    execute, queue, terminal,
};
use std::io::{stdout, Write};

use crate::widget::widget::{ProcessEvent, WidgetType};

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
    pub focused_widget_id: usize,
    old_cursor_position: CursorPosition,
    biggest_id: usize,
}

impl TextEditor {
    pub fn new(save_path: &PathBuf) -> Self {
        Self {
            running: true,
            save_path: save_path.clone(),
            widgets: Vec::new(),
            new_widgets: Vec::new(),
            focused_widget_id: 0,
            old_cursor_position: (0, 0),
            written: false,
            saved: true,
            biggest_id: 0,
        }
    }

    pub fn add_widget(&mut self, widget: Box<dyn ProcessEvent>) -> usize {
        self.widgets.push(widget);
        let idx = self.biggest_id;
        self.biggest_id += 1;
        self.widgets.last_mut().unwrap().set_id(idx);
        eprintln!(
            "add_widget: {}, type {}",
            idx,
            self.widgets.last().unwrap().get_type()
        );
        let pos = self
            .widgets
            .last_mut()
            .unwrap()
            .update_cursor_position_and_view();
        execute!(stdout(), cursor::MoveTo(pos.0 as u16, pos.1 as u16)).unwrap();
        stdout().flush().unwrap();
        return idx;
    }

    pub fn get_widget(&self, id: WidgetType) -> Option<&Box<dyn ProcessEvent>> {
        for widget in &self.widgets {
            if widget.get_type() == id {
                return Some(widget);
            }
        }
        for widget in &self.new_widgets {
            if widget.get_type() == id {
                return Some(widget);
            }
        }

        None
    }

    pub fn get_widget_id(
        &self,
        id: usize,
        expected_type: WidgetType,
    ) -> Option<&Box<dyn ProcessEvent>> {
        for widget in &self.widgets {
            if widget.get_id() == id && widget.get_type() == expected_type {
                return Some(widget);
            }
        }
        for widget in &self.new_widgets {
            if widget.get_id() == id && widget.get_type() == expected_type {
                return Some(widget);
            }
        }

        None
    }

    pub fn get_widget_mut(&mut self, id: WidgetType) -> Option<&mut Box<dyn ProcessEvent>> {
        for widget in &mut self.widgets {
            if widget.get_type() == id {
                return Some(widget);
            }
        }
        for widget in &mut self.new_widgets {
            if widget.get_type() == id {
                return Some(widget);
            }
        }

        None
    }

    pub fn get_widget_id_mut(
        &mut self,
        id: usize,
        expected_type: WidgetType,
    ) -> Option<&mut Box<dyn ProcessEvent>> {
        for widget in &mut self.widgets {
            if widget.get_id() == id && widget.get_type() == expected_type {
                return Some(widget);
            }
        }
        for widget in &mut self.new_widgets {
            if widget.get_id() == id && widget.get_type() == expected_type {
                return Some(widget);
            }
        }

        None
    }

    pub fn remove_widget_id(&mut self, id: usize, expected_type: WidgetType) {
        let mut idx = 0;
        eprintln!("remove_widget_id: {}, type {}", id, expected_type);
        for widget in &self.widgets {
            eprintln!("widget: {}, type {}", widget.get_id(), widget.get_type());
            if widget.get_id() == id && widget.get_type() == expected_type {
                self.widgets.remove(idx);
                return;
            }
            idx += 1;
        }
        idx = 0;
        for widget in &self.new_widgets {
            if widget.get_id() == id && widget.get_type() == expected_type {
                self.new_widgets.remove(idx);
                return;
            }
            idx += 1;
        }
    }

    pub fn render(&mut self, cursor_position: CursorPosition, is_cursor_visible: bool) {
        queue!(std::io::stdout(), terminal::Clear(terminal::ClearType::All)).unwrap();
        for widget in &mut self.widgets {
            widget.render();
        }
        if is_cursor_visible {
            queue!(stdout(), cursor::Show).unwrap();
        } else {
            queue!(stdout(), cursor::Hide).unwrap();
        }
        queue!(
            stdout(),
            cursor::MoveTo(cursor_position.0 as u16, cursor_position.1 as u16)
        )
        .unwrap();

        // Flush the terminal
        stdout().flush().unwrap();
        self.old_cursor_position = cursor_position;
    }

    fn process_uncaught_event(
        &mut self,
        widget: &mut Box<dyn ProcessEvent>,
        event: &Event,
    ) -> Option<CursorPosition> {
        match event {
            Event::Resize(_x, _y) => {
                for widget in &mut self.widgets {
                    // widget.set_width(*x as usize);
                    // widget.set_height(*y as usize);
                    eprintln!("Resizing: {}, {}", widget.get_width(), widget.get_height());
                }
            }
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
        let mut cursor_position: (i32, i32) = (0, 0);
        let mut is_cursor_visible = true;
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
                    } else {
                        cursor_position = self.old_cursor_position;
                    }
                    is_cursor_visible = widget.is_cursor_visible();
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
        // sort widgets by "get_z_idx" and by id
        self.widgets
            .sort_by(|a, b| a.get_z_idx().cmp(&b.get_z_idx()));
        self.widgets.sort_by(|a, b| a.get_id().cmp(&b.get_id()));
        self.render(cursor_position, is_cursor_visible);
    }
}
