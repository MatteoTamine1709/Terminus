use crossterm::event::{Event, KeyModifiers};

use crate::{
    editor::TextEditor,
    widget::widget::{CursorPosition, ShouldExit, Widget},
};

pub fn process_event(
    editor: &mut TextEditor,
    widget: &mut Widget,
    event: Event,
) -> (CursorPosition, ShouldExit) {
    match event {
        Event::Key(key) => {
            if key.modifiers == KeyModifiers::CONTROL {
                match key.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Char('c') => {
                        editor.running = false;
                        return ((0, 0), true);
                    }
                    crossterm::event::KeyCode::Right => {
                        while widget.text_position < widget.buffer.len_chars()
                            && widget.buffer.char(widget.text_position).is_whitespace()
                        {
                            widget.text_position += 1;
                        }
                        let mut has_punc = false;
                        while widget.text_position < widget.buffer.len_chars()
                            && widget
                                .buffer
                                .char(widget.text_position)
                                .is_ascii_punctuation()
                        {
                            has_punc = true;
                            widget.text_position += 1;
                        }
                        while !has_punc
                            && widget.text_position < widget.buffer.len_chars()
                            && widget.buffer.char(widget.text_position).is_alphanumeric()
                        {
                            widget.text_position += 1;
                        }
                    }
                    crossterm::event::KeyCode::Left => {
                        while widget.text_position > 0
                            && widget.buffer.char(widget.text_position - 1).is_whitespace()
                        {
                            widget.text_position -= 1;
                        }
                        let mut has_punc = false;
                        while widget.text_position > 0
                            && widget
                                .buffer
                                .char(widget.text_position - 1)
                                .is_ascii_punctuation()
                        {
                            has_punc = true;
                            widget.text_position -= 1;
                        }
                        while !has_punc
                            && widget.text_position > 0
                            && widget
                                .buffer
                                .char(widget.text_position - 1)
                                .is_alphanumeric()
                        {
                            widget.text_position -= 1;
                        }
                    }
                    _ => {}
                }
            }
            if key.modifiers == KeyModifiers::NONE {
                match key.code {
                    crossterm::event::KeyCode::Right => {
                        if widget.text_position < widget.buffer.len_chars() {
                            widget.text_position += 1;
                        }
                    }
                    crossterm::event::KeyCode::Left => {
                        if widget.text_position > 0 {
                            widget.text_position -= 1;
                        }
                    }
                    crossterm::event::KeyCode::Up => {
                        let line = widget.buffer.byte_to_line(widget.text_position);
                        if line > 0 {
                            let line_start = widget.buffer.line_to_byte(line);
                            let mut column = widget.text_position - line_start;
                            let len_prev_line = widget.buffer.line(line - 1).len_chars();
                            if column >= len_prev_line {
                                column = len_prev_line - 1;
                            }
                            if len_prev_line > 1 {
                                widget.text_position =
                                    widget.buffer.line_to_byte(line - 1) + column;
                            } else {
                                widget.text_position = widget.buffer.line_to_byte(line - 1);
                            }
                        } else {
                            widget.text_position = 0;
                        }
                    }
                    crossterm::event::KeyCode::PageUp => {
                        for _i in 0..widget.height - 1 {
                            let line = widget.buffer.byte_to_line(widget.text_position);
                            if line > 0 {
                                let line_start = widget.buffer.line_to_byte(line);
                                let mut column = widget.text_position - line_start;
                                let len_prev_line = widget.buffer.line(line - 1).len_chars();
                                if column >= len_prev_line {
                                    column = len_prev_line - 1;
                                }
                                if len_prev_line > 1 {
                                    widget.text_position =
                                        widget.buffer.line_to_byte(line - 1) + column;
                                } else {
                                    widget.text_position = widget.buffer.line_to_byte(line - 1);
                                }
                            } else {
                                widget.text_position = 0;
                            }
                        }
                    }
                    crossterm::event::KeyCode::Down => {
                        let line = widget.buffer.byte_to_line(widget.text_position);
                        if widget.buffer.len_lines() >= 2 && line < widget.buffer.len_lines() - 2 {
                            let line_start = widget.buffer.line_to_byte(line);
                            let mut column = widget.text_position - line_start;
                            let len_next_line = widget.buffer.line(line + 1).len_chars();
                            if column >= len_next_line {
                                column = len_next_line - 1;
                            }
                            if len_next_line > 1 {
                                widget.text_position =
                                    widget.buffer.line_to_byte(line + 1) + column;
                            } else {
                                widget.text_position = widget.buffer.line_to_byte(line + 1);
                            }
                        } else {
                            widget.text_position = widget.buffer.len_chars();
                        }
                    }
                    crossterm::event::KeyCode::PageDown => {
                        for _i in 0..widget.height - 1 {
                            let line = widget.buffer.byte_to_line(widget.text_position);
                            if widget.buffer.len_lines() >= 2
                                && line < widget.buffer.len_lines() - 2
                            {
                                let line_start = widget.buffer.line_to_byte(line);
                                let mut column = widget.text_position - line_start;
                                let len_next_line = widget.buffer.line(line + 1).len_chars();
                                if column >= len_next_line {
                                    column = len_next_line - 1;
                                }
                                if len_next_line > 1 {
                                    widget.text_position =
                                        widget.buffer.line_to_byte(line + 1) + column;
                                } else {
                                    widget.text_position = widget.buffer.line_to_byte(line + 1);
                                }
                            } else {
                                widget.text_position = widget.buffer.len_chars();
                            }
                        }
                    }
                    crossterm::event::KeyCode::Home => {
                        widget.text_position = 0;
                    }

                    crossterm::event::KeyCode::End => {
                        widget.text_position = widget.buffer.len_chars();
                    }
                    _ => return (widget.event)(widget, editor, event),
                }
                return (widget.update_cursor_position_and_view(), false);
            }
        }
        _ => return (widget.event)(widget, editor, event),
    }
    (widget.event)(widget, editor, event);
    return (widget.update_cursor_position_and_view(), false);
}
