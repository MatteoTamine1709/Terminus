use crate::widget::widget::{CursorPositionByte, ProcessEvent, WidgetType};

#[derive(Clone, PartialEq, Debug)]
pub enum ActionType {
    None,
    Insert,
    Delete,
    MoveCursor,
}

#[derive(Clone, Debug)]
pub struct Action {
    pub typ: ActionType,
    pub widget_id: usize,
    pub widget_type: WidgetType,
    pub cursor_position_byte: CursorPositionByte,
    pub text: String,
    pub done: bool,
    pub started: bool,
}

impl Action {
    pub fn new(typ: ActionType, cursor_position_byte: CursorPositionByte, text: String) -> Self {
        Self {
            typ,
            cursor_position_byte,
            text,
            started: true,
            ..Default::default()
        }
    }
}

impl Default for Action {
    fn default() -> Self {
        Self {
            typ: ActionType::None,
            widget_id: 0,
            widget_type: WidgetType::None,
            cursor_position_byte: 0,
            text: String::new(),
            done: false,
            started: false,
        }
    }
}
