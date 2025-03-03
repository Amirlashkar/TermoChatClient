use std::u16;

use crate::components::{
    app::App,
    states,
};

use ratatui::{
    layout::{Constraint, Direction, Layout, Position},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};


pub fn draw_ui(f: &mut Frame, app: &App) {
    match app.selected_screen {
        states::Screen::Main => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(70),
                ])
                .split(f.area());

            let rooms = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(
                    match app.selected_block {
                        states::Block::Rooms => Color::Red,
                        _ => Color::White,
                    }
                ))
                .title(Line::from("Rooms").centered());
            f.render_widget(rooms, chunks[0]);

            let chat_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(chunks[1]);

            let chat = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(
                    match app.selected_block {
                        states::Block::Chat => Color::Red,
                        _ => Color::White,
                    }
                ))
                .title(Line::from("Messages").centered());

            let messages: Vec<ListItem> = app.messages
                .iter()
                .enumerate()
                .map(|(_, m)| {
                    let content = Line::from(Span::raw(format!("{m}")));
                    ListItem::new(content)
                })
                .collect();
            let chat = List::new(messages).block(chat.clone());
            f.render_widget(chat, chat_chunks[0]);

            let typing_blk = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(
                    match app.selected_block {
                        states::Block::Typing => {
                            match app.mode {
                                states::Modes::Normal => Color::Red,
                                states::Modes::Insert => Color::Green,
                            }
                        },
                        _ => Color::White,
                    }
                ));

            // What to show on typing box
            let showing_text = match app.mode {
                states::Modes::Normal => {
                    match app.all_input[app.line_index].as_str() {
                        // Keep the draft message
                        "" => {vec![Line::from("Type here ...").style(Style::new().dark_gray())]}
                        _ => {vec![Line::from(app.all_input[app.line_index].as_str()).style(Style::new().red())]}
                    }
                },
                states::Modes::Insert => vec![Line::from(app.all_input[app.line_index].as_str()).style(Style::new().red())]
            };
            let typing_para = Paragraph::new(
                    showing_text
                )
                .block(typing_blk);
            f.render_widget(typing_para, chat_chunks[1]);

            // To set a cursor on typing box
            match app.mode {
                states::Modes::Insert => {
                    #[allow(clippy::cast_possible_truncation)]
                    f.set_cursor_position(Position::new(
                        chat_chunks[1].x + app.char_index as u16 + 1,
                        chat_chunks[1].y + 1
                    ));
                },
                _ => {}
            }

        },
        _ => {}
    }
}
