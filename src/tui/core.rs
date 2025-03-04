use std::{fmt::format, u16};

use crate::components::{
    app::App,
    states,
};

use ratatui::{
    layout::{Alignment, Constraint, Direction, Flex, Layout, Position},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};


// Custom colors to use
const BORDER:       Color = Color::Rgb(11, 255, 37);
const TYPING_BORDER:Color = Color::Rgb(253, 242, 83);
const CHAT_FG:      Color = Color::Rgb(203, 3, 8);
const CHAT_BG:      Color = Color::Rgb(203, 3, 8);
const TYPING_FG:    Color = Color::Rgb(203, 3, 8);
const TYPING_BG:    Color = Color::Rgb(203, 3, 8);
const USER_FORM:    Color = Color::Rgb(247, 155, 35);

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
                        states::Block::Rooms => BORDER,
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
                        states::Block::Chat => BORDER,
                        _ => Color::White,
                    }
                ))
                .title(Line::from("Messages").centered());

            let messages: Vec<ListItem> = app.messages
                .iter()
                .enumerate()
                .map(|(_, m)| {
                    let content = Line::from(Span::raw(format!("{m}"))
                        .style(
                            match app.is_user_msg {
                                true => Style::new().fg(CHAT_FG).bg(BORDER),
                                false => Style::new().bg(Color::Gray).fg(Color::Black)
                            }
                        ));
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
                                states::Modes::Normal => BORDER,
                                states::Modes::Insert => TYPING_BORDER,
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
                        _ => {vec![Line::from(app.all_input[app.line_index].as_str()).style(Style::new().fg(CHAT_FG))]}
                    }
                },
                states::Modes::Insert => vec![Line::from(app.all_input[app.line_index].as_str()).style(CHAT_FG)]
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
        states::Screen::UserForm => {

            // To draw center layout ----------
            let vchunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(60),
                ])
                .flex(Flex::Center)
                .split(f.area());

            let hchunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(40),
                ])
                .flex(Flex::Center)
                .split(vchunk[0]);
            // ---------------------------------

            let form_blk = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(USER_FORM)
                .title(Line::from(
                    match app.form.kind {
                        Some(states::Forms::SignUp) => "Sign Up Form",
                        _                     => "Sign In Form",
                    }
                ).centered());
            f.render_widget(form_blk, hchunk[0]);

            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(70),
                ])
                .margin(1)
                .split(hchunk[0]);

            // To draw inner layout ------------
            let titles: Vec<String> = match app.form.kind {
                Some(states::Forms::SignUp) => vec![format!("Username:"),
                                                    format!(""), format!("Password:"),
                                                    format!(""), format!("Question:"),
                                                    format!(""), format!("Answer:")],
                _                           => vec![format!("Username:"), format!(""),
                                                    format!("Password:")],
            };

            let rows: Vec<Constraint> = vec![Constraint::Percentage(100 / titles.len() as u16); titles.len()];
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints(rows);

            for (i, title) in titles.iter().enumerate() {
                if i % 2 == 0 {
                    let border: Borders;
                    if i == app.form.selected_input * 2 {
                        border = Borders::ALL;
                        match app.mode {
                            states::Modes::Insert => {
                                f.set_cursor_position(Position::new(
                                    rows.clone().split(cols[1])[i].x + app.char_index as u16 + 1,
                                    rows.clone().split(cols[1])[i].y + 1,
                                ));
                            },
                            _                     => {}
                        };
                    } else {
                        border = Borders::NONE;
                    };

                    let input_blk = Block::default()
                        .borders(border)
                        .border_type(BorderType::Rounded);

                    let title_line = Paragraph::new(
                        vec![
                            Line::from(Span::from("")), // To align title vertically
                            Line::from(Span::from(title))]
                    )
                    .alignment(Alignment::Center)
                    .block(input_blk.clone().borders(Borders::NONE));

                    f.render_widget(title_line, rows.clone().split(cols[0])[i]);
                    f.render_widget(input_blk, rows.clone().split(cols[1])[i]);
                }
            }
            // ---------------------------------

        },
        _ => {}
    }
}
