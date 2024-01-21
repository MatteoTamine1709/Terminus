use crossterm::event::Event;
use crossterm::style::Color;
use ropey::Rope;

use crate::{editor::TextEditor, widget::widget::WidgetType};

use super::widget::{
    BorderStyle, ColorText, CursorPosition, CursorPositionByte, ProcessEvent, ShouldExit,
};

pub struct Panel {
    pub typ: WidgetType,
    pub id: usize,
    /// the text
    pub buffer: Rope,
    pub colors: Vec<Vec<ColorText>>,

    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,

    /// the color
    pub default_fg: Color,
    pub default_bg: Color,

    pub focused: bool,
    pub targetable: bool,

    /// scolled lines
    pub scroll_lines: usize,

    /// scrolled columns
    pub scroll_columns: usize,

    pub boder_style: BorderStyle,
    pub text_position: CursorPositionByte,

    pub z_index: usize,
}

impl Panel {
    pub fn new(
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
    ) -> Box<Self> {
        let buffer = Rope::from_str(&text);
        Box::new(Self {
            typ: WidgetType::Panel,
            buffer,
            x,
            y,
            width,
            height,
            default_fg: fg,
            default_bg: bg,
            focused,
            targetable,
            boder_style,
            ..Default::default()
        })
    }
}

impl Default for Panel {
    fn default() -> Self {
        // Return a new Widget with default values here
        Self {
            typ: WidgetType::Panel,
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
            focused: false,
            targetable: false,
            boder_style: BorderStyle::None,
            text_position: 0,
            z_index: 0,
            colors: vec![],
        }
    }
}

impl ProcessEvent for Panel {
    fn get_border_style(&self) -> BorderStyle {
        self.boder_style
    }
    fn get_buffer(&self) -> &Rope {
        &self.buffer
    }
    fn get_height(&self) -> usize {
        self.height
    }
    fn get_width(&self) -> usize {
        self.width
    }
    fn get_x(&self) -> usize {
        self.x
    }
    fn get_y(&self) -> usize {
        self.y
    }
    fn get_scroll_lines(&self) -> usize {
        self.scroll_lines
    }
    fn get_scroll_columns(&self) -> usize {
        self.scroll_columns
    }
    fn get_default_fg(&self) -> Color {
        self.default_fg
    }
    fn get_default_bg(&self) -> Color {
        self.default_bg
    }
    fn get_text_position(&self) -> CursorPositionByte {
        self.text_position
    }
    fn get_focused(&self) -> bool {
        self.focused
    }
    fn get_targetable(&self) -> bool {
        self.targetable
    }
    fn get_type(&self) -> WidgetType {
        WidgetType::Panel
    }
    fn get_id(&self) -> usize {
        self.id
    }
    fn get_z_idx(&self) -> usize {
        self.z_index
    }

    fn get_colors(&self) -> Vec<Vec<ColorText>> {
        self.colors.clone()
    }
    fn get_colors_mut(&mut self) -> &mut Vec<Vec<ColorText>> {
        &mut self.colors
    }
    fn set_colors(&mut self, colors: Vec<Vec<ColorText>>) {
        self.colors = colors;
    }

    fn set_border_style(&mut self, border_style: BorderStyle) {
        self.boder_style = border_style;
    }
    fn set_buffer(&mut self, buffer: Rope) {
        self.buffer = buffer;
    }
    fn set_height(&mut self, height: usize) {
        self.height = height;
    }
    fn set_width(&mut self, width: usize) {
        self.width = width;
    }
    fn set_x(&mut self, x: usize) {
        self.x = x;
    }
    fn set_y(&mut self, y: usize) {
        self.y = y;
    }
    fn set_scroll_lines(&mut self, scroll_lines: usize) {
        self.scroll_lines = scroll_lines;
    }
    fn set_scroll_columns(&mut self, scroll_columns: usize) {
        self.scroll_columns = scroll_columns;
    }
    fn set_default_fg(&mut self, default_fg: Color) {
        self.default_fg = default_fg;
    }
    fn set_default_bg(&mut self, default_bg: Color) {
        self.default_bg = default_bg;
    }
    fn set_text_position(&mut self, text_position: CursorPositionByte) {
        self.text_position = text_position;
    }
    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
    fn set_targetable(&mut self, targetable: bool) {
        self.targetable = targetable;
    }
    fn set_type(&mut self, id: WidgetType) {
        self.typ = id;
    }
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    fn set_z_idx(&mut self, z_index: usize) {
        self.z_index = z_index;
    }

    fn event(
        &mut self,
        editor: &mut TextEditor,
        event: &Event,
    ) -> Option<(CursorPosition, ShouldExit)> {
        if self.focused {
            if let Event::Key(key_event) = event {
                match key_event.modifiers {
                    crossterm::event::KeyModifiers::SHIFT => match key_event.code {
                        crossterm::event::KeyCode::Char(c) => {
                            self.buffer.insert_char(self.text_position, c);
                            self.text_position += 1;
                            editor.written = true;
                            return Some((self.update_cursor_position_and_view(), false));
                        }
                        _ => {}
                    },
                    crossterm::event::KeyModifiers::NONE => match key_event.code {
                        crossterm::event::KeyCode::Tab => {
                            // let targetable_widgets: usize = editor
                            //     .widgets
                            //     .iter()
                            //     .filter(|w| w.get_targetable())
                            //     .map(|w| w.get_id())
                            //     .count();
                            // if targetable_widgets == 0 {
                            //     return Some((self.update_cursor_position_and_view(), true));
                            // }
                            // editor.widgets[0].set_focused(true);
                            // return Some((
                            //     editor.widgets[0].update_cursor_position_and_view(),
                            //     false,
                            // ));
                        }
                        crossterm::event::KeyCode::Char(c) => {
                            self.buffer.insert_char(self.text_position, c);
                            self.text_position += 1;
                            editor.written = true;
                            return Some((self.update_cursor_position_and_view(), false));
                        }
                        crossterm::event::KeyCode::Enter => {
                            self.buffer.insert_char(self.text_position, '\n');
                            self.text_position += 1;
                            editor.written = true;
                            return Some((self.update_cursor_position_and_view(), false));
                        }
                        crossterm::event::KeyCode::Backspace => {
                            if self.text_position > 0 {
                                self.buffer
                                    .remove(self.text_position - 1..self.text_position);
                                self.text_position -= 1;
                                editor.written = true;
                            }
                            return Some((self.update_cursor_position_and_view(), false));
                        }
                        _ => {}
                    },
                    crossterm::event::KeyModifiers::CONTROL => match key_event.code {
                        crossterm::event::KeyCode::Char(c) => match c {
                            'e' => {
                                if let Some(command_line) =
                                    editor.get_widget_mut(WidgetType::CommandLine)
                                {
                                    command_line.set_focused(true);
                                    // command_line.set_scroll_columns(0);
                                    // command_line.set_buffer(Rope::from_str(""));
                                    // command_line.set_text_position(0);
                                    let cursor_position =
                                        command_line.update_cursor_position_and_view();
                                    editor.focused_widget_id = command_line.get_id();
                                    return Some((cursor_position, false));
                                }
                            }
                            's' => {
                                let saved_path = editor.save_path.clone();
                                if let Some(command_line) =
                                    editor.get_widget_mut(WidgetType::CommandLine)
                                {
                                    command_line.set_focused(true);
                                    command_line.set_scroll_columns(0);
                                    let save = format!(":save {}", saved_path.to_str().unwrap());
                                    command_line.set_buffer(Rope::from_str(save.as_str()));
                                    command_line.set_text_position(save.len());
                                    let cursor_position =
                                        command_line.update_cursor_position_and_view();
                                    editor.focused_widget_id = command_line.get_id();
                                    return Some((cursor_position, false));
                                }
                            }
                            'f' => {
                                if let Some(command_line) =
                                    editor.get_widget_mut(WidgetType::CommandLine)
                                {
                                    command_line.set_focused(true);
                                    command_line.set_scroll_columns(0);
                                    let find = ":find ";
                                    command_line.set_buffer(Rope::from_str(find));
                                    command_line.set_text_position(find.len());
                                    let cursor_position =
                                        command_line.update_cursor_position_and_view();
                                    editor.focused_widget_id = command_line.get_id();
                                    return Some((cursor_position, false));
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                    _ => {}
                }
            }
            if let Event::Mouse(mouse_event) = event {
                if mouse_event.kind == crossterm::event::MouseEventKind::ScrollDown {
                    if self.scroll_lines < self.buffer.len_lines() {
                        self.scroll_lines += 1;
                        // return Some((self.update_cursor_position_and_view(), false));
                    }
                }
                if mouse_event.kind == crossterm::event::MouseEventKind::ScrollUp {
                    if self.scroll_lines > 0 {
                        self.scroll_lines -= 1;
                        // return Some((self.update_cursor_position_and_view(), false));
                    }
                }
            }
        }
        None
    }
}
