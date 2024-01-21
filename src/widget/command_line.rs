use std::{collections::HashMap, fs};

use crossterm::{event::Event, style::Color};
use regex::Regex;
use ropey::Rope;
use syntect::parsing::SyntaxReference;

use crate::{editor::TextEditor, widget::popup::Popup};

use super::{
    status_bar,
    widget::{
        BorderStyle, ColorText, ColorTextTag, CursorPosition, CursorPositionByte, ProcessEvent,
        ShouldExit, WidgetType, PS,
    },
};

trait CommandLineCommands {
    fn quit(
        command_line: &mut CommandLine,
        editor: &mut TextEditor,
        args: Vec<String>,
        event: Event,
    ) -> Option<CursorPosition>;
    fn find(
        command_line: &mut CommandLine,
        editor: &mut TextEditor,
        args: Vec<String>,
        event: Event,
    ) -> Option<CursorPosition>;
    fn open(
        command_line: &mut CommandLine,
        editor: &mut TextEditor,
        args: Vec<String>,
        event: Event,
    ) -> Option<CursorPosition>;
    fn save(
        command_line: &mut CommandLine,
        editor: &mut TextEditor,
        args: Vec<String>,
        event: Event,
    ) -> Option<CursorPosition>;
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
        fn(
            command_line: &mut Self,
            editor: &mut TextEditor,
            args: Vec<String>,
            event: Event,
        ) -> Option<CursorPosition>,
    >,
    positions: Vec<(CursorPositionByte, usize)>,
    position_idx: usize,

    list_popup: Option<(WidgetType, usize)>,
    bash_popup: Option<(WidgetType, usize)>,

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
                ) -> Option<CursorPosition>,
            > = HashMap::new();
            m.insert(":quit".to_string(), Self::quit);
            m.insert(":find".to_string(), Self::find);
            m.insert(":open".to_string(), Self::open);
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

    fn parse_command_line(&self) -> Vec<String> {
        let buffer = self.buffer.to_string();
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut in_regex = false;
        let mut chars = buffer.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                // Handle space outside of quotes
                ' ' if !in_single_quote && !in_double_quote && !in_regex => {
                    if !current_arg.is_empty() {
                        args.push(current_arg.clone());
                        current_arg.clear();
                    }
                }
                // Handle slash
                '/' if !in_single_quote && !in_double_quote => {
                    in_regex = !in_regex;
                    current_arg.push(c);
                }
                // Handle end of regex
                '\'' if !in_double_quote && !in_regex => {
                    in_single_quote = !in_single_quote;
                    current_arg.push(c);
                }
                // Handle double quote
                '"' if !in_single_quote && !in_regex => {
                    in_double_quote = !in_double_quote;
                    current_arg.push(c);
                }
                // Handle escape sequence
                '\\' if in_double_quote && chars.peek() == Some(&'"') => {
                    current_arg.push(chars.next().unwrap());
                }
                '\\' if in_single_quote && chars.peek() == Some(&'\'') => {
                    current_arg.push(chars.next().unwrap());
                }
                // Any other character
                _ => current_arg.push(c),
            }
        }

        if !current_arg.is_empty() {
            args.push(current_arg);
        }

        args
    }

    fn execute_command(
        &mut self,
        editor: &mut TextEditor,
        event: &Event,
    ) -> Option<CursorPosition> {
        if self.buffer.chars().next() == Some(':') {
            let args: Vec<String> = self.parse_command_line();
            eprintln!("args: {:?}", args);
            if args.len() > 0 {
                if let Some(command) = self.commands.get(&args[0]) {
                    let res = command(self, editor, args, event.clone());
                    self.old_buffer = self.buffer.clone();
                    return res;
                }
            }
        } else {
            match event {
                Event::Key(key_event) => {
                    if key_event.code != crossterm::event::KeyCode::Enter {
                        return None;
                    }
                }

                _ => return None,
            }
            // This is a bash command, execute it, dont need to parse
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(self.buffer.to_string())
                .output()
                .expect("failed to execute process");
            let output = String::from_utf8_lossy(&output.stdout);
            eprintln!("output: {:?}", output);
            return Some(self.create_output_bash_popup(editor, output.to_string()));
        }

        None
    }

    fn next_position(&mut self, editor: &mut TextEditor) -> (CursorPositionByte, usize) {
        if self.positions.len() > 0 {
            self.position_idx += 1;
            if self.position_idx >= self.positions.len() {
                self.position_idx = 0;
            }
            if let Some(panel) = editor.get_widget_mut(WidgetType::Panel) {
                panel.set_text_position(self.positions[self.position_idx].0);
                panel.update_cursor_position_and_view();
                return self.positions[self.position_idx];
            }
        }
        (0, 0)
    }

    fn prev_position(&mut self, editor: &mut TextEditor) {
        if self.positions.len() > 0 {
            if self.position_idx == 0 {
                self.position_idx = self.positions.len() - 1;
            } else {
                self.position_idx -= 1;
            }
            if let Some(panel) = editor.get_widget_mut(WidgetType::Panel) {
                panel.set_text_position(self.positions[self.position_idx].0);
                panel.update_cursor_position_and_view();
            }
        }
    }

    fn create_output_bash_popup(
        &mut self,
        editor: &mut TextEditor,
        text: String,
    ) -> CursorPosition {
        // Delete the old popup
        if let Some((typ, id)) = self.bash_popup {
            editor.remove_widget_id(id, typ);
            self.bash_popup = None;
        }
        let mut new_line_count = 0;
        let mut max_len = 0;
        let mut curr_len = 0;
        text.chars().for_each(|c| {
            curr_len += 1;
            if c == '\n' {
                if curr_len > max_len {
                    max_len = curr_len;
                }
                new_line_count += 1;
            }
        });

        new_line_count += 1;
        let y = if self.get_cursor_view().1 - new_line_count - 2 < 0 {
            0
        } else {
            self.get_cursor_view().1 - new_line_count - 2
        };
        if new_line_count > editor.height as i32 - 2 {
            new_line_count = editor.height as i32 - 2;
        }
        max_len = if max_len > self.get_width() {
            self.get_width()
        } else {
            max_len
        };
        let mut widget = Popup::new(
            text,
            0,
            y as usize,
            max_len,
            new_line_count as usize,
            Color::Grey,
            Color::Blue,
            true,
            true,
            BorderStyle::Dashed,
        );
        let syntax: &SyntaxReference = unsafe {
            PS.as_ref()
                .unwrap()
                .find_syntax_by_extension("sh")
                .unwrap_or(PS.as_ref().unwrap().find_syntax_plain_text())
        };
        widget.set_syntax(Some(syntax));
        widget.set_theme(Some("base16-eighties.dark".to_string()));
        widget.set_z_idx(10);
        let new_position = widget.get_cursor_view();
        self.bash_popup = Some((WidgetType::Popup, editor.add_widget(widget)));
        new_position
    }

    fn create_list_command_popup(&mut self, editor: &mut TextEditor) {
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

    fn update_command_list_popup(&mut self, editor: &mut TextEditor) {
        if let Some((typ, id)) = self.list_popup {
            if let Some(widget) = editor.get_widget_id_mut(id, typ) {
                let current_buffer = self.buffer.to_string();
                let mut text = String::new();
                let mut max_len = 0;
                let mut number_of_commands = 0;
                for (key, _) in &self.commands {
                    if key.starts_with(&current_buffer) || current_buffer.starts_with(key) {
                        if key.len() > max_len {
                            max_len = key.len();
                        }
                        text.push_str(key);
                        text.push_str("\n");
                        number_of_commands += 1;
                    }
                }
                if max_len == 0 {
                    editor.remove_widget_id(id, typ);
                    self.list_popup = None;
                    return;
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
    ) -> Option<CursorPosition> {
        if command_line.old_buffer.cmp(&command_line.buffer) == std::cmp::Ordering::Equal {
            editor.running = false;
        }
        None
    }

    fn find(
        command_line: &mut CommandLine,
        editor: &mut TextEditor,
        args: Vec<String>,
        _event: Event,
    ) -> Option<CursorPosition> {
        if args.len() < 2 {
            command_line.positions.clear();
            command_line.position_idx = 0;
            if let Some(panel) = editor.get_widget_mut(WidgetType::Panel) {
                panel.remove_color(&|c: &ColorText| {
                    c.tag == ColorTextTag::Selection || c.tag == ColorTextTag::Find
                });
            }
            return None;
        }
        let search_term = &args[1];
        if command_line.positions.len() > 0
            && command_line.old_buffer.cmp(&command_line.buffer) == std::cmp::Ordering::Equal
        {
            let (new_position, new_len) = command_line.next_position(editor);
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
                        len: new_len,
                        z_index: 10,
                        tag: ColorTextTag::Selection,
                    },
                );
            }
            return Some(command_line.update_cursor_position_and_view());
        }
        command_line.positions.clear();
        command_line.position_idx = 0;
        if let Some(panel) = editor.get_widget_mut(WidgetType::Panel) {
            let mut found_pos = panel.get_text_position();
            let mut found = false;
            let mut found_len = 0;
            // searchterm === mod
            // searchterm === "mod"
            // searchterm === 'mod'
            // searchterm === /mod/
            // let pattern

            panel.remove_color(&|c: &ColorText| c.tag == ColorTextTag::Find);
            panel.remove_color(&|c: &ColorText| c.tag == ColorTextTag::Selection);

            if search_term.starts_with('/') && search_term.ends_with('/') && search_term.len() > 2 {
                let pattern = &search_term[1..search_term.len() - 1];
                let regex = Regex::new(pattern);
                if let Ok(regex) = regex {
                    regex
                        .find_iter(&panel.get_buffer().to_string())
                        .for_each(|m| {
                            let pos = m.start();
                            let y = panel.get_buffer().byte_to_line(pos);
                            let x = panel.get_buffer().byte_to_char(pos)
                                - panel.get_buffer().line_to_char(y);
                            panel.push_color(
                                y,
                                ColorText {
                                    x,
                                    fg: Color::Red,
                                    bg: Color::Reset,
                                    len: m.len(),
                                    z_index: 5,
                                    tag: ColorTextTag::Find,
                                },
                            );
                            if !found && pos >= found_pos {
                                found = true;
                                found_pos = pos;
                                found_len = m.len();
                                command_line.position_idx = command_line.positions.len();
                            }
                            command_line.positions.push((pos, m.len()));
                            // found_positions.push(m.start());
                        });
                }
            } else {
                let search_term = if search_term.starts_with('"')
                    && search_term.ends_with('"')
                    && search_term.len() > 2
                {
                    &search_term[1..search_term.len() - 1]
                } else if search_term.starts_with('\'')
                    && search_term.ends_with('\'')
                    && search_term.len() > 2
                {
                    &search_term[1..search_term.len() - 1]
                } else {
                    search_term
                };
                panel
                    .get_buffer()
                    .to_string()
                    .match_indices(search_term)
                    .for_each(|(pos, _)| {
                        let y = panel.get_buffer().byte_to_line(pos);
                        let x = panel.get_buffer().byte_to_char(pos)
                            - panel.get_buffer().line_to_char(y);
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
                            found_len = search_term.len();
                            command_line.position_idx = command_line.positions.len();
                        }
                        command_line.positions.push((pos, search_term.len()));
                    });
            }
            if !found && command_line.positions.len() > 0 {
                found_pos = command_line.positions[0].0;
                found_len = command_line.positions[0].1;
                command_line.position_idx = 0;
            }

            if found {
                let y = panel.get_buffer().byte_to_line(found_pos);
                let x =
                    panel.get_buffer().byte_to_char(found_pos) - panel.get_buffer().line_to_char(y);
                panel.push_color(
                    y,
                    ColorText {
                        x,
                        fg: Color::Blue,
                        bg: Color::Reset,
                        len: found_len,
                        z_index: 10,
                        tag: ColorTextTag::Selection,
                    },
                );
                panel.set_text_position(found_pos);
                panel.update_cursor_position_and_view();
            }
        }
        None
    }

    fn open(
        command_line: &mut CommandLine,
        editor: &mut TextEditor,
        args: Vec<String>,
        event: Event,
    ) -> Option<CursorPosition> {
        if args.len() < 2 {
            return None;
        }
        // if event != Enter
        match event {
            Event::Key(key_event) => {
                if key_event.code != crossterm::event::KeyCode::Enter {
                    return None;
                }
            }

            _ => return None,
        }
        let path = &args[1];
        // Open the file
        if let Some(panel) = editor.get_widget(WidgetType::Panel) {
            editor.focused_widget_id = panel.get_id();
        }
        if let Some(panel) = editor.get_widget_mut(WidgetType::Panel) {
            let buffer = fs::read_to_string(path).unwrap();
            eprintln!("buffer: {:?}", buffer);
            panel.set_buffer(Rope::from_str(&buffer));
            panel.set_text_position(0);
            let res = panel.update_cursor_position_and_view();
            panel.remove_color(&|c: &ColorText| {
                c.tag == ColorTextTag::Selection || c.tag == ColorTextTag::Find
            });
            editor.written = false;
            command_line.buffer = Rope::from_str("");
            command_line.text_position = 0;
            command_line.focused = false;
            command_line.old_buffer = command_line.buffer.clone();
            return Some(res);
        }
        None
    }

    fn save(
        command_line: &mut CommandLine,
        editor: &mut TextEditor,
        args: Vec<String>,
        event: Event,
    ) -> Option<CursorPosition> {
        if args.len() < 2 {
            return None;
        }
        // if event != Enter
        match event {
            Event::Key(key_event) => {
                if key_event.code != crossterm::event::KeyCode::Enter {
                    return None;
                }
            }

            _ => return None,
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
        if let Some(status_bar) = editor.get_widget_mut(WidgetType::StatusBar) {
            status_bar.set_buffer(Rope::from_str(path));
        }
        None
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
            bash_popup: None,
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
                if key_event.modifiers == crossterm::event::KeyModifiers::SHIFT {
                    match key_event.code {
                        crossterm::event::KeyCode::Char(c) => {
                            self.buffer.insert_char(self.text_position, c);
                            self.text_position += 1;
                            let position = self.execute_command(editor, event);
                            self.update_command_list_popup(editor);
                            if let Some(position) = position {
                                return Some((position, false));
                            }
                            return Some((self.update_cursor_position_and_view(), false));
                        }
                        _ => {}
                    }
                }
                if key_event.modifiers == crossterm::event::KeyModifiers::NONE {
                    match key_event.code {
                        crossterm::event::KeyCode::Tab => {
                            eprintln!("command line tab");
                            if let Some((bash_popup_type, bash_popup_id)) = self.bash_popup {
                                editor.focused_widget_id = bash_popup_id;
                                if let Some(widget) =
                                    editor.get_widget_id_mut(bash_popup_id, bash_popup_type)
                                {
                                    self.focused = false;
                                    widget.set_focused(true);
                                    return Some((widget.update_cursor_position_and_view(), false));
                                }
                            }
                            self.create_list_command_popup(editor)
                        }
                        crossterm::event::KeyCode::Esc => {
                            if let Some((typ, id)) = self.list_popup {
                                editor.remove_widget_id(id, typ);
                                self.list_popup = None;
                                return Some((self.update_cursor_position_and_view(), false));
                            }
                            if let Some((typ, id)) = self.bash_popup {
                                editor.remove_widget_id(id, typ);
                                self.bash_popup = None;
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
                            let position = self.execute_command(editor, event);
                            self.update_command_list_popup(editor);
                            if let Some(position) = position {
                                return Some((position, false));
                            }
                            return Some((self.update_cursor_position_and_view(), false));
                        }
                        crossterm::event::KeyCode::Backspace => {
                            if self.text_position > 0 {
                                self.buffer
                                    .remove(self.text_position - 1..self.text_position);
                                self.text_position -= 1;
                            }
                            let position = self.execute_command(editor, event);
                            self.update_command_list_popup(editor);
                            if let Some(position) = position {
                                return Some((position, false));
                            }
                            return Some((self.update_cursor_position_and_view(), false));
                        }
                        crossterm::event::KeyCode::Enter => {
                            let position = self.execute_command(editor, event);
                            self.update_command_list_popup(editor);
                            if let Some(position) = position {
                                return Some((position, false));
                            }
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
            self.update_command_list_popup(editor);
        }
        return None;
    }
}
