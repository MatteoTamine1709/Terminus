mod editor;
mod event;
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
use widget::widget::{BorderStyle, Widget};

use crate::widget::{
    command_line::command_line, no_event::no_event, panel::panel_event, widget::WidgetID,
};

// use widget::panel::{panel_event, panel_render};

pub fn main_loop(file_content: String, save_path: &PathBuf) {
    let (mut width, height) = size().unwrap();
    width -= 1;
    println!("width: {}, height: {}", width, height);
    terminal::setup_terminal(true);
    let mut editor = TextEditor::new(save_path);
    let mut main = Widget::new(
        WidgetID::Main as usize,
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
        panel_event,
    );

    let status_bar = Widget::new(
        WidgetID::Status as usize,
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
        no_event,
    );

    let line_number = Widget::new(
        WidgetID::LineNumber as usize,
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
        no_event,
    );

    let command_line = Widget::new(
        WidgetID::CommandLine as usize,
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
        command_line,
    );
    main.add_widget(command_line);
    main.add_widget(status_bar);
    main.add_widget(line_number);
    let pos = main.update_cursor_position_and_view();
    editor.add_widget(main);

    editor.event(crossterm::event::Event::FocusGained);
    editor.render(pos);

    while editor.running {
        if (poll(std::time::Duration::from_millis(100))).unwrap() {
            editor.event(read().unwrap());
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
