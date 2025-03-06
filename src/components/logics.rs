use super::states::{Block, Modes, Screen, Forms};
use super::app::App;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{io, usize};


pub fn key_bindings(app: &mut App, e: KeyEvent) -> io::Result<()> {
    match app.mode {
        Modes::Normal => {

            match e.code {
                KeyCode::Char('q')  => {
                    app.exit = true;
                },
                _ => {}
            }

            match app.selected_screen {
                Screen::Main => {
                    match e.code {

                        KeyCode::Tab => {
                            match app.selected_block {
                                Block::Rooms  => {app.selected_block = Block::Chat},
                                Block::Chat   => {app.selected_block = Block::Typing},
                                Block::Typing => {app.selected_block = Block::Rooms},
                            }
                        },

                        KeyCode::BackTab => {
                            match app.selected_block {
                                Block::Rooms  => {app.selected_block = Block::Typing},
                                Block::Chat   => {app.selected_block = Block::Rooms},
                                Block::Typing => {app.selected_block = Block::Chat},
                            }
                        },

                        KeyCode::Enter => match app.selected_block {
                            Block::Typing => {
                                app.mode = Modes::Insert;
                                app.set_curser();
                            },
                            _ => {}
                        }

                        _ => {}
                    }
                },

                Screen::Form => match e.code {

                    KeyCode::Up => {
                        app.form_field_hover(false);
                    },

                    KeyCode::Down => {
                        app.form_field_hover(true);
                    },

                    KeyCode::Enter => {
                        match app.form.kind {
                            Forms::RoomCreator | Forms::RoomEdit => {
                                let is_last = app.form.selected_input == app.form.inputs.len() - 1;
                                if is_last {
                                    app.form.switch_pub();
                                } else {
                                    app.mode = Modes::Insert;
                                    app.set_curser();
                                }
                            },
                            _ => {
                                app.mode = Modes::Insert;
                                app.set_curser();
                            },
                        }
                    }

                    _ => {}
                },
                _ => {}
            }
        },

        Modes::Insert => {
            match e.modifiers {

                KeyModifiers::CONTROL => {
                    match e.code {
                        KeyCode::Char('w')       => app.delete_word(),
                        _ => {}
                    }
                },

                KeyModifiers::SHIFT => {
                    match e.code {
                        KeyCode::Right           => app.foreword(),
                        KeyCode::Left            => app.backword(),

                        // Allow uppercase letters to happen
                        KeyCode::Char(to_insert) => app.insert_char(to_insert),
                        _ => {}
                    }
                },

                _ => {
                    match e.code {
                        KeyCode::Char(to_insert) => app.insert_char(to_insert),
                        KeyCode::Right           => app.move_cursor_right(),
                        KeyCode::Left            => app.move_cursor_left(),
                        KeyCode::Up              => app.go_top_line(),
                        KeyCode::Down            => app.go_bottom_line(),
                        KeyCode::Backspace       => app.delete_char(),
                        KeyCode::Esc             => app.mode = Modes::Normal,
                        KeyCode::Enter           => app.submit_message(),
                        _ => {}
                    }
                }

            }
        }
    }
    Ok(())
}
