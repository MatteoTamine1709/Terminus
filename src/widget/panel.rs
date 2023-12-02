use crossterm::event::Event;
use once_cell::sync::Lazy;

use crate::widget::command_line::write_to_command_line;
use crate::{editor::TextEditor, event::process_event, widget::widget::WidgetID};

use super::widget::Widget;

type ShouldExit = bool;
type CursorPosition = (usize, usize);

use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;
use std::time::{Duration, Instant};

static mut CLEAR_COMMAND_LINE_TIMING: Lazy<Instant> = Lazy::new(|| Instant::now());
static mut CLEAR_COMMAND_LINE_DURATION: Lazy<Duration> = Lazy::new(|| Duration::from_secs(5));

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
                        update_status_bar(widget, true, false);
                    }
                    crossterm::event::KeyCode::Enter => {
                        widget.buffer.insert_char(widget.text_position, '\n');
                        widget.text_position += 1;
                        update_status_bar(widget, true, false);
                    }
                    crossterm::event::KeyCode::Backspace => {
                        if widget.text_position > 0 {
                            widget
                                .buffer
                                .remove(widget.text_position - 1..widget.text_position);
                            widget.text_position -= 1;
                            update_status_bar(widget, true, false);
                        }
                    }
                    _ => {}
                },
                crossterm::event::KeyModifiers::CONTROL => match key_event.code {
                    crossterm::event::KeyCode::Char(c) => match c {
                        's' => {
                            for i in 0..widget.widgets.len() {
                                if widget.widgets[i].id == WidgetID::CommandLine as usize {
                                    write_to_command_line(&mut widget.widgets[i], "Saving...");
                                    unsafe {
                                        CLEAR_COMMAND_LINE_TIMING = Lazy::new(|| Instant::now());
                                        CLEAR_COMMAND_LINE_DURATION =
                                            Lazy::new(|| Duration::from_secs(2));
                                    };
                                    break;
                                }
                            }
                            let writer = fs::File::create(&editor.save_path).unwrap();
                            widget.buffer.write_to(writer).unwrap();
                            update_status_bar(widget, false, true);
                        }
                        'f' => {
                            for i in 0..widget.widgets.len() {
                                if widget.widgets[i].id == WidgetID::CommandLine as usize {
                                    widget.widgets[i].focused = true;
                                    return (
                                        widget.widgets[i].update_cursor_position_and_view(),
                                        false,
                                    );
                                }
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }
        }
        widget.processed = true;
        let res = process_event(editor, widget, event);
        widget.processed = false;
        update_line_number(widget);
        update_status_bar(widget, false, false);

        if unsafe { CLEAR_COMMAND_LINE_TIMING.elapsed() } > unsafe { *CLEAR_COMMAND_LINE_DURATION }
        {
            for i in 0..widget.widgets.len() {
                if widget.widgets[i].id == WidgetID::CommandLine as usize {
                    write_to_command_line(&mut widget.widgets[i], "");
                }
            }
        }
        return res;
    }
    panic!("Unreachable code")
}

fn get_git_branch_name(repo_path: &Path) -> io::Result<String> {
    let head_path = repo_path.join(".git/HEAD");
    let content = fs::read_to_string(head_path)?;
    content
        .split_whitespace()
        .last()
        .and_then(|s| s.split('/').last())
        .map(String::from)
        .ok_or(io::Error::new(ErrorKind::Other, "Branch name not found"))
}

static TOTAL_FILE_INFO_WIDTH: usize = 25;
static TOTAL_POS_INFO_WIDTH: usize = 20;

fn update_status_bar(widget: &mut Widget, written: bool, saved: bool) {
    for i in 0..widget.widgets.len() {
        if widget.widgets[i].id == WidgetID::Status as usize {
            let current = widget.widgets[i].buffer.to_string();
            let parts = current.split(' ').collect::<Vec<&str>>();
            let mut file_info = parts[0].to_string();
            let last_char = file_info.chars().last().unwrap();
            if last_char != '*' && written {
                file_info.push('*');
            }
            if saved {
                file_info = file_info.replace("*", "");
            }

            let mut status_bar = String::new();
            status_bar.push_str(&file_info);
            status_bar.push_str(&" ".repeat(TOTAL_FILE_INFO_WIDTH - file_info.len()));

            let mut pos_info = String::new();
            let pos = widget.update_cursor_position_and_view();
            let x = pos.0 + 1 - widget.x + widget.scroll_columns;
            let y = pos.1 + 1 - widget.y + widget.scroll_lines;
            let percent = y * 100 / widget.buffer.len_lines();
            pos_info.push_str(&format!("{}% ({},{})", percent, x, y));
            status_bar.push_str(&pos_info);
            status_bar.push_str(&" ".repeat(TOTAL_POS_INFO_WIDTH - pos_info.len()));

            // Get git branch

            match get_git_branch_name(Path::new(".")) {
                Ok(branch_name) => {
                    status_bar.push_str(&format!("Git: {}", branch_name));
                }
                Err(_e) => {
                    status_bar.push_str(&format!("Git: /"));
                }
            }

            widget.widgets[i].buffer = ropey::Rope::from_str(&status_bar);
            break;
        }
    }
}

fn update_line_number(widget: &mut Widget) {
    let is_relative = true;
    for i in 0..widget.widgets.len() {
        if widget.widgets[i].id == WidgetID::LineNumber as usize {
            widget.widgets[i].colors = vec![(None, None); widget.buffer.len_lines()];
            let mut line_number = String::new();
            for j in widget.scroll_lines..(widget.scroll_lines + widget.height) {
                // Padded to the right
                if is_relative {
                    let pos = widget.update_cursor_position_and_view();
                    let v: i32 = (j as i32) - ((pos.1 + widget.scroll_lines) as i32);
                    let value: String = if v == 0 {
                        (j + 1).to_string()
                    } else {
                        (v.abs()).to_string()
                    };
                    line_number.push_str(&" ".repeat(widget.widgets[i].width - value.len()));
                    line_number.push_str(&value);
                    line_number.push('\n');
                } else {
                    let value: String = (j + 1).to_string();
                    line_number.push_str(&" ".repeat(widget.widgets[i].width - value.len()));
                    line_number.push_str(&value);
                    line_number.push('\n');
                }
            }
            widget.widgets[i].buffer = ropey::Rope::from_str(&line_number);
            break;
        }
    }
}
