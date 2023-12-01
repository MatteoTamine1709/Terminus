use crossterm::event::Event;

use crate::editor::TextEditor;

use super::widget::{CursorPosition, ShouldExit, Widget};

pub fn no_event(
    widget: &mut Widget,
    _editor: &mut TextEditor,
    _event: Event,
) -> (CursorPosition, ShouldExit) {
    (widget.update_cursor_position_and_view(), true)
}
