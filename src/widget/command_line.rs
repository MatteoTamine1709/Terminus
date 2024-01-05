use std::{collections::HashMap, fs};

use crossterm::{event::Event, style::Color};
use ropey::Rope;

use crate::{editor::TextEditor, widget::popup::Popup};

use super::widget::{
    BorderStyle, ColorText, ColorTextTag, CursorPosition, CursorPositionByte, ProcessEvent,
    ShouldExit, WidgetType,
};

trait CommandLineCommands {
    fn quit(
        command_line: &mut CommandLine,
        editor: &mut TextEditor,
        args: Vec<String>,
        event: Event,
    );
    fn find(
        command_line: &mut CommandLine,
        editor: &mut TextEditor,
        args: Vec<String>,
        event: Event,
    );
    fn save(
        command_line: &mut CommandLine,
        editor: &mut TextEditor,
        args: Vec<String>,
        event: Event,
    );
}

pub struct CommandLine {
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

    old_buffer: Rope,

    commands: HashMap<
        String,
        fn(command_line: &mut Self, editor: &mut TextEditor, args: Vec<String>, event: Event),
    >,
    positions: Vec<CursorPositionByte>,
    position_idx: usize,

    list_popup: Option<(WidgetType, usize)>,

    z_idx: usize,
}

impl CommandLine {
    pub fn new(
        id: usize,
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
        let commands = {
            let mut m: HashMap<
                String,
                fn(
                    command_line: &mut Self,
                    editor: &mut TextEditor,
                    args: Vec<String>,
                    event: Event,
                ),
            > = HashMap::new();
            m.insert(":quit".to_string(), Self::quit);
            m.insert(":find".to_string(), Self::find);
            m.insert(":save".to_string(), Self::save);
            m
        };
        Box::new(Self {
            typ: WidgetType::CommandLine,
            id,
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
            commands,
            ..Default::default()
        })
    }

    fn execute_command(&mut self, editor: &mut TextEditor, event: &Event) {
        let buffer = self.buffer.to_string();
        let args = buffer
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        if args.len() > 0 {
            if let Some(command) = self.commands.get(&args[0]) {
                command(self, editor, args, event.clone())
            }
        }
        self.old_buffer = self.buffer.clone();
    }

    fn next_position(&mut self, editor: &mut TextEditor) -> CursorPositionByte {
        if self.positions.len() > 0 {
            self.position_idx += 1;
            if self.position_idx >= self.positions.len() {
                self.position_idx = 0;
            }
            if let Some(panel) = editor.get_widget_mut(WidgetType::Panel) {
                panel.set_text_position(self.positions[self.position_idx]);
                panel.update_cursor_position_and_view();
                return self.positions[self.position_idx];
            }
        }
        0
    }

    fn prev_position(&mut self, editor: &mut TextEditor) {
        if self.positions.len() > 0 {
            if self.position_idx == 0 {
                self.position_idx = self.positions.len() - 1;
            } else {
                self.position_idx -= 1;
            }
            if let Some(panel) = editor.get_widget_mut(WidgetType::Panel) {
                panel.set_text_position(self.positions[self.position_idx]);
                panel.update_cursor_position_and_view();
            }
        }
    }

    fn create_popup(&mut self, editor: &mut TextEditor) {
        // Delete the old popup
        if let Some((typ, id)) = self.list_popup {
            editor.remove_widget_id(id, typ);
            self.list_popup = None;
        }
        let mut text = String::new();
        let mut max_len = 0;
        for (key, _) in &self.commands {
            if key.len() > max_len {
                max_len = key.len();
            }
            text.push_str(key);
            text.push_str("\n");
        }
        let mut widget = Popup::new(
            text,
            self.get_cursor_view().0 as usize,
            self.get_cursor_view().1 as usize - self.commands.len() - 2,
            max_len + 1,
            self.commands.len() + 1,
            Color::Grey,
            Color::Blue,
            false,
            false,
            BorderStyle::Dashed,
        );
        widget.set_z_idx(10);
        self.list_popup = Some((WidgetType::Popup, editor.add_widget(widget)));
        eprintln!("popup: {:?}", self.list_popup)
    }

    fn update_popup(&mut self, editor: &mut TextEditor) {
        if let Some((typ, id)) = self.list_popup {
            if let Some(widget) = editor.get_widget_id_mut(id, typ) {
                let current_buffer = self.buffer.to_string();
                let mut text = String::new();
                let mut max_len = 0;
                let mut number_of_commands = 0;
                for (key, _) in &self.commands {
                    if key.len() > max_len {
                        max_len = key.len();
                    }
                    if key.starts_with(&current_buffer) || current_buffer.starts_with(key) {
                        text.push_str(key);
                        text.push_str("\n");
                        number_of_commands += 1;
                    }
                }
                widget.set_x(self.get_cursor_view().0 as usize);
                widget.set_y(self.get_cursor_view().1 as usize - number_of_commands - 2);
                widget.set_width(max_len + 1);
                widget.set_height(number_of_commands + 1);
                widget.set_buffer(Rope::from_str(&text));
            }
        }
    }
}

impl CommandLineCommands for CommandLine {
    fn quit(
        command_line: &mut CommandLine,
        editor: &mut TextEditor,
        _args: Vec<String>,
        _event: Event,
    ) {
        if command_line.old_buffer.cmp(&command_line.buffer) == std::cmp::Ordering::Equal {
            editor.running = false;
        }
    }

    fn find(
        command_line: &mut CommandLine,
        editor: &mut TextEditor,
        args: Vec<String>,
        _event: Event,
    ) {
        if args.len() < 2 {
            return;
        }
        let search_term = &args[1];
        if command_line.positions.len() > 0
            && command_line.old_buffer.cmp(&command_line.buffer) == std::cmp::Ordering::Equal
        {
            let new_position = command_line.next_position(editor);
            if let Some(panel) = editor.get_widget_mut(WidgetType::Panel) {
                let y = panel.get_buffer().byte_to_line(new_position);
                let x = panel.get_buffer().byte_to_char(new_position)
                    - panel.get_buffer().line_to_char(y);
                panel.remove_color(&|c: &ColorText| c.tag == ColorTextTag::Selection);

                panel.push_color(
                    y,
                    ColorText {
                        x,
                        fg: Color::Blue,
                        bg: Color::Reset,
                        len: search_term.len(),
                        z_index: 10,
                        tag: ColorTextTag::Selection,
                    },
                );
            }
            return;
        }
        command_line.positions.clear();
        command_line.position_idx = 0;
        if let Some(panel) = editor.get_widget_mut(WidgetType::Panel) {
            let mut found_pos = panel.get_text_position();
            let mut found = false;
            let found_positions = panel
                .get_buffer()
                .chars()
                .collect::<String>()
                .to_lowercase()
                .match_indices(search_term.to_lowercase().as_str())
                .map(|(pos, _)| pos)
                .collect::<Vec<_>>();

            panel.remove_color(&|c: &ColorText| c.tag == ColorTextTag::Find);
            panel.remove_color(&|c: &ColorText| c.tag == ColorTextTag::Selection);

            for pos in found_positions {
                let y = panel.get_buffer().byte_to_line(pos);
                let x = panel.get_buffer().byte_to_char(pos) - panel.get_buffer().line_to_char(y);
                panel.push_color(
                    y,
                    ColorText {
                        x,
                        fg: Color::Red,
                        bg: Color::Reset,
                        len: search_term.len(),
                        z_index: 5,
                        tag: ColorTextTag::Find,
                    },
                );
                if !found && pos >= found_pos {
                    found = true;
                    found_pos = pos;
                    command_line.position_idx = command_line.positions.len();
                }
                command_line.positions.push(pos);
            }

            if !found && command_line.positions.len() > 0 {
                found_pos = command_line.positions[0];
                command_line.position_idx = 0;
            }
            let y = panel.get_buffer().byte_to_line(found_pos);
            let x = panel.get_buffer().byte_to_char(found_pos) - panel.get_buffer().line_to_char(y);

            panel.push_color(
                y,
                ColorText {
                    x,
                    fg: Color::Blue,
                    bg: Color::Reset,
                    len: search_term.len(),
                    z_index: 10,
                    tag: ColorTextTag::Selection,
                },
            );
            panel.set_text_position(found_pos);
            panel.update_cursor_position_and_view();
        }
    }

    fn save(
        command_line: &mut CommandLine,
        editor: &mut TextEditor,
        args: Vec<String>,
        event: Event,
    ) {
        if args.len() < 2 {
            return;
        }
        // if event != Enter
        match event {
            Event::Key(key_event) => {
                if key_event.code != crossterm::event::KeyCode::Enter {
                    return;
                }
            }

            _ => return,
        }
        let path = &args[1];
        // Save the file
        if let Some(panel) = editor.get_widget(WidgetType::Panel) {
            editor.focused_widget_id = panel.get_id();
        }
        if let Some(panel) = editor.get_widget(WidgetType::Panel) {
            fs::write(path, panel.get_buffer().to_string()).unwrap();
            editor.render(panel.get_cursor_view(), panel.is_cursor_visible());
            editor.written = false;
            command_line.buffer = Rope::from_str("");
            command_line.text_position = 0;
            command_line.focused = false;
            command_line.old_buffer = command_line.buffer.clone();
        }
    }
}

impl Default for CommandLine {
    fn default() -> Self {
        // Return a new Widget with default values here
        Self {
            typ: WidgetType::CommandLine,
            id: 0,
            buffer: Rope::from_str(""),
            old_buffer: Rope::from_str(""),
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
            commands: HashMap::new(),
            positions: Vec::new(),
            position_idx: 0,
            list_popup: None,
            z_idx: 0,
            colors: Vec::new(),
        }
    }
}

impl ProcessEvent for CommandLine {
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
        WidgetType::CommandLine
    }
    fn get_id(&self) -> usize {
        self.id
    }
    fn get_z_idx(&self) -> usize {
        self.z_idx
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
        self.z_idx = z_idx;
    }

    fn event(
        &mut self,
        editor: &mut TextEditor,
        event: &Event,
    ) -> Option<(CursorPosition, ShouldExit)> {
        if self.focused {
            if let Event::Key(key_event) = event {
                if key_event.modifiers == crossterm::event::KeyModifiers::NONE {
                    match key_event.code {
                        crossterm::event::KeyCode::Tab => self.create_popup(editor),
                        crossterm::event::KeyCode::Esc => {
                            if let Some((typ, id)) = self.list_popup {
                                editor.remove_widget_id(id, typ);
                                self.list_popup = None;
                                return Some((self.update_cursor_position_and_view(), false));
                            }
                            self.focused = false;

                            if let Some(panel) = editor.get_widget(WidgetType::Panel) {
                                editor.focused_widget_id = panel.get_id();
                            }
                            if let Some(panel) = editor.get_widget_mut(WidgetType::Panel) {
                                panel.set_focused(true);
                                panel.remove_color(&|c: &ColorText| {
                                    c.tag == ColorTextTag::Selection
                                });
                                return Some((panel.update_cursor_position_and_view(), false));
                            }
                        }
                        crossterm::event::KeyCode::Char(c) => {
                            self.buffer.insert_char(self.text_position, c);
                            self.text_position += 1;
                            self.execute_command(editor, event);
                            self.update_popup(editor);
                            return Some((self.update_cursor_position_and_view(), false));
                        }
                        crossterm::event::KeyCode::Backspace => {
                            if self.text_position > 0 {
                                self.buffer
                                    .remove(self.text_position - 1..self.text_position);
                                self.text_position -= 1;
                            }
                            self.execute_command(editor, event);
                            self.update_popup(editor);
                            return Some((self.update_cursor_position_and_view(), false));
                        }
                        crossterm::event::KeyCode::Enter => {
                            self.execute_command(editor, event);
                        }
                        _ => {}
                    }
                }
                if key_event.modifiers == crossterm::event::KeyModifiers::CONTROL {
                    match key_event.code {
                        crossterm::event::KeyCode::Char('j') => {
                            // TODO: repace this
                            self.execute_command(editor, event);
                        }
                        _ => {}
                    }
                }
            }
            self.update_popup(editor);
        }
        return None;
    }
}
