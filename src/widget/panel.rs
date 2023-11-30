use std::io::{stdout, Write};

use crossterm::{
    cursor::{self},
    event::Event,
    queue,
    style::{self, Color, PrintStyledContent, Stylize},
    terminal,
};
use ropey::RopeSlice;

use crate::editor::{self, TextEditor, Widget};

type ShouldExit = bool;
type CursorPositionByte = usize;
type CursorPosition = (usize, usize);

fn render_box(widget: &Widget, chars: [&str; 6]) {
    let mut stdout = stdout();
    queue!(stdout, cursor::MoveTo(widget.x as u16, widget.y as u16)).unwrap();
    queue!(stdout, style::Print(chars[0])).unwrap();
    queue!(
        stdout,
        cursor::MoveTo(widget.x as u16 + widget.width as u16, widget.y as u16)
    )
    .unwrap();
    queue!(stdout, style::Print(chars[1])).unwrap();
    queue!(
        stdout,
        cursor::MoveTo(widget.x as u16, widget.y as u16 + widget.height as u16)
    )
    .unwrap();
    queue!(stdout, style::Print(chars[2])).unwrap();
    queue!(
        stdout,
        cursor::MoveTo(
            widget.x as u16 + widget.width as u16,
            widget.y as u16 + widget.height as u16
        )
    )
    .unwrap();
    queue!(stdout, style::Print(chars[3])).unwrap();
    for i in 1..widget.width {
        queue!(
            stdout,
            cursor::MoveTo(widget.x as u16 + i as u16, widget.y as u16)
        )
        .unwrap();
        queue!(stdout, style::Print(chars[4])).unwrap();
        queue!(
            stdout,
            cursor::MoveTo(
                widget.x as u16 + i as u16,
                widget.y as u16 + widget.height as u16
            )
        )
        .unwrap();
        queue!(stdout, style::Print(chars[4])).unwrap();
    }
    for i in 1..widget.height {
        queue!(
            stdout,
            cursor::MoveTo(widget.x as u16, widget.y as u16 + i as u16)
        )
        .unwrap();
        queue!(stdout, style::Print(chars[5])).unwrap();
        queue!(
            stdout,
            cursor::MoveTo(
                widget.x as u16 + widget.width as u16,
                widget.y as u16 + i as u16
            )
        )
        .unwrap();
        queue!(stdout, style::Print(chars[5])).unwrap();
    }
}

pub fn panel_render(widget: &mut Widget, text_editor: &mut TextEditor) {
    let mut stdout = stdout();
    queue!(stdout, cursor::MoveTo(widget.x as u16, widget.y as u16)).unwrap();
    let mut height = widget.height;
    let mut width = widget.width;
    let mut x = widget.x;
    let mut y = widget.y;
    match widget.boder_style {
        editor::BorderStyle::None => {}
        editor::BorderStyle::Solid => {
            render_box(widget, ["┌", "┐", "└", "┘", "─", "│"]);
            height -= 1;
            width -= 1;
            x += 1;
            y += 1;
        }
        editor::BorderStyle::Dashed => {
            render_box(widget, ["┌", "┐", "└", "┘", "┄", "┆"]);
            height -= 1;
            width -= 1;
            x += 1;
            y += 1;
        }
    }
    let mut lines: Vec<RopeSlice> = widget
        .buffer
        .lines()
        .skip(widget.scroll_lines)
        .take(height)
        .collect();
    for line in lines.iter_mut() {
        let mut line_to_display: String = line
            .chars()
            .skip(widget.scroll_columns)
            .take(width)
            .collect();
        if line_to_display.len() < width {
            line_to_display.push_str(&" ".repeat(width - line_to_display.len()));
        }
        queue!(stdout, cursor::MoveTo(x as u16, y as u16)).unwrap();
        queue!(
            stdout,
            style::PrintStyledContent(
                line_to_display
                    .with(widget.color)
                    .on(widget.background_color)
            )
        )
        .unwrap();
        y += 1;
    }

    for child in widget.widgets.iter_mut() {
        (child.render)(child, text_editor);
    }
}

pub fn panel_event(
    widget: &mut Widget,
    text_editor: &mut TextEditor,
    event: Event,
) -> (CursorPosition, ShouldExit) {
    if widget.focused {
        if let Event::Key(key_event) = event {
            match key_event.modifiers {
                crossterm::event::KeyModifiers::CONTROL => match key_event.code {
                    crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Char('c') => {
                        return ((0, 0), true);
                    }
                    _ => {}
                },
                crossterm::event::KeyModifiers::NONE => match key_event.code {
                    crossterm::event::KeyCode::Tab => {
                        if let Some(focused_widget) = widget.focused_widget {
                            // widget.widgets[focused_widget].focused = false;

                            let mut res = (widget.widgets[focused_widget].event)(
                                &mut widget.widgets[focused_widget],
                                text_editor,
                                event,
                            );
                            if res.1 {
                                widget.widgets[focused_widget].focused = false;
                                widget.focused_widget = None;
                                res.0 = widget.cursor_position;
                            }
                            res.1 = false;
                            return res;
                        } else {
                            if widget.widgets.len() == 0 {
                                return (widget.cursor_position, true);
                            }
                            widget.focused_widget = Some(0);
                            widget.widgets[0].focused = true;

                            let y_offset = match widget.widgets[0].boder_style {
                                editor::BorderStyle::None => 0,
                                _ => 1,
                            };
                            let x_offset = match widget.widgets[0].boder_style {
                                editor::BorderStyle::None => 0,
                                _ => 1,
                            };
                            return (
                                (
                                    widget.widgets[0].cursor_position.0
                                        + widget.widgets[0].x
                                        + x_offset,
                                    widget.widgets[0].cursor_position.1
                                        + widget.widgets[0].y
                                        + y_offset,
                                ),
                                false,
                            );
                        }
                    }
                    crossterm::event::KeyCode::Right => {
                        if let Some(focused_widget) = widget.focused_widget {
                            return (widget.widgets[focused_widget].event)(
                                &mut widget.widgets[focused_widget],
                                text_editor,
                                event,
                            );
                        } else {
                            let y_offset = match widget.boder_style {
                                editor::BorderStyle::None => 0,
                                _ => 1,
                            };
                            let x_offset = match widget.boder_style {
                                editor::BorderStyle::None => 0,
                                _ => 1,
                            };
                            let mut res = (widget.cursor_position, false);

                            let old = widget.buffer.char_to_line(widget.cursor);
                            widget.cursor += 1;
                            let new = widget.buffer.char_to_line(widget.cursor);

                            res.0 .0 += 1;
                            if old != new {
                                widget.scroll_columns = 0;
                                res.0 .0 = 0;
                                res.0 .1 += 1;
                            }
                            if res.0 .1 as usize >= widget.height - y_offset {
                                widget.scroll_lines += 1;
                                res.0 .1 -= 1;
                            }
                            if res.0 .0 >= widget.width {
                                widget.scroll_columns += 1;
                                res.0 .0 -= 1;
                            }
                            widget.cursor_position = res.0;
                            res.0 .0 += widget.x + x_offset;
                            res.0 .1 += widget.y + y_offset;
                            return res;
                        }
                    }
                    crossterm::event::KeyCode::Left => {
                        if let Some(focused_widget) = widget.focused_widget {
                            return (widget.widgets[focused_widget].event)(
                                &mut widget.widgets[focused_widget],
                                text_editor,
                                event,
                            );
                        } else {
                            let y_offset = match widget.boder_style {
                                editor::BorderStyle::None => 0,
                                _ => 1,
                            };
                            let x_offset = match widget.boder_style {
                                editor::BorderStyle::None => 0,
                                _ => 1,
                            };
                            let mut res = (widget.cursor_position, false);
                            if widget.cursor > 0 {
                                let old = widget.buffer.char_to_line(widget.cursor);
                                widget.cursor -= 1;
                                let new = widget.buffer.char_to_line(widget.cursor);

                                if res.0 .0 == 0 && widget.scroll_columns > 0 {
                                    widget.scroll_columns -= 1;
                                } else if res.0 .0 > 0 {
                                    res.0 .0 -= 1;
                                }
                                if old != new {
                                    if widget.width < widget.buffer.line(new).len_chars() - 1 {
                                        widget.scroll_columns =
                                            widget.buffer.line(new).len_chars() - widget.width;
                                    } else {
                                        widget.scroll_columns = 0;
                                    }
                                    res.0 .0 = widget.buffer.line(new).len_chars() - 1;
                                    if (res.0 .0 > widget.width - x_offset) {
                                        res.0 .0 = widget.width - x_offset;
                                    }
                                    if res.0 .1 > 0 {
                                        res.0 .1 -= 1;
                                    } else if widget.scroll_lines > 0 {
                                        widget.scroll_lines -= 1;
                                    }
                                }
                                widget.cursor_position = res.0;
                            }
                            res.0 .0 += widget.x + x_offset;
                            res.0 .1 += widget.y + y_offset;
                            return res;
                        }
                    }
                    crossterm::event::KeyCode::Up => {
                        if let Some(focused_widget) = widget.focused_widget {
                            return (widget.widgets[focused_widget].event)(
                                &mut widget.widgets[focused_widget],
                                text_editor,
                                event,
                            );
                        } else {
                            let y_offset = match widget.boder_style {
                                editor::BorderStyle::None => 0,
                                _ => 1,
                            };
                            let x_offset = match widget.boder_style {
                                editor::BorderStyle::None => 0,
                                _ => 1,
                            };
                            let mut res = (widget.cursor_position, false);
                            if widget.cursor > 0 {
                                let old = widget.buffer.char_to_line(widget.cursor);
                                eprintln!("old: {} {}", old, widget.cursor_position.1);
                                if widget.cursor_position.1 <= 1 || old == 0 {
                                    eprintln!("widget.scroll_lines: {}", widget.scroll_lines);
                                    if widget.cursor_position.1 <= 1 && widget.scroll_lines > 0 {
                                        widget.scroll_lines -= 1;

                                        return (
                                            (
                                                widget.x + x_offset + widget.cursor_position.0,
                                                widget.y + y_offset + widget.cursor_position.1,
                                            ),
                                            false,
                                        );
                                    }
                                    if widget.cursor_position.1 == 0 && old == 0 {
                                        widget.cursor = 0;
                                        widget.cursor_position = (0, 0);
                                        return (
                                            (
                                                widget.x + x_offset + widget.cursor_position.0,
                                                widget.y + y_offset + widget.cursor_position.1,
                                            ),
                                            false,
                                        );
                                    }
                                }
                                let ltb = widget.buffer.line_to_char(old); // 12
                                let current_line_idx = widget.cursor - ltb; // 24 - 12 = 12
                                let new = old - 1;

                                let len_prev_line = widget.buffer.line(new).len_chars() - 1; // 11
                                widget.cursor = widget.buffer.line_to_char(new) + current_line_idx;
                                if current_line_idx >= len_prev_line {
                                    widget.cursor = ltb - 1;
                                    if widget.scroll_columns > len_prev_line {
                                        widget.scroll_columns = len_prev_line;
                                    }
                                    widget.cursor_position.0 =
                                        len_prev_line - widget.scroll_columns;
                                    res.0 .0 = len_prev_line - widget.scroll_columns;
                                }
                                widget.cursor_position.1 -= 1;
                                res.0 .1 -= 1;
                            }
                            res.0 .0 += widget.x + x_offset;
                            res.0 .1 += widget.y + y_offset;
                            return res;
                        }
                    }
                    crossterm::event::KeyCode::Down => {
                        if let Some(focused_widget) = widget.focused_widget {
                            return (widget.widgets[focused_widget].event)(
                                &mut widget.widgets[focused_widget],
                                text_editor,
                                event,
                            );
                        } else {
                            let y_offset = match widget.boder_style {
                                editor::BorderStyle::None => 0,
                                _ => 1,
                            };
                            let x_offset = match widget.boder_style {
                                editor::BorderStyle::None => 0,
                                _ => 1,
                            };
                            let mut res = (widget.cursor_position, false);
                            if widget.cursor < widget.buffer.len_chars() - 1 {
                                if widget.cursor_position.1 >= widget.height - 1 - y_offset {
                                    if widget.cursor_position.1 >= widget.height - 1 - y_offset
                                        && widget.scroll_lines < widget.buffer.len_lines() - 1
                                    {
                                        widget.scroll_lines += 1;
                                    } else {
                                        widget.cursor = widget.buffer.len_chars() - 1;
                                        widget.cursor_position =
                                            (widget.width - 1, widget.height - 1);
                                    }
                                    return (
                                        (
                                            widget.x + x_offset + widget.cursor_position.0,
                                            widget.y + y_offset + widget.cursor_position.1,
                                        ),
                                        false,
                                    );
                                }
                                let old = widget.buffer.char_to_line(widget.cursor);
                                let ltb = widget.buffer.line_to_char(old);
                                let current_line_idx = widget.cursor - ltb;
                                let new = old + 1;

                                let old_len = widget.buffer.line(old).len_chars();
                                let new_len = widget.buffer.line(new).len_chars();

                                widget.cursor = widget.buffer.line_to_char(new) + current_line_idx;
                                if new_len < old_len && current_line_idx >= new_len {
                                    widget.cursor = widget.buffer.line_to_char(new) + new_len - 1;
                                    if widget.scroll_columns > new_len - 1 {
                                        widget.scroll_columns = new_len - 1;
                                    }
                                    widget.cursor_position.0 = new_len - 1 - widget.scroll_columns;

                                    res.0 .0 = new_len - 1 - widget.scroll_columns;
                                }
                                widget.cursor_position.1 += 1;
                                res.0 .1 += 1;
                            }
                            res.0 .0 += widget.x + x_offset;
                            res.0 .1 += widget.y + y_offset;
                            return res;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        return (widget.cursor_position, false);
    }
    panic!("Unreachable code")
}
