mod app_context;
mod gaiji;
mod home;
mod scopenize;
mod sync;
mod theme;
mod tokenize;

use std::io;

use crate::app_context::AppContext;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

pub enum Screen {
    Home,
    Gaiji,
    Tokenize,
    Scopenize,
    Sync,
    Exit,
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut context = AppContext::new();
    // Try initialize (ignore error here, will be handled/retried in screens or empty list)
    let _ = context.initialize();

    let mut current_screen = Screen::Home;

    loop {
        current_screen = match current_screen {
            Screen::Home => home::run(&mut terminal)?,
            Screen::Gaiji => gaiji::run(&mut terminal)?,
            Screen::Tokenize => tokenize::run(&mut terminal, &mut context)?,
            Screen::Scopenize => scopenize::run(&mut terminal, &mut context)?,
            Screen::Sync => sync::run(&mut terminal)?,
            Screen::Exit => break,
        };
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
