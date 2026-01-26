use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use super::Screen;

struct HomeApp {
    menu_items: Vec<&'static str>,
    selected: usize,
}

impl HomeApp {
    fn new() -> Self {
        Self {
            menu_items: vec![
                "外字マップ プレビュー",
                "Tokenize プレビュー",
                "Scopenize プレビュー",
                "Retokenize プレビュー",
                "青空文庫 同期",
            ],
            selected: 0,
        }
    }

    fn next(&mut self) {
        if self.selected < self.menu_items.len() - 1 {
            self.selected += 1;
        }
    }

    fn prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn get_screen(&self) -> Screen {
        match self.selected {
            0 => Screen::Gaiji,
            1 => Screen::Tokenize,
            2 => Screen::Scopenize,
            3 => Screen::Retokenize,
            4 => Screen::Sync,
            _ => Screen::Home,
        }
    }
}

pub fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<Screen> {
    let mut app = HomeApp::new();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(Screen::Exit),
                    KeyCode::Up | KeyCode::Char('k') => app.prev(),
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Enter => return Ok(app.get_screen()),
                    KeyCode::Char('1') => return Ok(Screen::Gaiji),
                    KeyCode::Char('2') => return Ok(Screen::Tokenize),
                    KeyCode::Char('3') => return Ok(Screen::Scopenize),
                    KeyCode::Char('4') => return Ok(Screen::Retokenize),
                    KeyCode::Char('5') => return Ok(Screen::Sync),
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &HomeApp) {
    let chunks = Layout::vertical([
        Constraint::Length(5),
        Constraint::Min(1),
        Constraint::Length(3),
    ])
    .split(f.area());

    // Header
    let header_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "aozora-rs-tester プレビュー",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
    ];
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::BOTTOM))
        .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // Menu
    let items: Vec<ListItem> = app
        .menu_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let prefix = format!("[{}] ", i + 1);
            let style = if i == app.selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                Span::styled(*item, style),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected));

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" メニュー "))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    f.render_stateful_widget(list, chunks[1], &mut state);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("↑/↓: 選択  ", Style::default().fg(Color::Gray)),
        Span::styled("Enter: 決定  ", Style::default().fg(Color::Gray)),
        Span::styled("1-4: 直接選択  ", Style::default().fg(Color::Gray)),
        Span::styled("q: 終了", Style::default().fg(Color::Gray)),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}
