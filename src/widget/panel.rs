use crossterm::event::Event;

use crate::{editor::TextEditor, event::process_event, widget::widget::WidgetID};

use super::widget::Widget;

type ShouldExit = bool;
type CursorPosition = (usize, usize);

pub fn panel_event(
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
                    crossterm::event::KeyCode::Tab => {
                        let targetable_widgets: usize = widget
                            .widgets
                            .iter()
                            .filter(|w| w.targetable)
                            .map(|w| w.id)
                            .sum();
                        if targetable_widgets == 0 {
                            return (widget.update_cursor_position_and_view(), true);
                        }
                        widget.widgets[0].focused = true;
                        return (widget.widgets[0].update_cursor_position_and_view(), false);
                    }
                    crossterm::event::KeyCode::Char(c) => {
                        widget.buffer.insert_char(widget.text_position, c);
                        widget.text_position += 1;
                        update_status_bar(widget);
                    }
                    crossterm::event::KeyCode::Backspace => {
                        if widget.text_position > 0 {
                            widget
                                .buffer
                                .remove(widget.text_position - 1..widget.text_position);
                            widget.text_position -= 1;
                            update_status_bar(widget);
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
        update_line_number(widget);

        return res;
    }
    panic!("Unreachable code")
}

fn update_status_bar(widget: &mut Widget) {
    for i in 0..widget.widgets.len() {
        if widget.widgets[i].id == WidgetID::Status as usize {
            let len = widget.widgets[i].buffer.len_chars();
            let last_char = widget.widgets[i].buffer.char(len - 1);
            if last_char != '*' {
                widget.widgets[i].buffer.insert_char(len, '*');
            }
            break;
        }
    }
}

fn update_line_number(widget: &mut Widget) {
    for i in 0..widget.widgets.len() {
        if widget.widgets[i].id == WidgetID::LineNumber as usize {
            let mut line_number = String::new();
            for i in widget.scroll_lines..(widget.scroll_lines + widget.height) {
                line_number.push_str(&format!("{}\n", i + 1));
            }
            widget.widgets[i].buffer = ropey::Rope::from_str(&line_number);
            break;
        }
    }
}
