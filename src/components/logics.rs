use super::states::{Block, RoomForm, Screen, UserForm};
use super::app::App;

use crossterm::event::{KeyCode, KeyEvent};
use std::io;


pub fn key_bindings(app: &mut App, e: KeyEvent) -> io::Result<()> {
    match app.selected_screen {
        Screen::Main => match e.code {

            KeyCode::Char('q') => {
                app.exit = true;
            },

            KeyCode::Tab => {
                match app.selected_block {
                    Block::Rooms => {app.selected_block = Block::Chat},
                    Block::Chat => {app.selected_block = Block::Typing},
                    Block::Typing => {app.selected_block = Block::Rooms},
                }
            },

            _ => {}
        },
        _ => {}
    };
    Ok(())
}
