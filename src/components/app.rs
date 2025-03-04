use crate::tui::core::draw_ui;
use super::{
    forms::Form, logics, states::{
        Block, Forms, Modes, Screen
    }
};

use ratatui::{
    backend::Backend,
    widgets::ScrollbarState,
    Terminal,
};
use crossterm::event::{self, Event};
use dotenv::dotenv;
use std::{char, io::Result};


pub struct App {
    pub exit:             bool,
    pub form:             Form,
    pub selected_block:   Block,
    pub selected_screen:  Screen,
    pub mode:             Modes,

    // Coordination of showing line
    pub all_input:        Vec<String>,
    pub char_index:       usize,
    pub line_index:       usize,

    pub messages:         Vec<String>,
    pub is_user_msg:      bool,
    pub chat_scroll_state:ScrollbarState,
    pub chat_scroll_index:usize,
}

impl App {
    pub fn new() -> Self {
        dotenv().ok();
        let screen:  Screen; // This kind of approach is needed for future token conditions
        let formm: Form; // If you know, you know
        match std::env::var("TOKEN") {
            Ok(_value) => {
                screen = Screen::Main;
                formm = Form::new(None, None);
            },
            Err(_error) => {
                screen = Screen::UserForm;
                let form_kind = Forms::SignUp; // TODO: We should check if the user already exist or what
                let n_inputs = match form_kind {
                    // Create a form with different number of inputs with
                    // respect to UserForm kind
                    Forms::SignUp => Some(4),
                    _             => Some(2), // We never hit rooms form here so its ok
                };
                formm = Form::new(Some(form_kind), n_inputs);
            }
        };

        Self {
            exit:             false,
            form:             formm,
            selected_block:   Block::Rooms,
            selected_screen:  screen,
            mode:             Modes::Normal,
            all_input:        vec!["".to_string()],
            char_index:       0,
            line_index:       0,
            messages:         Vec::new(),
            is_user_msg:      true,
            chat_scroll_state:ScrollbarState::new(0),
            chat_scroll_index:0,
        }
    }

    pub fn form_field_hover(&mut self, go_next: bool) {
        let mut selected = self.form.selected_input;
        let last = self.form.inputs.len() - 1;
        if go_next {
            if selected != last {
                selected = selected.saturating_add(1);
            } else {
                selected = 0;
            }
        } else {
            if selected != 0 {
                selected = selected.saturating_sub(1);
            } else {
                selected = last;
            }
        }

        self.form.selected_input = selected;
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.char_index.saturating_sub(1);
        self.char_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.char_index.saturating_add(1);
        self.char_index = self.clamp_cursor(cursor_moved_right);
    }

    // Avoids cursor from going out of bound
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.all_input[self.line_index].chars().count())
    }

    fn reset_cursor(&mut self) {
        self.char_index = 0;
    }

    fn reset_line(&mut self) {
        self.line_index = 0;
    }

    // Gets index of selected char with respect to self.char_index
    fn byte_index(&self) -> usize {
        self.all_input[self.line_index]
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.char_index)
            .unwrap_or(self.all_input[self.line_index].len())
    }

    pub fn insert_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.all_input[self.line_index].insert(index, new_char);
        self.move_cursor_right();
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.char_index != 0;
        if is_not_cursor_leftmost {

            let current_index = self.char_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.all_input[self.line_index].chars().take(from_left_to_current_index);
            let after_char_to_delete = self.all_input[self.line_index].chars().skip(current_index);

            self.all_input[self.line_index] = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    pub fn new_line(&mut self) {
        self.all_input.push("".to_string());
        self.line_index = self.line_index.saturating_add(1);
        self.reset_cursor();
    }

    pub fn go_top_line(&mut self) {
        if self.line_index != 0 {
            self.line_index = self.line_index.saturating_sub(1);
            self.clamp_cursor(self.char_index);
        }
    }

    pub fn go_bottom_line(&mut self) {
        let new_line_index = self.line_index.saturating_add(1);
        if new_line_index < self.all_input.len() {
            self.line_index = new_line_index;
            self.clamp_cursor(self.char_index);
        }
    }

    // Removes one word behind
    pub fn delete_word(&mut self) {
        let is_not_cursor_leftmost = self.char_index != 0;
        if is_not_cursor_leftmost {
            let before_cursor: String = self.all_input[self.line_index].chars().take(self.char_index).collect();
            let new_cursor_pos = before_cursor
                .trim_end()              // Remove trailing spaces
                .rfind(' ')              // Find last space
                .map(|i| i + 1)          // Position after space
                .unwrap_or(0);           // If no space, go to start

            let after_cursor: String = self.all_input[self.line_index].chars().skip(self.char_index).collect();

            self.all_input[self.line_index] = format!(
                "{}{}",
                &before_cursor[..new_cursor_pos],
                after_cursor
            );

            self.char_index = new_cursor_pos;
        }
    }

    // Going one word foreward
    pub fn foreword(&mut self) {
        let space_indices: Vec<usize> = self.all_input[self.line_index][self.char_index..]
            .trim_start()
            .match_indices(" ")
            .map(|(i, _)| i + self.char_index)
            .collect();

        if space_indices.len() != 0 {
            self.char_index = space_indices[0];
        } else {
            self.char_index = self.all_input[self.line_index].len();
        }
    }

    // Going one word backward
    pub fn backword(&mut self) {
        let before_cursor: String = self.all_input[self.line_index].chars().take(self.char_index).collect();
        let new_cursor_pos = before_cursor
            .trim_end()
            .rfind(' ')
            .map(|i| i + 1)
            .unwrap_or(0);

        self.char_index = new_cursor_pos
    }

    pub fn submit_message(&mut self) {
        if self.all_input[self.line_index].ends_with('\\') {
            self.new_line();
        } else {
            self.messages.push("User1:".to_string());
            for (i, _) in self.all_input.clone().iter().enumerate() {
                self.all_input[i] = self.all_input[i].replace("\\", "");
                self.messages.push(self.all_input[i].to_string())
            }
            self.messages.push("".to_string());
            self.all_input = vec!["".to_string()];
            self.reset_cursor();
            self.reset_line();
        }
    }
}

// Not an struct method!
pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    while !app.exit {
        terminal.draw(|frame| draw_ui(frame, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {continue}
            let _res = logics::key_bindings(app, key);
        }
    }
    Ok(())
}
