use crate::components::{
    app::App,
    states,
};

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Color},
    text::{Line, Span, Text},
    widgets::{Block, Clear, Borders, List, ListItem, Paragraph, Wrap},
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

            // TODO: Use chunks to embed other widgets inside them
        },
        _ => {}
    }
}
