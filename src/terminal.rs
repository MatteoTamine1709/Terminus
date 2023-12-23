use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture, PushKeyboardEnhancementFlags},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::stdout;

pub fn setup_terminal(disable_mouse_interaction: bool) {
    // set panic hook
    std::panic::set_hook(Box::new(|info| {
        // clean up the terminal
        cleanup_terminal("Panic!");

        // pring panic info, if any
        if let Some(msg) = info.payload().downcast_ref::<&str>() {
            println!("Cause: {:?}", msg);
        }

        if let Some(loc) = info.location() {
            println!("Location: {}", loc);
        }
    }));

    // set raw mode
    enable_raw_mode().unwrap();

    execute!(
        stdout(),
        // so it can be restored
        cursor::SavePosition,
        // so it won't clutter other activities
        EnterAlternateScreen,
        EnableMouseCapture,
        PushKeyboardEnhancementFlags(
            crossterm::event::KeyboardEnhancementFlags::REPORT_EVENT_TYPES
        ),
        PushKeyboardEnhancementFlags(
            crossterm::event::KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
        ),
        // change cursor to a bar, as that's more clear
        cursor::SetCursorStyle::BlinkingBar,
    )
    .unwrap();

    // allow mouse usage
    if !disable_mouse_interaction {
        execute!(stdout(), EnableMouseCapture).unwrap();
    }
}

/// clean up the terminal
pub fn cleanup_terminal(message: &str) {
    execute!(
        stdout(),
        // go back to the normal screen
        LeaveAlternateScreen,
        // disable mouse
        DisableMouseCapture,
        // restore old cursor position
        cursor::RestorePosition,
        // restore cursor style
        cursor::SetCursorStyle::BlinkingBar,
        // restore visibility
        cursor::Show,
        // reset colors
        crossterm::style::SetForegroundColor(crossterm::style::Color::Reset),
        crossterm::style::SetBackgroundColor(crossterm::style::Color::Reset),
    )
    .unwrap();

    // leave raw mode
    disable_raw_mode().unwrap();

    // print the final message
    println!("{}", message);
}
