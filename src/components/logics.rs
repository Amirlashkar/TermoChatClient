use super::forms::Form;
use super::states::{Block, Modes, Screen, Forms};
use super::app::{
    App,
    hover_over,
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io;


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
                            Block::Rooms => {
                                app.enter_room();
                            }
                            _ => {}
                        },

                        _ => {}
                    }

                    match app.selected_block {
                        Block::Rooms => match e.code {
                            KeyCode::Up        => {
                                hover_over(app.room_names.len() - 1, &mut app.room_index, false);
                            },
                            KeyCode::Down      => {
                                hover_over(app.room_names.len() - 1, &mut app.room_index, true);
                            },
                            KeyCode::Char('c') => {
                                app.selected_screen = Screen::Form;
                                app.form = Form::new(Some(Forms::RoomCreator), Some(2), None);
                            },
                            KeyCode::Char('e') => {
                                app.selected_screen = Screen::Form;
                                app.form = Form::new(Some(Forms::RoomEdit), Some(2), None);
                            },
                            _ => {}
                        },
                        _ => {},
                    }
                },

                Screen::Form => match e.code {
                    KeyCode::Tab => {
                        hover_over(app.form.inputs.len() - 1, &mut app.form.selected_input, false);
                    },

                    KeyCode::BackTab => {
                        hover_over(app.form.inputs.len() - 1, &mut app.form.selected_input, true);
                    },

                    KeyCode::Enter => match app.form.kind {
                        Forms::RoomCreator | Forms::RoomEdit => {
                            app.toggle_form_bool();
                        },
                        _ => {
                            app.mode = Modes::Insert;
                            app.set_curser();
                        },
                    },

                    KeyCode::Char(' ') => {
                        app.submit_form();
                    },

                    _ => {},
                },

                Screen::FormChoose => match e.code {
                    KeyCode::Up => {
                        hover_over(app.form.options.len() - 1, &mut app.form.selected_input, false);
                    },

                    KeyCode::Down => {
                        hover_over(app.form.options.len() - 1, &mut app.form.selected_input, true);
                    },

                    KeyCode::Enter => {
                        app.jump2form();
                    },

                    _ => {},
                },
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
                        KeyCode::Enter           => {
                            app.send_message();
                            app.submit_message();
                        },
                        _ => {}
                    }
                }

            }
        }
    }
    Ok(())
}
