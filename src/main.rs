mod editor;
mod terminal;
mod widget;

use std::{env, path::PathBuf};

use crossterm::{
    event::{poll, read},
    style::Color,
    terminal::size,
};
use editor::TextEditor;
use terminal::cleanup_terminal;
use widget::widget::BorderStyle;

use crate::widget::{
    command_line::CommandLine, line_number::LineNumber, panel::Panel, status_bar::StatusBar,
    widget::ProcessEvent,
};

pub fn main_loop(file_content: String, save_path: &PathBuf) {
    let (mut width, height) = size().unwrap();
    width -= 1;
    println!("width: {}, height: {}", width, height);
    terminal::setup_terminal(true);
    let mut editor = TextEditor::new(save_path);
    let mut main = Panel::new(
        file_content.clone(),
        5,
        0,
        width as usize,
        height as usize - 2,
        Color::White,
        Color::Reset,
        true,
        true,
        BorderStyle::None,
    );

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

    let line_number = LineNumber::new(
        String::new(),
        0,
        0,
        4 as usize,
        height as usize - 2,
        Color::DarkGrey,
        Color::Black,
        false,
        false,
        BorderStyle::None,
    );

    let command_line = CommandLine::new(
        String::new(),
        0,
        height as usize - 1,
        width as usize,
        1 as usize,
        Color::White,
        Color::Black,
        false,
        false,
        BorderStyle::None,
    );
    let pos = main.update_cursor_position_and_view();
    editor.add_widget(status_bar);
    editor.add_widget(line_number);
    editor.add_widget(main);
    editor.add_widget(command_line);
    // editor.add_widget(tmp);

    editor.event(&crossterm::event::Event::FocusGained);
    editor.render(pos);

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
    let filepath = args.next().expect("ERROR: No file given");
    let (file_content, _) = match std::fs::read_to_string(&filepath) {
        Ok(x) => (x, false),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (String::new(), true),
        Err(e) => {
            println!("Failed to open file: {:?}", e);
            return;
        }
    };

    main_loop(file_content, &PathBuf::from(filepath));
}
