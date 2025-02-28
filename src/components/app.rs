use crate::tui::core::draw_ui;
use super::{
    logics,
    states::{Block, Screen}
};

use ratatui::{
    backend::Backend,
    Terminal,
};
use crossterm::{
    event,
    event::Event,
};
use dotenv::dotenv;
use std::io::Result;


pub struct App {
    pub exit:             bool,
    pub selected_block:   Block,
    pub selected_screen:  Screen,
    pub input_paragraph:  String,
}

impl App {
    pub fn new() -> App {
        dotenv().ok();
        let token = match std::env::var("TOKEN") {
            Ok(value) => value,
            Err(e) => {
                eprintln!("ERROR: {:?}", e);
                e.to_string()
            }
        };

        // This block checks it there is a token available ;
        // TODO: Instead it should first token availablity then check
        // its exiration date.
        let sc = match token.contains("ERROR") {
            true => Screen::UserForm,
            false => Screen::Main
        };

        App {
            exit:             false,
            selected_block:   Block::Rooms,
            selected_screen:  sc,
            input_paragraph:  String::new(),
        }
    }
}

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
