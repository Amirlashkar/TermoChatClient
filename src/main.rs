mod components;
mod tui;

use components::app::{
    App,
    run_app,
};
use ratatui::{
    prelude::CrosstermBackend,
    Terminal,
};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen}
};
use std::{
    io,
    io::Result,
};


fn main() -> Result<()>{
    ratatui::init();
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen)?;

    // core
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();
    let _res = run_app(&mut terminal, &mut app);

    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    ratatui::restore();
    Ok(())
}
