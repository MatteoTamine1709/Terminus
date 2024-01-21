// 'aaaa' -> 'aaaa'
mod editor;
mod terminal;
mod widget;

use std::{env, io::stdout, path::PathBuf};

use crossterm::{
    event::{poll, read},
    style::{Color, Print, PrintStyledContent},
    terminal::size,
};
use editor::TextEditor;
use terminal::cleanup_terminal;
use widget::widget::BorderStyle;

use crate::widget::{
    command_line::CommandLine, line_number::LineNumber, panel::Panel, status_bar::StatusBar,
    widget::ProcessEvent,
}; // test
use crossterm::execute;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

pub fn main_loop(file_content: String, save_path: PathBuf, new_load: bool) {
    let aaaa = "aaaa";
    let (width, height) = size().unwrap();
    println!("width: {}, height: {}", width, height);
    terminal::setup_terminal(true);
    let mut editor = TextEditor::new(&save_path);
    editor.written = new_load;
    let line_number_width = 8;
    eprintln!("line_number_width: {}", line_number_width);
    eprintln!("width: {}, height: {}", width, height);
    let mut main = Panel::new(
        file_content.clone(),
        line_number_width,
        0,
        width as usize - line_number_width,
        height as usize - 2,
        Color::White,
        Color::Reset,
        true,
        true,
        BorderStyle::None,
    );
    main.set_z_idx(1);

    let status_bar: Box<StatusBar> = StatusBar::new(
        save_path.to_str().unwrap().to_string(),
        0,
        height as usize - 2,
        width as usize,
        1 as usize,
        Color::Black,
        Color::White,
        false,
        false,
        BorderStyle::None,
    );
    //ici

    let line_number = LineNumber::new(
        String::new(),
        0,
        0,
        line_number_width as usize,
        height as usize - 2,
        Color::DarkGrey,
        Color::Black,
        false,
        false,
        BorderStyle::None,
    );

    let command_line = CommandLine::new(
        0,
        String::new(),
        0,
        height as usize - 1,
        width as usize,
        1,
        Color::White,
        Color::Black,
        false,
        false,
        BorderStyle::None,
    );
    let pos = main.update_cursor_position_and_view();
    editor.add_widget(status_bar);
    editor.add_widget(line_number);
    editor.add_widget(command_line);
    editor.focused_widget_id = editor.add_widget(main);
    // editor.add_widget(tmp);

    editor.event(&crossterm::event::Event::FocusGained);
    editor.render(pos, true);

    while editor.running {
        if (poll(std::time::Duration::from_millis(100))).unwrap() {
            editor.event(&read().unwrap());
        }
    }
    cleanup_terminal("Done");
}

fn main() {
    let mut args = env::args();
    let _ = args.next().unwrap();
    let filepath = args.next();
    let pathbuf: PathBuf;
    let mut file_content = String::new();
    let new_load;
    if let Some(filepath) = &filepath {
        (file_content, new_load) = match std::fs::read_to_string(&filepath) {
            Ok(content) => {
                pathbuf = PathBuf::from(filepath);
                (content, false)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                pathbuf = PathBuf::from(filepath);
                (String::new(), true)
            }
            Err(e) => {
                println!("Failed to open file: {:?}", e);
                return;
            }
        };
    } else {
        pathbuf = PathBuf::from("untitled");
        new_load = true;
    }
    // ICI

    main_loop(file_content, pathbuf, new_load);
}
