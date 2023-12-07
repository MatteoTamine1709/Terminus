use crossterm::event::Event;
use crossterm::style::Color;
use once_cell::sync::Lazy;
use ropey::Rope;

use crate::{editor::TextEditor, widget::widget::WidgetID};

use super::widget::{BorderStyle, CursorPositionByte, ProcessEvent};

type ShouldExit = bool;
type CursorPosition = (usize, usize);

use std::time::{Duration, Instant};

static mut CLEAR_COMMAND_LINE_TIMING: Lazy<Instant> = Lazy::new(|| Instant::now());
static mut CLEAR_COMMAND_LINE_DURATION: Lazy<Duration> = Lazy::new(|| Duration::from_secs(5));

pub struct Panel {
    pub id: WidgetID,
    /// the text
    pub buffer: Rope,

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
            id: WidgetID::Panel,
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
            id: WidgetID::Panel,
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
    fn get_id(&self) -> WidgetID {
        WidgetID::Panel
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
    fn set_id(&mut self, id: WidgetID) {
        self.id = id;
    }

    fn event(
        &mut self,
        _editor: &mut TextEditor,
        event: &Event,
    ) -> Option<(CursorPosition, ShouldExit)> {
        if self.focused {
            if let Event::Key(key_event) = event {
                match key_event.modifiers {
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
                            // update_status_bar(self, editor, true, false);
                        }
                        crossterm::event::KeyCode::Enter => {
                            self.buffer.insert_char(self.text_position, '\n');
                            self.text_position += 1;
                            // update_status_bar(self, editor, true, false);
                        }
                        crossterm::event::KeyCode::Backspace => {
                            if self.text_position > 0 {
                                self.buffer
                                    .remove(self.text_position - 1..self.text_position);
                                self.text_position -= 1;
                                // update_status_bar(widget, editor, true, false);
                            }
                        }
                        _ => {}
                    },
                    crossterm::event::KeyModifiers::CONTROL => match key_event.code {
                        crossterm::event::KeyCode::Char(c) => match c {
                            's' => {
                                // for i in 0..editor.widgets.len() {
                                //     if editor.widgets[i].get_id() == WidgetID::CommandLine {
                                //         // write_to_command_line(&mut editor.widgets[i], "Saving...");
                                //         unsafe {
                                //             CLEAR_COMMAND_LINE_TIMING =
                                //                 Lazy::new(|| Instant::now());
                                //             CLEAR_COMMAND_LINE_DURATION =
                                //                 Lazy::new(|| Duration::from_secs(2));
                                //         };
                                //         break;
                                //     }
                                // }
                                // let writer = fs::File::create(&editor.save_path).unwrap();
                                // self.buffer.write_to(writer).unwrap();
                                // update_status_bar(widget, editor, false, true);
                            }
                            'f' => {
                                // for i in 0..editor.widgets.len() {
                                //     if editor.widgets[i].get_id() == WidgetID::CommandLine {
                                //         editor.widgets[i].set_focused(true);
                                //         return Some((
                                //             editor.widgets[i].update_cursor_position_and_view(),
                                //             false,
                                //         ));
                                //     }
                                // }
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                    _ => {}
                }
            }
            // update_line_number(widget, editor);
            // update_status_bar(widget, editor, false, false);

            if unsafe { CLEAR_COMMAND_LINE_TIMING.elapsed() }
                > unsafe { *CLEAR_COMMAND_LINE_DURATION }
            {
                // for i in 0..editor.widgets.len() {
                //     if editor.widgets[i].get_id() == WidgetID::CommandLine {
                //         write_to_command_line(&mut editor.widgets[i], "");
                //     }
                // }
            }
        }
        None
    }
}
