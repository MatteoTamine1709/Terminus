use crossterm::{event::Event, style::Color};
use ropey::Rope;

use crate::editor::TextEditor;

use super::widget::{
    BorderStyle, ColorText, CursorPosition, CursorPositionByte, ProcessEvent, ShouldExit,
    WidgetType,
};

pub struct LineNumber {
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

impl LineNumber {
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
            typ: WidgetType::LineNumber,
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

impl Default for LineNumber {
    fn default() -> Self {
        // Return a new Widget with default values here
        Self {
            typ: WidgetType::LineNumber,
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
            colors: Vec::new(),
        }
    }
}

impl ProcessEvent for LineNumber {
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
        WidgetType::LineNumber
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
    fn set_z_idx(&mut self, z_idx: usize) {
        self.z_index = z_idx;
    }

    fn event(
        &mut self,
        editor: &mut TextEditor,
        _event: &Event,
    ) -> Option<(CursorPosition, ShouldExit)> {
        {
            if let Some(panel) = editor.get_widget(WidgetType::Panel) {
                let is_relative = true;
                let mut line_number = String::new();
                let max_lines = panel.get_buffer().len_lines();
                let max_v = if max_lines < panel.get_height() {
                    max_lines
                } else {
                    panel.get_height()
                };
                for j in panel.get_scroll_lines()..(panel.get_scroll_lines() + max_v) {
                    // Padded to the right
                    if is_relative {
                        let pos = panel.get_cursor_view();
                        let v: i32 = (j as i32) - (pos.1 + panel.get_scroll_lines() as i32);
                        let value: String = if v == 0 {
                            (j + 1).to_string()
                        } else {
                            (v.abs()).to_string()
                        };
                        line_number.push_str(&" ".repeat(self.width - value.len()));
                        line_number.push_str(&value);
                        line_number.push('\n');
                    } else {
                        let value: String = (j + 1).to_string();
                        line_number.push_str(&" ".repeat(self.width - value.len()));
                        line_number.push_str(&value);
                        line_number.push('\n');
                    }
                }
                self.set_buffer(ropey::Rope::from_str(&line_number));
            }
        }
        None
    }
}
