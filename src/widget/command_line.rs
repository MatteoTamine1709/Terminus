use crossterm::event::Event;

use crate::{editor::TextEditor, event::process_event};

use super::widget::{CursorPosition, ShouldExit, Widget};

static mut IS_INTERRUPTED: bool = false;

pub fn command_line(
    widget: &mut Widget,
    editor: &mut TextEditor,
    event: Event,
) -> (CursorPosition, ShouldExit) {
    if widget.processed {
        return (widget.update_cursor_position_and_view(), false);
    }
    if widget.focused {
        for i in 0..widget.widgets.len() {
            if widget.widgets[i].focused {
                let mut res = (widget.widgets[i].event)(&mut widget.widgets[i], editor, event);
                if res.1 {
                    widget.widgets[i].focused = false;
                    res.0 = widget.update_cursor_position_and_view();
                }
                res.1 = false;
                return res;
            }
        }
        if let Event::Key(key_event) = event {
            match key_event.modifiers {
                crossterm::event::KeyModifiers::NONE => match key_event.code {
                    crossterm::event::KeyCode::Esc => {
                        widget.focused = false;
                        return (widget.update_cursor_position_and_view(), false);
                    }
                    crossterm::event::KeyCode::Char(c) => {
                        widget.buffer.insert_char(widget.text_position, c);
                        widget.text_position += 1;
                    }
                    crossterm::event::KeyCode::Backspace => {
                        if widget.text_position > 0 {
                            widget
                                .buffer
                                .remove(widget.text_position - 1..widget.text_position);
                            widget.text_position -= 1;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        widget.processed = true;
        let res = process_event(editor, widget, event);
        widget.processed = false;

        return res;
    }
    panic!("Unreachable code")
}

pub fn write_to_command_line(widget: &mut Widget, text: &str) {
    widget.buffer = ropey::Rope::from_str(text);
}
