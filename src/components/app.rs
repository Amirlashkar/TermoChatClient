use crate::tui::core::draw_ui;
use crate::server_talk::session::Session;
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
use crossbeam_channel::{
    unbounded,
    Receiver,
    Sender,
};
use crossterm::event::{self, Event};
use std::{
    vec,
    char,
    thread,
    cell::RefCell,
    io::Result,
    rc::Rc,
};


pub struct App {
    pub exit:             bool,
    pub form:             Form,
    pub session:          Session,
    pub selected_block:   Block,
    pub selected_screen:  Screen,
    pub mode:             Modes,

    // Coordination of showing line
    pub all_input:        Rc<RefCell<Vec<String>>>,
    pub char_index:       usize,
    pub line_index:       usize,
    pub room_index:       usize,

    pub messages:         Vec<String>,
    pub to_ws:            Sender<String>,
    pub from_ws:          Receiver<String>,
    pub stop_to_ws:       Sender<bool>,
    pub room_names:       Vec<String>,
    pub room_hashes:      Vec<String>,
    pub is_user_msg:      bool,
    pub chat_scroll_state:ScrollbarState,
    pub chat_scroll_index:usize,
}

impl App {
    pub fn new() -> Self {
        let (tx_to_ws, rx_from_cli)         = unbounded::<String>(); // CLI -> WebSocket
        let (tx_to_cli, rx_from_ws)         = unbounded::<String>(); // WebSocket -> CLI
        let (stop_sender, stop_from_cli)    = unbounded::<bool>();   // Breaking channels

        let screen:      Screen; // This kind of approach is needed for future token conditions
        let formm:       Form;   // If you know, you know
        let mut rnames:  Vec<String> = vec!["".to_string()];
        let mut rhashes: Vec<String> = vec!["".to_string()];

        let sess = Session::new(rx_from_cli, tx_to_cli, stop_from_cli);
        match sess.token.clone() {
            Some(_value) => {
                let pong = sess.ping();
                if pong.contains_key("ok") {
                    screen = Screen::Main;
                    formm  = Form::new(None, None, None);
                    let rooms = sess.room_publist();
                    if rooms.contains_key("ok") {
                        rnames  = rooms.get("names").unwrap().to_vec();
                        rhashes = rooms.get("hashes").unwrap().to_vec();
                    }
                } else {
                    screen = Screen::Form;
                    formm  = Form::new(Some(Forms::SignIn), Some(2), None);
                }
            },

            None         => {
                screen = Screen::FormChoose;
                let opts = vec!["SignUp".to_string(), "LogIn".to_string()];
                formm  = Form::new(None, None, Some(opts));
            },
        }

        let inp = Rc::clone(&formm.inputs[0]);

        Self {
            exit:             false,
            form:             formm,
            session:          sess,
            selected_block:   Block::Rooms,
            selected_screen:  screen,
            mode:             Modes::Normal,
            all_input:        inp,
            char_index:       0,
            line_index:       0,
            room_index:       0,
            messages:         Vec::new(),
            to_ws:            tx_to_ws,
            from_ws:          rx_from_ws,
            stop_to_ws:       stop_sender,
            room_names:       rnames,
            room_hashes:      rhashes,
            is_user_msg:      true,
            chat_scroll_state:ScrollbarState::new(0),
            chat_scroll_index:0,
        }
    }

    pub fn update_input(&mut self) {
        self.all_input = Rc::clone(&self.form.inputs[self.form.selected_input]);
    }

    pub fn jump2form(&mut self) {
        let selected_form = match self.form.options[self.form.selected_input].as_str() {
            "SignUp" => Forms::SignUp,
            _        => Forms::SignIn,
        };

        let n_inputs = match selected_form {
            Forms::SignUp => 4,
            _             => 2,
        };

        self.selected_screen = Screen::Form;
        self.form = Form::new(Some(selected_form), Some(n_inputs), None);
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
        new_cursor_pos.clamp(0, self.all_input.borrow_mut()[self.line_index].chars().count())
    }

    fn reset_cursor(&mut self) {
        self.char_index = 0;
    }

    fn reset_line(&mut self) {
        self.line_index = 0;
    }

    pub fn set_curser(&mut self) {
        self.char_index = self.all_input.borrow()[self.line_index].len();
    }

    // Gets index of selected char with respect to self.char_index
    fn byte_index(&self) -> usize {
        let borrowed = self.all_input.borrow();
        borrowed[self.line_index]
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.char_index)
            .unwrap_or(borrowed[self.line_index].len())
    }

    pub fn insert_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.all_input.borrow_mut()[self.line_index].insert(index, new_char);
        self.move_cursor_right();
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.char_index != 0;
        if is_not_cursor_leftmost {

            let current_index = self.char_index;
            let from_left_to_current_index = current_index - 1;

            { // to surpress borrowed variable after we were done with it
            let mut borrowed = self.all_input.borrow_mut();
            let before_char_to_delete = borrowed[self.line_index].chars().take(from_left_to_current_index);
            let after_char_to_delete = borrowed[self.line_index].chars().skip(current_index);
            borrowed[self.line_index] = before_char_to_delete.chain(after_char_to_delete).collect();
            }

            self.move_cursor_left();
        }
    }

    pub fn new_line(&mut self) {
        self.all_input.borrow_mut().push("".to_string());
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
        if new_line_index < self.all_input.borrow().len() {
            self.line_index = new_line_index;
            self.clamp_cursor(self.char_index);
        }
    }

    // Removes one word behind
    pub fn delete_word(&mut self) {
        let is_not_cursor_leftmost = self.char_index != 0;
        if is_not_cursor_leftmost {
            let mut borrowed = self.all_input.borrow_mut();
            let before_cursor: String = borrowed[self.line_index].chars().take(self.char_index).collect();
            let new_cursor_pos = before_cursor
                .trim_end()              // Remove trailing spaces
                .rfind(' ')              // Find last space
                .map(|i| i + 1)          // Position after space
                .unwrap_or(0);           // If no space, go to start

            let after_cursor: String = borrowed[self.line_index].chars().skip(self.char_index).collect();

            borrowed[self.line_index] = format!(
                "{}{}",
                &before_cursor[..new_cursor_pos],
                after_cursor
            );

            self.char_index = new_cursor_pos;
        }
    }

    // Going one word foreward
    pub fn foreword(&mut self) {
        let space_indices: Vec<usize> = self.all_input.borrow()[self.line_index][self.char_index..]
            .trim_start()
            .match_indices(" ")
            .map(|(i, _)| i + self.char_index)
            .collect();

        if space_indices.len() != 0 {
            self.char_index = space_indices[0];
        } else {
            self.char_index = self.all_input.borrow()[self.line_index].len();
        }
    }

    // Going one word backward
    pub fn backword(&mut self) {
        let before_cursor: String = self.all_input.borrow()[self.line_index].chars().take(self.char_index).collect();
        let new_cursor_pos = before_cursor
            .trim_end()
            .rfind(' ')
            .map(|i| i + 1)
            .unwrap_or(0);

        self.char_index = new_cursor_pos
    }

    pub fn toggle_form_bool(&mut self) {
        let is_last = self.form.selected_input == self.form.inputs.len() - 1;
        if is_last {
            self.form.switch_pub();
        } else {
            self.mode = Modes::Insert;
            self.set_curser();
        }
    }

    fn vec2str(&self, vec: Vec<String>) -> String {
        let mut res = "".to_string();
        for s in vec.clone().iter() {
            res += s;
        }
        res
    }

    fn str2vec(&self, vec: String) -> Vec<String> {
        let parts: Vec<String> = vec.split("\\")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        parts
    }

    fn add_msg(&mut self, msg: &mut Vec<String>) {
        self.messages.push("User1:".to_string());
        {
            for (i, _) in msg.clone().iter().enumerate() {
                msg[i] = msg[i].replace("\\", "");
                self.messages.push(msg[i].to_string())
            }
            self.messages.push("".to_string());
        }
        self.reset_cursor();
        self.reset_line();
    }

    pub fn submit_message(&mut self) {
        let ends_with_slash = {
            let borrowed = self.all_input.borrow_mut();
            borrowed[self.line_index].ends_with('\\')
        };

        if ends_with_slash {
            self.new_line();
        } else {

            match self.selected_screen {
                Screen::Main => {
                    let mut borrowed = self.all_input.borrow_mut().to_vec();
                    self.add_msg(&mut borrowed);
                    *self.all_input.borrow_mut() = vec!["".to_string()];
                },

                Screen::Form => {
                    match self.form.kind {
                        Forms::SignIn => {
                            let show_name = &self.form.inputs[0].borrow().clone()[self.line_index];
                            let password  = &self.form.inputs[1].borrow().clone()[self.line_index];
                            self.session.login(show_name, password);
                        },
                        _ => {}
                    }
                },

                _ => {},
            }
        }
    }

    pub fn submit_form(&mut self) {
        match self.form.kind {
            Forms::SignUp => {
                let show_name        = &self.form.inputs[0].borrow_mut()[0].clone();
                let password         = &self.form.inputs[1].borrow_mut()[0].clone();
                let related_question = &self.form.inputs[2].borrow_mut()[0].clone();
                let related_answer   = &self.form.inputs[3].borrow_mut()[0].clone();

                let response = self.session.signup(show_name, password,
                    related_question, related_answer);

                if response.contains_key("ok") {
                    self.selected_screen = Screen::FormChoose;
                }

                let response = self.session.room_publist();
                if response.contains_key("ok") {
                    self.room_names  = response.get("names").unwrap().to_vec();
                    self.room_hashes = response.get("hashes").unwrap().to_vec();
                }
            },

            Forms::SignIn => {
                let show_name        = &self.form.inputs[0].borrow_mut()[0].clone();
                let password         = &self.form.inputs[1].borrow_mut()[0].clone();

                let response = self.session.login(show_name, password);

                if response.contains_key("ok") {
                    self.selected_screen = Screen::Main;
                    *self.all_input.borrow_mut() = vec![String::new()];
                    let response = self.session.room_publist();
                    if response.contains_key("ok") {
                        self.room_names  = response.get("names").unwrap().to_vec();
                        self.room_hashes = response.get("hashes").unwrap().to_vec();
                    }
                }
            },

            Forms::RoomCreator => {
                let name      = &self.form.inputs[0].borrow_mut()[0].clone();
                let is_public = &self.form.inputs[1].borrow_mut()[0].clone();

                let response  = self.session.room_build(name, is_public);

                if response.contains_key("ok") {
                    self.selected_screen = Screen::Main;
                    *self.all_input.borrow_mut() = vec![String::new()];
                }
            },

            _ => {},
        }
    }

    pub fn enter_room(&mut self) {
        let ses = self.session.clone();
        let room_hash = self.room_hashes[self.room_index].to_owned();
        thread::spawn(move || {
            if let Err(e) = ses.chat_connect(&room_hash) {
                eprintln!("WebSocket thread error: {}", e);
            }
        });

        // Shift cursor to typing box
        self.selected_block = Block::Typing;
        self.mode = Modes::Insert;
        self.set_curser();
    }

    pub fn leave_room(&mut self) {
        let _ = self.stop_to_ws.send(true);
    }

    pub fn send_message(&self) {
        let msg = self.all_input.borrow();
        let msg = self.vec2str(msg.to_vec());
        let err = self.to_ws.send(msg); // Send to WebSocket thread
        match err {
            Ok(_val) => {}
            Err(crossbeam_channel::SendError(er)) => {println!("ERROR: {}", er)}
        }
    }

    pub fn receive_message(&mut self) {
        let msg = self.from_ws.try_recv().ok(); // Non-blocking receive
        match msg {
            Some(text) => {
                println!("{text}");
                let mut msg = self.str2vec(text);
                self.add_msg(&mut msg);
            }
            _ => {}
        }
    }
}

pub fn hover_over(last: usize, selected_index: &mut usize, go_next: bool) {
    let mut selected = *selected_index;

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

    *selected_index = selected;
}


// Not an struct method!
pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    while !app.exit {
        // Link form and input field
        match app.selected_screen {
            Screen::FormChoose => {},
            Screen::Main       => {
                app.update_input();
                app.receive_message();
            }
            _                  => {app.update_input();},
        }

        terminal.draw(|frame| draw_ui(frame, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {continue}
            let _res = logics::key_bindings(app, key);
        }
    }
    Ok(())
}
