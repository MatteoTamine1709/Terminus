mod editor;
mod terminal;
mod widget;

use std::{env, path::PathBuf};

use crossterm::{
    event::{poll, read},
    style::Color,
    terminal::size,
};
use editor::{TextEditor, Widget};
use terminal::cleanup_terminal;

use widget::panel::{panel_event, panel_render};

pub fn main_loop(file_content: String, save_path: &PathBuf) {
    let (width, height) = size().unwrap();
    println!("width: {}, height: {}", width, height);
    terminal::setup_terminal(false);
    let mut editor = TextEditor::new(save_path);
    let mut main = Widget::new(
        file_content.clone(),
        0,
        0,
        width as usize - 1,
        height as usize - 1,
        Color::White,
        Color::Reset,
        true,
        editor::BorderStyle::None,
        panel_render,
        panel_event,
    );

    let tmp = Widget::new(
        file_content.clone(),
        80,
        6,
        6 as usize,
        6 as usize,
        Color::Blue,
        Color::Red,
        false,
        editor::BorderStyle::Dashed,
        panel_render,
        panel_event,
    );
    main.add_widget(tmp);
    editor.add_widget(main);

    editor.render((0, 0));

    loop {
        if (poll(std::time::Duration::from_millis(100))).unwrap() {
            if editor.event(read().unwrap()) {
                break;
            }
        }
    }
    cleanup_terminal("Done");
}

fn main() {
    // match enable_raw_mode() {
    //     Ok(_) => {},
    //     Err(e) => panic!("ERROR: Could not enable raw mode: {}", e),
    // }

    let mut args = env::args();
    let _ = args.next().unwrap();
    let filepath = args.next().expect("ERROR: No file given");
    let (file_content, newly_loaded) = match std::fs::read_to_string(&filepath) {
        Ok(x) => (x, false),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (String::new(), true),
        Err(e) => {
            println!("Failed to open file: {:?}", e);
            return;
        }
    };

    main_loop(file_content, &PathBuf::from(filepath));
}
