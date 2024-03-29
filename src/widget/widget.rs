use std::io::stdout;

use crossterm::{
    cursor::{self},
    event::Event,
    queue,
    style::{self, Color, ContentStyle, StyledContent, Stylize},
};

use ropey::{Rope, RopeSlice};
use syntect::{
    easy::HighlightLines,
    highlighting::{Style, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
};

use super::super::editor::TextEditor;

pub static mut PS: Option<SyntaxSet> = None;
pub static mut TS: Option<ThemeSet> = None;

pub type ShouldExit = bool;
pub type CursorPosition = (i32, i32);
pub type CursorPositionByte = usize;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ColorTextTag {
    None,
    Cursor,
    Selection,
    Find,
}

#[derive(Clone, Copy, Debug)]
pub struct ColorText {
    pub x: usize,
    pub fg: Color,
    pub bg: Color,
    pub len: usize,
    pub z_index: i16,
    pub tag: ColorTextTag,
}

impl ColorText {
    pub fn new(
        x: usize,
        fg: Color,
        bg: Color,
        len: usize,
        z_index: i16,
        tag: ColorTextTag,
    ) -> Self {
        Self {
            x,
            fg,
            bg,
            len,
            z_index,
            tag,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum WidgetType {
    None,
    Popup,
    Panel,
    StatusBar,
    LineNumber,
    CommandLine,
    _WidgetCount,
}

// Display trait
impl std::fmt::Display for WidgetType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            WidgetType::None => write!(f, "None"),
            WidgetType::Popup => write!(f, "Popup"),
            WidgetType::Panel => write!(f, "Panel"),
            WidgetType::StatusBar => write!(f, "StatusBar"),
            WidgetType::LineNumber => write!(f, "LineNumber"),
            WidgetType::CommandLine => write!(f, "CommandLine"),
            WidgetType::_WidgetCount => write!(f, "WidgetCount"),
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum BorderStyle {
    None,
    Solid,
    Dashed,
}

// https://doc.rust-lang.org/book/ch10-02-traits.html

pub trait ProcessEvent {
    fn event(
        &mut self,
        editor: &mut TextEditor,
        event: &Event,
    ) -> Option<(CursorPosition, ShouldExit)>;

    fn get_border_style(&self) -> BorderStyle;
    fn get_x(&self) -> usize;
    fn get_y(&self) -> usize;
    fn get_width(&self) -> usize;
    fn get_height(&self) -> usize;
    fn get_scroll_lines(&self) -> usize;
    fn get_scroll_columns(&self) -> usize;
    fn get_default_fg(&self) -> Color;
    fn get_default_bg(&self) -> Color;
    fn get_buffer(&self) -> &Rope;
    fn get_text_position(&self) -> CursorPositionByte;
    fn get_focused(&self) -> bool;
    fn get_targetable(&self) -> bool;
    fn get_type(&self) -> WidgetType;
    fn get_id(&self) -> usize;
    fn get_z_idx(&self) -> usize;
    fn get_syntax(&self) -> Option<&SyntaxReference> {
        None
    }
    fn get_theme(&self) -> Option<String> {
        None
    }

    fn get_colors(&self) -> Vec<Vec<ColorText>>;
    fn get_colors_mut(&mut self) -> &mut Vec<Vec<ColorText>>;
    fn push_color(&mut self, y: usize, color: ColorText) {
        let colors = self.get_colors_mut();
        if colors.len() <= y {
            colors.resize(y + 1, Vec::<ColorText>::new());
        }
        colors[y].push(color);
    }
    fn clear_colors(&mut self) {
        self.get_colors_mut().clear();
    }
    fn remove_line_colors(&mut self, y: usize) {
        self.get_colors_mut().remove(y);
    }
    fn remove_color(&mut self, predicate: &dyn Fn(&ColorText) -> bool) {
        for line in self.get_colors_mut() {
            line.retain(|c| !predicate(c));
        }
    }
    fn set_colors(&mut self, colors: Vec<Vec<ColorText>>);

    fn set_border_style(&mut self, border_style: BorderStyle);
    fn set_x(&mut self, x: usize);
    fn set_y(&mut self, y: usize);
    fn set_width(&mut self, width: usize);
    fn set_height(&mut self, height: usize);
    fn set_scroll_lines(&mut self, scroll_lines: usize);
    fn set_scroll_columns(&mut self, scroll_columns: usize);
    fn set_default_fg(&mut self, default_fg: Color);
    fn set_default_bg(&mut self, default_bg: Color);
    fn set_buffer(&mut self, buffer: Rope);
    fn set_text_position(&mut self, text_position: CursorPositionByte);
    fn set_focused(&mut self, focused: bool);
    fn set_targetable(&mut self, targetable: bool);
    fn set_type(&mut self, id: WidgetType);
    fn set_id(&mut self, id: usize);
    fn set_z_idx(&mut self, z_idx: usize);
    fn set_syntax(&mut self, _syntax: Option<&SyntaxReference>) {}
    fn set_theme(&mut self, _theme: Option<String>) {}

    fn get_offset(&self) -> usize {
        match self.get_border_style() {
            BorderStyle::None => 0,
            BorderStyle::Solid | BorderStyle::Dashed => 1,
        }
    }

    fn undo(&mut self) -> Option<CursorPosition> {
        None
    }
    fn redo(&mut self) -> Option<CursorPosition> {
        None
    }

    fn render_box(&self, chars: [&str; 6]) {
        let mut stdout = stdout();
        queue!(
            stdout,
            cursor::MoveTo(self.get_x() as u16, self.get_y() as u16)
        )
        .unwrap();
        queue!(stdout, style::Print(chars[0])).unwrap();
        queue!(
            stdout,
            cursor::MoveTo(
                self.get_x() as u16 + self.get_width() as u16,
                self.get_y() as u16
            )
        )
        .unwrap();
        queue!(stdout, style::Print(chars[1])).unwrap();
        queue!(
            stdout,
            cursor::MoveTo(
                self.get_x() as u16,
                self.get_y() as u16 + self.get_height() as u16
            )
        )
        .unwrap();
        queue!(stdout, style::Print(chars[2])).unwrap();
        queue!(
            stdout,
            cursor::MoveTo(
                self.get_x() as u16 + self.get_width() as u16,
                self.get_y() as u16 + self.get_height() as u16
            )
        )
        .unwrap();
        queue!(stdout, style::Print(chars[3])).unwrap();
        for i in 1..self.get_width() {
            queue!(
                stdout,
                cursor::MoveTo(self.get_x() as u16 + i as u16, self.get_y() as u16)
            )
            .unwrap();
            queue!(stdout, style::Print(chars[4])).unwrap();
            queue!(
                stdout,
                cursor::MoveTo(
                    self.get_x() as u16 + i as u16,
                    self.get_y() as u16 + self.get_height() as u16
                )
            )
            .unwrap();
            queue!(stdout, style::Print(chars[4])).unwrap();
        }
        for i in 1..self.get_height() {
            queue!(
                stdout,
                cursor::MoveTo(self.get_x() as u16, self.get_y() as u16 + i as u16)
            )
            .unwrap();
            queue!(stdout, style::Print(chars[5])).unwrap();
            queue!(
                stdout,
                cursor::MoveTo(
                    self.get_x() as u16 + self.get_width() as u16,
                    self.get_y() as u16 + i as u16
                )
            )
            .unwrap();
            queue!(stdout, style::Print(chars[5])).unwrap();
        }
    }

    fn render(&self) {
        let mut stdout = stdout();
        queue!(
            stdout,
            cursor::MoveTo(self.get_x() as u16, self.get_y() as u16)
        )
        .unwrap();
        let offset = self.get_offset();
        let height = self.get_height() - offset;
        let width = self.get_width() - offset;
        let x = self.get_x() + offset;
        let mut y = self.get_y() + offset;

        let ps = unsafe { PS.as_ref().unwrap() };
        let ts = unsafe { TS.as_ref().unwrap() };
        let syntax = self.get_syntax().unwrap_or(&ps.find_syntax_plain_text());
        let mut h = HighlightLines::new(
            syntax,
            &ts.themes[&self.get_theme().unwrap_or("base16-ocean.dark".to_string())],
        );

        let mut colors = self.get_colors();
        for color_line in &mut colors {
            color_line.sort_by_key(|c| -c.z_index);
            color_line.sort_by_key(|c| c.x);
        }

        let buffer = self.get_buffer();
        let num_lines = buffer.lines().len();
        let lines = buffer.lines().skip(self.get_scroll_lines()).take(height);
        for line in lines {
            let mut line_to_display: String = line
                .chars()
                .skip(self.get_scroll_columns())
                .take(width)
                .collect();
            if line_to_display.ends_with('\n') {
                line_to_display.pop();
            }
            if line_to_display.len() < width {
                line_to_display.push_str(&" ".repeat(width - line_to_display.len()));
            }

            let color_line = colors.get(y + self.get_scroll_lines() - offset);
            let mut intermediate_color_line = Vec::<ColorText>::new();
            if color_line.unwrap_or(&Vec::<ColorText>::new()).len() > 0 {
                let color_line = color_line.unwrap();
                let color_line_contained = color_line
                    .iter()
                    .skip_while(|c| c.x < self.get_scroll_columns())
                    .take_while(|c| c.x < self.get_scroll_columns() + line_to_display.len());
                let mut x_color = 0;
                for color in color_line_contained {
                    if color.x == 0 && x_color == 0 {
                        intermediate_color_line.push(ColorText::new(
                            x_color, color.fg, color.bg, color.len, 0, color.tag,
                        ));
                        x_color = color.x + color.len;
                    } else if color.x >= x_color {
                        x_color = color.x;
                        intermediate_color_line.push(ColorText::new(
                            x_color, color.fg, color.bg, color.len, 0, color.tag,
                        ));
                        x_color = color.x + color.len;
                    } else if color.x < x_color && color.x + color.len > x_color {
                        let len = color.x + color.len - x_color;
                        intermediate_color_line.push(ColorText::new(
                            x_color,
                            color.fg,
                            color.bg,
                            len,
                            color.z_index,
                            color.tag,
                        ));
                        x_color = len;
                    }
                }
            }
            let line = &line_to_display;
            let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
            queue!(stdout, cursor::MoveTo(x as u16, y as u16)).unwrap();
            let mut total_read = 0;
            let mut next_color_idx = 0;
            for (style, mut text) in ranges {
                while next_color_idx < intermediate_color_line.len()
                    && intermediate_color_line[next_color_idx].x >= total_read
                    && intermediate_color_line[next_color_idx].x < total_read + text.len()
                {
                    let def_style: ContentStyle = ContentStyle {
                        foreground_color: Some(Color::Rgb {
                            r: style.foreground.r,
                            g: style.foreground.g,
                            b: style.foreground.b,
                        }),
                        background_color: Some(Color::Rgb {
                            r: style.background.r,
                            g: style.background.g,
                            b: style.background.b,
                        }),
                        ..ContentStyle::default()
                    };

                    let style: ContentStyle = ContentStyle {
                        foreground_color: Some(intermediate_color_line[next_color_idx].fg),
                        background_color: Some(intermediate_color_line[next_color_idx].bg),
                        ..ContentStyle::default()
                    };

                    // eprintln!("text: {:?}", text);
                    let end_of_first_part = intermediate_color_line[next_color_idx].x - total_read;
                    let text_to_print = StyledContent::new(def_style, &text[..end_of_first_part]);
                    queue!(stdout, style::PrintStyledContent(text_to_print)).unwrap();

                    let mut end_of_second_part = intermediate_color_line[next_color_idx].x
                        - total_read
                        + intermediate_color_line[next_color_idx].len;
                    if end_of_second_part > text.len() {
                        end_of_second_part = text.len();
                    }
                    let text_to_print =
                        StyledContent::new(style, &text[end_of_first_part..end_of_second_part]);
                    queue!(stdout, style::PrintStyledContent(text_to_print)).unwrap();

                    text = &text[end_of_second_part..];
                    // let text_to_print = StyledContent::new(def_style, &text[end_of_second_part..]);
                    // queue!(stdout, style::PrintStyledContent(text_to_print)).unwrap();

                    total_read += end_of_second_part;
                    if y == 0 {
                        eprintln!(
                        "total_read: {}, next_color_idx: {}, intermediate_color_line.len(): {:?}, end_of_first_part: {}, end_of_second_part: {}, text.len(): {}",
                        total_read, next_color_idx, intermediate_color_line[next_color_idx], end_of_first_part, end_of_second_part, text.len()
                    );
                    }
                    intermediate_color_line[next_color_idx].len -=
                        end_of_second_part - end_of_first_part;
                    intermediate_color_line[next_color_idx].x +=
                        end_of_second_part - end_of_first_part;
                    if intermediate_color_line[next_color_idx].len == 0 {
                        next_color_idx += 1;
                    }
                }
                let style: ContentStyle = ContentStyle {
                    foreground_color: Some(Color::Rgb {
                        r: style.foreground.r,
                        g: style.foreground.g,
                        b: style.foreground.b,
                    }),
                    background_color: Some(Color::Rgb {
                        r: style.background.r,
                        g: style.background.g,
                        b: style.background.b,
                    }),
                    ..ContentStyle::default()
                };
                let text_to_print = StyledContent::new(style, &text);
                queue!(stdout, style::PrintStyledContent(text_to_print)).unwrap();
                total_read += text.len();
            }
            // }
            y += 1;
        }
        let line_rendered = if num_lines - self.get_scroll_lines() > height {
            height
        } else {
            num_lines - self.get_scroll_lines()
        };
        for i in 0..(height - line_rendered) {
            queue!(stdout, cursor::MoveTo(x as u16, (y + i) as u16)).unwrap();
            queue!(
                stdout,
                style::PrintStyledContent(
                    " ".repeat(width)
                        .with(self.get_default_fg())
                        .on(self.get_default_bg())
                )
            )
            .unwrap();
        }

        match self.get_border_style() {
            BorderStyle::None => {}
            BorderStyle::Solid => {
                self.render_box(["┌", "┐", "└", "┘", "─", "│"]);
            }
            BorderStyle::Dashed => {
                self.render_box(["┌", "┐", "└", "┘", "┄", "┆"]);
            }
        }
    }

    fn update_cursor_position_and_view(&mut self) -> CursorPosition {
        let offset = self.get_offset();
        let mut y = self.get_buffer().byte_to_line(self.get_text_position());
        let mut x = self.get_text_position() - self.get_buffer().line_to_byte(y);

        eprintln!(
            "y: {}, x: {}, scroll_lines: {}, scroll_columns: {}",
            y,
            x,
            self.get_scroll_lines(),
            self.get_scroll_columns()
        );
        if y < self.get_scroll_lines() {
            self.set_scroll_lines(y);
        }
        if y > self.get_scroll_lines() + (self.get_height() - 1) - offset - offset {
            self.set_scroll_lines(y - (self.get_height() - 1 - offset));
        }
        if x < self.get_scroll_columns() {
            self.set_scroll_columns(x);
        }
        if x > self.get_scroll_columns() + self.get_width() - offset - offset {
            self.set_scroll_columns((x + offset) - self.get_width());
        }
        x += offset + self.get_x();
        y += offset + self.get_y();
        y -= self.get_scroll_lines();
        x -= self.get_scroll_columns();
        (x as i32, y as i32)
    }

    fn get_cursor_view(&self) -> CursorPosition {
        let offset = self.get_offset();
        let y = self.get_buffer().byte_to_line(self.get_text_position());
        let x = self.get_text_position() - self.get_buffer().line_to_byte(y);
        let scroll_lines = self.get_scroll_lines();
        let scroll_columns = self.get_scroll_columns();
        // if '\n' we need to go to the next line

        // if y < scroll_lines {
        //     scroll_lines = y;
        // }
        // if y > scroll_lines + (self.get_height() - 1) - offset - offset {
        //     scroll_lines = y - (self.get_height() - 1 - offset);
        // }
        // if x < scroll_columns {
        //     scroll_columns = x;
        // }
        // if x > scroll_columns + self.get_width() - offset - offset {
        //     scroll_columns = (x + offset) - self.get_width();
        // }

        let y: i32 = y as i32 + offset as i32 + self.get_y() as i32 - scroll_lines as i32;
        let x: i32 = x as i32 + offset as i32 + self.get_x() as i32 - scroll_columns as i32;
        (x, y)
    }

    fn is_cursor_visible(&self) -> bool {
        let y = self.get_buffer().byte_to_line(self.get_text_position());
        let x = self.get_text_position() - self.get_buffer().line_to_byte(y);
        let scroll_lines = self.get_scroll_lines();
        let scroll_columns = self.get_scroll_columns();

        if y < scroll_lines {
            return false;
        }
        if y > scroll_lines + (self.get_height() - 1) {
            return false;
        }
        if x < scroll_columns {
            return false;
        }
        if x > scroll_columns + self.get_width() {
            return false;
        }
        true
    }
}
